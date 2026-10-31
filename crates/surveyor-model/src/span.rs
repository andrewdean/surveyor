//! Byte and line spans inside a source unit, and where a node lives.
//!
//! [`TextSpan`] pairs a byte range with the lines it covers so the two can never
//! disagree; [`SourceLocation`] adds the file; [`Location`] is what a node has: a whole
//! file, or a span inside one.

use serde::{Deserialize, Serialize};

use crate::RepoPath;

/// A half-open byte range `[start, end)` inside a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ByteSpan {
    /// First byte, inclusive.
    pub start: u64,
    /// Last byte, exclusive.
    pub end: u64,
}

impl ByteSpan {
    /// Creates a span; `end` must not precede `start`.
    ///
    /// # Panics
    /// Panics when `end < start`, which is a programming error.
    #[must_use]
    pub fn new(start: u64, end: u64) -> Self {
        assert!(end >= start, "byte span end precedes start");
        Self { start, end }
    }

    /// Number of bytes covered.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.end - self.start
    }

    /// True when the span covers no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// True when `other` lies entirely inside this span.
    #[must_use]
    pub fn contains(&self, other: ByteSpan) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Slices `source` to this span, or `None` when the span is out of bounds.
    #[must_use]
    pub fn slice<'a>(&self, source: &'a [u8]) -> Option<&'a [u8]> {
        let start = usize::try_from(self.start).ok()?;
        let end = usize::try_from(self.end).ok()?;
        source.get(start..end)
    }
}

/// A closed, one-based line range inside a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineSpan {
    /// First line, one-based, inclusive.
    pub start: u32,
    /// Last line, one-based, inclusive.
    pub end: u32,
}

impl LineSpan {
    /// Creates a line span; `end` must not precede `start`.
    ///
    /// # Panics
    /// Panics when `end < start`, which is a programming error.
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        assert!(end >= start, "line span end precedes start");
        Self { start, end }
    }

    /// Creates a line span from zero-based line indexes, as Tree-sitter reports them.
    #[must_use]
    pub fn from_zero_based(start_row: usize, end_row: usize) -> Self {
        let start = u32::try_from(start_row)
            .unwrap_or(u32::MAX - 1)
            .saturating_add(1);
        let end = u32::try_from(end_row)
            .unwrap_or(u32::MAX - 1)
            .saturating_add(1);
        Self::new(start, end.max(start))
    }

    /// Number of lines covered.
    #[must_use]
    pub fn count(&self) -> u32 {
        self.end - self.start + 1
    }
}

/// A byte range together with the one-based lines it covers.
///
/// Constructed by [`SourceUnit::text_span`](crate::SourceUnit::text_span), which counts
/// the lines, so the two halves cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TextSpan {
    /// Byte range.
    pub bytes: ByteSpan,
    /// Line range.
    pub lines: LineSpan,
}

impl TextSpan {
    /// Pairs a byte range with the lines it covers.
    #[must_use]
    pub const fn new(bytes: ByteSpan, lines: LineSpan) -> Self {
        Self { bytes, lines }
    }

    /// First byte of the span.
    #[must_use]
    pub const fn start(&self) -> u64 {
        self.bytes.start
    }

    /// One past the last byte of the span.
    #[must_use]
    pub const fn end(&self) -> u64 {
        self.bytes.end
    }
}

/// A span inside a named file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Repository-relative file.
    pub file: RepoPath,
    /// Span inside the file.
    pub span: TextSpan,
}

impl SourceLocation {
    /// Places `span` inside `file`.
    #[must_use]
    pub const fn new(file: RepoPath, span: TextSpan) -> Self {
        Self { file, span }
    }

    /// A `file:lines` citation.
    #[must_use]
    pub fn citation(&self) -> String {
        format!("{}:{}", self.file, self.span.lines)
    }
}

/// Where a node lives: a whole file, or a span inside one.
///
/// A span without a file, or lines without bytes, cannot be expressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "kebab-case")]
pub enum Location {
    /// The node is the file itself, or has no narrower span.
    File {
        /// Repository-relative file.
        file: RepoPath,
    },
    /// The node is a span inside a file.
    Span(SourceLocation),
}

impl Location {
    /// The file the node lives in.
    #[must_use]
    pub fn file(&self) -> &RepoPath {
        match self {
            Self::File { file } => file,
            Self::Span(loc) => &loc.file,
        }
    }

    /// The span, when the node is narrower than a file.
    #[must_use]
    pub fn span(&self) -> Option<TextSpan> {
        match self {
            Self::File { .. } => None,
            Self::Span(loc) => Some(loc.span),
        }
    }

    /// The source location, when the node is narrower than a file.
    #[must_use]
    pub fn source(&self) -> Option<&SourceLocation> {
        match self {
            Self::File { .. } => None,
            Self::Span(loc) => Some(loc),
        }
    }

    /// `file:lines`, or just the file.
    #[must_use]
    pub fn citation(&self) -> String {
        match self {
            Self::File { file } => file.to_string(),
            Self::Span(loc) => loc.citation(),
        }
    }
}

impl std::fmt::Display for LineSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_measure_and_slice() {
        let span = ByteSpan::new(2, 5);
        assert_eq!(span.len(), 3);
        assert_eq!(span.slice(b"0123456"), Some(&b"234"[..]));
        assert!(span.contains(ByteSpan::new(3, 4)));
        assert_eq!(LineSpan::from_zero_based(0, 2), LineSpan::new(1, 3));
        assert_eq!(LineSpan::new(4, 4).to_string(), "4");
        assert_eq!(LineSpan::new(4, 9).to_string(), "4-9");
    }
}
