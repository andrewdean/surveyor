//! Architectural intent as rules an agent cannot argue with.
//!
//! A rule is a Rhai script that receives the graph and returns violations. The script
//! declares its identity in top-level constants and its logic in `fn check(g)`:
//!
//! ```rhai
//! const ID = "only-the-store-names-lance";
//! const DESC = "lancedb is a dependency of surveyor-store and nothing else";
//! const SEVERITY = "fail";          // fail | warn | budget
//!
//! fn check(g) {
//!     let out = [];
//!     for e in g.edges_to("pkg:cargo/lancedb", "DEPENDS_ON") {
//!         if e.src != "pkg:cargo/surveyor-store" {
//!             out.push(violation(e, "depends on lancedb directly"));
//!         }
//!     }
//!     out
//! }
//! ```
//!
//! `g` answers the questions the CLI answers: `nodes(kind)`, `nodes_under(kind, path)`,
//! `node(id)`, `edges(kind)`, `edges_from(id, kind)`, `edges_to(id, kind)`, `out(id,
//! kind)`, `inn(id, kind)`, `package_of(id)`, `reaches(from, to, kinds, depth)` and
//! `cycles(kinds)`. Nodes and edges are maps with the fields the JSON output has, and
//! every edge carries its `citation`, so a violation points at a line. `violation(at,
//! message)` builds one from a node or an edge.
//!
//! `cargo_build(package, default_features)` resolves what a build of a Cargo package
//! pulls in from the manifest facts on `DEPENDS_ON` edges and Package nodes (see
//! [`cargo`]): a map with `packages` (each package reached, to the features enabled on
//! it) and `via` (each reached package, to the one it was first reached from).
//!
//! Severity `fail` fails the check on any violation; `warn` reports without failing;
//! `budget` fails when the count exceeds `const BUDGET`, which is how a known debt is
//! held and burned down. Scripts run sandboxed: no imports, no file access, bounded
//! operations.

// Rust guideline compliant 2026-02-21

pub mod cargo;

use std::backtrace::Backtrace;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rhai::module_resolvers::DummyModuleResolver;
use rhai::{AST, Array, Dynamic, Engine, Map, Scope};
use serde::{Deserialize, Serialize};
use surveyor_graph::{CodeGraph, Direction, EdgeFilter};
use surveyor_model::{Edge, EdgeKind, Node, NodeId, NodeKind};

/// Operations one rule may perform before it is stopped as runaway.
///
/// Generous enough for a loop over every edge of a large repository with a few
/// lookups each; a rule that needs more is doing graph work the engine should offer.
const MAX_OPERATIONS: u64 = 50_000_000;

/// How a rule's violations are judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Any violation fails the check.
    Fail,
    /// Violations are reported; the check passes.
    Warn,
    /// More violations than `budget` fail the check.
    Budget,
}

impl Severity {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "fail" => Some(Self::Fail),
            "warn" => Some(Self::Warn),
            "budget" => Some(Self::Budget),
            _ => None,
        }
    }
}

/// One finding of a rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    /// What the rule says about the subject.
    pub message: String,
    /// `file:lines` of the node or edge evidence, when it has one.
    pub citation: Option<String>,
    /// Identifier of the node, or the edge as `src -KIND-> dst`.
    pub subject: String,
}

/// A loaded, compiled rule.
pub struct Rule {
    /// Identifier, from `const ID` or the file stem.
    pub id: String,
    /// One-line description, from `const DESC`.
    pub desc: String,
    /// How violations are judged.
    pub severity: Severity,
    /// Allowed violations for a `budget` rule.
    pub budget: Option<i64>,
    /// Where it was loaded from.
    pub path: PathBuf,
    ast: AST,
    /// The script's top-level constants, visible to `check` as globals.
    scope: Scope<'static>,
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rule")
            .field("id", &self.id)
            .field("severity", &self.severity)
            .field("budget", &self.budget)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// The result of running one rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// Rule identifier.
    pub id: String,
    /// Rule description.
    pub desc: String,
    /// How it was judged.
    pub severity: Severity,
    /// Budget for `budget` rules.
    pub budget: Option<i64>,
    /// What it found.
    pub violations: Vec<Violation>,
    /// Whether the rule passed under its severity.
    pub passed: bool,
}

