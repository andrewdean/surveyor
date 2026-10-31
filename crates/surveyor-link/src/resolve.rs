//! The symbol index and candidate ranking: pure decisions, no graph mutation.

use std::collections::{HashMap, HashSet};

use surveyor_model::{CallForm, Confidence, Node, NodeKind, RepoPath, UnresolvedReason};

/// Where a candidate sits relative to the reference site; lower is closer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Tier {
    SameFile,
    SamePackage,
    /// In a package the referencing file imports or its package depends on.
    Imported,
    SameRepo,
}

/// In-memory index over the nodes built so far.
#[derive(Debug, Default)]
pub(crate) struct SymbolIndex {
    /// Node index by short name.
    by_name: HashMap<String, Vec<usize>>,
    /// Node index by `sql:` qualified table name (`schema.table`).
    tables_by_qualified: HashMap<String, usize>,
    /// Node indexes by bare table name.
    tables_by_name: HashMap<String, Vec<usize>>,
    /// Owning package node id per file.
    pub(crate) package_of_file: HashMap<RepoPath, String>,
    /// Package node ids a file imports, by file.
    pub(crate) imports_of_file: HashMap<RepoPath, HashSet<String>>,
    /// Package node ids a package depends on, by package node id.
    pub(crate) deps_of_package: HashMap<String, HashSet<String>>,
}

impl SymbolIndex {
    pub(crate) fn insert(&mut self, idx: usize, node: &Node) {
        // Impl blocks are containers named after their type; they are never a target.
        let nameable = (node.kind.is_symbol() && node.kind != NodeKind::Impl)
            || matches!(node.kind, NodeKind::Module | NodeKind::Query);
        if nameable {
            self.by_name.entry(node.name.clone()).or_default().push(idx);
        }
        if node.kind == NodeKind::Table {
            self.tables_by_qualified
                .insert(node.qualified_name.clone(), idx);
            self.tables_by_name
                .entry(node.name.clone())
                .or_default()
                .push(idx);
        }
    }

    pub(crate) fn by_name(&self, name: &str) -> &[usize] {
        self.by_name.get(name).map_or(&[], Vec::as_slice)
    }

    /// The table `schema.name`, or the unique table called `name` when no schema is given.
    pub(crate) fn table(&self, schema: Option<&str>, name: &str) -> Option<usize> {
        match schema {
            Some(s) => self
                .tables_by_qualified
                .get(&format!("{s}.{name}"))
                .copied(),
            None => match self.tables_by_name.get(name).map(Vec::as_slice) {
                Some([only]) => Some(*only),
                _ => None,
            },
        }
    }

    pub(crate) fn tier(&self, from: &RepoPath, candidate: &Node) -> Tier {
        let Some(file) = candidate.file() else {
            return Tier::SameRepo;
        };
        if file == from {
            return Tier::SameFile;
        }
        let a = self.package_of_file.get(from);
        let b = self.package_of_file.get(file);
        match (a, b) {
            (Some(a), Some(b)) if a == b => Tier::SamePackage,
            // Without manifests, files under one top-level directory are as close as a package.
            (None, None) if top_dir(from) == top_dir(file) => Tier::SamePackage,
            (_, Some(b))
                if self
                    .imports_of_file
                    .get(from)
                    .is_some_and(|s| s.contains(b)) =>
            {
                Tier::Imported
            }
            (Some(a), Some(b)) if self.deps_of_package.get(a).is_some_and(|s| s.contains(b)) => {
                Tier::Imported
            }
            _ => Tier::SameRepo,
        }
    }
}

fn top_dir(path: &RepoPath) -> &str {
    path.as_str().split_once('/').map_or("", |(dir, _)| dir)
}

/// Keeps the candidates of the closest tier, after an optional qualifier filter.
///
/// `qualifier` is the path segment before the name (`Relay` in `Relay::drain`); when
/// some candidates match it, the others are dropped.
pub(crate) fn rank<'a>(
    index: &SymbolIndex,
    from: &RepoPath,
    candidates: impl IntoIterator<Item = (usize, &'a Node)>,
    qualifier: Option<&str>,
) -> Vec<(Tier, usize)> {
    let all: Vec<(Tier, usize, &Node)> = candidates
        .into_iter()
        .map(|(i, n)| (index.tier(from, n), i, n))
        .collect();
    let qualified: Vec<(Tier, usize, &Node)> = qualifier
        .map(|q| {
            all.iter()
                .filter(|(_, _, n)| matches_qualifier(n, q))
                .copied()
                .collect()
        })
        .filter(|v: &Vec<_>| !v.is_empty())
        .unwrap_or(all);
    let Some(best) = qualified.iter().map(|(t, _, _)| *t).min() else {
        return Vec::new();
    };
    qualified
        .into_iter()
        .filter(|(t, _, _)| *t == best)
        .map(|(t, i, _)| (t, i))
        .collect()
}

