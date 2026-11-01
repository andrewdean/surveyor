//! What the linker consumes.

use surveyor_model::{CommitId, ContentHash, FactSet, Language, RepoId, RepoPath};

/// One file the walk visited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Repository-relative path.
    pub path: RepoPath,
    /// Language, if detected.
    pub language: Option<Language>,
    /// Hash of the file content.
    pub content_hash: ContentHash,
    /// Size in bytes.
    pub bytes: u64,
    /// Number of lines.
    pub lines: u64,
}

impl Default for FileEntry {
    fn default() -> Self {
        Self {
            path: RepoPath::new(""),
            language: None,
            content_hash: ContentHash::of(b""),
            bytes: 0,
            lines: 0,
        }
    }
}

/// Knobs that change how aggressively names are matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOptions {
    /// A reference with more candidates than this stays unresolved as ambiguous.
    pub max_candidates: usize,
    /// Whether type mentions in signatures produce `USES_TYPE` edges.
    pub link_type_uses: bool,
}

impl Default for LinkOptions {
    fn default() -> Self {
        // Five candidates still gives each edge a 0.2 confidence an agent can act on;
        // beyond that the name is effectively a common word.
        Self {
            max_candidates: 5,
            link_type_uses: true,
        }
    }
}

/// Everything extraction produced for one repository.
#[derive(Debug, Clone)]
pub struct LinkInput {
    /// Repository being linked.
    pub repo: RepoId,
    /// Commit the files were read at.
    pub commit: CommitId,
    /// Files visited, with or without facts.
    pub files: Vec<FileEntry>,
    /// Facts per extractor per file.
    pub fact_sets: Vec<FactSet>,
    /// Matching knobs.
    pub options: LinkOptions,
}

impl LinkInput {
    /// An empty input for `repo` at `commit`.
    #[must_use]
    pub fn new(repo: RepoId, commit: CommitId) -> Self {
        Self {
            repo,
            commit,
            files: Vec::new(),
            fact_sets: Vec::new(),
            options: LinkOptions::default(),
        }
    }

    /// Adds files and returns the input.
    #[must_use]
    pub fn with_files(mut self, files: impl IntoIterator<Item = FileEntry>) -> Self {
        self.files.extend(files);
        self
    }

    /// Adds fact sets and returns the input.
    #[must_use]
    pub fn with_fact_sets(mut self, sets: impl IntoIterator<Item = FactSet>) -> Self {
        self.fact_sets.extend(sets);
        self
    }

    /// Sets the options and returns the input.
    #[must_use]
    pub fn with_options(mut self, options: LinkOptions) -> Self {
        self.options = options;
        self
    }
}
