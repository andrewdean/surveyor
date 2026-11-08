//! Session ledgers: what Surveyor told an agent during one session.
//!
//! A session is a file of JSON lines, `<store>/sessions/<id>.jsonl`, one [`Record`]
//! per answer Surveyor gave: when, which command or tool, its arguments, and the
//! nodes the answer named with the citation each carried. The record is a pure
//! function of the answer the command already renders; nothing is planned after the
//! fact. A pull request body built from the ledger therefore cites what the agent
//! read, and a reviewer can check the claim against the file.
//!
//! The CLI appends a record for every query command and the MCP server for every
//! tool call when `SURVEYOR_SESSION` (or `--session`) names a session; without one
//! nothing is written. [`summarize`] folds a ledger into the consulted set, and
//! [`consulted_in`] reads node references out of any JSON answer, which is how a
//! hook records a tool response it did not produce.

// Rust guideline compliant 2026-02-21

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use surveyor_context::ContextPack;
use surveyor_model::{Edge, Location, Node};

/// Directory under a store that holds the ledgers.
pub const SESSIONS_DIR: &str = "sessions";

/// Longest session identifier accepted; Claude Code's are 36-character UUIDs and a
/// generated one is shorter, so this bounds the file name without excluding either.
const MAX_ID_LEN: usize = 128;

/// A session identifier that is safe as a file name.
///
/// One to 128 ASCII letters, digits, `.`, `_` or `-`, not starting with a dot, so
/// the ledger path `<store>/sessions/<id>.jsonl` never leaves the directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Parses an identifier, refusing anything that is not a plain file name.
    ///
    /// # Errors
    /// Returns [`ErrorKind::InvalidId`] when the text is empty, longer than 128
    /// bytes, starts with a dot or holds a character outside `[A-Za-z0-9._-]`.
    pub fn parse(raw: &str) -> Result<Self, SessionError> {
        let plain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
        let ok = !raw.is_empty()
            && raw.len() <= MAX_ID_LEN
            && !raw.starts_with('.')
            && raw.chars().all(plain);
        if ok {
            Ok(Self(raw.to_owned()))
        } else {
            Err(SessionError::new(ErrorKind::InvalidId(raw.to_owned())))
        }
    }

    /// A fresh identifier from the clock and the process: `s-<unix seconds>-<pid>`.
    #[must_use]
    pub fn generate() -> Self {
        Self(format!("s-{}-{}", now_ms() / 1000, std::process::id()))
    }

    /// The identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SessionId {
    type Err = SessionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

/// One node an answer named, with the citation the answer carried for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Consulted {
    /// Node identifier.
    pub id: String,
    /// `file:lines` the answer cited for the node, when it cited one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
}

impl Consulted {
    /// A consulted node with a citation when `citation` is not empty.
    #[must_use]
    pub fn new(id: impl Into<String>, citation: Option<String>) -> Self {
        Self {
            id: id.into(),
            citation: citation.filter(|c| !c.is_empty()),
        }
    }

    /// A graph node, cited at its own location.
    #[must_use]
    pub fn of_node(node: &Node) -> Self {
        Self::new(
            node.id.to_string(),
            node.location.as_ref().map(Location::citation),
        )
    }

    /// A graph node, cited at the evidence of the edge that brought it.
    #[must_use]
    pub fn of_node_via(node: &Node, edge: &Edge) -> Self {
        let evidence = edge
            .provenance
            .evidence
            .first()
            .map(surveyor_model::Evidence::citation);
        Self::new(
            node.id.to_string(),
            evidence.or_else(|| node.location.as_ref().map(Location::citation)),
        )
    }
}

/// Who wrote a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// A `surveyor` command run with `--session`.
    Cli,
    /// A tool call on the MCP server.
    Mcp,
    /// A hook that recorded a tool response it observed.
    Hook,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cli => "cli",
            Self::Mcp => "mcp",
            Self::Hook => "hook",
        })
    }
}

