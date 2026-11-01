//! Package manifests: `Cargo.toml`, `package.json` and `pyproject.toml`.
//!
//! Each manifest declares one `Package` (or a `Workspace`) and a `DEPENDS_ON`
//! relation per dependency. Dependencies are named by registry coordinate so a shared
//! library links its consumers across repositories. Evidence points at the line that
//! names the dependency.
//!
//! A Cargo dependency also carries what its entry says, as attributes of the relation:
//! `section` (`normal`, `dev` or `build`), `key` (the name the entry uses, which is what
//! `dep:key` and `key/feature` refer to), `optional`, `default_features`, the requested
//! `features` (comma-separated, when any) and `workspace = "true"` when the entry
//! inherits a `[workspace.dependencies]` definition this extractor does not see. The
//! package's `[features]` table becomes one `feature.<name>` attribute per feature,
//! holding the comma-separated list it enables.

use std::collections::BTreeMap;

use surveyor_model::{
    ByteSpan, DeclarationFact, DeclaredRelation, DeclaredSource, EdgeKind, ExtractorId, Fact,
    FactSet, NodeKind, RuleId, SourceUnit,
};

use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "manifest";

/// Reads package manifests into package declarations and dependency relations.
#[derive(Debug, Clone, Default)]
pub struct ManifestExtractor;

impl ManifestExtractor {
    /// Creates the extractor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extracts facts from a manifest unit.
    ///
    /// # Errors
    /// Returns a parse error when the manifest is not valid TOML or JSON.
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("manifest is not UTF-8"))?;
        let mut set = FactSet::empty(ExtractorId::new(EXTRACTOR_ID), unit);
        match unit.path.file_name() {
            Some("Cargo.toml") => cargo_facts(text, unit, &mut set)?,
            Some("package.json") => npm_facts(text, unit, &mut set)?,
            Some("pyproject.toml") => pyproject_facts(text, unit, &mut set)?,
            _ => {}
        }
        Ok(set)
    }
}

impl Extractor for ManifestExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_ID)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        matches!(
            unit.path.file_name(),
            Some("Cargo.toml" | "package.json" | "pyproject.toml")
        )
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

/// Registry a package coordinate belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Registry {
    Cargo,
    Npm,
    PyPi,
}

impl Registry {
    fn coordinate(self, name: &str) -> String {
        match self {
            Self::Cargo => format!("cargo/{name}"),
            Self::Npm => format!("npm/{name}"),
            Self::PyPi => format!("pypi/{}", name.to_lowercase().replace('_', "-")),
        }
    }
}

fn package_decl(
    unit: &SourceUnit,
    kind: NodeKind,
    coordinate: &str,
    display: &str,
    bytes: ByteSpan,
    attrs: BTreeMap<String, String>,
) -> Fact {
    Fact::Declaration(DeclarationFact {
        kind,
        name: display.to_owned(),
        scope: Vec::new(),
        span: unit.text_span(bytes),
        name_span: bytes,
        signature: Some(coordinate.to_owned()),
        doc: None,
        rule: RuleId::new("manifest.package"),
        attributes: attrs,
        hash: unit.hash_of(bytes),
    })
}

fn depends_on(
    unit: &SourceUnit,
    src: &str,
    dst: &str,
    bytes: ByteSpan,
    attributes: BTreeMap<String, String>,
) -> Fact {
    Fact::Relation(DeclaredRelation {
        kind: EdgeKind::DependsOn,
        src_kind: NodeKind::Package,
        src_name: src.to_owned(),
        dst_kind: NodeKind::Package,
        dst_name: dst.to_owned(),
        source: DeclaredSource::Manifest,
        span: unit.text_span(bytes),
        hash: unit.hash_of(bytes),
        attributes,
    })
}

/// Which Cargo manifest section names a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CargoSection {
    Normal,
    Dev,
    Build,
}

impl CargoSection {
    const ALL: [Self; 3] = [Self::Normal, Self::Dev, Self::Build];

    /// The table name in the manifest.
    fn table(self) -> &'static str {
        match self {
            Self::Normal => "dependencies",
            Self::Dev => "dev-dependencies",
            Self::Build => "build-dependencies",
        }
    }

    /// The value of the relation's `section` attribute.
    fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Dev => "dev",
            Self::Build => "build",
        }
    }
}

/// Comma-separated string values of a TOML array; empty when `value` is not one.
fn joined_strings(value: Option<&toml::Value>) -> String {
    value
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default()
}

