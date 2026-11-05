//! In-memory graph compute over `petgraph`.
//!
//! [`CodeGraph`] loads the nodes and edges of one or more repositories and answers the
//! relationship questions agents ask: neighbors by edge kind, reverse impact to a
//! depth, the shortest path between two symbols, hubs by `PageRank`, communities by
//! label propagation, and cycles by strongly connected components. Every answer
//! returns the edges traversed, so the caller can cite their evidence.
//!
//! # Examples
//!
//! ```
//! use surveyor_graph::{CodeGraph, EdgeFilter};
//! use surveyor_model::GraphDelta;
//!
//! let graph = CodeGraph::from_delta(GraphDelta::new());
//! assert_eq!(graph.node_count(), 0);
//! assert!(graph.find("anything", None, 5).is_empty());
//! let _ = EdgeFilter::default();
//! ```

// Rust guideline compliant 2026-02-21

mod card;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use surveyor_model::{Edge, EdgeKind, GraphDelta, Node, NodeId, NodeKind};

#[doc(inline)]
pub use card::{Card, Count, RuleNote, RuleStatus, SAMPLES_PER_COUNT, Sample, first_paragraph};

/// The direction of an edge relative to a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Edges that start at the node.
    Outgoing,
    /// Edges that end at the node.
    Incoming,
}

impl From<Direction> for petgraph::Direction {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Outgoing => petgraph::Direction::Outgoing,
            Direction::Incoming => petgraph::Direction::Incoming,
        }
    }
}

/// Which edges a traversal may follow.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeFilter {
    /// Edge kinds allowed; `None` allows every kind.
    pub kinds: Option<Vec<EdgeKind>>,
    /// Drop edges whose derivation is not deterministic (inferred or observed).
    pub deterministic_only: bool,
    /// Drop edges below this confidence.
    pub min_confidence: f32,
    /// Follow `CONTAINS` and `DEFINES` edges.
    pub include_structural: bool,
}

impl Default for EdgeFilter {
    fn default() -> Self {
        Self {
            kinds: None,
            deterministic_only: false,
            min_confidence: 0.0,
            include_structural: true,
        }
    }
}

impl EdgeFilter {
    /// Only the kinds along which a change propagates, deterministic edges only.
    #[must_use]
    pub fn impact() -> Self {
        Self {
            kinds: Some(EdgeKind::impact_kinds().to_vec()),
            deterministic_only: true,
            min_confidence: 0.0,
            include_structural: false,
        }
    }

    /// Certain, deterministic, non-structural edges that are not test coverage.
    ///
    /// The first tier a path search tries: a chain of these is a dependency an engineer
    /// would recognize, with no name-matching guesses in it.
    #[must_use]
    pub fn precise() -> Self {
        Self {
            kinds: Some(Self::dependency_kinds()),
            deterministic_only: true,
            min_confidence: 1.0,
            include_structural: false,
        }
    }

    /// Non-structural edges that are not test coverage, at any confidence.
    #[must_use]
    pub fn loose() -> Self {
        Self {
            kinds: Some(Self::dependency_kinds()),
            ..Self::default()
        }
    }

    fn dependency_kinds() -> Vec<EdgeKind> {
        EdgeKind::builtin()
            .iter()
            .filter(|k| !k.is_structural() && **k != EdgeKind::TestedBy)
            .cloned()
            .collect()
    }

    /// Restricts to the given kinds and returns the filter.
    #[must_use]
    pub fn with_kinds(mut self, kinds: Vec<EdgeKind>) -> Self {
        self.kinds = Some(kinds);
        self
    }

    /// Requires at least `confidence` and returns the filter.
    #[must_use]
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence;
        self
    }

    fn accepts(&self, edge: &Edge) -> bool {
        if !self.include_structural && edge.kind.is_structural() {
            return false;
        }
        if let Some(kinds) = &self.kinds
            && !kinds.contains(&edge.kind)
        {
            return false;
        }
        if self.deterministic_only && !edge.provenance.is_deterministic() {
            return false;
        }
        edge.provenance.confidence.get() >= self.min_confidence
    }
}

/// A node reached from another, with the edge that reached it.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor<'a> {
    /// The node reached.
    pub node: &'a Node,
    /// The edge traversed.
    pub edge: &'a Edge,
    /// True when the edge points from the start node to this one.
    pub outgoing: bool,
}

/// How far a reverse impact walk goes, and how weak a path it still reports.
///
/// Confidence multiplies along a path, so two name-matched hops at 0.5 reach a node at
/// 0.25 and a third drops it below the default floor. A node pruned through a weak path
/// is still reported if a stronger path reaches it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reach {
    /// Hops to follow backwards.
    pub depth: usize,
    /// Smallest path confidence still reported; `0.0` keeps everything.
    pub min_confidence: f32,
}

impl Reach {
    /// The floor `depth` applies: a path of two guessed hops, no weaker.
    pub const DEFAULT_MIN_CONFIDENCE: f32 = 0.25;

    /// `depth` hops with the default floor.
    #[must_use]
    pub const fn depth(depth: usize) -> Self {
        Self {
            depth,
            min_confidence: Self::DEFAULT_MIN_CONFIDENCE,
        }
    }

    /// Reports paths down to `min_confidence` and returns the reach.
    #[must_use]
    pub const fn with_min_confidence(self, min_confidence: f32) -> Self {
        Self {
            min_confidence,
            ..self
        }
    }
}

/// A node in the reverse impact set of a change.
#[derive(Debug, Clone, PartialEq)]
pub struct Affected<'a> {
    /// The node that may break.
    pub node: &'a Node,
    /// Hops from the changed node.
    pub depth: usize,
    /// The edge that brought the impact one hop closer to this node.
    pub via: &'a Edge,
    /// Product of confidences along the path.
    pub confidence: f32,
}