/// What happened, without the time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// The session opened.
    Started {
        /// The task, when the session was started for one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        task: Option<String>,
        /// Repository identifier of the dataset.
        repo: String,
        /// Commit the dataset was at.
        commit: String,
    },
    /// A context pack was planned and handed to the agent.
    Pack {
        /// The task the pack was planned for.
        task: String,
        /// Budget in tokens.
        budget: usize,
        /// Tokens the pack used.
        tokens: usize,
        /// Seed node identifiers, best first.
        seeds: Vec<String>,
        /// The nodes in the pack with their citations.
        consulted: Vec<Consulted>,
        /// Unresolved references the pack reported near the seeds, as
        /// `name (file:lines)`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        gaps: Vec<String>,
    },
    /// A command or tool answered.
    Answer {
        /// Who answered.
        source: Source,
        /// The command or tool name.
        op: String,
        /// Its arguments, as given.
        #[serde(default, skip_serializing_if = "Value::is_null")]
        args: Value,
        /// The nodes the answer named.
        consulted: Vec<Consulted>,
    },
    /// The agent read something outside Surveyor and said so.
    Note {
        /// Node identifiers or `file:lines` it read.
        consulted: Vec<Consulted>,
    },
}

impl Event {
    /// The record of a planned context pack: its seeds, its items with their
    /// citations, and the gaps it reported.
    #[must_use]
    pub fn pack_of(pack: &ContextPack) -> Self {
        Self::Pack {
            task: pack.task.clone(),
            budget: pack.budget,
            tokens: pack.used.tokens,
            seeds: pack.seeds.iter().map(|s| s.node.clone()).collect(),
            consulted: pack
                .items
                .iter()
                .map(|item| {
                    let card = &item.card;
                    let citation = match (&card.file, &card.lines) {
                        (Some(file), Some(lines)) => Some(format!("{file}:{lines}")),
                        (Some(file), None) => Some(file.clone()),
                        _ => None,
                    };
                    Consulted::new(card.id.clone(), citation)
                })
                .collect(),
            gaps: pack
                .gaps
                .iter()
                .map(|g| format!("{} ({})", g.name, g.citation))
                .collect(),
        }
    }
}

/// One line of the ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Milliseconds since the Unix epoch.
    pub at_ms: u64,
    /// What happened.
    #[serde(flatten)]
    pub event: Event,
}

impl Record {
    /// Stamps an event with the current time.
    #[must_use]
    pub fn now(event: Event) -> Self {
        Self {
            at_ms: now_ms(),
            event,
        }
    }
}

/// Milliseconds since the Unix epoch, zero when the clock is before it.
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// A session's ledger file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    id: SessionId,
    path: PathBuf,
}

impl Ledger {
    /// The ledger of session `id` under `store`: `<store>/sessions/<id>.jsonl`.
    ///
    /// Opening touches nothing on disk; the directory and file appear on the first
    /// append.
    #[must_use]
    pub fn in_store(store: &Path, id: SessionId) -> Self {
        let path = store
            .join(SESSIONS_DIR)
            .join(format!("{}.jsonl", id.as_str()));
        Self { id, path }
    }

    /// The session identifier.
    #[must_use]
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Where the ledger lives.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The file a summary is written to beside the ledger, with `ext` (`md` or `json`).
    #[must_use]
    pub fn summary_path(&self, ext: &str) -> PathBuf {
        self.path
            .with_file_name(format!("{}.summary.{ext}", self.id.as_str()))
    }

    /// True once a record was written.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    /// Appends one record, creating the directory and file on the first call.
    ///
    /// # Errors
    /// Returns [`ErrorKind::Io`] when the directory or file cannot be written.
    pub fn append(&self, event: Event) -> Result<Record, SessionError> {
        let record = Record::now(event);
        let line = serde_json::to_string(&record)
            .map_err(|e| SessionError::new(ErrorKind::Json(e.to_string())).at(&self.path))?;
        let io = |e: std::io::Error| SessionError::new(ErrorKind::Io(e)).at(&self.path);
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(io)?;
        writeln!(file, "{line}").map_err(io)?;
        Ok(record)
    }

