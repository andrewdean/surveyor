//! A graph over more than one repository: partitions keyed by `(repo, id)`, with the
//! external package stubs folded into the packages other repositories declare.
//!
//! A single-repository graph is [`CodeGraph::new`] and knows nothing of this. When a
//! store holds several repositories, [`CodeGraph::federated`] takes the primary
//! repository's partition, any other partitions loaded (`--all-repos`, or just the
//! foreign endpoints of the federated edges), and the federated edges themselves.
//! Node identifiers are unprefixed, so two repositories may both hold
//! `file:README.md`; every edge resolves inside its own partition, and a lookup by
//! identifier alone prefers the primary repository.

// Rust guideline compliant 2026-02-21

use std::collections::{HashMap, HashSet};

use petgraph::graph::{DiGraph, NodeIndex};
use surveyor_model::{Edge, FederatedEdge, Node, NodeId, NodeKind, RepoId};

use crate::CodeGraph;

/// One repository's nodes and edges, as the store partitions them.
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    /// The repository.
    pub repo: RepoId,
    /// Its nodes.
    pub nodes: Vec<Node>,
    /// Its edges; every endpoint is one of `nodes` or a folded stub.
    pub edges: Vec<Edge>,
}

fn is_stub(node: &Node) -> bool {
    node.kind == NodeKind::Package && node.attributes.get("external").is_some_and(|v| v == "true")
}

/// The stubs that fold: `(repo, id)` of each external stub whose identifier exactly one
/// other partition declares, mapped to that partition's repository.
fn folded_stubs(partitions: &[Partition]) -> HashMap<(RepoId, NodeId), RepoId> {
    let mut declarers: HashMap<&NodeId, Vec<&RepoId>> = HashMap::new();
    for p in partitions {
        for n in p
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Package && !is_stub(n))
        {
            declarers.entry(&n.id).or_default().push(&p.repo);
        }
    }
    partitions
        .iter()
        .flat_map(|p| {
            p.nodes.iter().filter(|n| is_stub(n)).filter_map(|n| {
                let others: Vec<&RepoId> = declarers
                    .get(&n.id)
                    .map(|list| list.iter().copied().filter(|r| **r != p.repo).collect())
                    .unwrap_or_default();
                match others.as_slice() {
                    [one] => Some(((p.repo.clone(), n.id.clone()), (*one).clone())),
                    _ => None,
                }
            })
        })
        .collect()
}

impl CodeGraph {
    /// Builds a graph over several partitions with `primary` as the repository a bare
    /// identifier refers to.
    ///
    /// An external stub whose identifier exactly one other loaded partition declares
    /// is folded: the stub is dropped and its edges point at the declared node, in
    /// memory only. With two or more declarers the stub stays as the ambiguous hub and
    /// the federated edges, at their stored `1/n` confidence, point at each declarer.
    /// A federated edge that duplicates a folded local edge is not added twice.
    #[must_use]
    pub fn federated(
        primary: RepoId,
        partitions: Vec<Partition>,
        federated: Vec<FederatedEdge>,
    ) -> Self {
        let folded = folded_stubs(&partitions);
        let total_nodes: usize = partitions.iter().map(|p| p.nodes.len()).sum();
        let total_edges: usize =
            partitions.iter().map(|p| p.edges.len()).sum::<usize>() + federated.len();
        let mut graph = DiGraph::with_capacity(total_nodes, total_edges);
        let mut nodes: Vec<Node> = Vec::with_capacity(total_nodes);
        let mut by_key: HashMap<(RepoId, NodeId), NodeIndex> = HashMap::with_capacity(total_nodes);
        let mut by_id: HashMap<NodeId, NodeIndex> = HashMap::with_capacity(total_nodes);
        let mut by_name: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        // Primary first, so a bare identifier resolves there; nodes move, never clone.
        let (first, rest): (Vec<Partition>, Vec<Partition>) =
            partitions.into_iter().partition(|p| p.repo == primary);
        let mut pending_edges: Vec<(RepoId, Vec<Edge>)> = Vec::new();
        for p in first.into_iter().chain(rest) {
            for node in p.nodes {
                if folded.contains_key(&(p.repo.clone(), node.id.clone())) {
                    continue;
                }
                let ix = graph.add_node(nodes.len());
                by_key.insert((p.repo.clone(), node.id.clone()), ix);
                by_id.entry(node.id.clone()).or_insert(ix);
                by_name
                    .entry(node.name.to_lowercase())
                    .or_default()
                    .push(ix);
                nodes.push(node);
            }
            pending_edges.push((p.repo, p.edges));
        }
        let resolve = |repo: &RepoId, id: &NodeId| -> Option<NodeIndex> {
            by_key
                .get(&(repo.clone(), id.clone()))
                .copied()
                .or_else(|| {
                    folded
                        .get(&(repo.clone(), id.clone()))
                        .and_then(|to| by_key.get(&(to.clone(), id.clone())).copied())
                })
        };
        let mut edges: Vec<Edge> = Vec::with_capacity(total_edges);
        let mut seen: HashSet<(NodeIndex, NodeIndex, String)> = HashSet::new();
        for (repo, list) in pending_edges {
            for edge in list {
                let (Some(s), Some(d)) = (resolve(&repo, &edge.src), resolve(&repo, &edge.dst))
                else {
                    continue;
                };
                seen.insert((s, d, edge.kind.to_string()));
                graph.add_edge(s, d, edges.len());
                edges.push(edge);
            }
        }
        for f in federated {
            let (Some(s), Some(d)) = (
                resolve(&f.src_repo, &f.edge.src),
                by_key
                    .get(&(f.dst_repo.clone(), f.edge.dst.clone()))
                    .copied(),
            ) else {
                continue;
            };
            if !seen.insert((s, d, f.edge.kind.to_string())) {
                continue;
            }
            graph.add_edge(s, d, edges.len());
            edges.push(f.edge);
        }
        Self {
            graph,
            nodes,
            edges,
            by_id,
            by_name,
            by_key,
            primary: Some(primary),
        }
    }

