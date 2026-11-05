//! Context packs: the minimum sufficient context for one task, with evidence, inside
//! a token budget.
//!
//! This crate holds the token accounting the budget is kept in. The planner that
//! fills a budget from the graph lands beside it.

// Rust guideline compliant 2026-02-21

pub mod tokens;

pub use tokens::{Measured, Tokenizer};
