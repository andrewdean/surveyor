//! The change between a base commit and the working tree, as line ranges.
//!
//! `surveyor pr` measures a change against a base: the merge base of `HEAD` and a
//! ref, or the default branch when none is named. The working tree counts, so an
//! uncommitted edit is part of the change, as `update` already treats it. Git
//! supplies the candidate paths (the tree diff from base to `HEAD`, plus whatever
//! `status` reports between `HEAD` and the working tree); the line ranges come from a
//! histogram diff of the base blob against the file on disk.

// Rust guideline compliant 2026-02-21

use std::collections::BTreeSet;

use gix::bstr::ByteSlice;
use gix::diff::blob::sources::byte_lines;
use gix::diff::blob::{Algorithm, Diff, InternedInput};
use surveyor_model::{
    ChangeSet, CommitId, FileChange, LineSpan, RepoPath, SourceUnit, new_side_range,
};

use crate::ExtractError;
use crate::walk::{Repository, looks_binary};

/// The commit a change is measured against, and how it was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base {
    /// The ref that was asked for, or the default branch that was found.
    pub spec: String,
    /// The commit the change is measured against: the merge base of `HEAD` and `spec`,
    /// or `spec` itself when the two histories do not meet.
    pub commit: CommitId,
    /// True when `commit` is a merge base rather than `spec` itself.
    pub merge_base: bool,
}

impl Base {
    /// One phrase for a report: how the base was found.
    #[must_use]
    pub fn describe(&self) -> String {
        if self.merge_base {
            format!("merge base of HEAD and `{}`", self.spec)
        } else {
            format!("`{}`", self.spec)
        }
    }
}

/// Refs tried for the default branch, after `origin/HEAD`.
const DEFAULT_BRANCHES: [&str; 4] = ["origin/main", "origin/master", "main", "master"];

fn git_err(e: impl std::fmt::Display) -> ExtractError {
    ExtractError::git(e.to_string())
}

impl Repository {
    /// Resolves the base a change is measured against.
    ///
    /// With `spec`, the merge base of `HEAD` and that ref; without one, the merge base
    /// of `HEAD` and the default branch, read from `origin/HEAD` or the first of
    /// `origin/main`, `origin/master`, `main`, `master` that exists.
    ///
    /// # Errors
    /// Returns a git error when the root is not a repository, `spec` does not resolve,
    /// or no default branch can be found.
    pub fn resolve_base(&self, spec: Option<&str>) -> Result<Base, ExtractError> {
        let repo = gix::discover(self.root()).map_err(git_err)?;
        let head = repo.head_id().map_err(git_err)?.detach();
        let spec = match spec {
            Some(s) => s.to_owned(),
            None => default_branch(&repo).ok_or_else(|| {
                ExtractError::git(
                    "no default branch found (origin/HEAD, origin/main, origin/master, main, master); pass --base",
                )
            })?,
        };
        let target = repo
            .rev_parse_single(spec.as_str())
            .map_err(|e| ExtractError::git(format!("{spec}: {e}")))?
            .object()
            .map_err(git_err)?
            .peel_to_commit()
            .map_err(|e| ExtractError::git(format!("{spec}: {e}")))?
            .id;
        let merge_base = repo.merge_base(head, target).map_err(git_err)?;
        let (commit, is_merge_base) = match merge_base {
            Some(id) => (id.detach(), id.detach() != target),
            None => (target, false),
        };
        Ok(Base {
            spec,
            commit: CommitId::new(commit.to_hex().to_string()),
            merge_base: is_merge_base,
        })
    }

