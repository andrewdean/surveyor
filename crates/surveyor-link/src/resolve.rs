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
    /// Node index by `sql:` qualified relation name (`schema.table`); views included.
    tables_by_qualified: HashMap<String, usize>,
    /// Node indexes by bare relation name; views included.
    tables_by_name: HashMap<String, Vec<usize>>,
    /// Owning package node id per file.
    pub(crate) package_of_file: HashMap<RepoPath, String>,
    /// Package node ids a file imports, by file.
    pub(crate) imports_of_file: HashMap<RepoPath, HashSet<String>>,
    /// Every node id a file's imports resolved to (packages, files, symbols), by file.
    pub(crate) imported_by_file: HashMap<RepoPath, HashSet<String>>,
    /// Names a file imports from outside the repository (dependencies, the standard
    /// library), by file. A reference to one is that dependency's, not a gap.
    pub(crate) external_names_by_file: HashMap<RepoPath, HashSet<String>>,
    /// Node indexes each name a file imports from inside the repository stands for
    /// (a module file, a class, a function), by file and bound name. A receiver that
    /// is one of these names narrows a reference to that target's declarations.
    pub(crate) imported_targets_by_file: HashMap<RepoPath, HashMap<String, Vec<usize>>>,
    /// Package node ids a package depends on, by package node id.
    pub(crate) deps_of_package: HashMap<String, HashSet<String>>,
}

/// True for kinds a reference can resolve to by name.
///
/// Impl blocks are containers named after their type; they are never a target.
pub(crate) fn is_nameable(kind: &NodeKind) -> bool {
    (kind.is_symbol() && *kind != NodeKind::Impl)
        || kind.is_relation()
        || matches!(kind, NodeKind::Module | NodeKind::Query)
}

