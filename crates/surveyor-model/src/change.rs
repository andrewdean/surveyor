//! A change to a repository as line ranges: what a diff touched, before any graph.
//!
//! [`ChangeSet`] is the input `surveyor pr` maps onto nodes. It is produced from git
//! (the extract crate diffs a base commit against the working tree) or parsed from a
//! unified diff a harness already has, and it carries no graph knowledge: a file, the
//! line ranges on the new side that changed, and the files that are gone.

// Rust guideline compliant 2026-02-21

use serde::{Deserialize, Serialize};

use crate::ids::RepoPath;
use crate::span::LineSpan;

/// The line ranges a change touched in one file, on the new side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    /// Repository-relative path on the new side.
    pub path: RepoPath,
    /// True when the file did not exist on the old side.
    pub added: bool,
    /// Changed ranges, ascending and disjoint. A pure deletion is the one line where
    /// the removed lines used to be, so the declaration around it counts as changed.
    pub hunks: Vec<LineSpan>,
}

/// Every file a change touched.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChangeSet {
    /// Files present on the new side with their changed ranges, by path.
    pub files: Vec<FileChange>,
    /// Files present on the old side only, by path.
    pub deleted: Vec<RepoPath>,
}

impl ChangeSet {
    /// True when nothing changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.deleted.is_empty()
    }

    /// Number of files touched, deletions included.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len() + self.deleted.len()
    }

    /// Sorts the files by path and the ranges inside each file.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.files.sort_by(|a, b| a.path.cmp(&b.path));
        for f in &mut self.files {
            f.hunks.sort_by_key(|h| (h.start, h.end));
        }
        self.deleted.sort();
        self.deleted.dedup();
        self
    }

    /// Reads a unified diff (`git diff`, `diff -u`) into the ranges it changes.
    ///
    /// Only the headers matter: `+++ b/<path>` names the new side, `/dev/null` there
    /// marks a deletion, and each `@@ -a,b +c,d @@` names the new-side range `c..c+d`.
    /// A hunk with `d` of zero is a pure deletion and maps to line `c`. Prefixes `a/`
    /// and `b/` are stripped, as `git diff --no-prefix` would have left them off.
    ///
    /// # Examples
    ///
    /// ```
    /// use surveyor_model::ChangeSet;
    ///
    /// let diff = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,4 @@\n a\n+b\n c\n d\n";
    /// let set = ChangeSet::parse_unified(diff);
    /// assert_eq!(set.files[0].path.as_str(), "src/lib.rs");
    /// assert_eq!(set.files[0].hunks[0].to_string(), "1-4");
    /// ```
    #[must_use]
    pub fn parse_unified(text: &str) -> Self {
        let mut set = Self::default();
        let mut old_side: Option<String> = None;
        let mut current: Option<FileChange> = None;
        let flush = |current: &mut Option<FileChange>, set: &mut Self| {
            if let Some(f) = current.take() {
                set.files.push(f);
            }
        };
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("--- ") {
                flush(&mut current, &mut set);
                old_side = Some(diff_path(rest));
            } else if let Some(rest) = line.strip_prefix("+++ ") {
                let new_side = diff_path(rest);
                if new_side == "/dev/null" {
                    if let Some(old) = old_side.take().filter(|p| p != "/dev/null") {
                        set.deleted.push(RepoPath::new(old));
                    }
                    current = None;
                } else {
                    let added = old_side.as_deref() == Some("/dev/null");
                    current = Some(FileChange {
                        path: RepoPath::new(new_side),
                        added,
                        hunks: Vec::new(),
                    });
                }
            } else if let (Some(f), Some(range)) = (current.as_mut(), hunk_range(line)) {
                f.hunks.push(range);
            }
        }
        flush(&mut current, &mut set);
        set.normalized()
    }
}

/// The path in a `---`/`+++` header: the first field, without `a/` or `b/`.
fn diff_path(rest: &str) -> String {
    let field = rest.split('\t').next().unwrap_or(rest).trim();
    if field == "/dev/null" {
        return field.to_owned();
    }
    field
        .strip_prefix("a/")
        .or_else(|| field.strip_prefix("b/"))
        .unwrap_or(field)
        .to_owned()
}

/// The new-side range of a `@@ -a,b +c,d @@` header, or `None` for any other line.
fn hunk_range(line: &str) -> Option<LineSpan> {
    let rest = line.strip_prefix("@@ ")?;
    let plus = rest.split(' ').find_map(|f| f.strip_prefix('+'))?;
    let (start, count) = match plus.split_once(',') {
        Some((s, c)) => (s.parse::<u32>().ok()?, c.parse::<u32>().ok()?),
        None => (plus.parse::<u32>().ok()?, 1),
    };
    Some(new_side_range(start, count))
}

/// Lines `start..start+count` on the new side; a zero count is the one line at `start`
/// (or line 1 when the deletion emptied the file).
#[must_use]
pub fn new_side_range(start: u32, count: u32) -> LineSpan {
    if count == 0 {
        let at = start.max(1);
        LineSpan::new(at, at)
    } else {
        LineSpan::new(start, start + count - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modified_added_and_deleted_files() {
        let diff = "\
diff --git a/crates/outbox/src/relay.rs b/crates/outbox/src/relay.rs
--- a/crates/outbox/src/relay.rs
+++ b/crates/outbox/src/relay.rs
@@ -20,6 +20,7 @@ impl Relay {
 fn drain
+    batch
@@ -40,3 +41,0 @@
-gone
-gone
-gone
diff --git a/new.rs b/new.rs
--- /dev/null
+++ b/new.rs
@@ -0,0 +1,2 @@
+a
+b
diff --git a/old.rs b/old.rs
--- a/old.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-a
-b
";
        let set = ChangeSet::parse_unified(diff);
        assert_eq!(set.files.len(), 2);
        let relay = &set.files[0];
        assert_eq!(relay.path.as_str(), "crates/outbox/src/relay.rs");
        assert!(!relay.added);
        assert_eq!(
            relay
                .hunks
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["20-26", "41"]
        );
        let new = &set.files[1];
        assert!(new.added);
        assert_eq!(new.hunks[0].to_string(), "1-2");
        assert_eq!(set.deleted, vec![RepoPath::new("old.rs")]);
        assert_eq!(set.file_count(), 3);
    }

    #[test]
    fn a_header_without_a_count_is_one_line() {
        assert_eq!(hunk_range("@@ -3 +4 @@"), Some(LineSpan::new(4, 4)));
        assert_eq!(new_side_range(10, 3), LineSpan::new(10, 12));
        assert_eq!(new_side_range(0, 0), LineSpan::new(1, 1));
        assert_eq!(hunk_range("@@ -3,0 +0,0 @@"), Some(LineSpan::new(1, 1)));
        assert_eq!(hunk_range("+++ not a hunk"), None);
    }
}
