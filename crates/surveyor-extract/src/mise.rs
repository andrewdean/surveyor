//! Mise tasks: the TOML files (`mise.toml`, `.mise.toml`, `.mise/config.toml` and the
//! files a `[task_config] includes` list names, by convention under `.mise/`) and the
//! file tasks under `.mise/tasks/` or `mise-tasks/`, scripts whose `#MISE` header
//! lines carry the description and dependencies.
//!
//! Every task becomes a `Task` declaration named as `mise run` names it
//! (`devdata:load`), carrying its description as documentation and its `run` and
//! `depends` entries as attributes. Each `run` entry is kept as a `Command` literal
//! inside the task (a file task keeps its own path as a `Path` literal), and each
//! `depends` entry as a declared `DEPENDS_ON` relation to the task it names, so the
//! linker can draw the edges from a task to what it runs.

use std::collections::BTreeMap;

use surveyor_model::{
    ByteSpan, ContentHash, DeclarationFact, DeclaredRelation, DeclaredSource, EdgeKind,
    ExtractorId, Fact, FactSet, Language, LiteralFact, LiteralRole, NodeKind, RuleId, SourceUnit,
};

use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "mise";

/// The runner a task coordinate names: `mise/devdata:load`.
pub const RUNNER: &str = "mise";

/// Reads mise task files into task declarations.
#[derive(Debug, Clone, Default)]
pub struct MiseExtractor;

impl MiseExtractor {
    /// Creates the extractor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extracts facts from a mise configuration file or a file task.
    ///
    /// # Errors
    /// Returns a parse error when the file is not UTF-8 or not valid TOML.
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("mise file is not UTF-8"))?;
        let mut set = FactSet::empty(ExtractorId::new(EXTRACTOR_ID), unit);
        if let Some(name) = task_name(unit.path.as_str()) {
            set.facts.extend(file_task_facts(unit, text, &name)?);
            return Ok(set);
        }
        let value: toml::Value =
            toml::from_str(text).map_err(|e| ExtractError::parse(e.to_string()))?;
        let Some(root) = value.as_table() else {
            return Ok(set);
        };
        let headers = table_headers(text);
        for (name, task) in tasks_of(root) {
            let Some(span) = header_span(&headers, &name, text.len()) else {
                continue;
            };
            set.facts.extend(task_facts(unit, &name, task, span));
        }
        Ok(set)
    }
}

impl Extractor for MiseExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_ID)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        (unit.language == Some(Language::Toml) && is_mise_file(unit.path.as_str()))
            || task_name(unit.path.as_str()).is_some()
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

/// True for the files mise reads tasks from: its configuration files by name, and
/// anything under a `.mise/` directory, where include files live by convention.
#[must_use]
pub fn is_mise_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let toml = std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"));
    let by_name = matches!(name, "mise.toml" | ".mise.toml" | "mise.local.toml")
        || (toml && (name.starts_with("mise.") || name.starts_with(".mise.")));
    let under_mise =
        path.split('/').any(|segment| segment == ".mise") || path.starts_with(".config/mise/");
    by_name || under_mise
}

/// Directories whose files mise runs as tasks, named by the path below them.
const TASK_DIRS: &[&str] = &[
    ".mise/tasks",
    "mise-tasks",
    ".mise-tasks",
    ".config/mise/tasks",
];

/// The task name of a file task, or `None` when `path` is not under a task directory:
/// `.mise/tasks/validate/data-coherence` is `validate:data-coherence`, and a script's
/// extension is dropped as mise drops it.
#[must_use]
pub fn task_name(path: &str) -> Option<String> {
    let rest = TASK_DIRS
        .iter()
        .find_map(|dir| path.strip_prefix(dir).and_then(|r| r.strip_prefix('/')))?;
    if rest.is_empty() || rest.split('/').any(|s| s.is_empty() || s.starts_with('.')) {
        return None;
    }
    let (dirs, file) = rest.rsplit_once('/').unwrap_or(("", rest));
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    let name = dirs
        .split('/')
        .filter(|s| !s.is_empty())
        .chain(std::iter::once(stem))
        .collect::<Vec<_>>()
        .join(":");
    Some(name)
}

/// The `#MISE key=value` header lines of a file task, as one TOML document.
///
/// `#MISE description="..."` and `#MISE depends=["a"]` are TOML assignments once the
/// marker is removed; `//MISE` serves scripts whose comments start with slashes.
fn mise_header(text: &str) -> (String, ByteSpan) {
    let mut offset = 0usize;
    let mut toml = String::new();
    let mut span: Option<ByteSpan> = None;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim();
        if let Some(assignment) = trimmed
            .strip_prefix("#MISE ")
            .or_else(|| trimmed.strip_prefix("//MISE "))
        {
            toml.push_str(assignment.trim());
            toml.push('\n');
            let end = start + line.trim_end_matches(['\n', '\r']).len();
            span = Some(match span {
                Some(s) => ByteSpan::new(s.start, end as u64),
                None => ByteSpan::new(start as u64, end as u64),
            });
        } else if !(trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//")) {
            break;
        }
    }
    let first_line = text.find('\n').unwrap_or(text.len());
    (toml, span.unwrap_or(ByteSpan::new(0, first_line as u64)))
}

