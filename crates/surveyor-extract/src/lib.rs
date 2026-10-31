//! Extraction: turning source files into language-independent facts.
//!
//! Two tiers produce facts. The universal tier is Tree-sitter: a grammar plus a
//! Surveyor query pack per language (see the `languages/` directory) yields
//! declarations, references and tagged literals for any file the grammar parses, even
//! a broken one. Native enrichers handle formats where a crate gives a better model
//! than a grammar: `sqlparser` for SQL files and SQL string literals, the `rhai`
//! crate's own AST for Rhai, typed deserialization for package manifests, and a small
//! Markdown reader for design documents and runbooks.
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
mod pipeline;
mod profile;
mod rhai_lang;
mod sql;
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
pub use pipeline::{Extraction, UnitRecord, extract_repository, extract_units};
#[doc(inline)]
pub use profile::{DocsProfile, Profile, SqlDialect, SqlProfile, WalkProfile};
#[doc(inline)]
pub use rhai_lang::RhaiExtractor;
#[doc(inline)]
pub use sql::{SqlExtractor, StatementSummary, summarize_sql};
#[doc(inline)]
pub use treesitter::TreeSitterExtractor;
#[doc(inline)]
pub use walk::Repository;