    /// The repository a bare identifier refers to, when the graph spans several.
    #[must_use]
    pub fn primary(&self) -> Option<&RepoId> {
        self.primary.as_ref()
    }

    /// True when `node` belongs to a repository other than the primary one.
    #[must_use]
    pub fn is_foreign(&self, node: &Node) -> bool {
        self.primary.as_ref().is_some_and(|p| node.repo != *p)
    }

    /// The node with `id` in `repo`, when the graph spans several repositories.
    #[must_use]
    pub fn node_in(&self, repo: &RepoId, id: &NodeId) -> Option<&Node> {
        self.by_key
            .get(&(repo.clone(), id.clone()))
            .map(|ix| &self.nodes[self.graph[*ix]])
    }

    /// Repositories with at least one node in the graph, primary first.
    #[must_use]
    pub fn repos(&self) -> Vec<&RepoId> {
        let mut seen: Vec<&RepoId> = Vec::new();
        if let Some(p) = &self.primary {
            seen.push(p);
        }
        for n in &self.nodes {
            if !seen.contains(&&n.repo) {
                seen.push(&n.repo);
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use surveyor_model::{
        ByteSpan, CommitId, Confidence, ContentHash, DeclaredSource, EdgeKind, Evidence, LineSpan,
        Provenance, RepoPath, SourceLocation, TextSpan,
    };

    use super::*;
    use crate::{Direction, EdgeFilter};

    fn pkg(repo: &str, coordinate: &str, external: bool) -> Node {
        let n = Node::synthetic(
            NodeId::new(format!("pkg:{coordinate}")),
            NodeKind::Package,
            coordinate.rsplit('/').next().unwrap(),
            RepoId::new(repo),
        )
        .with_qualified_name(coordinate);
        if external {
            n.with_attribute("external", "true")
        } else {
            n
        }
    }

    fn readme(repo: &str) -> Node {
        Node::synthetic(
            NodeId::new("file:README.md"),
            NodeKind::File,
            "README.md",
            RepoId::new(repo),
        )
        .in_file(RepoPath::new("README.md"))
    }

    fn depends(repo: &str, from: &str, to: &str, line: u32, confidence: f32) -> Edge {
        let span = TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(line, line));
        let evidence = Evidence {
            repo: RepoId::new(repo),
            location: SourceLocation::new(RepoPath::new("package.json"), span),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(b"x"),
        };
        Edge::new(
            NodeId::new(from),
            EdgeKind::DependsOn,
            NodeId::new(to),
            Provenance::declared(DeclaredSource::Manifest, evidence)
                .with_confidence(Confidence::new(confidence)),
        )
    }

    fn documented(repo: &str, from: &str) -> Edge {
        let span = TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1));
        let evidence = Evidence {
            repo: RepoId::new(repo),
            location: SourceLocation::new(RepoPath::new("README.md"), span),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(b"y"),
        };
        Edge::new(
            NodeId::new(from),
            EdgeKind::DocumentedBy,
            NodeId::new("file:README.md"),
            Provenance::declared(DeclaredSource::Citation, evidence),
        )
    }

    fn cairn() -> Partition {
        Partition {
            repo: RepoId::new("cairn"),
            nodes: vec![
                pkg("cairn", "npm/cairn-vscode", false),
                pkg("cairn", "npm/@torrentfx/core", true),
                readme("cairn"),
            ],
            edges: vec![
                depends(
                    "cairn",
                    "pkg:npm/cairn-vscode",
                    "pkg:npm/@torrentfx/core",
                    99,
                    1.0,
                ),
                documented("cairn", "pkg:npm/cairn-vscode"),
            ],
        }
    }

    fn causeway() -> Partition {
        Partition {
            repo: RepoId::new("causeway"),
            nodes: vec![
                pkg("causeway", "npm/@torrentfx/core", false),
                readme("causeway"),
            ],
            edges: vec![documented("causeway", "pkg:npm/@torrentfx/core")],
        }
    }