/// The result of running every rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// One outcome per rule, in load order.
    pub outcomes: Vec<Outcome>,
}

impl Report {
    /// True when every rule passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.outcomes.iter().all(|o| o.passed)
    }

    /// Number of violations across all rules.
    #[must_use]
    pub fn violation_count(&self) -> usize {
        self.outcomes.iter().map(|o| o.violations.len()).sum()
    }
}

/// Raised when a rule file cannot be read, compiled or run.
#[derive(Debug)]
pub struct RuleError {
    path: PathBuf,
    message: String,
    backtrace: Backtrace,
}

impl RuleError {
    fn new(path: &Path, message: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            message: message.into(),
            backtrace: Backtrace::capture(),
        }
    }

    /// The rule file concerned.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for RuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for RuleError {}

/// The graph as the scripts see it.
#[derive(Clone)]
struct GraphHandle(Arc<CodeGraph>);

fn node_map(n: &Node) -> Map {
    let mut m = Map::new();
    m.insert("id".into(), Dynamic::from(n.id.to_string()));
    m.insert("kind".into(), Dynamic::from(n.kind.to_string()));
    m.insert("name".into(), Dynamic::from(n.name.clone()));
    m.insert(
        "qualified_name".into(),
        Dynamic::from(n.qualified_name.clone()),
    );
    m.insert(
        "file".into(),
        n.file()
            .map_or(Dynamic::UNIT, |f| Dynamic::from(f.to_string())),
    );
    m.insert(
        "lines".into(),
        n.lines()
            .map_or(Dynamic::UNIT, |l| Dynamic::from(l.to_string())),
    );
    m.insert(
        "signature".into(),
        n.signature.clone().map_or(Dynamic::UNIT, Dynamic::from),
    );
    m.insert(
        "doc".into(),
        n.doc.clone().map_or(Dynamic::UNIT, Dynamic::from),
    );
    m.insert(
        "citation".into(),
        n.location
            .as_ref()
            .map_or(Dynamic::UNIT, |l| Dynamic::from(l.citation())),
    );
    let attrs: Map = n
        .attributes
        .iter()
        .map(|(k, v)| (k.as_str().into(), Dynamic::from(v.clone())))
        .collect();
    m.insert("attributes".into(), Dynamic::from_map(attrs));
    m
}

fn edge_map(e: &Edge) -> Map {
    let mut m = Map::new();
    m.insert("id".into(), Dynamic::from(e.id.to_string()));
    m.insert("kind".into(), Dynamic::from(e.kind.to_string()));
    m.insert("src".into(), Dynamic::from(e.src.to_string()));
    m.insert("dst".into(), Dynamic::from(e.dst.to_string()));
    m.insert(
        "derivation".into(),
        Dynamic::from(e.provenance.derivation.kind().to_string()),
    );
    m.insert(
        "confidence".into(),
        Dynamic::from(f64::from(e.provenance.confidence.get())),
    );
    let first = e.provenance.evidence.first();
    m.insert(
        "citation".into(),
        first.map_or(Dynamic::UNIT, |ev| Dynamic::from(ev.citation())),
    );
    m.insert(
        "file".into(),
        first.map_or(Dynamic::UNIT, |ev| {
            Dynamic::from(ev.location.file.to_string())
        }),
    );
    let attrs: Map = e
        .attributes
        .iter()
        .map(|(k, v)| (k.as_str().into(), Dynamic::from(v.clone())))
        .collect();
    m.insert("attributes".into(), Dynamic::from_map(attrs));
    m
}

fn kinds_of(kind: &str) -> Option<EdgeFilter> {
    if kind.is_empty() {
        return Some(EdgeFilter::default());
    }
    kind.parse::<EdgeKind>()
        .ok()
        .map(|k| EdgeFilter::default().with_kinds(vec![k]))
}

fn kinds_array(kinds: &Array) -> Option<EdgeFilter> {
    let parsed: Option<Vec<EdgeKind>> = kinds
        .iter()
        .map(|k| {
            k.clone()
                .into_string()
                .ok()
                .and_then(|s| s.parse::<EdgeKind>().ok())
        })
        .collect();
    parsed.map(|ks| {
        if ks.is_empty() {
            EdgeFilter::default()
        } else {
            EdgeFilter::default().with_kinds(ks)
        }
    })
}

