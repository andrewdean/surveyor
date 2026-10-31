//! Identifier newtypes shared across the graph.
//!
//! Every identifier is a thin wrapper over `String` or a fixed hash so that a node
//! identifier can never be passed where a repository identifier is expected. Node
//! identifiers are stable: they derive from language, path and scope, never from
//! content, so an unchanged symbol keeps its identifier across commits.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps the given text.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrows the identifier as text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the identifier and returns the owned text.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
    };
}

string_id!(
    /// Identifies one repository inside a Surveyor dataset, such as `causeway`.
    RepoId
);
string_id!(
    /// Stable identifier of a node, such as `rust:crates/a/src/lib.rs::Relay::drain`.
    NodeId
);
string_id!(
    /// A commit identifier as the version control system renders it.
    CommitId
);
string_id!(
    /// Identifies an extractor implementation, such as `tree-sitter/rust`.
    ExtractorId
);
string_id!(
    /// Identifies a Tree-sitter grammar, such as `rust`.
    GrammarId
);
string_id!(
    /// Identifies a query rule inside a grammar pack, such as `definition.function`.
    RuleId
);
string_id!(
    /// Identifies a precise resolver, such as `scip/rust-analyzer` or `sqlparser`.
    ResolverId
);
string_id!(
    /// Identifies a runtime probe (Phase 4).
    ProbeId
);
string_id!(
    /// Identifies a model used for an inferred fact, such as `claude-fable-5-1`.
    ModelId
);

/// A repository-relative path with forward slashes and no leading `./`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepoPath(String);

impl RepoPath {
    /// Normalizes and wraps a repository-relative path.
    ///
    /// Backslashes become forward slashes and a leading `./` is dropped.
    pub fn new(value: impl AsRef<str>) -> Self {
        let raw = value.as_ref().replace('\\', "/");
        let trimmed = raw.strip_prefix("./").unwrap_or(&raw);
        Self(trimmed.to_owned())
    }

    /// Borrows the path as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The file name component, if any.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.0.rsplit('/').next().filter(|s| !s.is_empty())
    }

    /// The extension without the dot, if any.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;
        let (_, ext) = name.rsplit_once('.')?;
        (!ext.is_empty()).then_some(ext)
    }

    /// The parent directory, or an empty string at the root.
    #[must_use]
    pub fn parent(&self) -> &str {
        self.0.rsplit_once('/').map_or("", |(dir, _)| dir)
    }

    /// True when this path is inside the given directory prefix.
    #[must_use]
    pub fn starts_with_dir(&self, dir: &str) -> bool {
        let dir = dir.trim_end_matches('/');
        dir.is_empty() || self.0 == dir || self.0.starts_with(&format!("{dir}/"))
    }
}

impl Display for RepoPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RepoPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A BLAKE3 digest over content, rendered as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Hashes the given bytes.
    pub fn of(bytes: impl AsRef<[u8]>) -> Self {
        Self(*blake3::hash(bytes.as_ref()).as_bytes())
    }

    /// Hashes several parts in order with length separators.
    ///
    /// Use this to derive an identifier from fields without ambiguity between
    /// `("ab", "c")` and `("a", "bc")`.
    pub fn of_parts<'a>(parts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part.as_bytes());
        }
        Self(*hasher.finalize().as_bytes())
    }

    /// The raw digest bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The digest as 64 lowercase hex characters.
    #[must_use]
    pub fn to_hex(&self) -> String {
        use std::fmt::Write as _;
        let mut hex = String::with_capacity(64);
        for b in self.0 {
            write!(hex, "{b:02x}").expect("writing to a String cannot fail");
        }
        hex
    }

    /// The first 16 hex characters, enough to disambiguate within a repository.
    #[must_use]
    pub fn short(&self) -> String {
        let mut hex = self.to_hex();
        hex.truncate(16);
        hex
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.short())
    }
}

impl Display for ContentHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl From<ContentHash> for String {
    fn from(value: ContentHash) -> Self {
        value.to_hex()
    }
}

impl TryFrom<String> for ContentHash {
    type Error = crate::ModelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::str::FromStr for ContentHash {
    type Err = crate::ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 64 {
            return Err(crate::ModelError::invalid("ContentHash", s));
        }
        let mut out = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let text = std::str::from_utf8(chunk)
                .map_err(|_utf8| crate::ModelError::invalid("ContentHash", s))?;
            out[i] = u8::from_str_radix(text, 16)
                .map_err(|_radix| crate::ModelError::invalid("ContentHash", s))?;
        }
        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_path_normalizes() {
        let p = RepoPath::new("./crates\\a\\src\\lib.rs");
        assert_eq!(p.as_str(), "crates/a/src/lib.rs");
        assert_eq!(p.file_name(), Some("lib.rs"));
        assert_eq!(p.extension(), Some("rs"));
        assert_eq!(p.parent(), "crates/a/src");
        assert!(p.starts_with_dir("crates/a"));
        assert!(!p.starts_with_dir("crates/ab"));
    }

    #[test]
    fn content_hash_round_trips_hex() {
        let h = ContentHash::of(b"hello");
        let parsed: ContentHash = h.to_hex().parse().expect("valid hex");
        assert_eq!(h, parsed);
        assert_ne!(
            ContentHash::of_parts(["ab", "c"]),
            ContentHash::of_parts(["a", "bc"])
        );
        assert!("zz".parse::<ContentHash>().is_err());
    }
}
