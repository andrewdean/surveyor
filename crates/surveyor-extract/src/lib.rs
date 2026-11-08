//! Extraction: turning source files into language-independent facts.
//!
//! Two tiers produce facts. The universal tier is Tree-sitter: a grammar plus a
//! Surveyor query pack per language (see the `languages/` directory) yields
//! declarations, references and tagged literals for any file the grammar parses, even
//! a broken one. Native enrichers handle formats where a crate gives a better model
//! than a grammar: `sqlparser` for SQL files and SQL string literals, the `rhai`
//! crate's own AST for Rhai, typed deserialization for package manifests and mise task
//! files, and a small Markdown reader for design documents, runbooks and every other
//! Markdown document.
//!
//! The precise tier is SCIP: [`ScipExtractor`] reads an index an external indexer
//! wrote (`rust-analyzer scip`, `scip-typescript`, `scip-python`) and emits
//! [`Fact::Resolved`](surveyor_model::Fact::Resolved) facts that bind reference sites
//! to the file and line of their declaration, which the linker pairs with the
//! Tree-sitter references at the same sites. Without an index nothing changes.
//!
//! [`extract_repository`] walks a checkout, runs every accepting extractor over each
//! file in parallel and returns the fact sets the linker consumes.
//!
//! # Examples
//!
//! ```
//! use surveyor_extract::{Extractor, TreeSitterExtractor};
//! use surveyor_model::{CommitId, RepoId, RepoPath, SourceUnit};
//!
//! let unit = SourceUnit::new(
//!     RepoId::new("demo"),
//!     RepoPath::new("src/lib.rs"),
//!     CommitId::new("worktree"),
//!     b"pub fn drain() { flush(); }".to_vec(),
//! );
//! let extractor = TreeSitterExtractor::new();
//! let facts = extractor.extract(&unit).expect("rust parses");
//! assert_eq!(facts.declarations().count(), 1);
//! assert_eq!(facts.references().next().map(|r| r.name.as_str()), Some("flush"));
//! ```

// Rust guideline compliant 2026-02-21

mod error;
mod extractor;
mod manifest;
mod markdown;
mod mise;
mod pipeline;
mod profile;
mod rhai_lang;
mod scip_index;
mod sql;
mod template;
mod treesitter;

mod walk;

#[doc(inline)]
pub use error::ExtractError;
#[doc(inline)]
pub use extractor::{Extractor, Extractors};
#[doc(inline)]
pub use manifest::ManifestExtractor;
#[doc(inline)]
pub use markdown::MarkdownExtractor;
#[doc(inline)]
pub use mise::MiseExtractor;
#[doc(inline)]
pub use pipeline::{
    Extraction, UnitRecord, default_extractors, extract_paths, extract_paths_using,
    extract_repository, extract_repository_with, extract_units, extractors_with,
};
#[doc(inline)]
pub use profile::{
    DocsProfile, HarnessProfile, Profile, RulesProfile, ScipProfile, SqlDialect, SqlProfile,
    WalkProfile,
};
#[doc(inline)]
pub use rhai_lang::RhaiExtractor;
#[doc(inline)]
pub use scip_index::ScipExtractor;
#[doc(inline)]
pub use sql::{SqlExtractor, StatementSummary, summarize_sql};
#[doc(inline)]
pub use treesitter::TreeSitterExtractor;
#[doc(inline)]
pub use walk::{ChangeSource, FileChanges, Repository};

/// Bumped whenever an extractor produces different facts for the same file without a
/// query pack changing: a span computed differently, a new reference form.
///
/// Part of the engine fingerprint a dataset records, with [`query_pack_fingerprint`],
/// so `surveyor update` re-indexes in full rather than mixing old and new facts.
pub const EXTRACT_REVISION: u32 = 12;

/// A hash of the embedded query packs: part of the engine fingerprint a dataset records.
///
/// Two builds with the same version but different packs produce different facts for the
/// same file; an incremental update across such builds would mix vocabularies.
#[must_use]
pub fn query_pack_fingerprint() -> surveyor_model::ContentHash {
    surveyor_model::ContentHash::of_parts([
        include_str!("../../../languages/rust/surveyor.scm"),
        include_str!("../../../languages/typescript/surveyor.scm"),
        include_str!("../../../languages/javascript/surveyor.scm"),
        include_str!("../../../languages/python/surveyor.scm"),
        include_str!("../../../languages/bash/surveyor.scm"),
    ])
}