    /// Reads every record, oldest first; a ledger that was never written is empty.
    ///
    /// # Errors
    /// Returns [`ErrorKind::Io`] when the file exists and cannot be read, and
    /// [`ErrorKind::Json`] when a line does not parse.
    pub fn read(&self) -> Result<Vec<Record>, SessionError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(SessionError::new(ErrorKind::Io(e)).at(&self.path)),
        };
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, line)| {
                serde_json::from_str(line).map_err(|e| {
                    SessionError::new(ErrorKind::Json(format!("line {}: {e}", i + 1)))
                        .at(&self.path)
                })
            })
            .collect()
    }

    /// Folds the ledger into its summary.
    ///
    /// # Errors
    /// As [`read`](Self::read).
    pub fn summary(&self) -> Result<Summary, SessionError> {
        Ok(summarize(&self.id, &self.read()?))
    }
}

/// One node across a whole session: its first citation and who named it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsultedSummary {
    /// Node identifier.
    pub id: String,
    /// The first citation an answer carried for it, when one did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
    /// How many answers named it.
    pub times: usize,
    /// The commands and tools that named it.
    pub ops: BTreeSet<String>,
}

/// The pack a session started from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackSummary {
    /// The task.
    pub task: String,
    /// Budget in tokens.
    pub budget: usize,
    /// Tokens used.
    pub tokens: usize,
    /// Seed node identifiers.
    pub seeds: Vec<String>,
}

/// The consulted set of a session.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Summary {
    /// Session identifier.
    pub id: SessionId,
    /// The task the session was started for, from the start or the first pack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// Repository and commit of the dataset, when the session recorded its start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Commit of the dataset at the start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Records in the ledger.
    pub records: usize,
    /// Records per source, keyed by source name.
    pub answers_by_source: BTreeMap<String, usize>,
    /// The first pack, when one was planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackSummary>,
    /// Every node consulted, most often named first, then by identifier.
    pub consulted: Vec<ConsultedSummary>,
    /// Unresolved references the pack reported.
    pub gaps: Vec<String>,
    /// Time of the first record, milliseconds since the epoch.
    pub first_at_ms: u64,
    /// Time of the last record.
    pub last_at_ms: u64,
}

impl Summary {
    /// The summary as Markdown: a heading, the counts, the consulted table and the gaps.
    #[must_use]
    pub fn markdown(&self) -> String {
        use std::fmt::Write as _;
        // Writing to a String cannot fail, so the results are dropped.
        let mut s = String::new();
        let _ = writeln!(s, "## Surveyor session {}\n", self.id);
        if let Some(task) = &self.task {
            let _ = writeln!(s, "Task: {task}\n");
        }
        if let (Some(repo), Some(commit)) = (&self.repo, &self.commit) {
            let _ = writeln!(s, "Dataset: {repo} at {}\n", short(commit));
        }
        let by_source = self
            .answers_by_source
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            s,
            "{} records ({by_source}); {} nodes consulted, {} with a citation.\n",
            self.records,
            self.consulted.len(),
            self.consulted
                .iter()
                .filter(|c| c.citation.is_some())
                .count()
        );
        if let Some(pack) = &self.pack {
            let seeds = if pack.seeds.is_empty() {
                String::from("none")
            } else {
                pack.seeds.join(", ")
            };
            let _ = writeln!(
                s,
                "Context pack: {} of {} tokens, seeds {seeds}.\n",
                pack.tokens, pack.budget
            );
        }
        if !self.consulted.is_empty() {
            s.push_str("| Node | Citation | Named by |\n|---|---|---|\n");
            for c in &self.consulted {
                let ops = c.ops.iter().cloned().collect::<Vec<_>>().join(", ");
                let _ = writeln!(
                    s,
                    "| `{}` | {} | {ops} ×{} |",
                    c.id,
                    c.citation.as_deref().unwrap_or(""),
                    c.times
                );
            }
            s.push('\n');
        }
        if !self.gaps.is_empty() {
            s.push_str("Gaps the pack reported:\n");
            for g in &self.gaps {
                let _ = writeln!(s, "- {g}");
            }
            s.push('\n');
        }
        s
    }
}