    fn fed(dst_repo: &str, confidence: f32) -> FederatedEdge {
        let mut edge = depends(
            "cairn",
            "pkg:npm/cairn-vscode",
            "pkg:npm/@torrentfx/core",
            99,
            confidence,
        );
        edge.id = surveyor_model::EdgeId::from(format!("{}@{dst_repo}", edge.id.as_str()));
        FederatedEdge {
            src_repo: RepoId::new("cairn"),
            dst_repo: RepoId::new(dst_repo),
            edge,
        }
    }

    #[test]
    fn one_declarer_folds_the_stub_and_repoints_its_edges() {
        let g = CodeGraph::federated(
            RepoId::new("cairn"),
            vec![cairn(), causeway()],
            vec![fed("causeway", 1.0)],
        );
        // Three packages became two: the stub is gone, the declared node remains.
        let core = g
            .node(&NodeId::new("pkg:npm/@torrentfx/core"))
            .expect("declared node");
        assert_eq!(core.repo.as_str(), "causeway");
        assert!(g.is_foreign(core));
        assert_eq!(
            g.nodes()
                .iter()
                .filter(|n| n.kind == NodeKind::Package)
                .count(),
            2
        );
        // Cairn's own dependency edge now reaches the declared node, once.
        let inc = g.neighbors(&core.id, Direction::Incoming, &EdgeFilter::default());
        let deps: Vec<_> = inc
            .iter()
            .filter(|n| n.edge.kind == EdgeKind::DependsOn)
            .collect();
        assert_eq!(deps.len(), 1, "{inc:?}");
        assert_eq!(deps[0].node.id.as_str(), "pkg:npm/cairn-vscode");
        assert_eq!(
            deps[0].edge.provenance.evidence[0].citation(),
            "package.json:99"
        );
        // Two READMEs coexist, keyed by repository; the bare id is the primary's.
        assert_eq!(
            g.node(&NodeId::new("file:README.md"))
                .unwrap()
                .repo
                .as_str(),
            "cairn"
        );
        assert_eq!(
            g.node_in(&RepoId::new("causeway"), &NodeId::new("file:README.md"))
                .unwrap()
                .repo
                .as_str(),
            "causeway"
        );
        // Each partition's DOCUMENTED_BY edge reached its own README: the primary's
        // README has the one from cairn-vscode, and both edges are in the graph.
        let docs = g.neighbors(
            &NodeId::new("file:README.md"),
            Direction::Incoming,
            &EdgeFilter::default(),
        );
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].node.id.as_str(), "pkg:npm/cairn-vscode");
        assert_eq!(
            g.edges()
                .iter()
                .filter(|e| e.kind == EdgeKind::DocumentedBy)
                .count(),
            2
        );
        assert_eq!(
            g.repos().iter().map(|r| r.as_str()).collect::<Vec<_>>(),
            ["cairn", "causeway"]
        );
    }

    #[test]
    fn two_declarers_keep_the_stub_as_the_hub() {
        let torrentfx = Partition {
            repo: RepoId::new("torrentfx"),
            nodes: vec![pkg("torrentfx", "npm/@torrentfx/core", false)],
            edges: vec![],
        };
        let g = CodeGraph::federated(
            RepoId::new("cairn"),
            vec![cairn(), causeway(), torrentfx],
            vec![fed("causeway", 0.5), fed("torrentfx", 0.5)],
        );
        let stub = g
            .node(&NodeId::new("pkg:npm/@torrentfx/core"))
            .expect("stub stays");
        assert_eq!(stub.repo.as_str(), "cairn");
        assert!(!g.is_foreign(stub));
        let out = g.neighbors(
            &NodeId::new("pkg:npm/cairn-vscode"),
            Direction::Outgoing,
            &EdgeFilter::default(),
        );
        let mut reached: Vec<(String, f32)> = out
            .iter()
            .filter(|n| n.edge.kind == EdgeKind::DependsOn)
            .map(|n| (n.node.repo.to_string(), n.edge.provenance.confidence.get()))
            .collect();
        reached.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            reached,
            vec![
                ("cairn".to_owned(), 1.0),
                ("causeway".to_owned(), 0.5),
                ("torrentfx".to_owned(), 0.5)
            ]
        );
    }

    #[test]
    fn the_primary_alone_with_a_foreign_endpoint_shows_the_consumer() {
        // Causeway loaded whole, plus only the foreign consumer node and the federated edge.
        let consumer = Partition {
            repo: RepoId::new("cairn"),
            nodes: vec![pkg("cairn", "npm/cairn-vscode", false)],
            edges: vec![],
        };
        let g = CodeGraph::federated(
            RepoId::new("causeway"),
            vec![causeway(), consumer],
            vec![fed("causeway", 1.0)],
        );
        let core = g.node(&NodeId::new("pkg:npm/@torrentfx/core")).unwrap();
        assert!(!g.is_foreign(core));
        let inc = g.neighbors(&core.id, Direction::Incoming, &EdgeFilter::default());
        let consumer = inc
            .iter()
            .find(|n| n.edge.kind == EdgeKind::DependsOn)
            .expect("federated consumer");
        assert_eq!(consumer.node.repo.as_str(), "cairn");
        assert!(g.is_foreign(consumer.node));
    }
}
