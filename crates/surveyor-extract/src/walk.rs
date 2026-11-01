//! Walking a checkout into source units, with git coordinates from `gix`.
//!
//! [`Repository::open`] locates the git repository (or falls back to a plain
//! directory), records the `HEAD` commit, and [`Repository::units`] walks the tree
//! honoring `.gitignore` and the profile's exclusions. Hidden directories are skipped
//! unless the profile's `[walk] include` names them. Every file that has a known
//! language or is a recognized manifest becomes a [`SourceUnit`]; binary and oversized
//! files are skipped.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use globset::GlobSet;
use ignore::WalkBuilder;
use surveyor_model::{CommitId, ContentHash, Language, RepoId, RepoPath, SourceUnit};

use crate::ExtractError;
use crate::profile::Profile;

/// Commit identifier used when a directory is not a git checkout.
pub const NO_COMMIT: &str = "worktree";

/// File names that are extracted although their extension carries no language.
const MANIFEST_NAMES: &[&str] = &["Cargo.toml", "package.json", "pyproject.toml", "Dockerfile"];

/// How the changed files were found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeSource {
    /// `git diff` between the indexed commit and `HEAD` named the candidates, each then
    /// confirmed by content hash.
    Git { since: CommitId },
    /// Every indexable file was hashed and compared with the recorded hash.
    Hashes,
}

impl ChangeSource {
    /// The short name used in reports: `git` or `hashes`.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Git { .. } => "git",
            Self::Hashes => "hashes",
        }
    }
}

/// What changed in a checkout since its index was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChanges {
    /// How the candidates were found.
    pub source: ChangeSource,
    /// Indexed files whose content differs.
    pub modified: Vec<RepoPath>,
    /// Indexable files the index does not know.
    pub added: Vec<RepoPath>,
    /// Indexed files that are gone, binary or no longer indexable.
    pub deleted: Vec<RepoPath>,
    /// Candidates whose hash still matches.
    pub unchanged: usize,
}

impl FileChanges {
    /// True when nothing changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modified.is_empty() && self.added.is_empty() && self.deleted.is_empty()
    }

    /// Files that must be read again: the modified and the added ones.
    pub fn to_extract(&self) -> impl Iterator<Item = &RepoPath> {
        self.modified.iter().chain(&self.added)
    }

    /// Every path the change touches.
    pub fn touched(&self) -> impl Iterator<Item = &RepoPath> {
        self.to_extract().chain(&self.deleted)
    }
}

/// A checkout Surveyor indexes.
#[derive(Debug, Clone)]
pub struct Repository {
    root: PathBuf,
    id: RepoId,
    commit: CommitId,
    dirty: bool,
}

impl Repository {
    /// Opens the checkout at `root`.
    ///
    /// The repository identifier comes from `id`, else the profile, else the directory
    /// name. The commit is `HEAD` when `root` is inside a git repository, else
    /// [`NO_COMMIT`].
    pub fn open(root: impl AsRef<Path>, id: Option<RepoId>) -> Result<Self, ExtractError> {
        let profile = Profile::load(root.as_ref())?;
        Self::open_with(root, id, &profile)
    }