/// First twelve characters of a commit, or all of a shorter one.
fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

/// Folds records into a [`Summary`]; a pure function of the ledger.
#[must_use]
pub fn summarize(id: &SessionId, records: &[Record]) -> Summary {
    let mut task = None;
    let mut repo = None;
    let mut commit = None;
    let mut pack = None;
    let mut gaps = Vec::new();
    let mut by_source: BTreeMap<String, usize> = BTreeMap::new();
    let mut consulted: BTreeMap<String, ConsultedSummary> = BTreeMap::new();
    let mut note = |op: &str, items: &[Consulted]| {
        for c in items {
            let entry = consulted
                .entry(c.id.clone())
                .or_insert_with(|| ConsultedSummary {
                    id: c.id.clone(),
                    citation: None,
                    times: 0,
                    ops: BTreeSet::new(),
                });
            entry.times += 1;
            entry.ops.insert(op.to_owned());
            if entry.citation.is_none() {
                entry.citation.clone_from(&c.citation);
            }
        }
    };
    for r in records {
        match &r.event {
            Event::Started {
                task: t,
                repo: rp,
                commit: c,
            } => {
                task = task.or_else(|| t.clone());
                repo = Some(rp.clone());
                commit = Some(c.clone());
            }
            Event::Pack {
                task: t,
                budget,
                tokens,
                seeds,
                consulted: items,
                gaps: g,
            } => {
                task = task.or_else(|| Some(t.clone()));
                if pack.is_none() {
                    pack = Some(PackSummary {
                        task: t.clone(),
                        budget: *budget,
                        tokens: *tokens,
                        seeds: seeds.clone(),
                    });
                    gaps.clone_from(g);
                }
                *by_source.entry(String::from("pack")).or_default() += 1;
                note("context_pack", items);
            }
            Event::Answer {
                source,
                op,
                consulted: items,
                ..
            } => {
                *by_source.entry(source.to_string()).or_default() += 1;
                note(op, items);
            }
            Event::Note { consulted: items } => {
                *by_source.entry(String::from("note")).or_default() += 1;
                note("note", items);
            }
        }
    }
    let mut consulted: Vec<ConsultedSummary> = consulted.into_values().collect();
    consulted.sort_by(|a, b| b.times.cmp(&a.times).then_with(|| a.id.cmp(&b.id)));
    Summary {
        id: id.clone(),
        task,
        repo,
        commit,
        records: records.len(),
        answers_by_source: by_source,
        pack,
        consulted,
        gaps,
        first_at_ms: records.first().map_or(0, |r| r.at_ms),
        last_at_ms: records.last().map_or(0, |r| r.at_ms),
    }
}

/// The node references in any JSON answer Surveyor renders.
///
/// An object counts as a node when it has a string `id` beside a `kind`, `name`,
/// `qualified_name` or `file` field, which is the shape every node view, card and
/// pack item has; its citation is `file:lines` when both are present, else a
/// `citation` or `location` string. An object with `file`, `start_line` and
/// `end_line` and no `id` is a source window, as `read_project_source` returns,
/// consulted as `file:start-end`. Edges, whose `src` and `dst` are bare strings,
/// are not nodes: the node views beside them carry the identifiers. The walk is
/// depth first and keeps the first citation of a repeated identifier.
#[must_use]
pub fn consulted_in(value: &Value) -> Vec<Consulted> {
    let mut found: Vec<Consulted> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    walk(value, &mut |c| {
        if seen.insert(c.id.clone()) {
            found.push(c);
        }
    });
    found
}

fn walk(value: &Value, emit: &mut dyn FnMut(Consulted)) {
    match value {
        Value::Object(map) => {
            if let Some(c) = node_of(map) {
                emit(c);
            }
            map.values().for_each(|v| walk(v, emit));
        }
        Value::Array(items) => items.iter().for_each(|v| walk(v, emit)),
        _ => {}
    }
}

