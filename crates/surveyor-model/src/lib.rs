//! The language-independent graph model Surveyor builds from source.
//!
//! This crate holds the types every other Surveyor crate agrees on and performs no
//! I/O. Extractors produce [`FactSet`]s from [`SourceUnit`]s; linkers turn facts into
//! [`Node`]s and [`Edge`]s; every edge carries a [`Provenance`] that says how it was
//! derived and which source spans support it.
//!
//! # Examples
//!
//! ```
//! use surveyor_model::{Confidence, Derivation, EdgeKind, GrammarId, NodeId, RuleId};
//!
//! let derivation = Derivation::Syntactic {
//!     grammar: GrammarId::new("rust"),
//!     rule: RuleId::new("call.expression"),
//! };
//! assert!(derivation.is_deterministic());
//! assert_eq!(Confidence::CERTAIN.get(), 1.0);
//! assert_eq!(EdgeKind::Calls.as_str(), "CALLS");
//! let id = NodeId::new("rust:crates/a/src/lib.rs::Relay::drain");
//! assert_eq!(id.as_str(), "rust:crates/a/src/lib.rs::Relay::drain");
//! ```

// Rust guideline compliant 2026-02-21

mod change;
mod edge;
mod error;
mod fact;
mod graph;
mod ids;
mod language;
mod node;
mod provenance;
mod span;

#[doc(inline)]
pub use change::{ChangeSet, FileChange, new_side_range};
#[doc(inline)]
pub use edge::{Edge, EdgeId, EdgeKind};
#[doc(inline)]
pub use error::ModelError;
#[doc(inline)]
pub use fact::{
    CallForm, DeclarationFact, DeclaredRelation, Fact, FactSet, LiteralFact, LiteralRole,
    ReferenceFact, ReferenceRole, ResolvedReference, SourceUnit, SymbolTarget,
};
#[doc(inline)]
pub use graph::{FederatedEdge, GraphDelta, GraphStats, Unresolved, UnresolvedReason};
#[doc(inline)]
pub use ids::{
    CommitId, ContentHash, ExtractorId, GrammarId, ModelId, NodeId, ProbeId, RepoId, RepoPath,
    ResolverId, RuleId,
};
#[doc(inline)]
pub use language::Language;
#[doc(inline)]
pub use node::{Node, NodeKind};
#[doc(inline)]
pub use provenance::{
    Confidence, DeclaredSource, Derivation, DerivationKind, Evidence, Provenance, TimeRange,
    Timestamp,
};
#[doc(inline)]
pub use span::{ByteSpan, LineSpan, Location, SourceLocation, TextSpan};
