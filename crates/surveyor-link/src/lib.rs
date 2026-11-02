//! Linking: facts from many files become one graph with provenance.
//!
//! The linker is language independent. It builds structural nodes (repository,
//! packages, files), one node per declaration, and then resolves every reference by
//! name within scope heuristics: the same file first, then the same package, then the
//! repository. A reference with one candidate becomes a certain edge; one with a few
//! candidates becomes one edge per candidate with confidence `1/n`; one with none or
//! too many is recorded as [`Unresolved`](surveyor_model::Unresolved), never guessed.
//!
//! SQL references resolve against the schema model the SQL extractor produced, and
//! carry `Resolved` provenance because `sqlparser` did the work. Manifest and
//! documentation relationships carry `Declared` provenance.
//!
//! # Examples
//!
//! ```
//! use surveyor_link::{FileEntry, LinkInput, link};
//! use surveyor_model::{CommitId, RepoId};
//!
//! let input = LinkInput::new(RepoId::new("demo"), CommitId::new("worktree"));
//! let delta = link(&input);
//! assert_eq!(delta.nodes.len(), 1, "an empty repository still has its root node");
//! let _ = FileEntry::default();
//! ```

// Rust guideline compliant 2026-02-21

mod ids;
mod input;
mod linker;
mod resolve;
mod update;

#[doc(inline)]
pub use ids::{doc_id, file_id, package_id, repo_id, sql_id, symbol_id};
#[doc(inline)]
pub use input::{FileEntry, LinkInput, LinkOptions};
#[doc(inline)]
pub use linker::{Seed, link, relink};

/// Bumped whenever linking produces different nodes or edges for the same facts.
///
/// Part of the engine fingerprint a dataset records, so `surveyor update` knows that
/// facts linked by an older build cannot be mixed with new ones.
pub const LINK_REVISION: u32 = 4;
#[doc(inline)]
pub use update::{ChangedFiles, RelinkReason, UpdatePlan, is_manifest, plan_update};