    /// Opens the checkout at `root` under a profile loaded elsewhere.
    ///
    /// The profile only contributes the repository identifier here; pass the same
    /// profile to the walks.
    pub fn open_with(
        root: impl AsRef<Path>,
        id: Option<RepoId>,
        profile: &Profile,
    ) -> Result<Self, ExtractError> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|e| ExtractError::io(e).at(root.as_ref()))?;
        let id = id
            .or_else(|| profile.repository_id().map(RepoId::new))
            .unwrap_or_else(|| {
                RepoId::new(root.file_name().and_then(|n| n.to_str()).unwrap_or("repo"))
            });
        let (commit, dirty) = match gix::discover(&root) {
            Ok(repo) => {
                let head = repo.head_id().map_or_else(
                    |_| CommitId::new(NO_COMMIT),
                    |id| CommitId::new(id.to_hex().to_string()),
                );
                let dirty = repo.is_dirty().unwrap_or(false);
                (head, dirty)
            }
            Err(_) => (CommitId::new(NO_COMMIT), false),
        };
        Ok(Self {
            root,
            id,
            commit,
            dirty,
        })
    }

    /// The canonical root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The repository identifier.
    #[must_use]
    pub fn id(&self) -> &RepoId {
        &self.id
    }

    /// The `HEAD` commit the files were read at, or [`NO_COMMIT`].
    #[must_use]
    pub fn commit(&self) -> &CommitId {
        &self.commit
    }

    /// True when the working tree differs from `HEAD`.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Loads the repository profile from the root.
    pub fn profile(&self) -> Result<Profile, ExtractError> {
        Profile::load(&self.root)
    }

    /// Compares the checkout with the files recorded at index time.
    ///
    /// The walk always runs, so a file the profile newly includes or excludes shows up
    /// as added or deleted. When both the recorded index and the checkout are clean
    /// commits, git names the paths whose content may differ; otherwise every walked
    /// or known path is hashed. Either way a candidate counts as changed only when its
    /// content hash differs from `known`, so a commit that touches a file without
    /// changing it costs nothing.
    pub fn changes_since(
        &self,
        profile: &Profile,
        since: Option<&CommitId>,
        since_dirty: bool,
        known: &HashMap<RepoPath, ContentHash>,
    ) -> Result<FileChanges, ExtractError> {
        let walked: BTreeSet<RepoPath> = self.paths(profile)?.into_iter().collect();
        let clean = !self.dirty && !since_dirty && self.commit.as_str() != NO_COMMIT;
        let from_git = since
            .filter(|s| clean && s.as_str() != NO_COMMIT)
            .and_then(|s| {
                self.changed_paths_since(s)
                    .map_err(|e| tracing::info!(error = %e, "git diff unavailable; hashing"))
                    .ok()
                    .map(|paths| (s.clone(), paths))
            });
        // Paths the walk sees but the index does not, and the reverse, change whatever
        // git says: the profile or the tree moved them in or out.
        let moved = walked
            .iter()
            .filter(|p| !known.contains_key(*p))
            .chain(known.keys().filter(|p| !walked.contains(*p)))
            .cloned();
        let (source, candidates): (ChangeSource, BTreeSet<RepoPath>) = match from_git {
            Some((since, paths)) => (
                ChangeSource::Git { since },
                paths
                    .into_iter()
                    .filter(|p| known.contains_key(p) || walked.contains(p))
                    .chain(moved)
                    .collect(),
            ),
            None => (
                ChangeSource::Hashes,
                walked
                    .iter()
                    .cloned()
                    .chain(known.keys().cloned())
                    .collect(),
            ),
        };
        let mut changes = FileChanges {
            source,
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            unchanged: 0,
        };
        for path in candidates {
            let present = walked
                .contains(&path)
                .then(|| self.unit(&path))
                .transpose()?
                .flatten()
                .map(|u| u.content_hash);
            match (present, known.get(&path)) {
                (Some(now), Some(then)) if now == *then => changes.unchanged += 1,
                (Some(_), Some(_)) => changes.modified.push(path),
                (Some(_), None) => changes.added.push(path),
                (None, Some(_)) => changes.deleted.push(path),
                (None, None) => {}
            }
        }
        Ok(changes)
    }

    /// Lists the repository-relative paths of files worth extracting.
    ///
    /// Hidden files and directories are pruned unless the profile's `[walk] include`
    /// names them or one of their ancestors or descendants, so `.github/workflows`
    /// walks `.github` and `workflows` but not `.github/ISSUE_TEMPLATE`.
    pub fn paths(&self, profile: &Profile) -> Result<Vec<RepoPath>, ExtractError> {
        let exclude = profile.exclude_set()?;
        let mut builder = WalkBuilder::new(&self.root);
        builder
            .hidden(false)
            .git_ignore(profile.walk.respect_gitignore)
            .git_global(profile.walk.respect_gitignore)
            .git_exclude(profile.walk.respect_gitignore)
            .follow_links(false)
            .max_filesize(Some(profile.walk.max_file_bytes));
        let root = self.root.clone();
        let include = profile.walk.include.clone();
        // The predicate prunes hidden directories so `.git` is never descended; the
        // walker skips it for files once a size limit is set, so files are checked
        // again below.
        let (prune_root, prune_include) = (root.clone(), include.clone());
        builder.filter_entry(move |entry| {
            entry.depth() == 0
                || entry
                    .path()
                    .strip_prefix(&prune_root)
                    .is_ok_and(|rel| !hidden_outside(&rel.to_string_lossy(), &prune_include))
        });
        let mut paths: Vec<RepoPath> = builder
            .build()
            .filter_map(|entry| {
                entry
                    .map_err(|err| tracing::debug!(error = %err, "walk entry skipped"))
                    .ok()
            })
            .filter(|entry| entry.file_type().is_some_and(|t| t.is_file()))
            .filter_map(|entry| {
                entry
                    .path()
                    .strip_prefix(&root)
                    .ok()
                    .map(|rel| RepoPath::new(rel.to_string_lossy()))
            })
            .filter(|rel| {
                !hidden_outside(rel.as_str(), &include)
                    && !exclude.is_match(rel.as_str())
                    && is_extractable(rel)
            })
            .collect();
        paths.sort();
        Ok(paths)
    }

    /// Reads one file into a source unit, or `None` when it is binary.
    pub fn unit(&self, path: &RepoPath) -> Result<Option<SourceUnit>, ExtractError> {
        let full = self.root.join(path.as_str());
        let content = std::fs::read(&full).map_err(|e| ExtractError::io(e).at(&full))?;
        if looks_binary(&content) {
            return Ok(None);
        }
        Ok(Some(SourceUnit::new(
            self.id.clone(),
            path.clone(),
            self.commit.clone(),
            content,
        )))
    }

    /// Reads every extractable file into source units.
    pub fn units(&self, profile: &Profile) -> Result<Vec<SourceUnit>, ExtractError> {
        self.paths(profile)?
            .iter()
            .map(|path| self.unit(path))
            .filter_map(Result::transpose)
            .collect()
    }

    /// Paths changed between `since` and `HEAD`, for incremental update.
    ///
    /// Renames report both the old and the new path.
    ///
    /// # Errors
    /// Returns a git error when the directory is not a repository or `since` is not a
    /// commit it knows.
    pub fn changed_paths_since(&self, since: &CommitId) -> Result<Vec<RepoPath>, ExtractError> {
        let repo = gix::discover(&self.root).map_err(|e| ExtractError::git(e.to_string()))?;
        let old = repo
            .rev_parse_single(since.as_str())
            .map_err(|e| ExtractError::git(format!("{since}: {e}")))?
            .object()
            .map_err(|e| ExtractError::git(e.to_string()))?
            .peel_to_tree()
            .map_err(|e| ExtractError::git(e.to_string()))?;
        let new = repo
            .head_tree()
            .map_err(|e| ExtractError::git(e.to_string()))?;
        let changes = repo
            .diff_tree_to_tree(Some(&old), Some(&new), None)
            .map_err(|e| ExtractError::git(e.to_string()))?;
        let mut paths: Vec<RepoPath> = Vec::new();
        for change in &changes {
            let location = change.location().to_string();
            paths.push(RepoPath::new(location));
            if let gix::object::tree::diff::ChangeDetached::Rewrite {
                source_location, ..
            } = change
            {
                paths.push(RepoPath::new(source_location.to_string()));
            }
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

/// True when `rel` has a dot-component and no entry of `include` is it, contains it or
/// lies under it. Pure; `rel` is repository-relative with `/` separators.
fn hidden_outside(rel: &str, include: &[String]) -> bool {
    let hidden = rel
        .split('/')
        .any(|part| part.starts_with('.') && part != "." && part != "..");
    hidden
        && !include.iter().any(|entry| {
            let entry = entry.trim_matches('/');
            rel == entry
                || rel
                    .strip_prefix(entry)
                    .is_some_and(|rest| rest.starts_with('/'))
                || entry
                    .strip_prefix(rel)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

/// True for files with a known language or a recognized manifest name.
#[must_use]
pub(crate) fn is_extractable(path: &RepoPath) -> bool {
    if path
        .file_name()
        .is_some_and(|n| MANIFEST_NAMES.contains(&n))
    {
        return true;
    }
    Language::from_path(path).is_some()
}

/// A NUL byte in the first 8 KiB marks a file as binary, as git does.
fn looks_binary(content: &[u8]) -> bool {
    const PROBE: usize = 8 * 1024;
    content.iter().take(PROBE).any(|b| *b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_a_plain_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(dir.path().join("src/lib.rs"), "fn a() {}").expect("write");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").expect("write");
        std::fs::write(dir.path().join("notes.txt"), "ignored").expect("write");
        std::fs::write(dir.path().join("blob.rs"), b"\0\0binary").expect("write");
        let repo = Repository::open(dir.path(), None).expect("opens");
        assert_eq!(repo.commit().as_str(), NO_COMMIT);
        let profile = repo.profile().expect("profile");
        let paths = repo.paths(&profile).expect("paths");
        assert_eq!(
            paths.iter().map(RepoPath::as_str).collect::<Vec<_>>(),
            vec!["Cargo.toml", "blob.rs", "src/lib.rs"]
        );
        let units = repo.units(&profile).expect("units");
        assert_eq!(units.len(), 2, "binary file is dropped");
    }

    #[test]
    fn changes_are_found_by_hash_without_git() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(dir.path().join("src/a.rs"), "fn a() {}").expect("write");
        std::fs::write(dir.path().join("src/b.rs"), "fn b() {}").expect("write");
        let repo = Repository::open(dir.path(), None).expect("opens");
        let profile = repo.profile().expect("profile");
        let known: HashMap<RepoPath, ContentHash> = [
            (RepoPath::new("src/a.rs"), ContentHash::of(b"fn a() {}")),
            (RepoPath::new("src/b.rs"), ContentHash::of(b"old")),
            (RepoPath::new("src/gone.rs"), ContentHash::of(b"x")),
        ]
        .into_iter()
        .collect();
        std::fs::write(dir.path().join("src/c.rs"), "fn c() {}").expect("write");
        let changes = repo
            .changes_since(&profile, Some(&CommitId::new("abc")), false, &known)
            .expect("changes");
        assert_eq!(changes.source, ChangeSource::Hashes);
        assert_eq!(changes.unchanged, 1);
        assert_eq!(changes.modified, vec![RepoPath::new("src/b.rs")]);
        assert_eq!(changes.added, vec![RepoPath::new("src/c.rs")]);
        assert_eq!(changes.deleted, vec![RepoPath::new("src/gone.rs")]);
        assert_eq!(changes.touched().count(), 3);
    }

    fn tree(dir: &Path, files: &[(&str, &str)]) {
        for (path, text) in files {
            let full = dir.join(path);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
            std::fs::write(full, text).expect("write");
        }
    }

    fn with_include(include: &[&str]) -> Profile {
        let mut profile = Profile::default();
        profile.walk.include = include.iter().map(|s| (*s).to_owned()).collect();
        profile
    }

    #[test]
    fn hidden_paths_are_pruned_unless_included() {
        assert!(hidden_outside(".mise/tasks/a.rhai", &[]));
        assert!(!hidden_outside("src/lib.rs", &[]));
        let include = [".mise".to_owned(), ".github/workflows".to_owned()];
        assert!(!hidden_outside(".mise", &include));
        assert!(!hidden_outside(".mise/tasks/a.rhai", &include));
        assert!(!hidden_outside(".github", &include), "ancestor of an entry");
        assert!(!hidden_outside(".github/workflows/ci.yml", &include));
        assert!(hidden_outside(".github/ISSUE_TEMPLATE/bug.md", &include));
        assert!(
            hidden_outside(".misery/x.rhai", &include),
            "prefix is not a path"
        );
        assert!(hidden_outside(".git/config", &include));
    }

    #[test]
    fn hidden_directories_walk_only_when_included() {
        let dir = tempfile::tempdir().expect("tempdir");
        tree(
            dir.path(),
            &[
                ("src/lib.rs", "fn a() {}"),
                (".mise/tasks/deploy.rhai", "fn deploy() {}"),
                (".github/workflows/ci.yml", "on: push\n"),
                (".github/ISSUE_TEMPLATE/bug.md", "# bug\n"),
                (".idea/notes.md", "# ide\n"),
                (".tool.toml", "a = 1\n"),
            ],
        );
        let repo = Repository::open(dir.path(), None).expect("opens");
        let names = |profile: &Profile| -> Vec<String> {
            repo.paths(profile)
                .expect("paths")
                .iter()
                .map(|p| p.as_str().to_owned())
                .collect()
        };
        assert_eq!(names(&Profile::default()), vec!["src/lib.rs"]);
        assert_eq!(
            names(&with_include(&[".mise", ".github/workflows"])),
            vec![
                ".github/workflows/ci.yml",
                ".mise/tasks/deploy.rhai",
                "src/lib.rs"
            ]
        );
    }

    #[test]
    fn a_profile_change_shows_as_added_and_deleted_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        tree(
            dir.path(),
            &[
                ("src/lib.rs", "fn a() {}"),
                ("vendor/dep.rs", "fn v() {}"),
                (".mise/tasks/deploy.rhai", "fn deploy() {}"),
            ],
        );
        let repo = Repository::open(dir.path(), None).expect("opens");
        let known: HashMap<RepoPath, ContentHash> = [
            (RepoPath::new("src/lib.rs"), ContentHash::of(b"fn a() {}")),
            (
                RepoPath::new("vendor/dep.rs"),
                ContentHash::of(b"fn v() {}"),
            ),
        ]
        .into_iter()
        .collect();
        let mut profile = with_include(&[".mise"]);
        profile.walk.exclude.push("vendor/**".to_owned());
        let changes = repo
            .changes_since(&profile, Some(&CommitId::new("abc")), false, &known)
            .expect("changes");
        assert_eq!(changes.unchanged, 1);
        assert!(changes.modified.is_empty());
        assert_eq!(
            changes.added,
            vec![RepoPath::new(".mise/tasks/deploy.rhai")]
        );
        assert_eq!(
            changes.deleted,
            vec![RepoPath::new("vendor/dep.rs")],
            "a newly excluded file leaves the index"
        );
    }
}
