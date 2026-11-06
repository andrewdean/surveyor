//! Token counting for the budget a context pack is held to.
//!
//! Claude's tokenizer is not public, so the count is the `cl100k_base` byte-pair
//! encoding, the usual proxy; a pack's report carries its bytes beside the tokens so
//! the margin is visible.

use std::fmt::{self, Debug, Formatter};

use serde::Serialize;
use tiktoken_rs::CoreBPE;

/// Counts tokens the way a prompt budget is spent.
///
/// Building one loads the encoding's vocabulary, which takes a noticeable fraction
/// of a second, so a caller keeps one and counts many texts with it.
pub struct Tokenizer {
    bpe: CoreBPE,
}

impl Debug for Tokenizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tokenizer")
            .field("encoding", &"cl100k_base")
            .finish()
    }
}

/// A text's size as a budget sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Measured {
    /// Tokens under `cl100k_base`.
    pub tokens: usize,
    /// Bytes of UTF-8.
    pub bytes: usize,
}

impl Tokenizer {
    /// The `cl100k_base` encoding.
    ///
    /// # Panics
    /// Panics when the embedded vocabulary fails to load, which is a build defect,
    /// not a runtime condition.
    #[must_use]
    pub fn cl100k() -> Self {
        let bpe = tiktoken_rs::cl100k_base().expect("cl100k_base vocabulary is embedded");
        Self { bpe }
    }

    /// Tokens in `text`, with no special tokens recognized.
    #[must_use]
    pub fn count(&self, text: &str) -> usize {
        self.bpe.encode_ordinary(text).len()
    }

    /// Tokens and bytes of `text`.
    #[must_use]
    pub fn measure(&self, text: &str) -> Measured {
        Measured {
            tokens: self.count(text),
            bytes: text.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_tokens_and_bytes() {
        let t = Tokenizer::cl100k();
        assert_eq!(t.count(""), 0);
        let text = "fn drain(&mut self) -> Result<usize>";
        let m = t.measure(text);
        assert!(m.tokens > 5 && m.tokens < m.bytes, "{m:?}");
        assert_eq!(m.bytes, text.len());
        assert_eq!(m.tokens, t.count(text));
    }
}