fn to_array<T>(items: impl IntoIterator<Item = T>, f: impl Fn(T) -> Map) -> Array {
    items.into_iter().map(|i| Dynamic::from_map(f(i))).collect()
}

impl GraphHandle {
    fn nodes(&mut self, kind: &str) -> Array {
        match kind.parse::<NodeKind>() {
            Ok(k) => to_array(self.0.nodes_of_kind(&k), node_map),
            Err(_) if kind.is_empty() => to_array(self.0.nodes().iter(), node_map),
            Err(_) => Array::new(),
        }
    }

    fn nodes_under(&mut self, kind: &str, prefix: &str) -> Array {
        let wanted = kind.parse::<NodeKind>().ok();
        let under = self
            .0
            .nodes()
            .iter()
            .filter(|n| wanted.as_ref().is_none_or(|k| &n.kind == k))
            .filter(|n| {
                n.file()
                    .is_some_and(|f| f.starts_with_dir(prefix) || f.as_str().starts_with(prefix))
            });
        to_array(under, node_map)
    }

    fn node(&mut self, id: &str) -> Dynamic {
        self.0
            .node(&NodeId::new(id))
            .map_or(Dynamic::UNIT, |n| Dynamic::from_map(node_map(n)))
    }

    fn edges(&mut self, kind: &str) -> Array {
        let wanted = kind.parse::<EdgeKind>().ok();
        let edges = self
            .0
            .edges()
            .iter()
            .filter(|e| wanted.as_ref().is_none_or(|k| &e.kind == k));
        to_array(edges, edge_map)
    }

    fn edges_from(&mut self, id: &str, kind: &str) -> Array {
        let Some(filter) = kinds_of(kind) else {
            return Array::new();
        };
        to_array(
            self.0
                .neighbors(&NodeId::new(id), Direction::Outgoing, &filter),
            |n| edge_map(n.edge),
        )
    }

    fn edges_to(&mut self, id: &str, kind: &str) -> Array {
        let Some(filter) = kinds_of(kind) else {
            return Array::new();
        };
        to_array(
            self.0
                .neighbors(&NodeId::new(id), Direction::Incoming, &filter),
            |n| edge_map(n.edge),
        )
    }

    fn out(&mut self, id: &str, kind: &str) -> Array {
        let Some(filter) = kinds_of(kind) else {
            return Array::new();
        };
        to_array(
            self.0
                .neighbors(&NodeId::new(id), Direction::Outgoing, &filter),
            |n| node_map(n.node),
        )
    }

    fn inn(&mut self, id: &str, kind: &str) -> Array {
        let Some(filter) = kinds_of(kind) else {
            return Array::new();
        };
        to_array(
            self.0
                .neighbors(&NodeId::new(id), Direction::Incoming, &filter),
            |n| node_map(n.node),
        )
    }

    /// The package that owns a node's file, as its identifier, or `()`.
    fn package_of(&mut self, id: &str) -> Dynamic {
        let graph = &self.0;
        let contains = EdgeFilter::default().with_kinds(vec![EdgeKind::Contains]);
        let owner = |file_id: NodeId| {
            graph
                .neighbors(&file_id, Direction::Incoming, &contains)
                .into_iter()
                .find(|n| n.node.kind == NodeKind::Package)
                .map(|n| n.node.id.to_string())
        };
        let found = graph.node(&NodeId::new(id)).and_then(|n| match n.kind {
            NodeKind::Package => Some(n.id.to_string()),
            NodeKind::File => owner(n.id.clone()),
            _ => n
                .file()
                .and_then(|f| owner(NodeId::new(format!("file:{f}")))),
        });
        found.map_or(Dynamic::UNIT, Dynamic::from)
    }

