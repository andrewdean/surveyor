//! Planning an incremental update: which files to relink and what to drop.
//!
//! The plan is a pure function of the stored graph, the names each file references,
//! the changed paths and a probe link of the changed files. It applies four rules:
//! changed and added files are relinked; a file whose edge was resolved against a
//! declaration that changed is relinked; a file that references a name whose set of
//! declarations changed is relinked, unless the name is one the language provides and
//! the change is in an unrelated package; and an edge left dangling by a dropped node
//! is dropped, relinking its file when that file still has facts to give.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::BuildHasher;

use surveyor_model::{
    ContentHash, Edge, EdgeId, EdgeKind, GraphDelta, Node, NodeId, NodeKind, RepoPath,
};

use crate::linker::{Seed, is_builtin_name};
use crate::resolve::{SymbolIndex, Tier, is_nameable};

/// File names whose change restructures packages and so calls for a full index.
const MANIFESTS: &[&str] = &["Cargo.toml", "package.json", "pyproject.toml"];

/// True for package manifests, which an incremental update does not relink.
#[must_use]
pub fn is_manifest(path: &RepoPath) -> bool {
    path.file_name().is_some_and(|n| MANIFESTS.contains(&n))
}

/// The paths an update found changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangedFiles {
    /// Indexed files with new content.
    pub modified: BTreeSet<RepoPath>,
    /// Files the index did not know.
    pub added: BTreeSet<RepoPath>,
    /// Indexed files that are gone.
    pub deleted: BTreeSet<RepoPath>,
}

impl ChangedFiles {
    /// Modified and added paths: the ones with facts to extract.
    pub fn to_extract(&self) -> impl Iterator<Item = &RepoPath> {
        self.modified.iter().chain(&self.added)
    }

    /// Every path the change touches.
    pub fn touched(&self) -> impl Iterator<Item = &RepoPath> {
        self.to_extract().chain(&self.deleted)
    }

    /// True when a manifest changed, which the plan cannot absorb incrementally.
    #[must_use]
    pub fn touches_manifest(&self) -> bool {
        self.touched().any(is_manifest)
    }
}

/// Why a file is relinked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelinkReason {
    /// Its content changed.
    Modified,
    /// It is new.
    Added,
    /// One of its edges was resolved against a declaration that changed.
    DeclarationChanged(NodeId),
    /// It references a name whose declarations were added or removed.
    NameChanged(String),
    /// One of its edges lost an endpoint.
    DanglingEdge(EdgeId),
}

impl RelinkReason {
    /// A short label for reports.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Modified => "modified".to_owned(),
            Self::Added => "added".to_owned(),
            Self::DeclarationChanged(id) => format!("declaration changed: {id}"),
            Self::NameChanged(name) => format!("name changed: {name}"),
            Self::DanglingEdge(id) => format!("dangling edge: {id}"),
        }
    }
}

/// What an incremental update does.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdatePlan {
    /// Files to extract again and relink, with the first reason found.
    pub relink: BTreeMap<RepoPath, RelinkReason>,
    /// Files whose rows are dropped: the relinked and the deleted ones.
    pub drop_files: BTreeSet<RepoPath>,
    /// Dangling edges whose own file is not relinked, dropped by identifier.
    pub drop_edges: Vec<EdgeId>,
    /// The surviving graph the relink resolves against.
    pub seed: Seed,
}

impl UpdatePlan {
    /// Files to relink that did not change themselves.
    pub fn relink_unchanged(&self) -> impl Iterator<Item = &RepoPath> {
        self.relink
            .iter()
            .filter(|(_, r)| !matches!(r, RelinkReason::Modified | RelinkReason::Added))
            .map(|(p, _)| p)
    }
}

/// A declaration as the plan compares it across versions.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Decl {
    hash: ContentHash,
    name: String,
    kind: NodeKind,
    file: RepoPath,
}

fn decls<'a>(
    nodes: impl IntoIterator<Item = &'a Node>,
    files: &BTreeSet<RepoPath>,
) -> HashMap<NodeId, Decl> {
    nodes
        .into_iter()
        .filter(|n| n.kind != NodeKind::File)
        .filter_map(|n| {
            let file = n.file().filter(|f| files.contains(f))?;
            Some((
                n.id.clone(),
                Decl {
                    hash: n.content_hash,
                    name: n.name.clone(),
                    kind: n.kind.clone(),
                    file: file.clone(),
                },
            ))
        })
        .collect()
}