/// How trustworthy the edges of a found path are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathTier {
    /// Every hop is certain and deterministic, and none is test coverage.
    Precise,
    /// Dependency edges only, some resolved by name with less than full confidence.
    Loose,
    /// Any edge, including containment and test coverage.
    Any,
}

/// One step of a path: the node and the edge used to reach it (none for the start).
#[derive(Debug, Clone, PartialEq)]
pub struct PathStep<'a> {
    /// The node.
    pub node: &'a Node,
    /// The edge used to reach it, if any.
    pub edge: Option<&'a Edge>,
}

/// Whether a found path followed edge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathDirection {
    /// Every hop follows its edge's direction: a dependency chain.
    Directed,
    /// Some hop runs against its edge's direction: a lead, not a dependency chain.
    Undirected,
}

/// How a path search treats edge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PathMode {
    /// Follow edge direction, and search against it only when no directed path exists.
    #[default]
    DirectedFirst,
    /// Follow edge direction only.
    Directed,
    /// Ignore edge direction.
    Undirected,
}

impl PathMode {
    /// The directions to try, in order.
    fn directions(self) -> &'static [PathDirection] {
        match self {
            Self::DirectedFirst => &[PathDirection::Directed, PathDirection::Undirected],
            Self::Directed => &[PathDirection::Directed],
            Self::Undirected => &[PathDirection::Undirected],
        }
    }
}

/// A path a search found, with how far it can be trusted.
#[derive(Debug, Clone, PartialEq)]
pub struct FoundPath<'a> {
    /// How trustworthy the edges are.
    pub tier: PathTier,
    /// Whether every hop followed its edge's direction.
    pub direction: PathDirection,
    /// The start node, then one step per hop.
    pub steps: Vec<PathStep<'a>>,
}

/// What a name query resolved to.
#[derive(Debug, Clone, PartialEq)]
pub enum Lookup<'a> {
    /// The query is a node identifier.
    Exact(&'a Node),
    /// One node matches the query better than any other.
    Unique(&'a Node),
    /// Several nodes match the query equally well, best first.
    Ambiguous(Vec<&'a Node>),
    /// Nothing matches the query; `similar` are nodes of the same name whose
    /// qualifier differs, when the query carried one.
    NotFound {
        /// Same-named nodes the query's qualifier excluded.
        similar: Vec<&'a Node>,
    },
}

/// A node many others depend on.
#[derive(Debug, Clone, PartialEq)]
pub struct Hub<'a> {
    /// The node.
    pub node: &'a Node,
    /// Distinct files whose nodes have an accepted edge into this node.
    pub dependent_files: usize,
    /// Accepted incoming edges.
    pub in_degree: usize,
    /// Accepted outgoing edges.
    pub out_degree: usize,
    /// `PageRank` over the accepted edges, for reference.
    pub rank: f64,
}

/// A cluster of nodes that reference each other more than the rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Community {
    /// Stable label: the lexically smallest member identifier.
    pub id: String,
    /// Member node identifiers, sorted.
    pub members: Vec<NodeId>,
}

/// A loaded graph.
#[derive(Debug)]
pub struct CodeGraph {
    graph: DiGraph<usize, usize>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    by_id: HashMap<NodeId, NodeIndex>,
    by_name: HashMap<String, Vec<NodeIndex>>,
}