    /// True when `to` is reachable from `from` along `kinds` within `depth` hops.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "rhai passes script arguments by value"
    )]
    fn reaches(&mut self, from: &str, to: &str, kinds: Array, depth: i64) -> bool {
        let Some(filter) = kinds_array(&kinds) else {
            return false;
        };
        let (from, to) = (NodeId::new(from), NodeId::new(to));
        self.0
            .shortest_path(&from, &to, &filter, true)
            .is_some_and(|steps| {
                i64::try_from(steps.len().saturating_sub(1)).unwrap_or(i64::MAX) <= depth
            })
    }

    /// Strongly connected components of two or more nodes along `kinds`, as arrays of ids.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "rhai passes script arguments by value"
    )]
    fn cycles(&mut self, kinds: Array) -> Array {
        let Some(filter) = kinds_array(&kinds) else {
            return Array::new();
        };
        self.0
            .cycles(2, &filter)
            .into_iter()
            .map(|group| {
                Dynamic::from_array(
                    group
                        .into_iter()
                        .map(|n| Dynamic::from(n.id.to_string()))
                        .collect(),
                )
            })
            .collect()
    }

    fn count(&mut self, kind: &str) -> i64 {
        i64::try_from(self.nodes(kind).len()).unwrap_or(i64::MAX)
    }

    /// What a build of the Cargo package `root` pulls in, as `#{ packages, via }`.
    fn cargo_build(&mut self, root: &str, default_features: bool) -> Map {
        let graph = &self.0;
        let depends_on = EdgeFilter::default().with_kinds(vec![EdgeKind::DependsOn]);
        let features_of = |id: &str| {
            graph
                .node(&NodeId::new(id))
                .map(cargo::feature_table)
                .unwrap_or_default()
        };
        let dependencies_of = |id: &str| {
            graph
                .neighbors(&NodeId::new(id), Direction::Outgoing, &depends_on)
                .into_iter()
                .map(|n| cargo::Dependency::from_edge(n.edge, n.node))
                .collect()
        };
        let build = cargo::resolve(root, default_features, features_of, dependencies_of);
        let packages: Map = build
            .features
            .iter()
            .map(|(package, features)| {
                let list: Array = features.iter().map(|f| Dynamic::from(f.clone())).collect();
                (package.as_str().into(), Dynamic::from_array(list))
            })
            .collect();
        let via: Map = build
            .via
            .iter()
            .map(|(package, from)| (package.as_str().into(), Dynamic::from(from.clone())))
            .collect();
        let mut m = Map::new();
        m.insert("packages".into(), Dynamic::from_map(packages));
        m.insert("via".into(), Dynamic::from_map(via));
        m
    }
}

/// `violation(at, message)`: a finding about a node map or an edge map.
#[expect(
    clippy::needless_pass_by_value,
    reason = "rhai passes script arguments by value"
)]
fn violation(at: Map, message: &str) -> Map {
    let get = |k: &str| at.get(k).and_then(|d| d.clone().into_string().ok());
    let subject = match (get("src"), get("kind"), get("dst")) {
        (Some(s), Some(k), Some(d)) => format!("{s} -{k}-> {d}"),
        _ => get("id").unwrap_or_default(),
    };
    let mut m = Map::new();
    m.insert("message".into(), Dynamic::from(message.to_owned()));
    m.insert(
        "citation".into(),
        get("citation").map_or(Dynamic::UNIT, Dynamic::from),
    );
    m.insert("subject".into(), Dynamic::from(subject));
    m
}

fn engine() -> Engine {
    let mut engine = Engine::new();
    engine.set_module_resolver(DummyModuleResolver);
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(64);
    // Rules nest loops, conditions and calls freely; the default depth trips on four levels.
    engine.set_max_expr_depths(256, 256);
    engine
        .register_type_with_name::<GraphHandle>("Graph")
        .register_fn("nodes", GraphHandle::nodes)
        .register_fn("nodes_under", GraphHandle::nodes_under)
        .register_fn("node", GraphHandle::node)
        .register_fn("edges", GraphHandle::edges)
        .register_fn("edges_from", GraphHandle::edges_from)
        .register_fn("edges_to", GraphHandle::edges_to)
        .register_fn("out", GraphHandle::out)
        .register_fn("inn", GraphHandle::inn)
        .register_fn("package_of", GraphHandle::package_of)
        .register_fn("reaches", GraphHandle::reaches)
        .register_fn("cycles", GraphHandle::cycles)
        .register_fn("count", GraphHandle::count)
        .register_fn("cargo_build", GraphHandle::cargo_build)
        .register_fn("violation", violation);
    engine
}