    /// The files and line ranges that differ between `base` and the working tree.
    ///
    /// Candidates are the paths the tree diff from `base` to `HEAD` names plus the
    /// paths `status` reports (staged, modified, untracked; ignored files are not).
    /// Each candidate is then compared as content, so a commit that touches a file
    /// without changing it, or an edit that was reverted, contributes nothing. A file
    /// that is binary on either side is listed with no ranges, so it maps to its file
    /// node. Directories and submodules are skipped.
    ///
    /// # Errors
    /// Returns a git error when the root is not a repository or `base` is not a
    /// commit it knows, and an I/O error when a candidate cannot be read.
    pub fn changes_against(&self, base: &CommitId) -> Result<ChangeSet, ExtractError> {
        let repo = gix::discover(self.root()).map_err(git_err)?;
        let base_tree = repo
            .rev_parse_single(base.as_str())
            .map_err(|e| ExtractError::git(format!("{base}: {e}")))?
            .object()
            .map_err(git_err)?
            .peel_to_tree()
            .map_err(git_err)?;
        let mut candidates: BTreeSet<String> = BTreeSet::new();
        if let Ok(head_tree) = repo.head_tree() {
            let changes = repo
                .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)
                .map_err(git_err)?;
            for change in &changes {
                candidates.insert(change.location().to_string());
                if let gix::object::tree::diff::ChangeDetached::Rewrite {
                    source_location, ..
                } = change
                {
                    candidates.insert(source_location.to_string());
                }
            }
        }
        let status = repo
            .status(gix::progress::Discard)
            .map_err(git_err)?
            .untracked_files(gix::status::UntrackedFiles::Files)
            .into_iter(None)
            .map_err(git_err)?;
        for item in status {
            let item = item.map_err(git_err)?;
            candidates.insert(item.location().to_string());
        }

        let mut set = ChangeSet::default();
        for path in candidates {
            let rel = RepoPath::new(&path);
            let old = blob_at(&base_tree, &path)?;
            let full = self.root().join(rel.as_str());
            let new = match std::fs::metadata(&full) {
                Ok(m) if m.is_file() => {
                    Some(std::fs::read(&full).map_err(|e| ExtractError::io(e).at(&full))?)
                }
                _ => None,
            };
            match (old, new) {
                (None, None) => {}
                (Some(_), None) => set.deleted.push(rel),
                (old, Some(new)) => {
                    let added = old.is_none();
                    if old.as_deref() == Some(new.as_slice()) {
                        continue;
                    }
                    let hunks = if looks_binary(&new) || old.as_deref().is_some_and(looks_binary) {
                        Vec::new()
                    } else {
                        line_hunks(old.as_deref().unwrap_or_default(), &new)
                    };
                    set.files.push(FileChange {
                        path: rel,
                        added,
                        hunks,
                    });
                }
            }
        }
        Ok(set.normalized())
    }

    /// The content of `path` at `base` as a source unit, or `None` when the file did not
    /// exist there or is binary.
    ///
    /// # Errors
    /// Returns a git error when the root is not a repository or `base` is unknown.
    pub fn unit_at(
        &self,
        base: &CommitId,
        path: &RepoPath,
    ) -> Result<Option<SourceUnit>, ExtractError> {
        let repo = gix::discover(self.root()).map_err(git_err)?;
        let tree = repo
            .rev_parse_single(base.as_str())
            .map_err(|e| ExtractError::git(format!("{base}: {e}")))?
            .object()
            .map_err(git_err)?
            .peel_to_tree()
            .map_err(git_err)?;
        Ok(
            blob_at(&tree, path.as_str())?
                .and_then(|bytes| self.unit_of(path, base.clone(), bytes)),
        )
    }
}

/// `origin/HEAD`'s target as `origin/<branch>`, else the first known default name.
fn default_branch(repo: &gix::Repository) -> Option<String> {
    let from_origin_head = repo
        .find_reference("refs/remotes/origin/HEAD")
        .ok()
        .and_then(|r| match r.target() {
            gix::refs::TargetRef::Symbolic(name) => {
                Some(name.as_bstr().to_str_lossy().into_owned())
            }
            gix::refs::TargetRef::Object(_) => None,
        })
        .map(|name| {
            name.strip_prefix("refs/remotes/")
                .map_or(name.clone(), str::to_owned)
        });
    from_origin_head.or_else(|| {
        DEFAULT_BRANCHES
            .into_iter()
            .find(|s| repo.rev_parse_single(*s).is_ok())
            .map(str::to_owned)
    })
}

/// The bytes of the blob at `path` in `tree`; `None` for a missing entry, a directory
/// or a submodule.
fn blob_at(tree: &gix::Tree<'_>, path: &str) -> Result<Option<Vec<u8>>, ExtractError> {
    let Some(entry) = tree.lookup_entry_by_path(path).map_err(git_err)? else {
        return Ok(None);
    };
    if !entry.mode().is_blob() {
        return Ok(None);
    }
    let object = entry.object().map_err(git_err)?;
    Ok(Some(object.detach().data))
}