/// Nameable declarations grouped by name: `name -> {(id, kind)}`.
fn by_name(decls: &HashMap<NodeId, Decl>) -> HashMap<&str, BTreeSet<(&NodeId, &NodeKind)>> {
    decls
        .iter()
        .filter(|(_, d)| is_nameable(&d.kind))
        .fold(HashMap::new(), |mut acc, (id, d)| {
            acc.entry(d.name.as_str())
                .or_default()
                .insert((id, &d.kind));
            acc
        })
}

/// Plans the update of a stored graph after `changed` files changed.
///
/// `names` maps every indexed file to the names it references; `probe` is a plain
/// link of the changed files' facts on their own, which supplies the declarations
/// they now hold with the identifiers the real relink will give them.
#[must_use]
pub fn plan_update<S: BuildHasher>(
    nodes: &[Node],
    edges: &[Edge],
    names: &HashMap<RepoPath, Vec<String>, S>,
    changed: &ChangedFiles,
    probe: &GraphDelta,
) -> UpdatePlan {
    let touched: BTreeSet<RepoPath> = changed.touched().cloned().collect();
    let old = decls(nodes, &touched);
    let new = decls(&probe.nodes, &touched);
    let has_facts = |path: &RepoPath| names.contains_key(path) && !changed.deleted.contains(path);

    let mut relink: BTreeMap<RepoPath, RelinkReason> = changed
        .modified
        .iter()
        .map(|p| (p.clone(), RelinkReason::Modified))
        .chain(
            changed
                .added
                .iter()
                .map(|p| (p.clone(), RelinkReason::Added)),
        )
        .collect();

    let changed_decls = changed_declarations(&old, &new);
    for (file, against) in by_declaration(edges, &changed_decls) {
        if !relink.contains_key(file) && has_facts(file) {
            relink.insert(
                file.clone(),
                RelinkReason::DeclarationChanged(against.clone()),
            );
        }
    }

    let closeness = Closeness::new(nodes, edges, names.keys(), probe);
    let changed_names = changed_names(&old, &new, changed);
    for (file, referenced) in names {
        if relink.contains_key(file) || !has_facts(file) {
            continue;
        }
        if let Some(name) = closeness.affected_name(file, referenced, &changed_names) {
            relink.insert(file.clone(), RelinkReason::NameChanged(name.to_owned()));
        }
    }

    let drop_files = |relink: &BTreeMap<RepoPath, RelinkReason>| -> BTreeSet<RepoPath> {
        relink.keys().chain(&changed.deleted).cloned().collect()
    };
    let final_ids = final_node_ids(nodes, probe, &touched, &relink, &drop_files(&relink));
    let mut drop_edges = Vec::new();
    let dropped = drop_files(&relink);
    for edge in edges {
        if final_ids.contains(&edge.src) && final_ids.contains(&edge.dst) {
            continue;
        }
        match evidence_file(edge) {
            Some(file) if dropped.contains(file) => {}
            Some(file) if has_facts(file) && !is_manifest(file) => {
                relink
                    .entry(file.clone())
                    .or_insert_with(|| RelinkReason::DanglingEdge(edge.id.clone()));
            }
            _ => drop_edges.push(edge.id.clone()),
        }
    }

    let drop_files = drop_files(&relink);
    let seed = surviving(nodes, edges, &drop_files, &drop_edges);
    UpdatePlan {
        relink,
        drop_files,
        drop_edges,
        seed,
    }
}

/// Rule 2: declarations whose hash changed, or that appeared or vanished.
fn changed_declarations<'a>(
    old: &'a HashMap<NodeId, Decl>,
    new: &'a HashMap<NodeId, Decl>,
) -> HashSet<&'a NodeId> {
    old.iter()
        .filter(|(id, d)| new.get(*id).is_none_or(|n| n.hash != d.hash))
        .map(|(id, _)| id)
        .chain(new.keys().filter(|id| !old.contains_key(*id)))
        .collect()
}

/// Files whose edges were resolved against a changed declaration, with that declaration.
fn by_declaration<'a>(
    edges: &'a [Edge],
    changed_decls: &HashSet<&NodeId>,
) -> Vec<(&'a RepoPath, &'a NodeId)> {
    edges
        .iter()
        .filter_map(|edge| {
            let file = evidence_file(edge)?;
            let against = edge
                .resolved_against
                .iter()
                .find(|id| changed_decls.contains(id))?;
            Some((file, against))
        })
        .collect()
}

