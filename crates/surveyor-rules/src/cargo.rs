//! Cargo feature resolution over the dependency facts manifests declare.
//!
//! A `DEPENDS_ON` edge from a Cargo manifest carries the entry's `section`, `key`,
//! `optional`, `default_features` and requested `features`; a Package node carries its
//! `[features]` table as `feature.<name>` attributes. From those, [`resolve`] computes
//! which packages a build of one package pulls in and with which features, the way
//! cargo unifies them: a package's features are the union of what every path requests,
//! an optional dependency is taken only when an enabled feature names it, and
//! dev-dependencies count at the root alone.
//!
//! The graph holds first-party manifests only, so a registry crate is a leaf: its own
//! dependencies and features are not seen.

// Rust guideline compliant 2026-02-21

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use surveyor_model::{Edge, Node};

/// Which manifest section a dependency entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// `[dependencies]`.
    Normal,
    /// `[dev-dependencies]`.
    Dev,
    /// `[build-dependencies]`.
    Build,
}

impl Section {
    /// Reads the `section` attribute; an absent or unknown value is a normal dependency.
    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("dev") => Self::Dev,
            Some("build") => Self::Build,
            _ => Self::Normal,
        }
    }
}

/// One dependency entry as the graph records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// Identifier of the package depended on.
    pub package: String,
    /// The name the entry uses, which `dep:key` and `key/feature` refer to.
    pub key: String,
    /// Which section declared it.
    pub section: Section,
    /// Whether a feature must name it for it to be built.
    pub optional: bool,
    /// Whether the entry requests the dependency's `default` feature.
    pub default_features: bool,
    /// Features the entry requests.
    pub features: Vec<String>,
}

impl Dependency {
    /// Reads a `DEPENDS_ON` edge to `target`; an edge without manifest facts is a
    /// required, default-features dependency with nothing requested.
    #[must_use]
    pub fn from_edge(edge: &Edge, target: &Node) -> Self {
        let attr = |name: &str| edge.attributes.get(name).map(String::as_str);
        Self {
            package: target.id.to_string(),
            key: attr("key").map_or_else(|| target.name.clone(), str::to_owned),
            section: Section::parse(attr("section")),
            optional: attr("optional") == Some("true"),
            default_features: attr("default_features") != Some("false"),
            features: attr("features").map(split_list).unwrap_or_default(),
        }
    }
}

/// A package's `[features]` table: each feature and what it enables.
pub type FeatureTable = BTreeMap<String, Vec<String>>;

/// The feature table a Package node's `feature.<name>` attributes describe.
#[must_use]
pub fn feature_table(node: &Node) -> FeatureTable {
    node.attributes
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("feature.")
                .map(|name| (name.to_owned(), split_list(value)))
        })
        .collect()
}

/// Items of a comma-separated attribute value.
fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

/// What a build of one package reaches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Build {
    /// Features enabled on each package reached, the root included.
    pub features: BTreeMap<String, BTreeSet<String>>,
    /// The package each reached package was first reached from.
    pub via: BTreeMap<String, String>,
}

impl Build {
    /// Whether the build pulls `package` in.
    #[must_use]
    pub fn reaches(&self, package: &str) -> bool {
        self.features.contains_key(package)
    }

    /// The packages from the root down to `package`, or nothing when it is not reached.
    #[must_use]
    pub fn path_to(&self, package: &str) -> Vec<String> {
        if !self.reaches(package) {
            return Vec::new();
        }
        let mut chain = vec![package.to_owned()];
        while let Some(parent) = self.via.get(chain.last().map_or("", String::as_str)) {
            chain.push(parent.clone());
        }
        chain.reverse();
        chain
    }
}

/// One item of a feature's list, as cargo reads it.
enum Enables {
    /// Another feature of the same package.
    Feature(String),
    /// An optional dependency, by key (`dep:key`, or a bare key with no feature of that name).
    Dependency(String),
    /// A feature of a dependency (`key/feature`); `weak` (`key?/feature`) does not enable it.
    DependencyFeature {
        key: String,
        feature: String,
        weak: bool,
    },
}

