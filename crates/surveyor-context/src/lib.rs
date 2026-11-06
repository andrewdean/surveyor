//! Context packs: the minimum sufficient context for one task, with evidence, inside
//! a token budget.
//!
//! [`Planner::pack`] builds a [`ContextPack`] over a loaded graph and its text index:
//! seeds from the identifiers the task names and from the BM25F ranking of its text,
//! expansion along the edges `explain` and `affected` cite, ranking by seed score,
//! path confidence, hop decay and kind prior, and a greedy fill of the budget with a
//! card per node and a source window for the best of them. The pack renders as
//! Markdown, which is what the budget is counted on with the `cl100k_base` encoding
//! in [`tokens`].

// Rust guideline compliant 2026-02-21

pub mod pack;
pub mod render;
pub mod text;
pub mod tokens;

#[doc(inline)]
pub use pack::{
    ContextPack, DEFAULT_BUDGET, Gap, Item, Omission, Omitted, Planner, Seed, SeedSource, Via,
    Window,
};
#[doc(inline)]
pub use text::identifiers_in;
#[doc(inline)]
pub use tokens::{Measured, Tokenizer};
