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

    /// Guesses the language from a script's first line, or `None` when it is not a
    /// shebang naming a known interpreter: `#!/usr/bin/env bash`, `#!/bin/sh`,
    /// `#!/usr/bin/env python3`, `#!/usr/bin/env node`, `#!/usr/bin/env cwsh` (a Rhai
    /// host).
    #[must_use]
    pub fn from_shebang(first_line: &str) -> Option<Self> {
        let rest = first_line.trim().strip_prefix("#!")?;
        let interpreter = rest
            .split_whitespace()
            .find(|word| !word.ends_with("/env") && !word.starts_with('-'))?
            .rsplit('/')
            .next()?;
        let name = interpreter
            .trim_end_matches(|c: char| c.is_ascii_digit() || c == '.')
            .to_ascii_lowercase();
        let lang = match name.as_str() {
            "bash" | "sh" | "zsh" | "dash" | "ksh" => Self::Bash,
            "python" => Self::Python,
            "node" | "nodejs" | "bun" | "deno" => Self::JavaScript,
            "rhai" | "cwsh" => Self::Rhai,
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
        assert_eq!(
            Language::from_shebang("#!/usr/bin/env bash\n"),
            Some(Language::Bash)
        );
        assert_eq!(Language::from_shebang("#!/bin/sh"), Some(Language::Bash));
        assert_eq!(
            Language::from_shebang("#!/usr/bin/env -S python3 -u"),
            Some(Language::Python)
        );
        assert_eq!(
            Language::from_shebang("#!/usr/bin/env cwsh"),
            Some(Language::Rhai)
        );
        assert_eq!(Language::from_shebang("#!/usr/bin/perl"), None);
        assert_eq!(Language::from_shebang("# not a shebang"), None);
        assert_eq!("rust".parse::<Language>().ok(), Some(Language::Rust));
        assert_eq!(
            "zig".parse::<Language>().ok(),
            Some(Language::Other("zig".into()))
        );
    }
}