impl SymbolIndex {
    pub(crate) fn insert(&mut self, idx: usize, node: &Node) {
        let nameable = is_nameable(&node.kind) && !node.kind.is_relation();
        if nameable {
            self.by_name.entry(node.name.clone()).or_default().push(idx);
        }
        if node.kind.is_relation() {
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

    /// The relation `schema.name`, or every relation called `name` when no schema is
    /// given. A view is a relation like a table: code that selects from it links to it.
    ///
    /// A name without a schema comes from a `search_path` lookup or a templated
    /// `{schema}.name`; either way the schema is decided at run time, so each schema
    /// that declares the relation is a candidate.
    pub(crate) fn tables(&self, schema: Option<&str>, name: &str) -> Vec<usize> {
        match schema {
            Some(s) => self
                .tables_by_qualified
                .get(&format!("{s}.{name}"))
                .into_iter()
                .copied()
                .collect(),
            None => self.tables_by_name.get(name).cloned().unwrap_or_default(),
        }
    }

    /// The normalized name of the package that owns `file`, if any.
    fn package_name_of(&self, file: &RepoPath) -> Option<String> {
        self.package_of_file
            .get(file)
            .map(|id| normalize_package(id.rsplit('/').next().unwrap_or(id)))
    }

    /// True when `from` imports `name` from outside the repository.
    pub(crate) fn imports_external(&self, from: &RepoPath, name: &str) -> bool {
        self.external_names_by_file
            .get(from)
            .is_some_and(|names| names.contains(name))
    }

    /// True when `from` imports `candidate` itself or the file that declares it.
    pub(crate) fn imports(&self, from: &RepoPath, candidate: &Node) -> bool {
        self.imported_by_file.get(from).is_some_and(|imported| {
            imported.contains(candidate.id.as_str())
                || candidate
                    .file()
                    .is_some_and(|f| imported.contains(crate::ids::file_id(f).as_str()))
        })
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

/// Whether something at the reference site points at its declaration.
///
/// A path qualifier (`Relay` in `Relay::drain`) or an import of the file (`from app.db
/// import connect`, `use crate::store::OutboxStore`) names the declaration; a bare name
/// matched across files is a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Anchor {
    /// Neither a qualifier nor an import points at a candidate.
    None,
    /// The qualifier, or failing that an import of the file, names the candidates kept.
    Matched,
    /// Declarations of the name exist, but none under the qualifier: it names
    /// something outside the repository, and none of them is the target.
    Mismatched,
}

/// The candidates of the closest tier, and what pointed at them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ranked {
    /// Candidates of the closest tier, all in that tier.
    pub(crate) candidates: Vec<(Tier, usize)>,
    /// What pointed at them.
    pub(crate) anchor: Anchor,
}

impl Ranked {
    /// The tier every candidate sits in, if any.
    fn tier(&self) -> Option<Tier> {
        self.candidates.first().map(|(t, _)| *t)
    }
}

/// Keeps the candidates of the closest tier, after an optional qualifier filter.
///
/// `qualifier` is the path segment before the name (`Relay` in `Relay::drain`); when
/// some candidates match it, the others are dropped, and when none does, nothing is
/// kept: a qualifier that names no declaration never resolves to another one. Without
/// a qualifier, candidates the file imports (themselves or their file) win over the
/// rest.
pub(crate) fn rank<'a>(
    index: &SymbolIndex,
    from: &RepoPath,
    candidates: impl IntoIterator<Item = (usize, &'a Node)>,
    qualifier: Option<&str>,
) -> Ranked {
    let all: Vec<(Tier, usize, &Node)> = candidates
        .into_iter()
        .map(|(i, n)| (index.tier(from, n), i, n))
        .collect();
    let keep = |pred: &dyn Fn(&Node) -> bool| -> Vec<(Tier, usize, &Node)> {
        all.iter().filter(|(_, _, n)| pred(n)).copied().collect()
    };
    let (qualified, anchor) = match qualifier {
        Some(q) if !all.is_empty() => {
            let under = keep(&|n| matches_qualifier(index, n, q));
            if under.is_empty() {
                (under, Anchor::Mismatched)
            } else {
                (under, Anchor::Matched)
            }
        }
        _ => {
            let imported = keep(&|n| index.imports(from, n));
            if imported.is_empty() {
                (all, Anchor::None)
            } else {
                (imported, Anchor::Matched)
            }
        }
    };
    let candidates = qualified
        .iter()
        .map(|(t, _, _)| *t)
        .min()
        .map(|best| {
            qualified
                .iter()
                .filter(|(t, _, _)| *t == best)
                .map(|(t, i, _)| (*t, *i))
                .collect()
        })
        .unwrap_or_default();
    Ranked { candidates, anchor }
}

/// True when `qualifier` names the scope, file, module directory or package of `node`.
fn matches_qualifier(index: &SymbolIndex, node: &Node, qualifier: &str) -> bool {
    // `causeway_platform::install_crypto_provider()`: the qualifier is the crate.
    if node
        .file()
        .and_then(|f| index.package_name_of(f))
        .is_some_and(|package| package == normalize_package(qualifier))
    {
        return true;
    }
    match node.qualified_name.rsplit_once("::") {
        Some((scope, _)) => scope == qualifier || scope.ends_with(&format!("::{qualifier}")),
        // Top-level item: the qualifier may name its file stem or a directory on its
        // module path (`routes::handler` for `routes/access.rs`, where `routes/mod.rs`
        // re-exports it).
        None => node.file().is_some_and(|f| {
            let stem = f
                .file_name()
                .and_then(|n| n.rsplit_once('.'))
                .map(|(s, _)| s);
            stem == Some(qualifier) || f.parent().split('/').any(|dir| dir == qualifier)
        }),
    }
}

/// Package names compare case-insensitively with `-` and `_` alike, as Cargo does.
fn normalize_package(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
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
    /// The name, its path's root or its receiver's root is one the file imports from
    /// outside the repository.
    pub(crate) external_import: bool,
    /// A call on `self`/`this` inside a class that extends a type from outside the
    /// repository, so a method declared nowhere here is inherited from it.
    pub(crate) inherits_external: bool,
    /// The candidates were narrowed by the receiver to the declarations of the
    /// imported module, the imported class or the enclosing class, so the receiver's
    /// type is not a guess.
    pub(crate) receiver_known: bool,
}

/// Confidence of a method call resolved by name to a declaration in another file of
/// the same or an imported package: the receiver's type is unknown, but the
/// declaration is one the file can reach.
const NEARBY_METHOD_CONFIDENCE: f32 = 0.75;

/// Confidence of a method call resolved by name to a declaration in an unrelated
/// package: the receiver's type is unknown, so it is a plausible guess, not a fact.
const REMOTE_METHOD_CONFIDENCE: f32 = 0.5;

/// Decides what to do with ranked candidates. Pure.
///
/// A reference is *anchored* when it points at its declaration rather than guessing
/// by name: the declaration is in the same file, or the path's qualifier named it.
/// Only an anchored reference makes a standard-library name resolve, and only an
/// anchored method call is certain; the rest share a lower confidence.
pub(crate) fn decide(ranked: &Ranked, site: Site, max_candidates: usize) -> Resolution {
    let report = |reason| {
        if site.report {
            Resolution::Unresolved(reason)
        } else {
            Resolution::Silent
        }
    };
    let candidates = ranked.candidates.as_slice();
    let remote = ranked.tier() == Some(Tier::SameRepo);
    let anchored = ranked.tier() == Some(Tier::SameFile) || ranked.anchor == Anchor::Matched;
    let linked = |confidence: f32| Resolution::Linked {
        targets: candidates.iter().map(|(_, i)| *i).collect(),
        confidence: Confidence::new(confidence / candidates.len() as f32),
    };
    match candidates {
        // A name imported from a dependency is that dependency's, whatever the
        // repository declares under the same name: neither an edge nor a gap.
        _ if site.external_import => Resolution::Silent,
        // `self.execute_unary()` in a class that extends a dependency's type: the
        // method is the base class's when nothing here declares it.
        [] if site.inherits_external => Resolution::Silent,
        // A standard-library name (`expect`, `Result`, `map`) with no declaration the
        // reference points at is the std item, not a gap.
        [] if site.builtin => Resolution::Silent,
        [] if ranked.anchor == Anchor::Mismatched => report(UnresolvedReason::QualifierMismatch),
        [] => report(UnresolvedReason::NoDeclaration),
        // `db.connect()` after `from . import db`: the receiver named the module, and
        // the candidates are its declarations, wherever that module lives.
        _ if site.receiver_known => linked(1.0),
        _ if site.builtin && !anchored => Resolution::Silent,
        many if many.len() > max_candidates => report(UnresolvedReason::Ambiguous),
        // A method call whose only matches are in unrelated packages is a guess about
        // the receiver's type: link it only when the name is unique, and say so.
        [_, _, ..] if site.method_call && remote => report(UnresolvedReason::AmbiguousReceiver),
        _ if site.method_call && remote => linked(REMOTE_METHOD_CONFIDENCE),
        _ if site.method_call && !anchored => linked(NEARBY_METHOD_CONFIDENCE),
        _ => linked(1.0),
    }
}

/// Decides what to do with the tables a reference may name. Pure.
///
/// A schema-qualified name has at most one candidate and links with certainty. A bare
/// name links to every schema that declares the table, each at `1/n`, because the
/// schema is chosen at run time and the graph cannot tell which.
pub(crate) fn decide_table(candidates: Vec<usize>, max_candidates: usize) -> Resolution {
    match candidates.len() {
        0 => Resolution::Unresolved(UnresolvedReason::NoTable),
        n if n > max_candidates => Resolution::Unresolved(UnresolvedReason::Ambiguous),
        n => Resolution::Linked {
            confidence: Confidence::new(1.0 / n as f32),
            targets: candidates,
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

    fn ranked(candidates: &[(Tier, usize)], anchor: Anchor) -> Ranked {
        Ranked {
            candidates: candidates.to_vec(),
            anchor,
        }
    }

    #[test]
    fn a_name_imported_from_outside_the_repository_is_silent() {
        let site = Site {
            method_call: false,
            builtin: false,
            report: true,
            external_import: true,
            inherits_external: false,
            receiver_known: false,
        };
        assert_eq!(
            decide(&ranked(&[], Anchor::None), site, 8),
            Resolution::Silent,
            "no gap for a dependency's name"
        );
        assert_eq!(
            decide(&ranked(&[(Tier::SameFile, 1)], Anchor::None), site, 8),
            Resolution::Silent,
            "no edge either, even to a same-named declaration next door"
        );
    }

    #[test]
    fn a_self_call_in_a_class_with_an_external_base_is_silent_only_when_undeclared() {
        let site = Site {
            method_call: true,
            builtin: false,
            report: true,
            external_import: false,
            inherits_external: true,
            receiver_known: false,
        };
        assert_eq!(
            decide(&ranked(&[], Anchor::None), site, 8),
            Resolution::Silent
        );
        assert!(
            matches!(
                decide(&ranked(&[(Tier::SameFile, 1)], Anchor::None), site, 8),
                Resolution::Linked { .. }
            ),
            "a method declared here still links"
        );
    }

    #[test]
    fn a_known_receiver_makes_a_remote_method_call_certain() {
        let site = Site {
            method_call: true,
            builtin: true,
            report: true,
            external_import: false,
            inherits_external: false,
            receiver_known: true,
        };
        assert_eq!(
            decide(&ranked(&[(Tier::SameRepo, 4)], Anchor::None), site, 8),
            Resolution::Linked {
                targets: vec![4],
                confidence: Confidence::CERTAIN,
            },
            "a builtin name on a known receiver is the receiver's own method"
        );
        assert_eq!(
            decide(&ranked(&[], Anchor::None), site, 8),
            Resolution::Silent,
            "with nothing declared, a builtin name stays silent as before"
        );
    }

    #[test]
    fn a_table_shares_confidence_across_the_schemas_that_declare_it() {
        assert_eq!(
            decide_table(vec![], 8),
            Resolution::Unresolved(UnresolvedReason::NoTable)
        );
        assert_eq!(
            decide_table(vec![3], 8),
            Resolution::Linked {
                targets: vec![3],
                confidence: Confidence::CERTAIN,
            }
        );
        assert_eq!(
            decide_table(vec![3, 5], 8),
            Resolution::Linked {
                targets: vec![3, 5],
                confidence: Confidence::new(0.5),
            }
        );
        assert_eq!(
            decide_table(vec![1, 2, 3], 2),
            Resolution::Unresolved(UnresolvedReason::Ambiguous)
        );
    }

    fn confidence_of(resolution: &Resolution) -> f32 {
        match resolution {
            Resolution::Linked { confidence, .. } => confidence.get(),
            other => panic!("not linked: {other:?}"),
        }
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "confidences are exact constants")]
    fn decisions_follow_the_rules() {
        let free = Site {
            method_call: false,
            builtin: false,
            report: true,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
        };
        let plain = |c: &[(Tier, usize)]| ranked(c, Anchor::None);
        assert_eq!(
            decide(&plain(&[]), free, 5),
            Resolution::Unresolved(UnresolvedReason::NoDeclaration)
        );
        assert_eq!(
            decide(&plain(&[(Tier::SameFile, 1)]), free, 5),
            Resolution::Linked {
                targets: vec![1],
                confidence: Confidence::CERTAIN
            }
        );
        // A free function or qualified path is certain at any distance.
        assert_eq!(
            confidence_of(&decide(&plain(&[(Tier::SameRepo, 1)]), free, 5)),
            1.0
        );
        let six: Vec<(Tier, usize)> = (0..6).map(|i| (Tier::SameRepo, i)).collect();
        assert_eq!(
            decide(&plain(&six), free, 5),
            Resolution::Unresolved(UnresolvedReason::Ambiguous)
        );
        let quiet = Site {
            report: false,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
            ..free
        };
        assert_eq!(decide(&plain(&[]), quiet, 5), Resolution::Silent);
    }

    #[test]
    fn a_qualifier_may_name_the_scope_file_directory_or_package() {
        use surveyor_model::{ByteSpan, LineSpan, NodeId, RepoId, TextSpan};
        let at = |id: &str, file: &str| {
            Node::synthetic(NodeId::new(id), NodeKind::Function, "f", RepoId::new("r"))
                .with_qualified_name(id.rsplit("::").next().unwrap_or(id))
                .at(
                    RepoPath::new(file),
                    TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1)),
                )
        };
        let mut index = SymbolIndex::default();
        index.package_of_file.insert(
            RepoPath::new("crates/platform/src/lib.rs"),
            "pkg:cargo/causeway-platform".to_owned(),
        );
        let in_lib = at(
            "rust:crates/platform/src/lib.rs::f",
            "crates/platform/src/lib.rs",
        );
        let in_module = at("rust:src/routes/access.rs::f", "src/routes/access.rs");
        let method = Node::synthetic(
            NodeId::new("rust:src/relay.rs::Relay::f"),
            NodeKind::Method,
            "f",
            RepoId::new("r"),
        )
        .with_qualified_name("Relay::f");
        assert!(matches_qualifier(&index, &in_lib, "causeway_platform"));
        assert!(matches_qualifier(&index, &in_lib, "lib"));
        assert!(!matches_qualifier(&index, &in_lib, "routes"));
        assert!(matches_qualifier(&index, &in_module, "access"));
        assert!(matches_qualifier(&index, &in_module, "routes"));
        assert!(!matches_qualifier(&index, &in_module, "server"));
        assert!(matches_qualifier(&index, &method, "Relay"));
        assert!(!matches_qualifier(&index, &method, "Other"));
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "confidences are exact constants")]
    fn a_qualifier_that_names_no_declaration_is_a_gap_not_a_guess() {
        let free = Site {
            method_call: false,
            builtin: false,
            report: true,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
        };
        assert_eq!(
            decide(&ranked(&[], Anchor::Mismatched), free, 5),
            Resolution::Unresolved(UnresolvedReason::QualifierMismatch)
        );
        assert_eq!(
            confidence_of(&decide(
                &ranked(&[(Tier::SameRepo, 1)], Anchor::Matched),
                free,
                5
            )),
            1.0
        );
    }

