//! Error type for model-level parsing and validation.

use std::backtrace::Backtrace;
use std::fmt::{self, Display, Formatter};

/// Raised when a textual form does not parse into a model value.
///
/// Produced by the `FromStr` implementations of identifiers and kinds. The input that
/// failed is available through [`ModelError::input`].
#[derive(Debug)]
pub struct ModelError {
    what: &'static str,
    input: String,
    backtrace: Backtrace,
}

impl ModelError {
    pub(crate) fn invalid(what: &'static str, input: impl Into<String>) -> Self {
        Self {
            what,
            input: input.into(),
            backtrace: Backtrace::capture(),
        }
    }

    /// The kind of value that was being parsed, such as `"NodeKind"`.
    #[must_use]
    pub fn what(&self) -> &'static str {
        self.what
    }

    /// The text that failed to parse.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {}: {:?}", self.what, self.input)
    }
}

impl std::error::Error for ModelError {}