fn node_of(map: &serde_json::Map<String, Value>) -> Option<Consulted> {
    let Some(id) = map.get("id") else {
        return window_of(map);
    };
    let id = id.as_str()?;
    let shaped = ["kind", "name", "qualified_name", "file"]
        .iter()
        .any(|k| map.contains_key(*k));
    if !shaped {
        return None;
    }
    let text = |k: &str| map.get(k).and_then(Value::as_str).map(str::to_owned);
    let citation = match (text("file"), text("lines")) {
        (Some(file), Some(lines)) => Some(format!("{file}:{lines}")),
        (Some(file), None) => Some(file),
        _ => text("citation").or_else(|| text("location")),
    };
    Some(Consulted::new(id, citation))
}

/// A source window (`file`, `start_line`, `end_line`) as a consulted citation.
fn window_of(map: &serde_json::Map<String, Value>) -> Option<Consulted> {
    let file = map.get("file")?.as_str()?;
    let start = map.get("start_line")?.as_u64()?;
    let end = map.get("end_line")?.as_u64()?;
    let citation = format!("{file}:{start}-{end}");
    Some(Consulted::new(citation.clone(), Some(citation)))
}

/// Why a ledger operation failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The identifier is not a plain file name.
    InvalidId(String),
    /// The ledger could not be read or written.
    Io(std::io::Error),
    /// A line of the ledger is not a record.
    Json(String),
}

/// Raised when a session identifier is invalid or a ledger cannot be read or written.
#[derive(Debug)]
pub struct SessionError {
    kind: ErrorKind,
    path: Option<PathBuf>,
    backtrace: std::backtrace::Backtrace,
}