/// What one Cargo dependency entry states, as relation attributes.
///
/// A plain version string (`tempfile = "3"`) has no table keys, so it is a required,
/// default-features dependency with no requested features.
fn cargo_dependency_attributes(
    section: CargoSection,
    key: &str,
    value: &toml::Value,
) -> BTreeMap<String, String> {
    let flag = |name: &str| value.get(name).and_then(toml::Value::as_bool);
    let mut attrs = BTreeMap::new();
    attrs.insert("section".to_owned(), section.as_str().to_owned());
    attrs.insert("key".to_owned(), key.to_owned());
    attrs.insert(
        "optional".to_owned(),
        flag("optional").unwrap_or(false).to_string(),
    );
    attrs.insert(
        "default_features".to_owned(),
        flag("default-features").unwrap_or(true).to_string(),
    );
    if flag("workspace") == Some(true) {
        attrs.insert("workspace".to_owned(), "true".to_owned());
    }
    let features = joined_strings(value.get("features"));
    if !features.is_empty() {
        attrs.insert("features".to_owned(), features);
    }
    attrs
}

/// The package's `[features]` table as `feature.<name>` attributes.
fn cargo_feature_attributes(doc: &toml::Table) -> impl Iterator<Item = (String, String)> + '_ {
    doc.get("features")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|features| features.iter())
        .map(|(name, enables)| (format!("feature.{name}"), joined_strings(Some(enables))))
}

/// Span of the first line matching `predicate`, searched after `after` (a byte offset).
fn line_span_where(text: &str, after: usize, predicate: impl Fn(&str) -> bool) -> Option<ByteSpan> {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if start < after {
            continue;
        }
        if predicate(line.trim_end_matches('\n')) {
            return Some(ByteSpan::new(
                start as u64,
                (start + line.trim_end().len()) as u64,
            ));
        }
    }
    None
}

fn section_offset(text: &str, header: &str) -> Option<usize> {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        if line.trim() == header {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn key_line(text: &str, after: usize, key: &str) -> Option<ByteSpan> {
    line_span_where(text, after, |l| {
        let l = l.trim_start();
        l.strip_prefix(key)
            .or_else(|| l.strip_prefix(&format!("\"{key}\"")))
            .or_else(|| l.strip_prefix(&format!("'{key}'")))
            .is_some_and(|rest| rest.trim_start().starts_with(['=', ':']))
    })
}

fn whole(unit: &SourceUnit) -> ByteSpan {
    ByteSpan::new(0, unit.content.len() as u64)
}

fn cargo_facts(text: &str, unit: &SourceUnit, set: &mut FactSet) -> Result<(), ExtractError> {
    let doc: toml::Table = toml::from_str(text).map_err(|e| ExtractError::parse(e.to_string()))?;
    let dir = unit.path.parent().to_owned();
    let mut attrs = BTreeMap::new();
    attrs.insert("manifest".to_owned(), unit.path.to_string());
    attrs.insert("dir".to_owned(), dir.clone());

    let package_name = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned);

    let src = match &package_name {
        Some(name) => {
            let coord = Registry::Cargo.coordinate(name);
            let span = section_offset(text, "[package]")
                .and_then(|o| key_line(text, o, "name"))
                .unwrap_or_else(|| whole(unit));
            if let Some(v) = doc
                .get("package")
                .and_then(|p| p.get("version"))
                .and_then(toml::Value::as_str)
            {
                attrs.insert("version".to_owned(), v.to_owned());
            }
            attrs.extend(cargo_feature_attributes(&doc));
            set.facts.push(package_decl(
                unit,
                NodeKind::Package,
                &coord,
                name,
                span,
                attrs,
            ));
            Some(coord)
        }
        None if doc.contains_key("workspace") => {
            let name = if dir.is_empty() {
                "workspace".to_owned()
            } else {
                dir.rsplit('/').next().unwrap_or("workspace").to_owned()
            };
            let coord = format!(
                "cargo-workspace/{}",
                if dir.is_empty() { "root" } else { dir.as_str() }
            );
            let span = section_offset(text, "[workspace]").map_or_else(
                || whole(unit),
                |o| ByteSpan::new(o as u64, (o + "[workspace]".len()) as u64),
            );
            set.facts.push(package_decl(
                unit,
                NodeKind::Workspace,
                &coord,
                &name,
                span,
                attrs,
            ));
            None
        }
        None => None,
    };

    let Some(src) = src else { return Ok(()) };
    let src = src.as_str();
    set.facts
        .extend(CargoSection::ALL.iter().flat_map(|&section| {
            let after = section_offset(text, &format!("[{}]", section.table())).unwrap_or(0);
            doc.get(section.table())
                .and_then(toml::Value::as_table)
                .into_iter()
                .flat_map(|deps| deps.iter())
                .map(move |(key, value)| {
                    // `foo = { package = "bar" }` renames the crate; the registry name is `bar`.
                    let crate_name = value
                        .get("package")
                        .and_then(toml::Value::as_str)
                        .unwrap_or(key.as_str());
                    let span = key_line(text, after, key).unwrap_or_else(|| whole(unit));
                    depends_on(
                        unit,
                        src,
                        &Registry::Cargo.coordinate(crate_name),
                        span,
                        cargo_dependency_attributes(section, key, value),
                    )
                })
                .collect::<Vec<_>>()
        }));
    Ok(())
}

fn npm_facts(text: &str, unit: &SourceUnit, set: &mut FactSet) -> Result<(), ExtractError> {
    let doc: serde_json::Value =
        serde_json::from_str(text).map_err(|e| ExtractError::parse(e.to_string()))?;
    let Some(name) = doc.get("name").and_then(serde_json::Value::as_str) else {
        return Ok(());
    };
    let coord = Registry::Npm.coordinate(name);
    let span = key_line(text, 0, "name").unwrap_or_else(|| whole(unit));
    let mut attrs = BTreeMap::new();
    attrs.insert("manifest".to_owned(), unit.path.to_string());
    attrs.insert("dir".to_owned(), unit.path.parent().to_owned());
    if let Some(v) = doc.get("version").and_then(serde_json::Value::as_str) {
        attrs.insert("version".to_owned(), v.to_owned());
    }
    set.facts.push(package_decl(
        unit,
        NodeKind::Package,
        &coord,
        name,
        span,
        attrs,
    ));
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        let Some(deps) = doc.get(section).and_then(serde_json::Value::as_object) else {
            continue;
        };
        let after = key_line(text, 0, section).map_or(0, |s| s.start as usize);
        for key in deps.keys() {
            let span = key_line(text, after, key).unwrap_or_else(|| whole(unit));
            set.facts.push(depends_on(
                unit,
                &coord,
                &Registry::Npm.coordinate(key),
                span,
                BTreeMap::new(),
            ));
        }
    }
    Ok(())
}