impl CodeGraph {
    /// Builds a graph from nodes and edges; edges with unknown endpoints are dropped.
    #[must_use]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut graph = DiGraph::with_capacity(nodes.len(), edges.len());
        let mut by_id = HashMap::with_capacity(nodes.len());
        let mut by_name: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            let ix = graph.add_node(i);
            by_id.insert(node.id.clone(), ix);
            by_name
                .entry(node.name.to_lowercase())
                .or_default()
                .push(ix);
        }
        let mut kept: Vec<Edge> = Vec::with_capacity(edges.len());
        for edge in edges {
            let (Some(&s), Some(&d)) = (by_id.get(&edge.src), by_id.get(&edge.dst)) else {
                continue;
            };
            let ei = kept.len();
            graph.add_edge(s, d, ei);
            kept.push(edge);
        }
        Self {
            graph,
            nodes,
            edges: kept,
            by_id,
            by_name,
        }
    }

    /// Builds a graph from a linker delta.
    #[must_use]
    pub fn from_delta(delta: GraphDelta) -> Self {
        Self::new(delta.nodes, delta.edges)
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// All nodes.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// All edges.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// The node with `id`, if present.
    #[must_use]
    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.by_id.get(id).map(|ix| &self.nodes[self.graph[*ix]])
    }

    /// The edge with the given identifier, if present.
    #[must_use]
    pub fn edge(&self, id: &surveyor_model::EdgeId) -> Option<&Edge> {
        self.edges.iter().find(|e| &e.id == id)
    }

    /// Finds nodes by name, qualified name or identifier.
    ///
    /// Exact identifier matches come first, then exact names, then qualified names
    /// that end with the query, then substrings. `kind` restricts the result.
    #[must_use]
    pub fn find(&self, query: &str, kind: Option<&NodeKind>, limit: usize) -> Vec<&Node> {
        self.scored(query, kind)
            .into_iter()
            .map(|(_, n)| n)
            .take(limit)
            .collect()
    }

    /// The nodes a question is about: `find` when the question is a name, otherwise
    /// the nodes its identifier-like terms name, those matching the most terms first.
    ///
    /// `who calls OutboxEventSink` seeds on `OutboxEventSink`; `outbox relay drain`
    /// seeds on every node one of those words names. Question words (`who`, `calls`,
    /// `table`) are dropped, so a name that is also such a word is found only when it
    /// is the whole query.
    #[must_use]
    pub fn seeds(&self, question: &str, limit: usize) -> Vec<&Node> {
        let whole = self.find(question, None, limit);
        if !whole.is_empty() {
            return whole;
        }
        let terms: Vec<&str> = question
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':' || c == '.'))
            .map(|t| t.trim_matches(|c: char| c == ':' || c == '.'))
            .filter(|t| t.len() >= 3 && !QUESTION_WORDS.contains(&t.to_lowercase().as_str()))
            .collect();
        let mut hits: HashMap<&NodeId, (usize, u8, &Node)> = HashMap::new();
        for term in terms {
            for (score, node) in self.scored(term, None).into_iter().take(limit * 4) {
                let entry = hits.entry(&node.id).or_insert((0, score, node));
                entry.0 += 1;
                entry.1 = entry.1.min(score);
            }
        }
        let mut ranked: Vec<(usize, u8, &Node)> = hits.into_values().collect();
        ranked.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then(a.1.cmp(&b.1))
                .then_with(|| kind_priority(&a.2.kind).cmp(&kind_priority(&b.2.kind)))
                .then_with(|| a.2.id.cmp(&b.2.id))
        });
        ranked.into_iter().map(|(_, _, n)| n).take(limit).collect()
    }

    /// Resolves a query to one node, or says why it cannot.
    ///
    /// An identifier resolves exactly. Otherwise the best-scoring matches are
    /// compared: one is `Unique`; several equally good are `Ambiguous`, in rank
    /// order; and a query whose qualifier (`path::`, `Type::`, `schema.`) matches no
    /// declaration of that name is `NotFound`, with the same-named nodes listed, so a
    /// mistyped qualifier is never silently answered with another file's node.
    #[must_use]
    pub fn lookup(&self, query: &str, kind: Option<&NodeKind>) -> Lookup<'_> {
        // An identifier needs no ranking; a batch of them (every card of a context
        // pack) would otherwise pay a substring scan of every node per query.
        if let Some(node) = self
            .node(&NodeId::new(query.trim()))
            .filter(|n| kind.is_none_or(|k| &n.kind == k))
        {
            return Lookup::Exact(node);
        }
        let scored = self.scored(query, kind);
        let Some(&(best, first)) = scored.first() else {
            return Lookup::NotFound {
                similar: Vec::new(),
            };
        };
        if best == 0 {
            return Lookup::Exact(first);
        }
        let qualified = query.contains("::") || query.contains('/') || query.contains('.');
        if best == 4 && qualified {
            return Lookup::NotFound {
                similar: scored.into_iter().map(|(_, n)| n).take(10).collect(),
            };
        }
        let ties: Vec<&Node> = scored
            .into_iter()
            .take_while(|(score, _)| *score == best)
            .map(|(_, n)| n)
            .collect();
        // A type's impl blocks, a test named after what it tests
        // (`describe("OutboxPanel")`), a task named after its script and a document
        // yield to the declaration; only same-named declarations are ambiguous.
        let mut declared: Vec<&Node> = ties
            .iter()
            .copied()
            .filter(|n| {
                !matches!(
                    n.kind,
                    NodeKind::Test | NodeKind::Impl | NodeKind::Task | NodeKind::Document
                )
            })
            .collect();
        if let [one] = declared.as_slice() {
            return Lookup::Unique(one);
        }
        if declared.is_empty() && ties.len() == 1 {
            return Lookup::Unique(first);
        }
        if declared.is_empty() {
            declared = ties;
        }
        declared.truncate(10);
        Lookup::Ambiguous(declared)
    }

    /// Every match of `query`, ranked: 0 identifier, 1 exact name or qualified
    /// name, 2 the same ignoring case, 3 qualified name or identifier ending with
    /// the query, 4 same simple name, 5 substring. Case decides between a type and
    /// the module named after it: `Dimension` is the enum, `dimension` the module.
    fn scored(&self, query: &str, kind: Option<&NodeKind>) -> Vec<(u8, &Node)> {
        let q = query.trim();
        if q.is_empty() {
            return Vec::new();
        }
        let ql = q.to_lowercase();
        let last = ql
            .rsplit("::")
            .next()
            .unwrap_or(&ql)
            .rsplit('.')
            .next()
            .unwrap_or(&ql)
            .to_owned();

        let exact_id = self.node(&NodeId::new(q)).map(|n| (0u8, n));
        let by_name = self.by_name.get(&last).into_iter().flatten().map(|ix| {
            let n = &self.nodes[self.graph[*ix]];
            let qn = n.qualified_name.to_lowercase();
            let score = if n.qualified_name == q || n.name == q {
                1
            } else if qn == ql || n.name.to_lowercase() == ql {
                2
            } else if qn.ends_with(&ql) || n.id.as_str().to_lowercase().ends_with(&ql) {
                3
            } else {
                4
            };
            (score, n)
        });
        let substrings = self
            .nodes
            .iter()
            .filter(|n| {
                n.qualified_name.to_lowercase().contains(&ql)
                    || n.id.as_str().to_lowercase().contains(&ql)
            })
            .map(|n| (5u8, n));

        let mut seen: HashSet<&NodeId> = HashSet::new();
        let mut scored: Vec<(u8, &Node)> = exact_id
            .into_iter()
            .chain(by_name)
            .chain(substrings)
            .filter(|(_, n)| seen.insert(&n.id))
            .filter(|(_, n)| kind.is_none_or(|k| &n.kind == k))
            .collect();
        scored.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| kind_priority(&a.1.kind).cmp(&kind_priority(&b.1.kind)))
                .then_with(|| a.1.qualified_name.len().cmp(&b.1.qualified_name.len()))
                .then_with(|| a.1.id.cmp(&b.1.id))
        });
        scored
    }

    /// Direct neighbors of `id` in `direction`, through edges the filter accepts.
    #[must_use]
    pub fn neighbors(
        &self,
        id: &NodeId,
        direction: Direction,
        filter: &EdgeFilter,
    ) -> Vec<Neighbor<'_>> {
        let Some(&ix) = self.by_id.get(id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for er in self.graph.edges_directed(ix, direction.into()) {
            let edge = &self.edges[*er.weight()];
            if !filter.accepts(edge) {
                continue;
            }
            let other = if direction == Direction::Outgoing {
                er.target()
            } else {
                er.source()
            };
            out.push(Neighbor {
                node: &self.nodes[self.graph[other]],
                edge,
                outgoing: direction == Direction::Outgoing,
            });
        }
        out.sort_by(|a, b| {
            a.edge
                .kind
                .cmp(&b.edge.kind)
                .then_with(|| a.node.id.cmp(&b.node.id))
        });
        out
    }

    /// Neighbors in both directions.
    #[must_use]
    pub fn all_neighbors(&self, id: &NodeId, filter: &EdgeFilter) -> Vec<Neighbor<'_>> {
        let mut out = self.neighbors(id, Direction::Outgoing, filter);
        out.extend(self.neighbors(id, Direction::Incoming, filter));
        out
    }

    /// Nodes that may break when `id` changes: reverse traversal within `reach`.
    ///
    /// Follows the filter's edge kinds backwards (a caller is affected by its callee,
    /// a query by its table), breadth first, reporting each node once at its shortest
    /// depth with the confidence product along the path found. A path whose product
    /// falls below the reach's floor is neither reported nor followed further.
    #[must_use]
    pub fn affected(&self, id: &NodeId, reach: Reach, filter: &EdgeFilter) -> Vec<Affected<'_>> {
        let Some(&start) = self.by_id.get(id) else {
            return Vec::new();
        };
        let mut seen: HashMap<NodeIndex, usize> = HashMap::new();
        seen.insert(start, 0);
        let mut queue: VecDeque<(NodeIndex, usize, f32)> = VecDeque::new();
        queue.push_back((start, 0, 1.0));
        let mut out: Vec<Affected<'_>> = Vec::new();
        while let Some((ix, d, conf)) = queue.pop_front() {
            if d == reach.depth {
                continue;
            }
            let mut incoming: Vec<(EdgeIndex, NodeIndex)> = self
                .graph
                .edges_directed(ix, petgraph::Direction::Incoming)
                .map(|er| (er.id(), er.source()))
                .collect();
            incoming.sort_by_key(|(e, _)| self.edges[self.graph[*e]].id.as_str().to_owned());
            for (ei, src) in incoming {
                let edge = &self.edges[self.graph[ei]];
                if !filter.accepts(edge) || seen.contains_key(&src) {
                    continue;
                }
                let c = conf * edge.provenance.confidence.get();
                if c < reach.min_confidence {
                    continue;
                }
                seen.insert(src, d + 1);
                let node = &self.nodes[self.graph[src]];
                out.push(Affected {
                    node,
                    depth: d + 1,
                    via: edge,
                    confidence: c,
                });
                // A file that imports the symbol is affected; the files that import
                // that file depend on everything it exports, which is too coarse to
                // count as affected by one symbol. Ask about the file itself to walk
                // its importers.
                if node.kind != NodeKind::File {
                    queue.push_back((src, d + 1, c));
                }
            }
        }
        out
    }

    /// Shortest path from `from` to `to`, treating edges as undirected unless `directed`.
    #[must_use]
    pub fn shortest_path(
        &self,
        from: &NodeId,
        to: &NodeId,
        filter: &EdgeFilter,
        directed: bool,
    ) -> Option<Vec<PathStep<'_>>> {
        let (&a, &b) = (self.by_id.get(from)?, self.by_id.get(to)?);
        let mut prev: HashMap<NodeIndex, (NodeIndex, usize)> = HashMap::new();
        let mut queue = VecDeque::from([a]);
        let mut seen: HashSet<NodeIndex> = HashSet::from([a]);
        'bfs: while let Some(ix) = queue.pop_front() {
            let dirs: &[petgraph::Direction] = if directed {
                &[petgraph::Direction::Outgoing]
            } else {
                &[petgraph::Direction::Outgoing, petgraph::Direction::Incoming]
            };
            for dir in dirs {
                for er in self.graph.edges_directed(ix, *dir) {
                    let edge = &self.edges[*er.weight()];
                    if !filter.accepts(edge) {
                        continue;
                    }
                    let next = if *dir == petgraph::Direction::Outgoing {
                        er.target()
                    } else {
                        er.source()
                    };
                    if seen.insert(next) {
                        prev.insert(next, (ix, *er.weight()));
                        if next == b {
                            break 'bfs;
                        }
                        queue.push_back(next);
                    }
                }
            }
        }
        if a != b && !prev.contains_key(&b) {
            return None;
        }
        let mut steps: Vec<PathStep<'_>> = Vec::new();
        let mut cur = b;
        while cur != a {
            let (p, e) = prev[&cur];
            steps.push(PathStep {
                node: &self.nodes[self.graph[cur]],
                edge: Some(&self.edges[e]),
            });
            cur = p;
        }
        steps.push(PathStep {
            node: &self.nodes[self.graph[a]],
            edge: None,
        });
        steps.reverse();
        Some(steps)
    }

    /// `PageRank` of every node, indexed like `self.nodes`, over the accepted edges.
    ///
    /// Power iteration over the edge list, linear in edges per round, so a graph of a
    /// few hundred thousand edges ranks in well under a second.
    fn page_rank_scores(&self, filter: &EdgeFilter) -> Vec<f64> {
        let n = self.graph.node_count();
        if n == 0 {
            return Vec::new();
        }
        let accepted: Vec<(usize, usize)> = self
            .graph
            .edge_references()
            .filter(|er| filter.accepts(&self.edges[*er.weight()]))
            .map(|er| (er.source().index(), er.target().index()))
            .collect();
        let mut out_degree = vec![0usize; n];
        for (s, _) in &accepted {
            out_degree[*s] += 1;
        }
        // 0.85 is the damping factor of the original PageRank paper; 30 rounds
        // converge well past three decimals on graphs of this size.
        const DAMPING: f64 = 0.85;
        const ROUNDS: usize = 30;
        let uniform = 1.0 / n as f64;
        let mut rank = vec![uniform; n];
        let mut next = vec![0.0f64; n];
        for _ in 0..ROUNDS {
            let dangling: f64 = (0..n)
                .filter(|i| out_degree[*i] == 0)
                .map(|i| rank[i])
                .sum();
            let base = (1.0 - DAMPING) * uniform + DAMPING * dangling * uniform;
            next.fill(base);
            for (s, d) in &accepted {
                next[*d] += DAMPING * rank[*s] / out_degree[*s] as f64;
            }
            std::mem::swap(&mut rank, &mut next);
        }
        rank
    }

    /// The `top` nodes by `PageRank` over the edges the filter accepts.
    #[must_use]
    pub fn page_rank(&self, top: usize, filter: &EdgeFilter) -> Vec<(&Node, f64)> {
        let mut scored: Vec<(&Node, f64)> = self
            .page_rank_scores(filter)
            .into_iter()
            .enumerate()
            .map(|(i, r)| (&self.nodes[self.graph[NodeIndex::new(i)]], r))
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.id.cmp(&b.0.id))
        });
        scored.truncate(top);
        scored
    }

    /// The `top` hubs: nodes ranked by how many distinct files depend on them.
    ///
    /// Fan-in across files is what a change ripples through; `PageRank` alone rewards
    /// small closed cycles (an error enum and its `Display` impl) and is kept only as a
    /// tiebreaker and for reference.
    #[must_use]
    pub fn hubs(&self, top: usize, filter: &EdgeFilter) -> Vec<Hub<'_>> {
        let ranks = self.page_rank_scores(filter);
        let mut hubs: Vec<Hub<'_>> = self
            .graph
            .node_indices()
            .map(|ix| {
                let node = &self.nodes[self.graph[ix]];
                let incoming: Vec<&Node> = self
                    .graph
                    .edges_directed(ix, petgraph::Direction::Incoming)
                    .filter(|er| filter.accepts(&self.edges[*er.weight()]))
                    .map(|er| &self.nodes[self.graph[er.source()]])
                    .collect();
                let dependent_files = incoming
                    .iter()
                    .map(|n| n.file().map_or(n.id.as_str(), |f| f.as_str()))
                    .collect::<HashSet<_>>()
                    .len();
                let out_degree = self
                    .graph
                    .edges_directed(ix, petgraph::Direction::Outgoing)
                    .filter(|er| filter.accepts(&self.edges[*er.weight()]))
                    .count();
                Hub {
                    node,
                    dependent_files,
                    in_degree: incoming.len(),
                    out_degree,
                    rank: ranks[ix.index()],
                }
            })
            .collect();
        hubs.sort_by(|a, b| {
            b.dependent_files
                .cmp(&a.dependent_files)
                .then_with(|| b.in_degree.cmp(&a.in_degree))
                .then_with(|| {
                    b.rank
                        .partial_cmp(&a.rank)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.node.id.cmp(&b.node.id))
        });
        hubs.truncate(top);
        hubs
    }

    /// The best path from `from` to `to`.
    ///
    /// Directions are tried in the order `mode` gives, and within a direction the
    /// tiers in order: precise edges, then any dependency edge, then any edge at all.
    /// The result says which direction and tier succeeded.
    #[must_use]
    pub fn best_path(&self, from: &NodeId, to: &NodeId, mode: PathMode) -> Option<FoundPath<'_>> {
        mode.directions()
            .iter()
            .flat_map(|direction| {
                [
                    (PathTier::Precise, EdgeFilter::precise()),
                    (PathTier::Loose, EdgeFilter::loose()),
                    (PathTier::Any, EdgeFilter::default()),
                ]
                .into_iter()
                .map(move |(tier, filter)| (*direction, tier, filter))
            })
            .find_map(|(direction, tier, filter)| {
                self.shortest_path(from, to, &filter, direction == PathDirection::Directed)
                    .map(|steps| FoundPath {
                        tier,
                        direction,
                        steps,
                    })
            })
    }

    /// Communities by label propagation over the edges the filter accepts.
    ///
    /// Deterministic: nodes are visited in identifier order and ties break toward the
    /// smallest label. Singleton communities are omitted.
    #[must_use]
    pub fn communities(&self, filter: &EdgeFilter) -> Vec<Community> {
        let n = self.nodes.len();
        let mut order: Vec<NodeIndex> = self.graph.node_indices().collect();
        order.sort_by(|a, b| {
            self.nodes[self.graph[*a]]
                .id
                .cmp(&self.nodes[self.graph[*b]].id)
        });
        let mut label: Vec<usize> = (0..n).collect();
        // Twenty sweeps bound the cost on large graphs; propagation on code graphs
        // settles in well under ten.
        for _ in 0..20 {
            let mut changed = false;
            for &ix in &order {
                let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
                for dir in [petgraph::Direction::Outgoing, petgraph::Direction::Incoming] {
                    for er in self.graph.edges_directed(ix, dir) {
                        if !filter.accepts(&self.edges[*er.weight()]) {
                            continue;
                        }
                        let other = if dir == petgraph::Direction::Outgoing {
                            er.target()
                        } else {
                            er.source()
                        };
                        *counts.entry(label[other.index()]).or_default() += 1;
                    }
                }
                let Some((&best, _)) = counts
                    .iter()
                    .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                else {
                    continue;
                };
                if label[ix.index()] != best {
                    label[ix.index()] = best;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut groups: BTreeMap<usize, Vec<NodeId>> = BTreeMap::new();
        for ix in self.graph.node_indices() {
            groups
                .entry(label[ix.index()])
                .or_default()
                .push(self.nodes[self.graph[ix]].id.clone());
        }
        let mut out: Vec<Community> = groups
            .into_values()
            .filter(|m| m.len() > 1)
            .map(|mut members| {
                members.sort();
                Community {
                    id: members[0].to_string(),
                    members,
                }
            })
            .collect();
        out.sort_by(|a, b| {
            b.members
                .len()
                .cmp(&a.members.len())
                .then_with(|| a.id.cmp(&b.id))
        });
        out
    }

    /// Strongly connected components with at least `min_size` nodes (dependency cycles).
    #[must_use]
    pub fn cycles(&self, min_size: usize, filter: &EdgeFilter) -> Vec<Vec<&Node>> {
        let mut sub: DiGraph<(), ()> =
            DiGraph::with_capacity(self.graph.node_count(), self.graph.edge_count());
        for _ in 0..self.graph.node_count() {
            sub.add_node(());
        }
        for er in self.graph.edge_references() {
            if filter.accepts(&self.edges[*er.weight()]) {
                sub.add_edge(er.source(), er.target(), ());
            }
        }
        let mut out: Vec<Vec<&Node>> = petgraph::algo::tarjan_scc(&sub)
            .into_iter()
            .filter(|c| c.len() >= min_size.max(2))
            .map(|c| {
                let mut nodes: Vec<&Node> = c
                    .into_iter()
                    .map(|ix| &self.nodes[self.graph[ix]])
                    .collect();
                nodes.sort_by(|a, b| a.id.cmp(&b.id));
                nodes
            })
            .collect();
        out.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a[0].id.cmp(&b[0].id)));
        out
    }

    /// Nodes of `kind`, in identifier order.
    #[must_use]
    pub fn nodes_of_kind(&self, kind: &NodeKind) -> Vec<&Node> {
        let mut out: Vec<&Node> = self.nodes.iter().filter(|n| &n.kind == kind).collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }
}