fn classify(item: &str, table: &FeatureTable) -> Enables {
    if let Some(key) = item.strip_prefix("dep:") {
        return Enables::Dependency(key.to_owned());
    }
    if let Some((key, feature)) = item.split_once('/') {
        let (key, weak) = key
            .strip_suffix('?')
            .map_or((key, false), |bare| (bare, true));
        return Enables::DependencyFeature {
            key: key.to_owned(),
            feature: feature.to_owned(),
            weak,
        };
    }
    if table.contains_key(item) {
        Enables::Feature(item.to_owned())
    } else {
        Enables::Dependency(item.to_owned())
    }
}

/// What a set of requested features turns on inside one package.
#[derive(Debug, Default)]
struct Enabled {
    /// Keys of optional dependencies that are built.
    dependencies: BTreeSet<String>,
    /// Features requested on dependencies, by key.
    dependency_features: BTreeMap<String, BTreeSet<String>>,
}

fn enable(table: &FeatureTable, requested: &BTreeSet<String>) -> Enabled {
    let mut enabled = Enabled::default();
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<String> = requested.iter().cloned().collect();
    while let Some(feature) = queue.pop_front() {
        if !seen.insert(feature.clone()) {
            continue;
        }
        let Some(items) = table.get(&feature) else {
            // A requested name that is no feature is the implicit feature of an optional
            // dependency; `default` on a package without one enables nothing.
            if feature != "default" {
                enabled.dependencies.insert(feature);
            }
            continue;
        };
        for item in items {
            match classify(item, table) {
                Enables::Feature(name) => queue.push_back(name),
                Enables::Dependency(key) => {
                    enabled.dependencies.insert(key);
                }
                Enables::DependencyFeature { key, feature, weak } => {
                    if !weak {
                        enabled.dependencies.insert(key.clone());
                    }
                    enabled
                        .dependency_features
                        .entry(key)
                        .or_default()
                        .insert(feature);
                }
            }
        }
    }
    enabled
}

