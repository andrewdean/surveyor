//! Error type for the store.

use std::backtrace::Backtrace;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Lance(lancedb::Error),
    Arrow(arrow_schema::ArrowError),
    Serde(serde_json::Error),
    Io(std::io::Error),
    Schema(String),
}

/// Raised when the dataset cannot be opened, written or read.
#[derive(Debug)]
pub struct StoreError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl StoreError {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn schema(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Schema(msg.into()))
    }

    /// True when the dataset on disk does not match the expected schema.
    #[must_use]
    pub fn is_schema(&self) -> bool {
        matches!(self.kind, ErrorKind::Schema(_))
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Lance(e) => write!(f, "lance: {e}"),
            ErrorKind::Arrow(e) => write!(f, "arrow: {e}"),
            ErrorKind::Serde(e) => write!(f, "serialization: {e}"),
            ErrorKind::Io(e) => write!(f, "I/O: {e}"),
            ErrorKind::Schema(m) => write!(f, "schema: {m}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Lance(e) => Some(e),
            ErrorKind::Arrow(e) => Some(e),
            ErrorKind::Serde(e) => Some(e),
            ErrorKind::Io(e) => Some(e),
            ErrorKind::Schema(_) => None,
        }
    }
}

impl From<lancedb::Error> for StoreError {
    fn from(e: lancedb::Error) -> Self {
        Self::new(ErrorKind::Lance(e))
    }
}

impl From<arrow_schema::ArrowError> for StoreError {
    fn from(e: arrow_schema::ArrowError) -> Self {
        Self::new(ErrorKind::Arrow(e))
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Serde(e))
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Io(e))
    }
}