fn matches_qualifier(node: &Node, qualifier: &str) -> bool {
    match node.qualified_name.rsplit_once("::") {
        Some((scope, _)) => scope == qualifier || scope.ends_with(&format!("::{qualifier}")),
        // Top-level item: the qualifier may name its file stem or module directory.
        None => node.file().is_some_and(|f| {
            let stem = f
                .file_name()
                .and_then(|n| n.rsplit_once('.'))
                .map(|(s, _)| s);
            stem == Some(qualifier)
                || (matches!(stem, Some("mod" | "lib" | "index" | "__init__"))
                    && f.parent().ends_with(qualifier))
        }),
    }
}

/// What a ranked set of candidates means for one reference.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Resolution {
    /// Link to each target with the given confidence.
    Linked {
        targets: Vec<usize>,
        confidence: Confidence,
    },
    /// Record a gap.
    Unresolved(UnresolvedReason),
    /// Neither link nor report: a standard-library name or a type mention.
    Silent,
}

/// How a reference was written, as far as resolution cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Site {
    /// Method call through a receiver, so the target depends on an unknown type.
    pub(crate) method_call: bool,
    /// The name is one the standard library or runtime provides.
    pub(crate) builtin: bool,
    /// Whether a miss should be reported as a gap.
    pub(crate) report: bool,
}

/// Confidence of a method call resolved by name to a unique declaration in another
/// package: the receiver's type is unknown, so it is a plausible guess, not a fact.
const REMOTE_METHOD_CONFIDENCE: f32 = 0.5;

/// Decides what to do with ranked candidates. Pure.
pub(crate) fn decide(ranked: &[(Tier, usize)], site: Site, max_candidates: usize) -> Resolution {
    let report = |reason| {
        if site.report {
            Resolution::Unresolved(reason)
        } else {
            Resolution::Silent
        }
    };
    let remote = ranked.iter().all(|(t, _)| *t == Tier::SameRepo);
    match ranked {
        // A standard-library name (`expect`, `Result`, `map`) with no local declaration is
        // the std item, not a gap; with only remote declarations it is still the std item.
        [] if site.builtin => Resolution::Silent,
        [] => report(UnresolvedReason::NoDeclaration),
        _ if site.builtin && remote => Resolution::Silent,
        many if many.len() > max_candidates => report(UnresolvedReason::Ambiguous),
        // A method call whose only matches are in other packages is a guess about the
        // receiver's type: link it only when the name is unique, and say so.
        [_, _, ..] if site.method_call && remote => report(UnresolvedReason::AmbiguousReceiver),
        [_] if site.method_call && remote => Resolution::Linked {
            targets: vec![ranked[0].1],
            confidence: Confidence::new(REMOTE_METHOD_CONFIDENCE),
        },
        some => Resolution::Linked {
            targets: some.iter().map(|(_, i)| *i).collect(),
            confidence: Confidence::new(1.0 / some.len() as f32),
        },
    }
}

/// Whether a call form is a method call.
pub(crate) const fn is_method(form: CallForm) -> bool {
    matches!(form, CallForm::Method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decisions_follow_the_rules() {
        let free = Site {
            method_call: false,
            builtin: false,
            report: true,
        };
        assert_eq!(
            decide(&[], free, 5),
            Resolution::Unresolved(UnresolvedReason::NoDeclaration)
        );
        assert_eq!(
            decide(&[(Tier::SameFile, 1)], free, 5),
            Resolution::Linked {
                targets: vec![1],
                confidence: Confidence::CERTAIN
            }
        );
        let six: Vec<(Tier, usize)> = (0..6).map(|i| (Tier::SameRepo, i)).collect();
        assert_eq!(
            decide(&six, free, 5),
            Resolution::Unresolved(UnresolvedReason::Ambiguous)
        );
        let builtin = Site {
            builtin: true,
            ..free
        };
        assert_eq!(decide(&[], builtin, 5), Resolution::Silent);
        assert_eq!(
            decide(&[(Tier::SameRepo, 1)], builtin, 5),
            Resolution::Silent
        );
        assert!(matches!(
            decide(&[(Tier::SamePackage, 1)], builtin, 5),
            Resolution::Linked { .. }
        ));
        let method = Site {
            method_call: true,
            ..free
        };
        assert_eq!(
            decide(&[(Tier::SameRepo, 1), (Tier::SameRepo, 2)], method, 5),
            Resolution::Unresolved(UnresolvedReason::AmbiguousReceiver)
        );
        assert!(matches!(
            decide(&[(Tier::SameRepo, 1)], method, 5),
            Resolution::Linked { confidence, .. } if (confidence.get() - 0.5).abs() < f32::EPSILON
        ));
        assert!(matches!(
            decide(&[(Tier::Imported, 1), (Tier::Imported, 2)], method, 5),
            Resolution::Linked { targets, .. } if targets.len() == 2
        ));
        let quiet = Site {
            report: false,
            ..free
        };
        assert_eq!(decide(&[], quiet, 5), Resolution::Silent);
    }
}
