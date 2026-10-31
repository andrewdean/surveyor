//! Error type for extraction.

use std::backtrace::Backtrace;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

/// What went wrong during extraction.
#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(std::io::Error),
    Parse(String),
    Query(String),
    Git(String),
    Config(String),
}

/// Raised when a repository, file or query cannot be processed.
///
/// Use [`ExtractError::path`] for the file involved, when there is one, and the
/// `is_*` predicates to branch on the cause without matching on internals.
#[derive(Debug)]
pub struct ExtractError {
    kind: ErrorKind,
    path: Option<PathBuf>,
    backtrace: Backtrace,
}

impl ExtractError {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            path: None,
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn io(err: std::io::Error) -> Self {
        Self::new(ErrorKind::Io(err))
    }

    pub(crate) fn parse(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Parse(msg.into()))
    }

    pub(crate) fn query(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Query(msg.into()))
    }

    pub(crate) fn git(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Git(msg.into()))
    }

    pub(crate) fn config(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config(msg.into()))
    }

    pub(crate) fn at(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// The file the error concerns, when known.
    #[must_use]
    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    /// True when the cause is an I/O failure.
    #[must_use]
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ErrorKind::Io(_))
    }

    /// True when the cause is a syntax error in the source being extracted.
    #[must_use]
    pub fn is_parse(&self) -> bool {
        matches!(self.kind, ErrorKind::Parse(_))
    }

    /// True when the cause is a malformed configuration or profile.
    #[must_use]
    pub fn is_config(&self) -> bool {
        matches!(self.kind, ErrorKind::Config(_))
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Io(e) => write!(f, "I/O error: {e}")?,
            ErrorKind::Parse(m) => write!(f, "parse error: {m}")?,
            ErrorKind::Query(m) => write!(f, "query pack error: {m}")?,
            ErrorKind::Git(m) => write!(f, "git error: {m}")?,
            ErrorKind::Config(m) => write!(f, "configuration error: {m}")?,
        }
        if let Some(path) = &self.path {
            write!(f, " ({})", path.display())?;
        }
        Ok(())
    }
}

impl std::error::Error for ExtractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ExtractError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err)
    }
}