fn pyproject_facts(text: &str, unit: &SourceUnit, set: &mut FactSet) -> Result<(), ExtractError> {
    let doc: toml::Table = toml::from_str(text).map_err(|e| ExtractError::parse(e.to_string()))?;
    let project = doc
        .get("project")
        .or_else(|| doc.get("tool").and_then(|t| t.get("poetry")));
    let Some(name) = project
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
    else {
        return Ok(());
    };
    let coord = Registry::PyPi.coordinate(name);
    let span = section_offset(text, "[project]")
        .or_else(|| section_offset(text, "[tool.poetry]"))
        .and_then(|o| key_line(text, o, "name"))
        .unwrap_or_else(|| whole(unit));
    let mut attrs = BTreeMap::new();
    attrs.insert("manifest".to_owned(), unit.path.to_string());
    attrs.insert("dir".to_owned(), unit.path.parent().to_owned());
    set.facts.push(package_decl(
        unit,
        NodeKind::Package,
        &coord,
        name,
        span,
        attrs,
    ));
    let after = section_offset(text, "[project]").unwrap_or(0);
    if let Some(deps) = project
        .and_then(|p| p.get("dependencies"))
        .and_then(toml::Value::as_array)
    {
        for dep in deps.iter().filter_map(toml::Value::as_str) {
            let dep_name = requirement_name(dep);
            if dep_name.is_empty() {
                continue;
            }
            let span =
                line_span_where(text, after, |l| l.contains(dep)).unwrap_or_else(|| whole(unit));
            set.facts.push(depends_on(
                unit,
                &coord,
                &Registry::PyPi.coordinate(&dep_name),
                span,
                BTreeMap::new(),
            ));
        }
    }
    Ok(())
}

/// The distribution name at the front of a PEP 508 requirement string.
fn requirement_name(requirement: &str) -> String {
    requirement
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect()
}

#[cfg(test)]
mod tests {
    use surveyor_model::{CommitId, RepoId, RepoPath};

    use super::*;