/// The facts of a file task: the declaration over its header, its own path as what it
/// runs, and its declared dependencies.
fn file_task_facts(unit: &SourceUnit, text: &str, name: &str) -> Result<Vec<Fact>, ExtractError> {
    let (header, span) = mise_header(text);
    let table: toml::Table =
        toml::from_str(&header).map_err(|e| ExtractError::parse(format!("#MISE header: {e}")))?;
    let description = table
        .get("description")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let depends: Vec<String> = strings(&table, "depends")
        .into_iter()
        .chain(strings(&table, "depends_post"))
        .collect();
    let mut attributes = BTreeMap::new();
    attributes.insert("runner".to_owned(), RUNNER.to_owned());
    attributes.insert("file".to_owned(), unit.path.to_string());
    if let Some(d) = &description {
        attributes.insert("description".to_owned(), d.clone());
    }
    if !depends.is_empty() {
        attributes.insert("depends".to_owned(), depends.join(","));
    }
    let declaration = Fact::Declaration(DeclarationFact {
        kind: NodeKind::Task,
        name: name.to_owned(),
        scope: Vec::new(),
        span: unit.text_span(span),
        name_span: span,
        signature: Some(format!("{RUNNER} run {name}")),
        doc: description,
        rule: RuleId::new("mise.file-task"),
        attributes,
        hash: unit.hash_of(span),
    });
    let own_path = Fact::Literal(LiteralFact {
        role: LiteralRole::Path,
        hash: ContentHash::of(unit.path.as_str()),
        value: unit.path.to_string(),
        span: unit.text_span(span),
        scope: vec![name.to_owned()],
        rule: RuleId::new("mise.file-task"),
    });
    let relations = depends.into_iter().map(|dep| {
        Fact::Relation(DeclaredRelation {
            kind: EdgeKind::DependsOn,
            src_kind: NodeKind::Task,
            src_name: format!("{RUNNER}/{name}"),
            dst_kind: NodeKind::Task,
            dst_name: format!("{RUNNER}/{dep}"),
            source: DeclaredSource::Other(RUNNER.to_owned()),
            span: unit.text_span(span),
            hash: unit.hash_of(span),
            attributes: BTreeMap::new(),
        })
    });
    Ok([declaration, own_path]
        .into_iter()
        .chain(relations)
        .collect())
}

/// The tasks a parsed file declares: the `[tasks]` table's entries, plus the top-level
/// tables of an include file, which mise reads as tasks when they have a `run`.
fn tasks_of(root: &toml::Table) -> Vec<(String, &toml::Table)> {
    let nested = root
        .get("tasks")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|t| t.iter())
        .filter_map(|(k, v)| v.as_table().map(|t| (k.clone(), t)));
    let top_level = root
        .iter()
        .filter(|(k, _)| k.as_str() != "tasks")
        .filter_map(|(k, v)| v.as_table().map(|t| (k.clone(), t)))
        .filter(|(_, t)| looks_like_task(t));
    nested.chain(top_level).collect()
}

/// A table is a task when it says what to run or what to run after.
fn looks_like_task(table: &toml::Table) -> bool {
    ["run", "file", "depends", "depends_post", "wait_for"]
        .iter()
        .any(|key| table.contains_key(*key))
}