/// Rule 3: names whose set of nameable declarations changed, each with the files the
/// change happened in. A cited path that appears or disappears counts as a name, since
/// documentation edges resolve paths the same way.
fn changed_names<'a>(
    old: &'a HashMap<NodeId, Decl>,
    new: &'a HashMap<NodeId, Decl>,
    changed: &'a ChangedFiles,
) -> HashMap<&'a str, BTreeSet<&'a RepoPath>> {
    let (old_names, new_names) = (by_name(old), by_name(new));
    old_names
        .keys()
        .chain(new_names.keys())
        .filter(|n| old_names.get(*n) != new_names.get(*n))
        .map(|n| {
            let files = old_names
                .get(n)
                .into_iter()
                .chain(new_names.get(n))
                .flatten()
                .filter_map(|(id, _)| old.get(*id).or_else(|| new.get(*id)))
                .map(|d| &d.file)
                .collect();
            (*n, files)
        })
        .chain(
            changed
                .added
                .iter()
                .chain(&changed.deleted)
                .map(|p| (p.as_str(), BTreeSet::from([p]))),
        )
        .collect()
}

/// Where files sit relative to each other, for the builtin-name exception.
struct Closeness<'a> {
    index: SymbolIndex,
    file_nodes: HashMap<&'a RepoPath, &'a Node>,
    probe: &'a GraphDelta,
}

impl<'a> Closeness<'a> {
    fn new(
        nodes: &'a [Node],
        edges: &'a [Edge],
        known: impl Iterator<Item = &'a RepoPath>,
        probe: &'a GraphDelta,
    ) -> Self {
        let file_nodes = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File)
            .filter_map(|n| n.file().map(|f| (f, n)))
            .collect();
        Self {
            index: relations(
                nodes,
                edges,
                known.chain(probe.nodes.iter().filter_map(Node::file)),
            ),
            file_nodes,
            probe,
        }
    }

    /// True when `to` is in `from`'s package or one it imports.
    fn close(&self, from: &RepoPath, to: &RepoPath) -> bool {
        self.file_nodes
            .get(to)
            .copied()
            .or_else(|| self.probe.nodes.iter().find(|n| n.file() == Some(to)))
            .is_some_and(|c| self.index.tier(from, c) != Tier::SameRepo)
    }

    /// The first referenced name whose change can alter `file`'s resolution: any
    /// changed name, except a builtin one changed in an unrelated package.
    fn affected_name<'n>(
        &self,
        file: &RepoPath,
        referenced: &'n [String],
        changed_names: &HashMap<&str, BTreeSet<&RepoPath>>,
    ) -> Option<&'n str> {
        let language = self.file_nodes.get(file).and_then(|n| n.language.as_ref());
        referenced.iter().map(String::as_str).find(|name| {
            changed_names.get(name).is_some_and(|where_| {
                !is_builtin_name(language, name) || where_.iter().any(|w| self.close(file, w))
            })
        })
    }
}

/// The node identifiers that exist after the update: what survives, what the probe
/// declares, and what the unchanged relinked files recreate.
fn final_node_ids<'a>(
    nodes: &'a [Node],
    probe: &'a GraphDelta,
    touched: &BTreeSet<RepoPath>,
    relink: &BTreeMap<RepoPath, RelinkReason>,
    dropped: &BTreeSet<RepoPath>,
) -> HashSet<&'a NodeId> {
    nodes
        .iter()
        .filter(|n| {
            n.file().is_none_or(|f| {
                !dropped.contains(f) || (relink.contains_key(f) && !touched.contains(f))
            })
        })
        .map(|n| &n.id)
        .chain(probe.nodes.iter().map(|n| &n.id))
        .collect()
}

/// The nodes and edges that stand: nothing from a dropped file, no dropped edge.
fn surviving(
    nodes: &[Node],
    edges: &[Edge],
    drop_files: &BTreeSet<RepoPath>,
    drop_edges: &[EdgeId],
) -> Seed {
    let dropped_edges: HashSet<&EdgeId> = drop_edges.iter().collect();
    Seed {
        nodes: nodes
            .iter()
            .filter(|n| n.file().is_none_or(|f| !drop_files.contains(f)))
            .cloned()
            .collect(),
        edges: edges
            .iter()
            .filter(|e| !dropped_edges.contains(&e.id))
            .filter(|e| evidence_file(e).is_none_or(|f| !drop_files.contains(f)))
            .cloned()
            .collect(),
    }
}