/// Symbols rank above files and files above structure when a query matches several.
/// Words a question is made of, never the name it asks about.
const QUESTION_WORDS: &[&str] = &[
    "who",
    "what",
    "where",
    "which",
    "how",
    "why",
    "when",
    "does",
    "did",
    "the",
    "and",
    "for",
    "that",
    "this",
    "with",
    "from",
    "into",
    "are",
    "was",
    "were",
    "can",
    "could",
    "should",
    "calls",
    "call",
    "called",
    "caller",
    "callers",
    "uses",
    "use",
    "used",
    "using",
    "reads",
    "read",
    "writes",
    "write",
    "written",
    "touches",
    "touch",
    "handles",
    "handled",
    "handle",
    "defines",
    "defined",
    "define",
    "implements",
    "implemented",
    "depends",
    "depend",
    "table",
    "function",
    "method",
    "type",
    "struct",
    "class",
    "module",
    "file",
    "code",
    "find",
    "show",
    "list",
    "about",
    "between",
    "path",
    "all",
    "any",
    "its",
    "their",
];

fn kind_priority(kind: &NodeKind) -> u8 {
    match kind {
        k if k.is_symbol() => 0,
        NodeKind::Table | NodeKind::View | NodeKind::Schema | NodeKind::Column => 1,

        NodeKind::Module | NodeKind::Package => 2,
        NodeKind::File => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use surveyor_model::{
        ByteSpan, CommitId, Confidence, ContentHash, Evidence, GrammarId, LineSpan, Provenance,
        RepoId, RepoPath, RuleId, SourceLocation, TextSpan,
    };

    use super::*;

    fn node(id: &str, kind: NodeKind) -> Node {
        let name = id.rsplit("::").next().unwrap_or(id).to_owned();
        Node::synthetic(NodeId::new(id), kind, name, RepoId::new("r")).with_qualified_name(id)
    }

    fn edge(src: &str, kind: EdgeKind, dst: &str, confidence: f32) -> Edge {
        let ev = Evidence {
            repo: RepoId::new("r"),
            location: SourceLocation::new(
                RepoPath::new("f.rs"),
                TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1)),
            ),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(b"x"),
        };
        let prov = Provenance::syntactic(GrammarId::new("rust"), RuleId::new("r"), ev)
            .with_confidence(Confidence::new(confidence));
        Edge::new(NodeId::new(src), kind, NodeId::new(dst), prov)
    }

    fn sample() -> CodeGraph {
        let nodes = vec![
            node("a::main", NodeKind::Function),
            node("a::service", NodeKind::Function),
            node("a::repo", NodeKind::Function),
            node("sql:s.t", NodeKind::Table),
            node("a::tests::t1", NodeKind::Test),
            node("file:a.rs", NodeKind::File),
        ];
        let edges = vec![
            edge("a::main", EdgeKind::Calls, "a::service", 1.0),
            edge("a::service", EdgeKind::Calls, "a::repo", 0.5),
            edge("a::repo", EdgeKind::Reads, "sql:s.t", 1.0),
            edge("a::tests::t1", EdgeKind::Calls, "a::service", 1.0),
            edge("file:a.rs", EdgeKind::Contains, "a::main", 1.0),
        ];
        CodeGraph::new(nodes, edges)
    }

    #[test]
    fn affected_walks_backwards_with_confidence() {
        let g = sample();
        let hits = g.affected(
            &NodeId::new("sql:s.t"),
            Reach::depth(3),
            &EdgeFilter::impact(),
        );
        let ids: Vec<&str> = hits.iter().map(|a| a.node.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["a::repo", "a::service", "a::main", "a::tests::t1"]
        );
        let main = hits
            .iter()
            .find(|a| a.node.id.as_str() == "a::main")
            .expect("main");
        assert_eq!(main.depth, 3);
        assert!((main.confidence - 0.5).abs() < 1e-6);
        assert_eq!(
            g.affected(
                &NodeId::new("sql:s.t"),
                Reach::depth(1),
                &EdgeFilter::impact()
            )
            .len(),
            1
        );
    }

    #[test]
    fn affected_reports_an_importing_file_but_does_not_walk_its_importers() {
        // `settings.py` declares S; `__init__.py` re-exports it; `app.py` and
        // `other.py` import from `__init__.py`, and only `app.py` uses S.
        let graph = CodeGraph::new(
            vec![
                node("py:settings.py::S", NodeKind::Class),
                node("file:pkg/__init__.py", NodeKind::File),
                node("file:app.py", NodeKind::File),
                node("file:other.py", NodeKind::File),
                node("py:app.py::Use", NodeKind::Class),
            ],
            vec![
                edge(
                    "file:pkg/__init__.py",
                    EdgeKind::Imports,
                    "py:settings.py::S",
                    1.0,
                ),
                edge(
                    "file:app.py",
                    EdgeKind::Imports,
                    "file:pkg/__init__.py",
                    1.0,
                ),
                edge(
                    "file:other.py",
                    EdgeKind::Imports,
                    "file:pkg/__init__.py",
                    1.0,
                ),
                edge(
                    "py:app.py::Use",
                    EdgeKind::Extends,
                    "py:settings.py::S",
                    1.0,
                ),
            ],
        );
        let mut hits: Vec<(String, usize)> = graph
            .affected(
                &NodeId::new("py:settings.py::S"),
                Reach::depth(3),
                &EdgeFilter::impact(),
            )
            .iter()
            .map(|a| (a.node.id.to_string(), a.depth))
            .collect();
        hits.sort();
        assert_eq!(
            hits,
            vec![
                ("file:pkg/__init__.py".to_owned(), 1),
                ("py:app.py::Use".to_owned(), 1)
            ],
            "the re-exporting file is affected; what imports that file is not, by this symbol"
        );
        let importers: Vec<String> = graph
            .affected(
                &NodeId::new("file:pkg/__init__.py"),
                Reach::depth(1),
                &EdgeFilter::impact(),
            )
            .iter()
            .map(|a| a.node.id.to_string())
            .collect();
        assert_eq!(
            importers.len(),
            2,
            "asked about the file, its importers are affected"
        );
    }

    #[test]
    fn affected_prunes_paths_below_the_confidence_floor() {
        // t <- repo (1.0) <- guess (0.5) <- far (0.4): far sits at 0.2, below the floor,
        // unless a second, stronger path reaches it.
        let nodes = vec![
            node("sql:s.t", NodeKind::Table),
            node("a::repo", NodeKind::Function),
            node("a::guess", NodeKind::Function),
            node("a::far", NodeKind::Function),
            node("a::direct", NodeKind::Function),
        ];
        let weak = CodeGraph::new(
            nodes.clone(),
            vec![
                edge("a::repo", EdgeKind::Reads, "sql:s.t", 1.0),
                edge("a::guess", EdgeKind::Calls, "a::repo", 0.5),
                edge("a::far", EdgeKind::Calls, "a::guess", 0.4),
            ],
        );
        let names = |hits: Vec<Affected<'_>>| -> Vec<(String, usize)> {
            hits.iter()
                .map(|a| (a.node.name.clone(), a.depth))
                .collect()
        };
        let table = NodeId::new("sql:s.t");
        assert_eq!(
            names(weak.affected(&table, Reach::depth(3), &EdgeFilter::impact())),
            vec![("repo".to_owned(), 1), ("guess".to_owned(), 2)]
        );
        let all = weak.affected(
            &table,
            Reach::depth(3).with_min_confidence(0.0),
            &EdgeFilter::impact(),
        );
        assert_eq!(all.len(), 3);
        assert!((all[2].confidence - 0.2).abs() < 1e-6);

        let strong = CodeGraph::new(
            nodes,
            vec![
                edge("a::repo", EdgeKind::Reads, "sql:s.t", 1.0),
                edge("a::guess", EdgeKind::Calls, "a::repo", 0.5),
                edge("a::far", EdgeKind::Calls, "a::guess", 0.4),
                edge("a::direct", EdgeKind::Calls, "a::repo", 1.0),
                edge("a::far", EdgeKind::Calls, "a::direct", 1.0),
            ],
        );
        let far = strong
            .affected(&table, Reach::depth(3), &EdgeFilter::impact())
            .into_iter()
            .find(|a| a.node.name == "far")
            .expect("far is reached through direct");
        assert_eq!(far.depth, 3);
        assert!((far.confidence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lookup_resolves_identifiers_unique_names_and_bad_qualifiers() {
        let g = sample();
        assert_eq!(
            g.lookup("a::service", None),
            Lookup::Exact(g.node(&NodeId::new("a::service")).unwrap())
        );
        assert_eq!(
            g.lookup("service", None),
            Lookup::Unique(g.node(&NodeId::new("a::service")).unwrap())
        );
        assert!(
            matches!(g.lookup("nowhere/else.rs::service", None), Lookup::NotFound { similar } if similar.len() == 1)
        );
        assert!(
            matches!(g.lookup("no_such_name", None), Lookup::NotFound { similar } if similar.is_empty())
        );
    }

    #[test]
    fn a_question_seeds_on_the_names_it_contains() {
        let g = CodeGraph::new(
            vec![
                node("x/outbox.rs::OutboxEventSink", NodeKind::Trait),
                node("x/relay.rs::drain", NodeKind::Function),
                node("y/other.rs::Other", NodeKind::Struct),
            ],
            Vec::new(),
        );
        let names = |nodes: Vec<&Node>| nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>();
        assert_eq!(
            names(g.seeds("OutboxEventSink", 5)),
            vec!["OutboxEventSink"]
        );
        assert_eq!(
            names(g.seeds("who calls OutboxEventSink?", 5)),
            vec!["OutboxEventSink"]
        );
        let both = names(g.seeds("where does the outbox relay drain", 5));
        assert!(
            both.contains(&"OutboxEventSink".to_owned()) && both.contains(&"drain".to_owned()),
            "{both:?}"
        );
        assert!(g.seeds("what is the weather", 5).is_empty());
    }

    #[test]
    fn exact_case_outranks_a_name_that_differs_only_in_case() {
        let g = CodeGraph::new(
            vec![
                node("x/dimension.rs::dimension", NodeKind::Module),
                node("x/dimension.rs::Dimension", NodeKind::Enum),
            ],
            Vec::new(),
        );
        assert!(
            matches!(g.lookup("Dimension", None), Lookup::Unique(n) if n.kind == NodeKind::Enum)
        );
        assert!(
            matches!(g.lookup("dimension", None), Lookup::Unique(n) if n.kind == NodeKind::Module)
        );
        assert!(matches!(g.lookup("DIMENSION", None), Lookup::Ambiguous(c) if c.len() == 2));
    }

    #[test]
    fn same_named_declarations_are_ambiguous_until_qualified() {
        let nodes = vec![
            node("x/a.rs::run", NodeKind::Function),
            node("x/b.rs::run", NodeKind::Function),
        ];
        let g = CodeGraph::new(nodes, Vec::new());
        assert!(matches!(g.lookup("run", None), Lookup::Ambiguous(c) if c.len() == 2));
        let named_test = CodeGraph::new(
            vec![
                node("x/a.rs::run", NodeKind::Function),
                node("x/a.rs::run#2", NodeKind::Test),
            ],
            Vec::new(),
        );
        assert!(matches!(
            named_test.lookup("run", None),
            Lookup::Unique(n) if n.kind == NodeKind::Function
        ));
        assert!(
            matches!(g.lookup("x/b.rs::run", None), Lookup::Exact(n) if n.id.as_str() == "x/b.rs::run")
        );
        assert!(
            matches!(g.lookup("b.rs::run", None), Lookup::Unique(n) if n.id.as_str() == "x/b.rs::run")
        );
    }

    #[test]
    fn path_find_rank_and_communities() {
        let g = sample();
        let path = g
            .shortest_path(
                &NodeId::new("a::main"),
                &NodeId::new("sql:s.t"),
                &EdgeFilter::default(),
                true,
            )
            .expect("path");
        assert_eq!(path.len(), 4);
        assert!(path[0].edge.is_none());
        assert_eq!(path[3].edge.map(|e| e.kind.clone()), Some(EdgeKind::Reads));
        assert!(
            g.shortest_path(
                &NodeId::new("sql:s.t"),
                &NodeId::new("a::main"),
                &EdgeFilter::default(),
                true
            )
            .is_none()
        );
        assert!(
            g.shortest_path(
                &NodeId::new("sql:s.t"),
                &NodeId::new("a::main"),
                &EdgeFilter::default(),
                false
            )
            .is_some()
        );

        let found = g
            .best_path(
                &NodeId::new("a::main"),
                &NodeId::new("sql:s.t"),
                PathMode::Directed,
            )
            .expect("path");
        assert_eq!(
            found.tier,
            PathTier::Loose,
            "the 0.5 call forces the second tier"
        );
        assert_eq!(found.direction, PathDirection::Directed);
        assert_eq!(found.steps.len(), 4);
        let found = g
            .best_path(
                &NodeId::new("a::main"),
                &NodeId::new("a::service"),
                PathMode::DirectedFirst,
            )
            .expect("path");
        assert_eq!(
            (found.tier, found.direction),
            (PathTier::Precise, PathDirection::Directed)
        );
        // Against the edges there is no directed path; the default falls back and says so.
        let found = g
            .best_path(
                &NodeId::new("sql:s.t"),
                &NodeId::new("a::main"),
                PathMode::DirectedFirst,
            )
            .expect("undirected fallback");
        assert_eq!(found.direction, PathDirection::Undirected);
        assert!(
            g.best_path(
                &NodeId::new("sql:s.t"),
                &NodeId::new("a::main"),
                PathMode::Directed
            )
            .is_none()
        );

        let found = g.find("service", None, 5);
        assert_eq!(found[0].id.as_str(), "a::service");
        assert_eq!(g.find("a::service", None, 5)[0].id.as_str(), "a::service");
        assert_eq!(g.find("t", Some(&NodeKind::Table), 5).len(), 1);

        let ranked = g.page_rank(2, &EdgeFilter::default());
        assert_eq!(ranked[0].0.id.as_str(), "sql:s.t");
        let hubs = g.hubs(3, &EdgeFilter::default());
        assert_eq!(
            hubs[0].node.id.as_str(),
            "a::service",
            "two callers plus the highest rank"
        );
        assert_eq!(hubs[0].in_degree, 2);

        let comms = g.communities(&EdgeFilter::default());
        assert_eq!(comms.len(), 1);
        assert_eq!(comms[0].members.len(), 6);
        assert!(g.cycles(2, &EdgeFilter::default()).is_empty());
        let n = g.neighbors(
            &NodeId::new("a::service"),
            Direction::Incoming,
            &EdgeFilter::default(),
        );
        assert_eq!(n.len(), 2);
    }
}
