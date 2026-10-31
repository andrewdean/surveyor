//! The repository profile: conventions the core cannot infer.
//!
//! A `surveyor.toml` at the repository root selects the SQL dialect, the directories
//! that hold migrations and documentation, and the paths to skip. Every field has a
//! default, so a repository without a profile still indexes.
//!
//! ```toml
//! [repository]
//! id = "causeway"
//!
//! [walk]
//! exclude = ["vendor/**"]
//!
//! [sql]
//! dialect = "postgresql"
//! default_schema = "public"
//!
//! [docs]
//! designs = ["docs/rfd/**/*.md", "docs/adr/**/*.md"]
//! runbooks = ["docs/runbooks/**/*.md"]
//!
//! [scip]
//! indexes = ["index.scip"]
//! ```

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

use crate::ExtractError;

/// The profile file name looked up at the repository root.
pub const PROFILE_FILE: &str = "surveyor.toml";

/// SQL dialects `sqlparser` can parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SqlDialect {
    #[default]
    #[serde(alias = "postgres", alias = "postgresql")]
    PostgreSql,
    MySql,
    Sqlite,
    DuckDb,
    Snowflake,
    BigQuery,
    Ansi,
    Generic,
}

/// Which files to visit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WalkProfile {
    /// Glob patterns to skip, relative to the repository root.
    pub exclude: Vec<String>,
    /// Files larger than this are skipped.
    pub max_file_bytes: u64,
    /// Whether to honor `.gitignore` files.
    pub respect_gitignore: bool,
}

impl Default for WalkProfile {
    fn default() -> Self {
        Self {
            exclude: vec![
                "**/node_modules/**".into(),
                "**/target/**".into(),
                "**/dist/**".into(),
                "**/build/**".into(),
                "**/.venv/**".into(),
                "**/__pycache__/**".into(),
                "**/.git/**".into(),
                "**/.surveyor/**".into(),
                "**/*.min.js".into(),
                "**/*.lock".into(),
                "**/package-lock.json".into(),
            ],
            // Generated bundles and data dumps above this size are noise, not code.
            max_file_bytes: 2 * 1024 * 1024,
            respect_gitignore: true,
        }
    }
}

/// SQL conventions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SqlProfile {
    /// Dialect used for files and literals.
    pub dialect: SqlDialect,
    /// Schema for unqualified table names.
    pub default_schema: String,
    /// Globs that identify migration files; others are still parsed as SQL.
    pub migrations: Vec<String>,
}

impl Default for SqlProfile {
    fn default() -> Self {
        Self {
            dialect: SqlDialect::PostgreSql,
            default_schema: "public".into(),
            migrations: vec!["**/migrations/**/*.sql".into()],
        }
    }
}

/// Documentation conventions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DocsProfile {
    /// Globs of design documents (RFDs, ADRs, design notes).
    pub designs: Vec<String>,
    /// Globs of operational runbooks.
    pub runbooks: Vec<String>,
}

impl Default for DocsProfile {
    fn default() -> Self {
        Self {
            designs: vec![
                "docs/rfd/**/*.md".into(),
                "docs/rfds/**/*.md".into(),
                "docs/adr/**/*.md".into(),
                "docs/adrs/**/*.md".into(),
                "docs/design/**/*.md".into(),
                "docs/designs/**/*.md".into(),
                "docs/architecture/**/*.md".into(),
            ],
            runbooks: vec!["docs/runbooks/**/*.md".into(), "runbooks/**/*.md".into()],
        }
    }
}

/// Precise indexes to merge when they exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ScipProfile {
    /// SCIP index files, relative to the repository root. A missing file is skipped,
    /// so a checkout without the toolchain indexes exactly as before.
    pub indexes: Vec<String>,
}

/// Where the repository keeps its architectural rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RulesProfile {
    /// Directories of `*.rhai` rule files, relative to the repository root.
    pub dirs: Vec<String>,
}

impl Default for RulesProfile {
    fn default() -> Self {
        Self {
            dirs: vec!["surveyor-rules".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
struct RepositorySection {
    id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
struct AdapterSection {
    name: Option<String>,
}

/// Everything a repository tells Surveyor about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Profile {
    repository: RepositorySection,
    /// Walk settings.
    pub walk: WalkProfile,
    /// SQL settings.
    pub sql: SqlProfile,
    /// Documentation settings.
    pub docs: DocsProfile,
    /// Precise index settings.
    pub scip: ScipProfile,
    /// Rule settings.
    pub rules: RulesProfile,
    adapter: AdapterSection,
}

impl Profile {
    /// Parses a profile from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, ExtractError> {
        toml::from_str(text).map_err(|e| ExtractError::config(format!("{PROFILE_FILE}: {e}")))
    }

    /// Loads `surveyor.toml` from `root`, or the defaults when the file is absent.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ExtractError> {
        let path = root.as_ref().join(PROFILE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::from_toml(&text).map_err(|e| e.at(path)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ExtractError::io(e).at(path)),
        }
    }

    /// The repository identifier the profile declares, if any.
    #[must_use]
    pub fn repository_id(&self) -> Option<&str> {
        self.repository.id.as_deref()
    }

    /// The adapter the profile asks for, if any.
    #[must_use]
    pub fn adapter(&self) -> Option<&str> {
        self.adapter.name.as_deref()
    }

    /// Compiled exclusion globs.
    pub fn exclude_set(&self) -> Result<GlobSet, ExtractError> {
        compile_globs(&self.walk.exclude)
    }

    /// Compiled design-document globs.
    pub fn design_set(&self) -> Result<GlobSet, ExtractError> {
        compile_globs(&self.docs.designs)
    }

    /// Compiled runbook globs.
    pub fn runbook_set(&self) -> Result<GlobSet, ExtractError> {
        compile_globs(&self.docs.runbooks)
    }

    /// Compiled migration globs.
    pub fn migration_set(&self) -> Result<GlobSet, ExtractError> {
        compile_globs(&self.sql.migrations)
    }
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet, ExtractError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| ExtractError::config(format!("bad glob {pattern:?}: {e}")))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| ExtractError::config(format!("glob set: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_and_overrides_parse() {
        let p = Profile::from_toml("[repository]\nid = \"x\"\n[sql]\ndialect = \"sqlite\"\n")
            .expect("parses");
        assert_eq!(p.repository_id(), Some("x"));
        assert_eq!(p.sql.dialect, SqlDialect::Sqlite);
        assert_eq!(p.sql.default_schema, "public");
        assert!(
            p.exclude_set()
                .expect("globs")
                .is_match("a/node_modules/b.js")
        );
        assert!(
            p.design_set()
                .expect("globs")
                .is_match("docs/rfd/0001-x.md")
        );
        assert!(Profile::from_toml("[sql]\ndialect = \"nope\"").is_err());
        let p = Profile::from_toml("[scip]\nindexes = [\"index.scip\"]\n").expect("parses");
        assert_eq!(p.scip.indexes, vec!["index.scip".to_owned()]);
    }
}