fn evidence_file(edge: &Edge) -> Option<&RepoPath> {
    edge.provenance.evidence.first().map(|ev| &ev.location.file)
}

/// The package, import and dependency relations of a stored graph, for ranking.
///
/// `paths` are placed by package directory when no `CONTAINS` edge places them, which
/// is the case for files the update adds.
fn relations<'a>(
    nodes: &[Node],
    edges: &[Edge],
    paths: impl IntoIterator<Item = &'a RepoPath>,
) -> SymbolIndex {
    let by_id: HashMap<&NodeId, &Node> = nodes.iter().map(|n| (&n.id, n)).collect();
    let mut index = SymbolIndex::default();
    for e in edges {
        let (Some(s), Some(d)) = (by_id.get(&e.src), by_id.get(&e.dst)) else {
            continue;
        };
        match (&e.kind, &s.kind, &d.kind) {
            (EdgeKind::Contains, NodeKind::Package, NodeKind::File) => {
                if let Some(file) = d.file() {
                    index.package_of_file.insert(file.clone(), s.id.to_string());
                }
            }
            (EdgeKind::Imports, NodeKind::File, kind) => {
                if let Some(file) = s.file() {
                    index
                        .imported_by_file
                        .entry(file.clone())
                        .or_default()
                        .insert(d.id.to_string());
                    if *kind == NodeKind::Package {
                        index
                            .imports_of_file
                            .entry(file.clone())
                            .or_default()
                            .insert(d.id.to_string());
                    }
                }
            }
            (EdgeKind::DependsOn, NodeKind::Package, NodeKind::Package) => {
                index
                    .deps_of_package
                    .entry(s.id.to_string())
                    .or_default()
                    .insert(d.id.to_string());
            }
            _ => {}
        }
    }
    let mut dirs: Vec<(&str, &Node)> = nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Package)
        .filter_map(|n| n.attributes.get("dir").map(|d| (d.as_str(), n)))
        .collect();
    dirs.sort_by_key(|(dir, _)| std::cmp::Reverse(dir.len()));
    for path in paths {
        if index.package_of_file.contains_key(path) {
            continue;
        }
        if let Some((_, pkg)) = dirs.iter().find(|(dir, _)| path.starts_with_dir(dir)) {
            index
                .package_of_file
                .insert(path.clone(), pkg.id.to_string());
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use surveyor_extract::{
        Extractors, ManifestExtractor, SqlDialect, SqlExtractor, TreeSitterExtractor, extract_units,
    };
    use surveyor_model::{CommitId, DerivationKind, FactSet, RepoId, SourceUnit};

    use super::*;
    use crate::input::{FileEntry, LinkInput};
    use crate::{link, relink};

    fn unit(path: &str, text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("demo"),
            RepoPath::new(path),
            CommitId::new("c0"),
            text.into(),
        )
    }

    fn extract(units: &[SourceUnit]) -> (Vec<FileEntry>, Vec<FactSet>) {
        let extractors = Extractors::new()
            .with(TreeSitterExtractor::new())
            .with(SqlExtractor::new(SqlDialect::PostgreSql, "public"))
            .with(ManifestExtractor::new());
        let extraction = extract_units(&extractors, units, SqlDialect::PostgreSql);
        assert!(extraction.errors.is_empty(), "{:?}", extraction.errors);
        let files = extraction
            .units
            .iter()
            .map(|u| FileEntry {
                path: u.path.clone(),
                language: u.language.clone(),
                content_hash: u.content_hash,
                bytes: u.bytes,
                lines: u.lines,
            })
            .collect();
        (files, extraction.fact_sets)
    }

    fn input(units: &[SourceUnit], commit: &str) -> LinkInput {
        let (files, facts) = extract(units);
        LinkInput::new(RepoId::new("demo"), CommitId::new(commit))
            .with_files(files)
            .with_fact_sets(facts)
    }

    fn names_of(facts: &[FactSet]) -> HashMap<RepoPath, Vec<String>> {
        facts.iter().fold(HashMap::new(), |mut acc, set| {
            let entry: &mut Vec<String> = acc.entry(set.path.clone()).or_default();
            entry.extend(set.references().map(|r| r.name.clone()));
            entry.sort();
            entry.dedup();
            acc
        })
    }

    /// The comparable shape of a graph: node identities and edge triples.
    fn shape(delta: &GraphDelta) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
        let nodes = delta
            .nodes
            .iter()
            .map(|n| format!("{} {} {}", n.id, n.kind, n.content_hash.short()))
            .collect();
        let edges = delta
            .edges
            .iter()
            .map(|e| {
                format!(
                    "{} -{}-> {} {} {:.2} {}",
                    e.src,
                    e.kind,
                    e.dst,
                    e.provenance.derivation.kind(),
                    e.provenance.confidence.get(),
                    e.provenance.evidence[0].citation()
                )
            })
            .collect();
        let unresolved = delta
            .unresolved
            .iter()
            .map(|u| format!("{} {} {}", u.name, u.site.citation(), u.reason))
            .collect();
        (nodes, edges, unresolved)
    }

    fn base() -> Vec<SourceUnit> {
        vec![
            unit(
                "Cargo.toml",
                "[package]\nname = \"outbox\"\n\n[dependencies]\nsqlx = \"0.8\"\n",
            ),
            unit(
                "src/lib.rs",
                "pub mod relay;\npub mod store;\npub mod util;\n",
            ),
            unit(
                "src/relay.rs",
                "use crate::store::Store;\npub struct Relay;\nimpl Relay {\n    pub fn drain(&self) -> usize { let s = Store; s.pending(); helper(); 1 }\n}\nfn helper() {}\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn drains() { Relay.drain(); }\n}\n",
            ),
            unit(
                "src/store.rs",
                "pub struct Store;\nimpl Store {\n    pub fn pending(&self) -> Vec<String> { Vec::new() }\n}\n",
            ),
            unit(
                "src/util.rs",
                "pub fn count(xs: &[u8]) -> usize { xs.len() }\n",
            ),
        ]
    }

    /// Indexes `before`, applies the change to `after` incrementally, and checks the
    /// result against a fresh link of `after`.
    fn check(before: &[SourceUnit], after: &[SourceUnit]) -> UpdatePlan {
        let full_before = link(&input(before, "c0"));
        let (_, before_facts) = extract(before);
        let names = names_of(&before_facts);

        let before_paths: HashMap<&RepoPath, ContentHash> =
            before.iter().map(|u| (&u.path, u.content_hash)).collect();
        let after_paths: HashMap<&RepoPath, ContentHash> =
            after.iter().map(|u| (&u.path, u.content_hash)).collect();
        let changed = ChangedFiles {
            modified: after
                .iter()
                .filter(|u| {
                    before_paths
                        .get(&u.path)
                        .is_some_and(|h| *h != u.content_hash)
                })
                .map(|u| u.path.clone())
                .collect(),
            added: after
                .iter()
                .filter(|u| !before_paths.contains_key(&u.path))
                .map(|u| u.path.clone())
                .collect(),
            deleted: before
                .iter()
                .filter(|u| !after_paths.contains_key(&u.path))
                .map(|u| u.path.clone())
                .collect(),
        };
        let changed_units: Vec<SourceUnit> = after
            .iter()
            .filter(|u| changed.to_extract().any(|p| *p == u.path))
            .cloned()
            .collect();
        let probe = link(&input(&changed_units, "c1"));
        let plan = plan_update(
            &full_before.nodes,
            &full_before.edges,
            &names,
            &changed,
            &probe,
        );
        let relink_units: Vec<SourceUnit> = after
            .iter()
            .filter(|u| plan.relink.contains_key(&u.path))
            .cloned()
            .collect();
        let delta = relink(&input(&relink_units, "c1"), plan.seed.clone());

        // Assemble what the store would hold afterwards.
        let replaced: HashSet<&NodeId> = delta.nodes.iter().map(|n| &n.id).collect();
        let dropped_edges: HashSet<&EdgeId> = plan.drop_edges.iter().collect();
        let combined = GraphDelta {
            nodes: full_before
                .nodes
                .iter()
                .filter(|n| n.file().is_none_or(|f| !plan.drop_files.contains(f)))
                .filter(|n| !replaced.contains(&n.id))
                .cloned()
                .chain(delta.nodes.iter().cloned())
                .collect(),
            edges: full_before
                .edges
                .iter()
                .filter(|e| !dropped_edges.contains(&e.id))
                .filter(|e| {
                    !plan
                        .drop_files
                        .contains(&e.provenance.evidence[0].location.file)
                })
                .cloned()
                .chain(delta.edges.iter().cloned())
                .collect(),
            unresolved: full_before
                .unresolved
                .iter()
                .filter(|u| !plan.drop_files.contains(&u.site.file))
                .cloned()
                .chain(delta.unresolved.iter().cloned())
                .collect(),
        };
        let fresh = link(&input(after, "c1"));
        let (want_n, want_e, want_u) = shape(&fresh);
        let (got_n, got_e, got_u) = shape(&combined);
        assert_eq!(got_n, want_n, "nodes differ; plan {:?}", plan.relink);
        assert_eq!(got_e, want_e, "edges differ; plan {:?}", plan.relink);
        assert_eq!(got_u, want_u, "unresolved differ; plan {:?}", plan.relink);
        assert_eq!(combined.nodes.len(), fresh.nodes.len());
        assert_eq!(combined.edges.len(), fresh.edges.len());
        plan
    }

    #[test]
    fn body_edit_relinks_the_callers_only() {
        let mut after = base();
        after[3] = unit(
            "src/store.rs",
            "pub struct Store;\nimpl Store {\n    pub fn pending(&self) -> Vec<String> { vec![String::new()] }\n}\n",
        );
        let plan = check(&base(), &after);
        assert_eq!(
            plan.relink.get(&RepoPath::new("src/store.rs")),
            Some(&RelinkReason::Modified)
        );
        assert!(matches!(
            plan.relink.get(&RepoPath::new("src/relay.rs")),
            Some(RelinkReason::DeclarationChanged(id)) if id.as_str().ends_with("Store::pending")
                || id.as_str().ends_with("::Store")
        ));
        assert!(!plan.relink.contains_key(&RepoPath::new("src/util.rs")));
    }

    #[test]
    fn new_declaration_relinks_files_that_name_it() {
        let mut after = base();
        // `helper` gains a second declaration elsewhere: the call in relay.rs becomes
        // ambiguous between two candidates and must be relinked.
        after[4] = unit(
            "src/util.rs",
            "pub fn count(xs: &[u8]) -> usize { xs.len() }\npub fn helper() {}\n",
        );
        let plan = check(&base(), &after);
        assert!(matches!(
            plan.relink.get(&RepoPath::new("src/relay.rs")),
            Some(RelinkReason::NameChanged(n)) if n == "helper"
        ));
    }

    #[test]
    fn deleted_file_drops_its_nodes_and_relinks_dependents() {
        let mut after = base();
        after.remove(3);
        after[1] = unit("src/lib.rs", "pub mod relay;\npub mod util;\n");
        let plan = check(&base(), &after);
        assert!(plan.drop_files.contains(&RepoPath::new("src/store.rs")));
        assert!(plan.relink.contains_key(&RepoPath::new("src/relay.rs")));
        assert!(
            plan.seed
                .nodes
                .iter()
                .all(|n| n.file() != Some(&RepoPath::new("src/store.rs")))
        );
    }

    #[test]
    fn added_file_links_into_the_existing_graph() {
        let mut after = base();
        after.push(unit(
            "src/extra.rs",
            "use crate::util::count;\npub fn twice() -> usize { count(&[1]) * 2 }\n",
        ));
        let plan = check(&base(), &after);
        assert_eq!(
            plan.relink.get(&RepoPath::new("src/extra.rs")),
            Some(&RelinkReason::Added)
        );
        assert_eq!(plan.relink.len(), 1, "{:?}", plan.relink);
    }

    #[test]
    fn unchanged_tree_changes_only_the_root_commit() {
        let plan = check(&base(), &base());
        assert!(plan.relink.is_empty());
        let delta = relink(
            &LinkInput::new(RepoId::new("demo"), CommitId::new("c9")),
            plan.seed,
        );
        assert_eq!(delta.nodes.len(), 1);
        assert_eq!(
            delta.nodes[0].attributes.get("commit").map(String::as_str),
            Some("c9")
        );
        assert!(delta.edges.is_empty());
        assert_eq!(
            delta.nodes[0].kind,
            NodeKind::Repository,
            "{:?}",
            DerivationKind::Syntactic
        );
    }
}