/// Resolves what a build of `root` pulls in.
///
/// `features_of` gives a package's feature table and `dependencies_of` its dependency
/// entries; a package unknown to either is a leaf. With `default_features` the root is
/// built as `cargo build` builds it, without it as `--no-default-features` does.
pub fn resolve(
    root: &str,
    default_features: bool,
    features_of: impl Fn(&str) -> FeatureTable,
    dependencies_of: impl Fn(&str) -> Vec<Dependency>,
) -> Build {
    let mut build = Build::default();
    let root_features: BTreeSet<String> = default_features
        .then(|| "default".to_owned())
        .into_iter()
        .collect();
    build.features.insert(root.to_owned(), root_features);
    let mut queue = VecDeque::from([root.to_owned()]);
    while let Some(package) = queue.pop_front() {
        let requested = build.features.get(&package).cloned().unwrap_or_default();
        let enabled = enable(&features_of(&package), &requested);
        for dep in dependencies_of(&package) {
            if dep.section == Section::Dev && package != root {
                continue;
            }
            if dep.optional && !enabled.dependencies.contains(&dep.key) {
                continue;
            }
            let mut wanted: BTreeSet<String> = dep.features.iter().cloned().collect();
            wanted.extend(
                enabled
                    .dependency_features
                    .get(&dep.key)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
            if dep.default_features {
                wanted.insert("default".to_owned());
            }
            let first = !build.features.contains_key(&dep.package);
            let features = build.features.entry(dep.package.clone()).or_default();
            let before = features.len();
            features.extend(wanted);
            let grew = features.len() > before;
            if first && dep.package != root {
                build.via.insert(dep.package.clone(), package.clone());
            }
            if first || grew {
                queue.push_back(dep.package.clone());
            }
        }
    }
    build
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(entries: &[(&str, &[&str])]) -> FeatureTable {
        entries
            .iter()
            .map(|(name, items)| {
                (
                    (*name).to_owned(),
                    items.iter().map(|i| (*i).to_owned()).collect(),
                )
            })
            .collect()
    }

    fn dep(package: &str, section: Section) -> Dependency {
        Dependency {
            package: package.to_owned(),
            key: package.to_owned(),
            section,
            optional: false,
            default_features: true,
            features: Vec::new(),
        }
    }

    fn optional(package: &str) -> Dependency {
        Dependency {
            optional: true,
            ..dep(package, Section::Normal)
        }
    }

    /// A server whose default feature pulls parquet in, over an ops crate whose default
    /// feature pulls a sniffer in; the server turns the ops defaults off.
    fn workspace() -> (
        BTreeMap<String, FeatureTable>,
        BTreeMap<String, Vec<Dependency>>,
    ) {
        let features = BTreeMap::from([
            (
                "server".to_owned(),
                table(&[
                    ("default", &["tabular"]),
                    ("tabular", &["dep:parquet"]),
                    ("kafka", &["ops?/kafka"]),
                    ("extra-z", &["extra/z"]),
                    ("extra-weak", &["extra?/z"]),
                ]),
            ),
            (
                "ops".to_owned(),
                table(&[
                    ("default", &["sniff"]),
                    ("sniff", &["dep:sniffer"]),
                    ("kafka", &["dep:rdkafka"]),
                ]),
            ),
        ]);
        let deps = BTreeMap::from([
            (
                "server".to_owned(),
                vec![
                    optional("parquet"),
                    Dependency {
                        default_features: false,
                        ..dep("ops", Section::Normal)
                    },
                    dep("tokio", Section::Normal),
                    dep("testkit", Section::Dev),
                    dep("cc", Section::Build),
                    optional("extra"),
                    optional("implicit"),
                ],
            ),
            (
                "ops".to_owned(),
                vec![
                    optional("sniffer"),
                    optional("rdkafka"),
                    dep("ops-dev-only", Section::Dev),
                ],
            ),
        ]);
        (features, deps)
    }

    fn build_with(root_features: &[&str], default_features: bool) -> Build {
        let (features, deps) = workspace();
        let mut deps = deps;
        if let Some(ops) = deps
            .get_mut("server")
            .and_then(|d| d.iter_mut().find(|d| d.package == "ops"))
        {
            ops.features = root_features.iter().map(|f| (*f).to_owned()).collect();
        }
        resolve(
            "server",
            default_features,
            |p| features.get(p).cloned().unwrap_or_default(),
            |p| deps.get(p).cloned().unwrap_or_default(),
        )
    }

    #[test]
    fn a_minimal_build_leaves_optional_and_default_gated_crates_out() {
        let build = build_with(&[], false);
        assert!(build.reaches("ops"));
        assert!(build.reaches("tokio"));
        assert!(build.reaches("cc"));
        assert!(
            build.reaches("testkit"),
            "dev-dependencies count at the root"
        );
        assert!(!build.reaches("ops-dev-only"), "and nowhere else");
        assert!(!build.reaches("parquet"));
        assert!(!build.reaches("sniffer"));
        assert!(!build.reaches("extra"));
        assert!(!build.reaches("implicit"));
    }

    #[test]
    fn default_features_pull_parquet_in_but_not_the_sniffer() {
        let build = build_with(&[], true);
        assert_eq!(build.path_to("parquet"), vec!["server", "parquet"]);
        assert!(
            !build.reaches("sniffer"),
            "ops is depended on without its defaults"
        );
    }

    #[test]
    fn a_requested_dependency_feature_enables_its_optional_dependency() {
        let build = build_with(&["sniff"], false);
        assert_eq!(build.path_to("sniffer"), vec!["server", "ops", "sniffer"]);
        assert_eq!(build.path_to("absent"), Vec::<String>::new());
    }

    #[test]
    fn weak_and_strong_dependency_features_differ() {
        let (features, deps) = workspace();
        let run = |root_feature: &str| {
            let mut features = features.clone();
            let server = features.get_mut("server").expect("server table");
            server.insert("default".to_owned(), vec![root_feature.to_owned()]);
            resolve(
                "server",
                true,
                |p| features.get(p).cloned().unwrap_or_default(),
                |p| deps.get(p).cloned().unwrap_or_default(),
            )
        };
        assert!(
            !run("extra-weak").reaches("extra"),
            "`extra?/z` alone builds nothing"
        );
        let strong = run("extra-z");
        assert!(strong.reaches("extra"));
        assert!(strong.features["extra"].contains("z"));
        // `ops?/kafka` applies because ops is built anyway.
        assert!(run("kafka").reaches("rdkafka"));
    }

    #[test]
    fn an_implicit_feature_names_its_optional_dependency() {
        let (features, deps) = workspace();
        let mut features = features;
        features
            .get_mut("server")
            .expect("server table")
            .insert("default".to_owned(), vec!["implicit".to_owned()]);
        let build = resolve(
            "server",
            true,
            |p| features.get(p).cloned().unwrap_or_default(),
            |p| deps.get(p).cloned().unwrap_or_default(),
        );
        assert!(build.reaches("implicit"));
    }

    #[test]
    fn dependency_cycles_terminate() {
        let deps = BTreeMap::from([
            ("a".to_owned(), vec![dep("b", Section::Normal)]),
            ("b".to_owned(), vec![dep("a", Section::Normal)]),
        ]);
        let build = resolve(
            "a",
            true,
            |_| FeatureTable::new(),
            |p| deps.get(p).cloned().unwrap_or_default(),
        );
        assert_eq!(build.path_to("b"), vec!["a", "b"]);
        assert_eq!(build.path_to("a"), vec!["a"]);
    }

    #[test]
    fn an_edge_without_manifest_facts_is_a_plain_dependency() {
        use surveyor_model::ByteSpan;
        use surveyor_model::{
            CommitId, ContentHash, DeclaredSource, EdgeKind, Evidence, LineSpan, NodeId, NodeKind,
            Provenance, RepoId, RepoPath, SourceLocation, TextSpan,
        };
        let repo = RepoId::new("r");
        let target = Node::synthetic(
            NodeId::new("pkg:cargo/tokio"),
            NodeKind::Package,
            "tokio",
            repo.clone(),
        );
        let evidence = Evidence {
            repo,
            location: SourceLocation::new(
                RepoPath::new("Cargo.toml"),
                TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1)),
            ),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(b"x"),
        };
        let edge = Edge::new(
            NodeId::new("pkg:cargo/a"),
            EdgeKind::DependsOn,
            target.id.clone(),
            Provenance::declared(DeclaredSource::Manifest, evidence),
        );
        let plain = Dependency::from_edge(&edge, &target);
        assert_eq!(plain.key, "tokio");
        assert_eq!(plain.section, Section::Normal);
        assert!(!plain.optional && plain.default_features);
        assert!(plain.features.is_empty());

        let edge = edge.with_attributes(BTreeMap::from([
            ("section".to_owned(), "dev".to_owned()),
            ("key".to_owned(), "tk".to_owned()),
            ("optional".to_owned(), "true".to_owned()),
            ("default_features".to_owned(), "false".to_owned()),
            ("features".to_owned(), "rt,macros".to_owned()),
        ]));
        let detailed = Dependency::from_edge(&edge, &target);
        assert_eq!(detailed.key, "tk");
        assert_eq!(detailed.section, Section::Dev);
        assert!(detailed.optional && !detailed.default_features);
        assert_eq!(detailed.features, vec!["rt", "macros"]);
    }
}