/// The string values of `key`: one string, or an array of strings.
fn strings(table: &toml::Table, key: &str) -> Vec<String> {
    match table.get(key) {
        Some(toml::Value::String(s)) => vec![s.clone()],
        Some(toml::Value::Array(items)) => items
            .iter()
            .filter_map(toml::Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

/// A table header line: its unquoted name and the byte range of the line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    name: String,
    line: ByteSpan,
}

/// Every `[header]` line with the task name it declares, in file order.
///
/// `[tasks.x]`, `[tasks."a:b"]`, `["a:b"]` and `[plain]` all name a task; the
/// `tasks.` prefix is dropped and quotes are removed.
fn table_headers(text: &str) -> Vec<Header> {
    let mut offset = 0usize;
    let mut out = Vec::new();
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim();
        if let Some(inner) = trimmed
            .strip_prefix('[')
            .filter(|rest| !rest.starts_with('['))
            .and_then(|rest| rest.split_once(']'))
            .map(|(inner, _)| inner.trim())
        {
            let name = inner.strip_prefix("tasks.").unwrap_or(inner);
            let name = name.trim_matches(['"', '\'']).to_owned();
            let end = start + line.trim_end_matches(['\n', '\r']).len();
            out.push(Header {
                name,
                line: ByteSpan::new(start as u64, end as u64),
            });
        }
    }
    out
}

/// The span of `name`'s table: from its header to the next header or the end.
fn header_span(headers: &[Header], name: &str, len: usize) -> Option<(ByteSpan, ByteSpan)> {
    let at = headers.iter().position(|h| h.name == name)?;
    let end = headers
        .get(at + 1)
        .map_or(len as u64, |next| next.line.start);
    Some((ByteSpan::new(headers[at].line.start, end), headers[at].line))
}

/// Byte range of the first line inside `table` that starts with `key`, else the header.
fn key_line(text: &str, table: ByteSpan, header: ByteSpan, key: &str) -> ByteSpan {
    let Some(body) = text.get(table.start as usize..table.end as usize) else {
        return header;
    };
    let mut offset = table.start as usize;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_start();
        if trimmed
            .strip_prefix(key)
            .is_some_and(|rest| rest.trim_start().starts_with('='))
        {
            let end = start + line.trim_end_matches(['\n', '\r']).len();
            return ByteSpan::new(start as u64, end as u64);
        }
    }
    header
}