    #[test]
    fn builtin_names_resolve_only_when_anchored() {
        let builtin = Site {
            method_call: false,
            builtin: true,
            report: true,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
        };
        assert_eq!(
            decide(&ranked(&[], Anchor::None), builtin, 5),
            Resolution::Silent
        );
        assert_eq!(
            decide(&ranked(&[], Anchor::Mismatched), builtin, 5),
            Resolution::Silent
        );
        for tier in [Tier::SamePackage, Tier::Imported, Tier::SameRepo] {
            assert_eq!(
                decide(&ranked(&[(tier, 1)], Anchor::None), builtin, 5),
                Resolution::Silent,
                "{tier:?}"
            );
        }
        assert!(matches!(
            decide(&ranked(&[(Tier::SameFile, 1)], Anchor::None), builtin, 5),
            Resolution::Linked { .. }
        ));
        assert!(matches!(
            decide(&ranked(&[(Tier::SameRepo, 1)], Anchor::Matched), builtin, 5),
            Resolution::Linked { .. }
        ));
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "confidences are exact constants")]
    fn method_calls_by_name_are_certain_only_in_their_own_file() {
        let method = Site {
            method_call: true,
            builtin: false,
            report: true,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
        };
        let one = |tier| ranked(&[(tier, 1)], Anchor::None);
        assert_eq!(confidence_of(&decide(&one(Tier::SameFile), method, 5)), 1.0);
        assert_eq!(
            confidence_of(&decide(&one(Tier::SamePackage), method, 5)),
            NEARBY_METHOD_CONFIDENCE
        );
        assert_eq!(
            confidence_of(&decide(&one(Tier::Imported), method, 5)),
            NEARBY_METHOD_CONFIDENCE
        );
        assert_eq!(
            confidence_of(&decide(&one(Tier::SameRepo), method, 5)),
            REMOTE_METHOD_CONFIDENCE
        );
        assert_eq!(
            decide(
                &ranked(&[(Tier::SameRepo, 1), (Tier::SameRepo, 2)], Anchor::None),
                method,
                5
            ),
            Resolution::Unresolved(UnresolvedReason::AmbiguousReceiver)
        );
        // Two candidates nearby share the nearby confidence between them.
        let two = decide(
            &ranked(&[(Tier::Imported, 1), (Tier::Imported, 2)], Anchor::None),
            method,
            5,
        );
        assert!(matches!(&two, Resolution::Linked { targets, .. } if targets.len() == 2));
        assert!((confidence_of(&two) - NEARBY_METHOD_CONFIDENCE / 2.0).abs() < 1e-6);
    }
}
