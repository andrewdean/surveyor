//! Source languages and formats Surveyor recognizes.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{ModelError, RepoPath};

/// A programming language or configuration format.
///
/// The variant set is open-ended through [`Language::Other`], so an adapter can
/// introduce a language without a model change.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Bash,
    Sql,
    Rhai,
    Toml,
    Yaml,
    Json,
    Markdown,
    Dockerfile,
    Proto,
    Go,
    Java,
    /// A language without a built-in variant, named by its lowercase identifier.
    Other(String),
}

impl Language {
    /// Guesses the language from a repository path, or `None` when unknown.
    #[must_use]
    pub fn from_path(path: &RepoPath) -> Option<Self> {
        let name = path.file_name()?;
        if name == "Dockerfile" || name.starts_with("Dockerfile.") || name.ends_with(".dockerfile")
        {
            return Some(Self::Dockerfile);
        }
        let lang = match path.extension()? {
            "rs" => Self::Rust,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "tsx" => Self::Tsx,
            "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "py" | "pyi" => Self::Python,
            "sh" | "bash" | "zsh" => Self::Bash,
            "sql" | "psql" => Self::Sql,
            "rhai" => Self::Rhai,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "json" => Self::Json,
            "md" | "markdown" => Self::Markdown,
            "proto" => Self::Proto,
            "go" => Self::Go,
            "java" => Self::Java,
            _ => return None,
        };
        Some(lang)
    }

    /// The lowercase identifier used in node identifiers, such as `rust` or `tsx`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Bash => "bash",
            Self::Sql => "sql",
            Self::Rhai => "rhai",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Dockerfile => "dockerfile",
            Self::Proto => "proto",
            Self::Go => "go",
            Self::Java => "java",
            Self::Other(name) => name,
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = match s {
            "rust" => Self::Rust,
            "typescript" => Self::TypeScript,
            "tsx" => Self::Tsx,
            "javascript" => Self::JavaScript,
            "python" => Self::Python,
            "bash" => Self::Bash,
            "sql" => Self::Sql,
            "rhai" => Self::Rhai,
            "toml" => Self::Toml,
            "yaml" => Self::Yaml,
            "json" => Self::Json,
            "markdown" => Self::Markdown,
            "dockerfile" => Self::Dockerfile,
            "proto" => Self::Proto,
            "go" => Self::Go,
            "java" => Self::Java,
            other
                if !other.is_empty()
                    && other.chars().all(|c| c.is_ascii_lowercase() || c == '-') =>
            {
                Self::Other(other.to_owned())
            }
            other => return Err(ModelError::invalid("Language", other)),
        };
        Ok(lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_from_path() {
        assert_eq!(
            Language::from_path(&RepoPath::new("a/b.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(&RepoPath::new("a/b.tsx")),
            Some(Language::Tsx)
        );
        assert_eq!(
            Language::from_path(&RepoPath::new("Dockerfile")),
            Some(Language::Dockerfile)
        );
        assert_eq!(Language::from_path(&RepoPath::new("a/noext")), None);
        assert_eq!("rust".parse::<Language>().ok(), Some(Language::Rust));
        assert_eq!(
            "zig".parse::<Language>().ok(),
            Some(Language::Other("zig".into()))
        );
    }
}