/// The new-side line ranges that differ between `old` and `new`, by a histogram diff
/// over lines with git's slider heuristics. A pure deletion is the one line it sat
/// after (line 1 when it was at the top), so the declaration around it is changed.
#[must_use]
pub fn line_hunks(old: &[u8], new: &[u8]) -> Vec<LineSpan> {
    let input = InternedInput::new(byte_lines(old), byte_lines(new));
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    diff.hunks()
        .map(|h| {
            let count = h.after.end - h.after.start;
            if count == 0 {
                new_side_range(h.after.start, 0)
            } else {
                new_side_range(h.after.start + 1, count)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(hunks: &[LineSpan]) -> Vec<String> {
        hunks.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn hunks_name_new_side_lines() {
        let old = b"a\nb\nc\nd\ne\n";
        // b changed, a line added after d, e removed.
        let new = b"a\nB\nc\nd\nd2\n";
        assert_eq!(spans(&line_hunks(old, new)), ["2", "5"]);
        // Pure deletion in the middle maps to the line before it.
        assert_eq!(spans(&line_hunks(b"a\nb\nc\n", b"a\nc\n")), ["1"]);
        // Deletion at the top maps to line 1.
        assert_eq!(spans(&line_hunks(b"a\nb\n", b"b\n")), ["1"]);
        // A new file is one range over all its lines; identical content is nothing.
        assert_eq!(spans(&line_hunks(b"", b"x\ny\nz\n")), ["1-3"]);
        assert!(line_hunks(b"same\n", b"same\n").is_empty());
    }

    #[test]
    fn a_repository_diffs_base_against_the_working_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.com")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.com")
                .status()
                .expect("git runs");
            assert!(status.success(), "git {args:?}");
        };
        git(&["init", "-q", "-b", "main"]);
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "fn a() {}\nfn b() {}\nfn gone() {}\n",
        )
        .unwrap();
        std::fs::write(root.join("src/old.rs"), "fn old() {}\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "base"]);
        // One committed change, one staged, one untracked, one deletion.
        std::fs::write(root.join("src/lib.rs"), "fn a() {}\nfn b() { 1 }\n").unwrap();
        git(&["commit", "-q", "-am", "edit b, drop gone"]);
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"1\"\n",
        )
        .unwrap();
        git(&["add", "Cargo.toml"]);
        std::fs::write(root.join("src/new.rs"), "fn new() {}\n").unwrap();
        std::fs::remove_file(root.join("src/old.rs")).unwrap();

        let repo = Repository::open(root, None).expect("opens");
        let base = repo.resolve_base(Some("HEAD~1")).expect("base");
        assert!(
            !base.merge_base,
            "HEAD~1 is an ancestor, so it is the base itself"
        );
        let set = repo.changes_against(&base.commit).expect("changes");
        let files: Vec<(String, bool, Vec<String>)> = set
            .files
            .iter()
            .map(|f| (f.path.to_string(), f.added, spans(&f.hunks)))
            .collect();
        assert_eq!(
            files,
            vec![
                ("Cargo.toml".to_owned(), false, vec!["3".to_owned()]),
                ("src/lib.rs".to_owned(), false, vec!["2".to_owned()]),
                ("src/new.rs".to_owned(), true, vec!["1".to_owned()]),
            ]
        );
        assert_eq!(set.deleted, vec![RepoPath::new("src/old.rs")]);
        // The base version is readable for removed-symbol extraction.
        let unit = repo
            .unit_at(&base.commit, &RepoPath::new("src/lib.rs"))
            .expect("unit")
            .expect("existed at base");
        assert!(unit.content.ends_with(b"fn gone() {}\n"));
        assert!(
            repo.unit_at(&base.commit, &RepoPath::new("src/new.rs"))
                .unwrap()
                .is_none()
        );
        // The default base is the branch itself here (no origin), so the change is empty.
        let default = repo.resolve_base(None).expect("default");
        assert_eq!(default.spec, "main");
        assert!(repo.resolve_base(Some("no-such-ref")).is_err());
    }
}