impl SessionError {
    fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            path: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }

    fn at(mut self, path: &Path) -> Self {
        self.path = Some(path.to_path_buf());
        self
    }

    /// What went wrong.
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::InvalidId(id) => write!(
                f,
                "session id {id:?} is not a plain file name (letters, digits, `.`, `_`, `-`; not starting with a dot; at most {MAX_ID_LEN} bytes)"
            ),
            ErrorKind::Io(e) => write!(f, "session ledger: {e}"),
            ErrorKind::Json(e) => write!(f, "session ledger is not valid: {e}"),
        }?;
        match &self.path {
            Some(p) => write!(f, " ({})", p.display()),
            None => Ok(()),
        }
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::InvalidId(_) | ErrorKind::Json(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_are_plain_file_names() {
        assert!(SessionId::parse("5e0d8f2a-1b3c-4d5e-8f90-123456789abc").is_ok());
        assert!(SessionId::parse("s-1700000000-42").is_ok());
        for bad in ["", ".hidden", "a/b", "a b", "..", &"x".repeat(129)] {
            assert!(SessionId::parse(bad).is_err(), "{bad:?}");
        }
        let generated = SessionId::generate();
        assert!(SessionId::parse(generated.as_str()).is_ok());
    }

    #[test]
    fn a_ledger_round_trips_and_summarizes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let id = SessionId::parse("t1").expect("id");
        let ledger = Ledger::in_store(dir.path(), id.clone());
        assert!(!ledger.exists());
        assert!(ledger.read().expect("empty read").is_empty());

        ledger
            .append(Event::Started {
                task: Some("make drain batch".into()),
                repo: "mini".into(),
                commit: "abc123".into(),
            })
            .expect("append");
        ledger
            .append(Event::Pack {
                task: "make drain batch".into(),
                budget: 12_000,
                tokens: 3_400,
                seeds: vec!["rust:a.rs::drain".into()],
                consulted: vec![
                    Consulted::new("rust:a.rs::drain", Some("a.rs:10-20".into())),
                    Consulted::new("rust:a.rs::Relay", Some("a.rs:1-30".into())),
                ],
                gaps: vec!["sqlx (a.rs:12)".into()],
            })
            .expect("append");
        ledger
            .append(Event::Answer {
                source: Source::Cli,
                op: "affected".into(),
                args: json!({"query": "drain"}),
                consulted: vec![
                    Consulted::new("rust:a.rs::drain", None),
                    Consulted::new("rust:b.rs::main", Some("b.rs:5".into())),
                ],
            })
            .expect("append");
        ledger
            .append(Event::Note {
                consulted: vec![Consulted::new("docs/x.md", None)],
            })
            .expect("append");

        assert!(ledger.exists());
        let records = ledger.read().expect("read");
        assert_eq!(records.len(), 4);
        assert!(records.windows(2).all(|w| w[0].at_ms <= w[1].at_ms));

        let summary = ledger.summary().expect("summary");
        assert_eq!(summary.task.as_deref(), Some("make drain batch"));
        assert_eq!(summary.repo.as_deref(), Some("mini"));
        assert_eq!(summary.records, 4);
        assert_eq!(summary.consulted.len(), 4);
        // Named twice, with the first citation kept.
        let drain = &summary.consulted[0];
        assert_eq!(drain.id, "rust:a.rs::drain");
        assert_eq!(drain.times, 2);
        assert_eq!(drain.citation.as_deref(), Some("a.rs:10-20"));
        assert!(drain.ops.contains("affected") && drain.ops.contains("context_pack"));
        assert_eq!(summary.gaps, vec!["sqlx (a.rs:12)".to_owned()]);
        assert_eq!(summary.answers_by_source.get("cli"), Some(&1));
        let md = summary.markdown();
        assert!(md.contains("## Surveyor session t1"));
        assert!(md.contains("| `rust:b.rs::main` | b.rs:5 | affected ×1 |"));
        assert_eq!(
            ledger.summary_path("md").file_name().unwrap(),
            "t1.summary.md"
        );
    }

    #[test]
    fn node_references_are_read_out_of_json_answers() {
        let answer = json!({
            "node": {"id": "rust:a.rs::f", "kind": "Function", "file": "a.rs", "lines": "3-9"},
            "relations": [
                {"kind": "CALLS", "node": {"id": "rust:b.rs::g", "kind": "Function", "name": "g"},
                 "evidence": ["a.rs:5"]},
                {"src": "rust:a.rs::f", "dst": "rust:c.rs::h", "kind": "CALLS"}
            ],
            "items": [{"card": {"id": "doc:README.md", "kind": "Document", "file": "README.md"}}],
            "hit": {"id": "x", "citation": "z.rs:1", "name": "x"},
            "plain": {"id": "not-a-node"},
            "window": {"file": "w.rs", "start_line": 4, "end_line": 9, "text": "..."}
        });
        let mut found = consulted_in(&answer);
        found.sort();
        let ids: Vec<&str> = found.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "doc:README.md",
                "rust:a.rs::f",
                "rust:b.rs::g",
                "w.rs:4-9",
                "x"
            ]
        );
        assert_eq!(found[0].citation.as_deref(), Some("README.md"));
        assert_eq!(found[1].citation.as_deref(), Some("a.rs:3-9"));
        assert_eq!(found[2].citation, None);
        assert_eq!(found[3].citation.as_deref(), Some("w.rs:4-9"));
        assert_eq!(found[4].citation.as_deref(), Some("z.rs:1"));
    }

    #[test]
    fn a_corrupt_line_is_reported_with_its_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger = Ledger::in_store(dir.path(), SessionId::parse("t2").expect("id"));
        ledger
            .append(Event::Note { consulted: vec![] })
            .expect("append");
        std::fs::OpenOptions::new()
            .append(true)
            .open(ledger.path())
            .and_then(|mut f| writeln!(f, "{{not json"))
            .expect("corrupt");
        let err = ledger.read().expect_err("corrupt line");
        assert!(
            matches!(err.kind(), ErrorKind::Json(m) if m.starts_with("line 2")),
            "{err}"
        );
    }
}
