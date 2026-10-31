//! Persistence: the graph as Lance tables on local disk or object storage.
//!
//! [`LanceStore`] keeps five tables (`nodes`, `edges`, `files`, `unresolved`, `meta`)
//! partitioned by repository. Replacing a repository deletes its rows and appends the
//! new ones, which Lance records as a new dataset version, so the previous index stays
//! queryable. Traversal does not happen here: [`LanceStore::load_graph`] hands the
//! rows to `surveyor-graph`.
//!
//! The store is also where vectors and full-text indexes will live (Phase 2); the
//! schema keeps a JSON `attributes` column per node so cards can be added without a
//! migration.

// Rust guideline compliant 2026-02-21

mod error;
mod lance;
mod schema;

#[doc(inline)]
pub use error::StoreError;
#[doc(inline)]
pub use lance::{FileRecord, LanceStore, RepositorySnapshot, StoreStats};