    fn unit(path: &str, text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new(path),
            CommitId::new("c"),
            text.into(),
        )
    }

    #[test]
    fn cargo_package_and_dependencies() {
        let text = "[package]\nname = \"outbox\"\nversion = \"0.1.0\"\n\n[dependencies]\ntokio = { version = \"1\" }\nmy-log = { package = \"tracing\", version = \"0.1\" }\n\n[dev-dependencies]\ntempfile = \"3\"\n";
        let u = unit("crates/outbox/Cargo.toml", text);
        let facts = ManifestExtractor::new().extract(&u).expect("parses");
        let pkg = facts.declarations().next().expect("package");
        assert_eq!(pkg.kind, NodeKind::Package);
        assert_eq!(pkg.name, "outbox");
        assert_eq!(pkg.signature.as_deref(), Some("cargo/outbox"));
        assert_eq!(u.slice(pkg.span.bytes), "name = \"outbox\"");
        let rels: Vec<_> = facts.relations().collect();
        assert_eq!(rels.len(), 3);
        assert!(rels.iter().any(|r| r.dst_name == "cargo/tracing"));
        let tokio = rels
            .iter()
            .find(|r| r.dst_name == "cargo/tokio")
            .expect("tokio");
        assert_eq!(u.slice(tokio.span.bytes), "tokio = { version = \"1\" }");
    }

    #[test]
    fn cargo_dependency_and_feature_facts() {
        let text = "[package]\nname = \"server\"\nversion = \"0.1.0\"\n\n[features]\ndefault = [\"sniff\"]\nsniff = [\"dep:sniffer\"]\n\n[dependencies]\ntokio = { workspace = true, features = [\"rt\", \"macros\"] }\nsniffer = { path = \"../sniffer\", optional = true }\nops = { path = \"../ops\", default-features = false }\n\n[dev-dependencies]\ntempfile = \"3\"\n\n[build-dependencies]\ncc = \"1\"\n";
        let u = unit("apps/server/Cargo.toml", text);
        let facts = ManifestExtractor::new().extract(&u).expect("parses");
        let pkg = facts.declarations().next().expect("package");
        assert_eq!(
            pkg.attributes.get("feature.default").map(String::as_str),
            Some("sniff")
        );
        assert_eq!(
            pkg.attributes.get("feature.sniff").map(String::as_str),
            Some("dep:sniffer")
        );

        let attrs = |dst: &str| {
            facts
                .relations()
                .find(|r| r.dst_name == dst)
                .map_or_else(|| panic!("relation to {dst}"), |r| r.attributes.clone())
        };
        let get = |m: &BTreeMap<String, String>, k: &str| m.get(k).cloned();
        let tokio = attrs("cargo/tokio");
        assert_eq!(get(&tokio, "section").as_deref(), Some("normal"));
        assert_eq!(get(&tokio, "key").as_deref(), Some("tokio"));
        assert_eq!(get(&tokio, "optional").as_deref(), Some("false"));
        assert_eq!(get(&tokio, "default_features").as_deref(), Some("true"));
        assert_eq!(get(&tokio, "workspace").as_deref(), Some("true"));
        assert_eq!(get(&tokio, "features").as_deref(), Some("rt,macros"));
        let sniffer = attrs("cargo/sniffer");
        assert_eq!(get(&sniffer, "optional").as_deref(), Some("true"));
        assert_eq!(get(&sniffer, "features"), None);
        assert_eq!(get(&sniffer, "workspace"), None);
        let ops = attrs("cargo/ops");
        assert_eq!(get(&ops, "default_features").as_deref(), Some("false"));
        assert_eq!(
            get(&attrs("cargo/tempfile"), "section").as_deref(),
            Some("dev")
        );
        assert_eq!(get(&attrs("cargo/cc"), "section").as_deref(), Some("build"));
    }

    #[test]
    fn npm_and_pyproject() {
        let u = unit(
            "web/package.json",
            "{\n  \"name\": \"@acme/web\",\n  \"dependencies\": {\n    \"react\": \"^18\"\n  }\n}\n",
        );
        let facts = ManifestExtractor::new().extract(&u).expect("parses");
        assert_eq!(
            facts
                .declarations()
                .next()
                .and_then(|d| d.signature.clone())
                .as_deref(),
            Some("npm/@acme/web")
        );
        let rel = facts.relations().next().expect("dep");
        assert_eq!(rel.dst_name, "npm/react");
        assert_eq!(u.slice(rel.span.bytes).trim(), "\"react\": \"^18\"");

        let u = unit(
            "svc/pyproject.toml",
            "[project]\nname = \"svc_api\"\ndependencies = [\n  \"fastapi>=0.100\",\n  \"SQLAlchemy[asyncio]\",\n]\n",
        );
        let facts = ManifestExtractor::new().extract(&u).expect("parses");
        let rels: Vec<_> = facts.relations().collect();
        assert!(rels.iter().any(|r| r.dst_name == "pypi/fastapi"));
        assert!(rels.iter().any(|r| r.dst_name == "pypi/sqlalchemy"));
        assert!(rels.iter().all(|r| r.src_name == "pypi/svc-api"));
    }
}