/// Rules loaded from files, ready to run against any graph.
#[derive(Debug)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    /// Loads every `*.rhai` file directly inside `dirs` (sorted by name) plus `files`.
    ///
    /// # Errors
    /// Returns the first file that does not compile or declares an invalid severity.
    pub fn load(dirs: &[PathBuf], files: &[PathBuf]) -> Result<Self, RuleError> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for dir in dirs {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut found: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("rhai"))
                })
                .collect();
            found.sort();
            paths.extend(found);
        }
        paths.extend(files.iter().cloned());
        let engine = engine();
        let rules = paths
            .iter()
            .map(|p| compile_rule(&engine, p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rules })
    }

    /// Compiles rules from source text, for tests and embedded rule sets.
    ///
    /// # Errors
    /// Returns the first source that does not compile.
    pub fn from_sources(sources: &[(&str, &str)]) -> Result<Self, RuleError> {
        let engine = engine();
        let rules = sources
            .iter()
            .map(|(name, text)| compile_source(&engine, Path::new(name), text))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rules })
    }

    /// The loaded rules.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Runs every rule against `graph`.
    ///
    /// # Errors
    /// Returns the first rule whose `check` raises a script error.
    pub fn run(&self, graph: Arc<CodeGraph>) -> Result<Report, RuleError> {
        let engine = engine();
        let handle = GraphHandle(graph);
        let outcomes = self
            .rules
            .iter()
            .map(|rule| run_rule(&engine, rule, &handle))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Report { outcomes })
    }
}

fn compile_rule(engine: &Engine, path: &Path) -> Result<Rule, RuleError> {
    let text = std::fs::read_to_string(path).map_err(|e| RuleError::new(path, e.to_string()))?;
    compile_source(engine, path, &text)
}

fn compile_source(engine: &Engine, path: &Path, text: &str) -> Result<Rule, RuleError> {
    let ast = engine
        .compile(text)
        .map_err(|e| RuleError::new(path, e.to_string()))?;
    let mut scope = Scope::new();
    engine
        .run_ast_with_scope(&mut scope, &ast)
        .map_err(|e| RuleError::new(path, e.to_string()))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rule")
        .to_owned();
    let id = scope.get_value::<String>("ID").unwrap_or(stem);
    let desc = scope.get_value::<String>("DESC").unwrap_or_default();
    let severity = match scope.get_value::<String>("SEVERITY") {
        None => Severity::Fail,
        Some(s) => Severity::parse(&s).ok_or_else(|| {
            RuleError::new(path, format!("SEVERITY {s:?} is not fail, warn or budget"))
        })?,
    };
    let budget = scope.get_value::<i64>("BUDGET");
    if severity == Severity::Budget && budget.is_none() {
        return Err(RuleError::new(path, "a budget rule needs `const BUDGET`"));
    }
    if !ast
        .iter_functions()
        .any(|f| f.name == "check" && f.params.len() == 1)
    {
        return Err(RuleError::new(path, "a rule needs `fn check(g)`"));
    }
    Ok(Rule {
        id,
        desc,
        severity,
        budget,
        path: path.to_path_buf(),
        ast,
        scope,
    })
}

fn run_rule(engine: &Engine, rule: &Rule, graph: &GraphHandle) -> Result<Outcome, RuleError> {
    let mut scope = rule.scope.clone();
    let raw: Array = engine
        .call_fn(&mut scope, &rule.ast, "check", (graph.clone(),))
        .map_err(|e| RuleError::new(&rule.path, format!("check: {e}")))?;
    let violations: Vec<Violation> = raw
        .into_iter()
        .filter_map(rhai::Dynamic::try_cast::<Map>)
        .map(|m| {
            let get = |k: &str| m.get(k).and_then(|d| d.clone().into_string().ok());
            Violation {
                message: get("message").unwrap_or_default(),
                citation: get("citation"),
                subject: get("subject").unwrap_or_default(),
            }
        })
        .collect();
    let count = i64::try_from(violations.len()).unwrap_or(i64::MAX);
    let passed = match rule.severity {
        Severity::Fail => violations.is_empty(),
        Severity::Warn => true,
        Severity::Budget => count <= rule.budget.unwrap_or(0),
    };
    Ok(Outcome {
        id: rule.id.clone(),
        desc: rule.desc.clone(),
        severity: rule.severity,
        budget: rule.budget,
        violations,
        passed,
    })
}

