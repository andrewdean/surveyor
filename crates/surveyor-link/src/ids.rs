//! Stable node identifier builders.
//!
//! Identifiers never depend on content, so an unchanged declaration keeps its
//! identifier across commits. They are readable on purpose: an agent that sees
//! `rust:crates/outbox/src/relay.rs::Relay::drain` knows where to look.

use surveyor_model::{Language, NodeId, RepoId, RepoPath};

/// `repo:<id>`.
#[must_use]
pub fn repo_id(repo: &RepoId) -> NodeId {
    NodeId::new(format!("repo:{repo}"))
}

/// `file:<path>`.
#[must_use]
pub fn file_id(path: &RepoPath) -> NodeId {
    NodeId::new(format!("file:{path}"))
}

/// `pkg:<registry>/<name>`, from a manifest's registry coordinate.
#[must_use]
pub fn package_id(coordinate: &str) -> NodeId {
    NodeId::new(format!("pkg:{coordinate}"))
}

/// `doc:<path>`.
#[must_use]
pub fn doc_id(path: &RepoPath) -> NodeId {
    NodeId::new(format!("doc:{path}"))
}

/// `sql:<schema>[.<table>[.<column>]]`, or `sql:<schema>.<table>#<name>` for members.
#[must_use]
pub fn sql_id(parts: &[&str], member: Option<&str>) -> NodeId {
    let base = parts.join(".");
    match member {
        Some(m) => NodeId::new(format!("sql:{base}#{m}")),
        None => NodeId::new(format!("sql:{base}")),
    }
}

/// `<language>:<path>::<qualified name>`.
#[must_use]
pub fn symbol_id(language: Option<&Language>, path: &RepoPath, qualified: &str) -> NodeId {
    let lang = language.map_or("src", Language::as_str);
    NodeId::new(format!("{lang}:{path}::{qualified}"))
}