fn task_facts(
    unit: &SourceUnit,
    name: &str,
    task: &toml::Table,
    (table, header): (ByteSpan, ByteSpan),
) -> Vec<Fact> {
    let text = unit.text().unwrap_or_default();
    let description = task
        .get("description")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let run = strings(task, "run");
    let depends: Vec<String> = strings(task, "depends")
        .into_iter()
        .chain(strings(task, "depends_post"))
        .collect();
    let mut attributes = BTreeMap::new();
    attributes.insert("runner".to_owned(), RUNNER.to_owned());
    if let Some(d) = &description {
        attributes.insert("description".to_owned(), d.clone());
    }
    if !run.is_empty() {
        attributes.insert("run".to_owned(), run.join("\n"));
    }
    if !depends.is_empty() {
        attributes.insert("depends".to_owned(), depends.join(","));
    }
    let declaration = Fact::Declaration(DeclarationFact {
        kind: NodeKind::Task,
        name: name.to_owned(),
        scope: Vec::new(),
        span: unit.text_span(table),
        name_span: header,
        signature: Some(format!("{RUNNER} run {name}")),
        doc: description,
        rule: RuleId::new("mise.task"),
        attributes,
        hash: unit.hash_of(table),
    });
    let run_line = key_line(text, table, header, "run");
    let commands = run.into_iter().map(move |command| {
        Fact::Literal(LiteralFact {
            role: LiteralRole::Command,
            hash: ContentHash::of(&command),
            value: command,
            span: unit.text_span(run_line),
            scope: vec![name.to_owned()],
            rule: RuleId::new("mise.run"),
        })
    });
    let depends_line = key_line(text, table, header, "depends");
    let relations = depends.into_iter().map(move |dep| {
        Fact::Relation(DeclaredRelation {
            kind: EdgeKind::DependsOn,
            src_kind: NodeKind::Task,
            src_name: format!("{RUNNER}/{name}"),
            dst_kind: NodeKind::Task,
            dst_name: format!("{RUNNER}/{dep}"),
            source: DeclaredSource::Other(RUNNER.to_owned()),
            span: unit.text_span(depends_line),
            hash: unit.hash_of(depends_line),
            attributes: BTreeMap::new(),
        })
    });
    std::iter::once(declaration)
        .chain(commands)
        .chain(relations)
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
    fn recognizes_mise_files_by_name_and_directory() {
        assert!(is_mise_file("mise.toml"));
        assert!(is_mise_file(".mise.toml"));
        assert!(is_mise_file("mise.python.toml"));
        assert!(is_mise_file(".mise/config.toml"));
        assert!(is_mise_file(".mise/toml/devdata.toml"));
        assert!(!is_mise_file("Cargo.toml"));
        assert!(!is_mise_file("config/mise-like.toml"));
    }

    #[test]
    fn file_tasks_are_named_by_their_path_and_described_by_their_header() {
        assert_eq!(
            task_name(".mise/tasks/validate/data-coherence").as_deref(),
            Some("validate:data-coherence")
        );
        assert_eq!(task_name("mise-tasks/build.sh").as_deref(), Some("build"));
        assert_eq!(task_name(".mise/tasks/.hidden"), None);
        assert_eq!(task_name(".mise/toml/dev.toml"), None);
        assert_eq!(task_name("src/tasks/x"), None);

        let script = "#!/usr/bin/env bash\n#MISE description=\"Report cross-source coherence of the workspace's development data.\"\n#MISE depends=[\"workspace:init\"]\n#\n# The checks live elsewhere.\nset -euo pipefail\n";
        let u = unit(".mise/tasks/validate/data-coherence", script);
        let ex = MiseExtractor::new();
        assert!(ex.accepts(&u));
        let facts = ex.extract(&u).expect("parses");
        let task = facts.declarations().next().expect("task");
        assert_eq!(task.name, "validate:data-coherence");
        assert_eq!(
            task.doc.as_deref(),
            Some("Report cross-source coherence of the workspace's development data.")
        );
        assert_eq!(task.span.lines, surveyor_model::LineSpan::new(2, 3));
        assert!(u.slice(task.span.bytes).starts_with("#MISE description"));
        assert!(
            u.slice(task.span.bytes)
                .ends_with("depends=[\"workspace:init\"]")
        );
        let own = facts.literals().next().expect("own path");
        assert_eq!(own.role, LiteralRole::Path);
        assert_eq!(own.value, ".mise/tasks/validate/data-coherence");
        assert_eq!(own.scope, vec!["validate:data-coherence".to_owned()]);
        let dep = facts.relations().next().expect("depends");
        assert_eq!(dep.dst_name, "mise/workspace:init");

        let bare = unit("mise-tasks/lint", "#!/bin/sh\necho lint\n");
        let facts = ex.extract(&bare).expect("parses without a header");
        let task = facts.declarations().next().expect("task");
        assert_eq!(task.name, "lint");
        assert_eq!(task.doc, None);
        assert_eq!(u.slice(ByteSpan::new(0, 2)), "#!");
    }

    #[test]
    fn nested_tasks_and_include_file_tasks_become_declarations() {
        let config = "[tools]\nrust = \"1.95\"\n\n[tasks.fmt]\ndescription = \"Format all sources\"\nrun = \"cargo fmt --all\"\n\n[tasks.\"ci\"]\ndepends = [\"fmt\", \"test\"]\n";
        let u = unit("mise.toml", config);
        let ex = MiseExtractor::new();
        assert!(ex.accepts(&u));
        let facts = ex.extract(&u).expect("parses");
        let decls: Vec<_> = facts.declarations().collect();
        assert_eq!(decls.len(), 2, "{decls:?}");
        let fmt = decls.iter().find(|d| d.name == "fmt").expect("fmt");
        assert_eq!(fmt.kind, NodeKind::Task);
        assert_eq!(fmt.doc.as_deref(), Some("Format all sources"));
        assert_eq!(
            fmt.attributes.get("run").map(String::as_str),
            Some("cargo fmt --all")
        );
        assert_eq!(u.slice(fmt.name_span), "[tasks.fmt]");
        assert!(u.slice(fmt.span.bytes).starts_with("[tasks.fmt]"));
        assert!(u.slice(fmt.span.bytes).ends_with("cargo fmt --all\"\n\n"));
        let command = facts.literals().next().expect("command literal");
        assert_eq!(command.role, LiteralRole::Command);
        assert_eq!(command.value, "cargo fmt --all");
        assert_eq!(command.scope, vec!["fmt".to_owned()]);
        assert_eq!(u.slice(command.span.bytes), "run = \"cargo fmt --all\"");
        let deps: Vec<_> = facts.relations().collect();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].src_name, "mise/ci");
        assert_eq!(deps[0].dst_name, "mise/fmt");
        assert_eq!(deps[0].kind, EdgeKind::DependsOn);
        assert_eq!(u.slice(deps[0].span.bytes), "depends = [\"fmt\", \"test\"]");

        let include = "# Development data.\n\n[\"devdata:load\"]\ndescription = \"Run every loader in derived order.\"\nrun = \".mise/scripts/tasks/devdata/loaders.rhai run\"\n\n[\"devdata:test\"]\nrun = [\n  \"cargo test -p cw-devdata\",\n  \"cwsh test loaders_test.rhai\",\n]\n\n[env]\nX = \"1\"\n";
        let u = unit(".mise/toml/devdata.toml", include);
        let facts = ex.extract(&u).expect("parses");
        let names: Vec<_> = facts.declarations().map(|d| d.name.clone()).collect();
        assert_eq!(
            names,
            vec!["devdata:load", "devdata:test"],
            "env is not a task"
        );
        let commands: Vec<_> = facts.literals().map(|l| l.value.clone()).collect();
        assert_eq!(
            commands,
            vec![
                ".mise/scripts/tasks/devdata/loaders.rhai run",
                "cargo test -p cw-devdata",
                "cwsh test loaders_test.rhai",
            ]
        );
        let load = facts.declarations().next().expect("load");
        assert_eq!(load.signature.as_deref(), Some("mise run devdata:load"));
        assert_eq!(load.span.lines.start, 3);
    }
}