#[cfg(test)]
mod tests {
    use surveyor_model::{
        ByteSpan, CommitId, ContentHash, DeclaredSource, Evidence, GraphDelta, LineSpan,
        Provenance, RepoId, RepoPath, SourceLocation, TextSpan,
    };

    use super::*;

    fn graph() -> Arc<CodeGraph> {
        let repo = RepoId::new("r");
        let pkg = |name: &str| {
            Node::synthetic(
                NodeId::new(format!("pkg:cargo/{name}")),
                NodeKind::Package,
                name,
                repo.clone(),
            )
        };
        let a = pkg("a");
        let b = pkg("b");
        let lance = pkg("lancedb");
        let f = Node::synthetic(
            NodeId::new("file:a/src/lib.rs"),
            NodeKind::File,
            "lib.rs",
            repo.clone(),
        )
        .in_file(RepoPath::new("a/src/lib.rs"));
        let func = Node::synthetic(
            NodeId::new("rust:a/src/lib.rs::go"),
            NodeKind::Function,
            "go",
            repo.clone(),
        )
        .at(
            RepoPath::new("a/src/lib.rs"),
            TextSpan::new(ByteSpan::new(0, 5), LineSpan::new(1, 1)),
        );
        let ev = |file: &str, line: u32| Evidence {
            repo: repo.clone(),
            location: SourceLocation::new(
                RepoPath::new(file),
                TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(line, line)),
            ),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(b"x"),
        };
        let dep = |s: &Node, d: &Node, line: u32| {
            Edge::new(
                s.id.clone(),
                EdgeKind::DependsOn,
                d.id.clone(),
                Provenance::declared(DeclaredSource::Manifest, ev("a/Cargo.toml", line)),
            )
        };
        let edges = vec![
            dep(&a, &b, 3),
            dep(&b, &a, 4),
            dep(&a, &lance, 5),
            Edge::new(
                a.id.clone(),
                EdgeKind::Contains,
                f.id.clone(),
                Provenance::declared(DeclaredSource::Manifest, ev("a/Cargo.toml", 1)),
            ),
            Edge::new(
                f.id.clone(),
                EdgeKind::Contains,
                func.id.clone(),
                Provenance::declared(DeclaredSource::Manifest, ev("a/src/lib.rs", 1)),
            ),
        ];
        Arc::new(CodeGraph::from_delta(GraphDelta {
            nodes: vec![a, b, lance, f, func],
            edges,
            unresolved: Vec::new(),
        }))
    }

    #[test]
    fn cargo_build_follows_dependency_facts() {
        let rules = RuleSet::from_sources(&[(
            "build.rhai",
            r#"
                fn check(g) {
                    let out = [];
                    let build = g.cargo_build("pkg:cargo/a", true);
                    for p in ["pkg:cargo/a", "pkg:cargo/b", "pkg:cargo/lancedb"] {
                        if !build.packages.contains(p) { out.push(#{ message: "missing " + p }); }
                    }
                    if build.via["pkg:cargo/lancedb"] != "pkg:cargo/a" { out.push(#{ message: "via" }); }
                    if build.packages["pkg:cargo/b"] != ["default"] { out.push(#{ message: "features" }); }
                    if g.cargo_build("pkg:cargo/nowhere", false).packages.len() != 1 { out.push(#{ message: "unknown root" }); }
                    out
                }
            "#,
        )])
        .expect("compiles");
        let report = rules.run(graph()).expect("runs");
        assert!(
            report.outcomes[0].violations.is_empty(),
            "{:?}",
            report.outcomes[0]
        );
    }

    #[test]
    fn rules_run_with_severities_and_citations() {
        let rules = RuleSet::from_sources(&[
            ("lance.rhai", r#"
                const ID = "only-store-names-lance";
                const DESC = "lancedb belongs to the store";
                fn check(g) {
                    let out = [];
                    for e in g.edges_to("pkg:cargo/lancedb", "DEPENDS_ON") {
                        if e.src != "pkg:cargo/store" { out.push(violation(e, "names lancedb")); }
                    }
                    out
                }
            "#),
            ("cycles.rhai", r#"
                const SEVERITY = "budget";
                const BUDGET = 1;
                fn check(g) {
                    let out = [];
                    for group in g.cycles(["DEPENDS_ON"]) { out.push(#{ message: "cycle " + group.len(), subject: group[0] }); }
                    out
                }
            "#),
            ("consts.rhai", r#"
                const WANTED = ["pkg:cargo/a"];
                const LIMIT = 1;
                fn check(g) {
                    let out = [];
                    if g.count("Package") < LIMIT { out.push(#{ message: "limit" }); }
                    for e in g.edges("DEPENDS_ON") { if e.src in WANTED && e.dst == "pkg:cargo/b" { out.push(violation(e, "a depends on b")); } }
                    out
                }
            "#),
            ("owner.rhai", r#"
                const SEVERITY = "warn";
                fn check(g) {
                    let out = [];
                    if g.package_of("rust:a/src/lib.rs::go") != "pkg:cargo/a" { out.push(violation(g.node("rust:a/src/lib.rs::go"), "wrong owner")); }
                    if !g.reaches("pkg:cargo/a", "pkg:cargo/b", ["DEPENDS_ON"], 1) { out.push(#{ message: "a should reach b" }); }
                    if g.count("Function") != 1 { out.push(#{ message: "count" }); }
                    out
                }
            "#),
        ])
        .expect("rules compile");
        assert_eq!(
            rules.rules()[1].id,
            "cycles",
            "the file stem names an anonymous rule"
        );
        let report = rules.run(graph()).expect("rules run");
        let by_id = |id: &str| report.outcomes.iter().find(|o| o.id == id).expect(id);
        let lance = by_id("only-store-names-lance");
        assert!(!lance.passed);
        assert_eq!(
            lance.violations[0].citation.as_deref(),
            Some("a/Cargo.toml:5")
        );
        assert_eq!(
            lance.violations[0].subject,
            "pkg:cargo/a -DEPENDS_ON-> pkg:cargo/lancedb"
        );
        let cycles = by_id("cycles");
        assert!(cycles.passed, "one cycle is within the budget of one");
        assert_eq!(cycles.violations.len(), 1);
        let owner = by_id("owner");
        assert!(
            owner.passed && owner.violations.is_empty(),
            "{:?}",
            owner.violations
        );
        let consts = by_id("consts");
        assert_eq!(
            consts.violations.len(),
            1,
            "constants are visible to check: {:?}",
            consts.violations
        );
        assert!(!report.passed());
        assert_eq!(report.violation_count(), 3);
    }

    /// The Causeway port under `examples/causeway-rules/` must keep compiling and
    /// running against the engine: every rule loads, declares an identifier, and
    /// finds nothing in a graph that has none of Causeway's structures.
    #[test]
    fn the_causeway_example_rules_load_and_run() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/causeway-rules");
        let rules = RuleSet::load(&[dir], &[]).expect("the example rules compile");
        let ids: Vec<&str> = rules.rules().iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "gate-staleness",
                "facade-size-budget",
                "runtime-env-resource-ctors",
                "component-cross-deps",
                "crypto-provider-installs",
                "cargo-feature-isolation",
                "server-rpc-contract-deps",
                "server-rpc-service-impls",
                "server-orchestrators",
                "island-cross-references",
                "cross-slice-schema-refs",
                "domain-god-crate-modules",
                "domain-god-crate-dependents",
            ],
            "one rule per ported check, in file order"
        );
        let report = rules.run(graph()).expect("the example rules run");
        let stale = &report.outcomes[0];
        assert!(
            !stale.passed && stale.violations.len() > 300,
            "the fixture graph has none of Causeway's paths: {}",
            stale.violations.len()
        );
        assert!(
            report.outcomes[1..]
                .iter()
                .all(|o| o.passed && o.violations.is_empty()),
            "{:?}",
            report
                .outcomes
                .iter()
                .filter(|o| !o.passed)
                .map(|o| &o.id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn malformed_rules_are_refused() {
        assert!(
            RuleSet::from_sources(&[("x.rhai", "const SEVERITY = \"loud\"; fn check(g) { [] }")])
                .is_err()
        );
        assert!(
            RuleSet::from_sources(&[("x.rhai", "const SEVERITY = \"budget\"; fn check(g) { [] }")])
                .is_err()
        );
        assert!(RuleSet::from_sources(&[("x.rhai", "const ID = \"no-check\";")]).is_err());
    }
}
