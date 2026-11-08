//! From changed lines to changed nodes, and from changed nodes to what they affect.
//!
//! `surveyor pr` reads a [`ChangeSet`] (line ranges per file) and asks two pure
//! questions of the graph: which declarations those lines fall in, and which nodes
//! the reverse impact walk reaches from them. Both are functions of the graph and
//! the change alone, so a test can run them on a fixture and a comment job on a
//! dataset without either knowing about git.

// Rust guideline compliant 2026-02-21

use std::collections::{BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use surveyor_model::{ChangeSet, Edge, LineSpan, Node, NodeId, NodeKind, RepoPath};

use crate::{CodeGraph, EdgeFilter, Reach};

/// One changed file with the nodes its changed lines fall in.
#[derive(Debug, Clone, PartialEq)]
pub struct ChangedFile<'a> {
    /// Repository-relative path.
    pub path: RepoPath,
    /// True when the file is new.
    pub added: bool,
    /// The changed ranges, as the change set gave them.
    pub hunks: Vec<LineSpan>,
    /// Innermost declarations a changed line falls in, by position; empty when the
    /// file holds none at those lines.
    pub symbols: Vec<&'a Node>,
    /// The file's own node, when the graph indexed the file.
    pub file: Option<&'a Node>,
}

impl<'a> ChangedFile<'a> {
    /// The nodes this file contributes to the changed set: its symbols, else its file
    /// node when the graph has one, else nothing.
    pub fn nodes(&self) -> impl Iterator<Item = &'a Node> + '_ {
        let fallback = self.symbols.is_empty().then_some(self.file).flatten();
        self.symbols.iter().copied().chain(fallback)
    }
}

/// The nodes a change touched, file by file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Changed<'a> {
    /// Files on the new side, by path.
    pub files: Vec<ChangedFile<'a>>,
    /// Files the change removed, by path.
    pub deleted: Vec<RepoPath>,
}

impl<'a> Changed<'a> {
    /// Every changed node once, symbols first then file nodes, each group by identifier.
    #[must_use]
    pub fn nodes(&self) -> Vec<&'a Node> {
        let mut seen: HashSet<&'a NodeId> = HashSet::new();
        let mut symbols: Vec<&'a Node> = Vec::new();
        let mut files: Vec<&'a Node> = Vec::new();
        for f in &self.files {
            for n in f.nodes() {
                if seen.insert(&n.id) {
                    if n.kind == NodeKind::File {
                        files.push(n);
                    } else {
                        symbols.push(n);
                    }
                }
            }
        }
        symbols.sort_by(|x, y| x.id.cmp(&y.id));
        files.sort_by(|x, y| x.id.cmp(&y.id));
        symbols.into_iter().chain(files).collect()
    }

    /// Changed files the graph holds nothing for: not indexed, so not mapped.
    pub fn unindexed(&self) -> impl Iterator<Item = &RepoPath> + '_ {
        self.files
            .iter()
            .filter(|f| f.symbols.is_empty() && f.file.is_none())
            .map(|f| &f.path)
    }

    /// Number of changed symbols (nodes narrower than a file).
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.nodes()
            .iter()
            .filter(|n| n.kind != NodeKind::File)
            .count()
    }
}

/// Maps every changed range onto the innermost declaration containing each of its
/// lines, the containment rule the linker's `enclosing` applies to references.
///
/// A range that touches an `impl` header and the first method inside it names both;
/// a range inside one method names the method alone; a range in a file with no
/// declaration at those lines (a manifest, a workflow, a design) names the file node.
/// Nodes without a span (packages, tables from a manifest) are never changed lines.
#[must_use]
pub fn changed<'a>(graph: &'a CodeGraph, set: &ChangeSet) -> Changed<'a> {
    let wanted: HashSet<&str> = set.files.iter().map(|f| f.path.as_str()).collect();
    let mut spans: HashMap<&str, Vec<&'a Node>> = HashMap::new();
    let mut file_nodes: HashMap<&str, &'a Node> = HashMap::new();
    for node in graph.nodes() {
        let Some(file) = node.file().map(RepoPath::as_str) else {
            continue;
        };
        if !wanted.contains(file) {
            continue;
        }
        match (&node.kind, node.lines()) {
            (NodeKind::File, _) => {
                file_nodes.insert(file, node);
            }
            (_, Some(_)) => spans.entry(file).or_default().push(node),
            _ => {}
        }
    }
    let files = set
        .files
        .iter()
        .map(|f| {
            let decls = spans.get(f.path.as_str()).map_or(&[][..], Vec::as_slice);
            let symbols = innermost(decls, &f.hunks);
            ChangedFile {
                path: f.path.clone(),
                added: f.added,
                hunks: f.hunks.clone(),
                symbols,
                file: file_nodes.get(f.path.as_str()).copied(),
            }
        })
        .collect();
    Changed {
        files,
        deleted: set.deleted.clone(),
    }
}

/// For each line of each range, the smallest declaration span containing it; the
/// distinct results by position.
fn innermost<'a>(decls: &[&'a Node], hunks: &[LineSpan]) -> Vec<&'a Node> {
    let candidates: Vec<(&'a Node, LineSpan)> = decls
        .iter()
        .filter_map(|n| n.lines().map(|l| (*n, l)))
        .filter(|(_, l)| hunks.iter().any(|h| intersects(*l, *h)))
        .collect();
    let mut hit: BTreeSet<(u32, u32, &str)> = BTreeSet::new();
    let mut by_key: HashMap<(u32, u32, &str), &'a Node> = HashMap::new();
    for h in hunks {
        for line in h.start..=h.end {
            let best = candidates
                .iter()
                .filter(|(_, l)| l.start <= line && line <= l.end)
                .min_by_key(|(n, l)| (l.count(), l.start, n.id.as_str()));
            if let Some((n, l)) = best {
                let key = (l.start, l.end, n.id.as_str());
                hit.insert(key);
                by_key.insert(key, n);
            }
        }
    }
    hit.into_iter()
        .filter_map(|k| by_key.get(&k).copied())
        .collect()
}

fn intersects(a: LineSpan, b: LineSpan) -> bool {
    a.start <= b.end && b.start <= a.end
}

/// How the affected nodes are grouped in a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactGroup {
    /// Functions, methods, types and the other symbols a developer searches for.
    Code,
    /// Schemas, tables, views, columns, migrations and queries.
    Data,
    /// Services, RPCs, messages, operations, topics and their producers and consumers.
    Contracts,
    /// Workloads, images, Dockerfiles, configuration, flags, policies, environments.
    Workloads,
    /// Tasks, scripts, workflows and host modules.
    Tasks,
    /// Designs, runbooks, documents, skills and plans.
    Documents,
    /// Files, modules, packages, workspaces.
    Files,
    /// Everything else: history, inferred and adapter kinds.
    Other,
}

impl ImpactGroup {
    /// Every group, in the order a comment lists them.
    pub const ALL: [Self; 8] = [
        Self::Code,
        Self::Data,
        Self::Contracts,
        Self::Workloads,
        Self::Tasks,
        Self::Documents,
        Self::Files,
        Self::Other,
    ];

    /// The group a node kind falls in.
    #[must_use]
    pub fn of(kind: &NodeKind) -> Self {
        use NodeKind as K;
        match kind {
            k if k.is_symbol() => Self::Code,
            K::Schema
            | K::Table
            | K::View
            | K::Column
            | K::Index
            | K::Constraint
            | K::Migration
            | K::Query => Self::Data,
            K::ProtoFile
            | K::Service
            | K::Rpc
            | K::Message
            | K::OpenApiOperation
            | K::GeneratedBinding
            | K::Topic
            | K::Queue
            | K::Producer
            | K::Consumer => Self::Contracts,
            K::Workload
            | K::Image
            | K::Dockerfile
            | K::ConfigMap
            | K::Secret
            | K::Flag
            | K::Policy
            | K::Namespace
            | K::Environment => Self::Workloads,
            K::Task | K::Script | K::Workflow | K::HostModule => Self::Tasks,
            K::Design | K::Runbook | K::Document | K::Skill | K::Plan => Self::Documents,
            K::Repository | K::Workspace | K::Package | K::Module | K::File => Self::Files,
            _ => Self::Other,
        }
    }

    /// The heading a comment uses.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Code => "Code symbols",
            Self::Data => "Tables, views and columns",
            Self::Contracts => "Contracts and topics",
            Self::Workloads => "Workloads",
            Self::Tasks => "Tasks and workflows",
            Self::Documents => "Documents and designs",
            Self::Files => "Files and packages",
            Self::Other => "Other",
        }
    }

    /// The groups no node of `graph` falls in: kinds the dataset cannot show.
    #[must_use]
    pub fn absent_from(graph: &CodeGraph) -> Vec<Self> {
        let present: HashSet<Self> = graph.nodes().iter().map(|n| Self::of(&n.kind)).collect();
        Self::ALL
            .into_iter()
            .filter(|g| !present.contains(g))
            .collect()
    }
}

/// A node the change may break, reached from one of the changed nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct Impact<'a> {
    /// The node reached.
    pub node: &'a Node,
    /// Hops from the changed node that reached it most strongly.
    pub depth: usize,
    /// Product of confidences along that path.
    pub confidence: f32,
    /// The edge that brought the impact one hop closer to this node.
    pub via: &'a Edge,
    /// The changed node the path started from.
    pub from: &'a Node,
}

impl Impact<'_> {
    /// The group the reached node reports under.
    #[must_use]
    pub fn group(&self) -> ImpactGroup {
        ImpactGroup::of(&self.node.kind)
    }
}

/// The reverse impact of every changed node, merged: a node reached from several
/// keeps its strongest path (highest confidence, then fewest hops), changed nodes
/// are never their own impact, and the rows come strongest first, then by identifier.
#[must_use]
pub fn impact<'a>(
    graph: &'a CodeGraph,
    changed: &[&'a Node],
    reach: Reach,
    filter: &EdgeFilter,
) -> Vec<Impact<'a>> {
    let changed_ids: HashSet<&NodeId> = changed.iter().map(|n| &n.id).collect();
    let mut best: HashMap<&NodeId, Impact<'a>> = HashMap::new();
    for from in changed {
        for hit in graph.affected(&from.id, reach, filter) {
            if changed_ids.contains(&hit.node.id) {
                continue;
            }
            let candidate = Impact {
                node: hit.node,
                depth: hit.depth,
                confidence: hit.confidence,
                via: hit.via,
                from,
            };
            let stronger = best.get(&hit.node.id).is_none_or(|cur| {
                (candidate.confidence, std::cmp::Reverse(candidate.depth))
                    > (cur.confidence, std::cmp::Reverse(cur.depth))
            });
            if stronger {
                best.insert(&hit.node.id, candidate);
            }
        }
    }
    let mut out: Vec<Impact<'a>> = best.into_values().collect();
    out.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then(a.depth.cmp(&b.depth))
            .then(a.node.id.cmp(&b.node.id))
    });
    out
}

#[cfg(test)]
mod tests {
    use surveyor_model::{
        ByteSpan, CommitId, Confidence, EdgeKind, Evidence, FileChange, GrammarId, Provenance,
        RepoId, RuleId, SourceLocation, TextSpan,
    };

    use super::*;

    fn span(start_line: u32, end_line: u32) -> TextSpan {
        TextSpan::new(
            ByteSpan::new(u64::from(start_line) * 100, u64::from(end_line) * 100 + 99),
            LineSpan::new(start_line, end_line),
        )
    }

    fn decl(id: &str, kind: NodeKind, file: &str, lines: (u32, u32)) -> Node {
        Node::synthetic(
            NodeId::new(id),
            kind,
            id.rsplit("::").next().unwrap(),
            RepoId::new("r"),
        )
        .at(RepoPath::new(file), span(lines.0, lines.1))
    }

    fn file_node(path: &str) -> Node {
        Node::synthetic(
            NodeId::new(format!("file:{path}")),
            NodeKind::File,
            path,
            RepoId::new("r"),
        )
        .in_file(RepoPath::new(path))
    }

    fn edge(src: &str, kind: EdgeKind, dst: &str, file: &str, line: u32, c: f32) -> Edge {
        let evidence = Evidence {
            repo: RepoId::new("r"),
            location: SourceLocation::new(RepoPath::new(file), span(line, line)),
            commit: CommitId::new("c"),
            content_hash: surveyor_model::ContentHash::of(b"x"),
        };
        let provenance =
            Provenance::syntactic(GrammarId::new("rust"), RuleId::new("call"), evidence)
                .with_confidence(Confidence::new(c));
        Edge::new(NodeId::new(src), kind, NodeId::new(dst), provenance)
    }

    fn graph() -> CodeGraph {
        let relay = "crates/outbox/src/relay.rs";
        let api = "crates/api/src/main.rs";
        let nodes = vec![
            file_node(relay),
            file_node(api),
            file_node("Cargo.toml"),
            decl("rust:relay::Relay", NodeKind::Struct, relay, (10, 12)),
            decl("rust:relay::Relay#impl", NodeKind::Impl, relay, (14, 30)),
            decl("rust:relay::Relay::new", NodeKind::Method, relay, (16, 18)),
            decl(
                "rust:relay::Relay::drain",
                NodeKind::Method,
                relay,
                (20, 28),
            ),
            decl("rust:api::main", NodeKind::Function, api, (3, 9)),
            decl("rust:api::report", NodeKind::Function, api, (11, 13)),
        ];
        let edges = vec![
            edge(
                "rust:api::main",
                EdgeKind::Calls,
                "rust:relay::Relay::drain",
                api,
                5,
                1.0,
            ),
            edge(
                "rust:api::report",
                EdgeKind::Calls,
                "rust:api::main",
                api,
                12,
                0.5,
            ),
            edge(
                "rust:api::report",
                EdgeKind::References,
                "rust:relay::Relay::drain",
                api,
                13,
                0.5,
            ),
        ];
        CodeGraph::new(nodes, edges)
    }

    fn set(path: &str, hunks: &[(u32, u32)]) -> ChangeSet {
        ChangeSet {
            files: vec![FileChange {
                path: RepoPath::new(path),
                added: false,
                hunks: hunks.iter().map(|(s, e)| LineSpan::new(*s, *e)).collect(),
            }],
            deleted: vec![],
        }
    }

    fn ids(nodes: &[&Node]) -> Vec<String> {
        nodes.iter().map(|n| n.id.to_string()).collect()
    }

    #[test]
    fn a_hunk_inside_a_method_names_the_method_alone() {
        let g = graph();
        let c = changed(&g, &set("crates/outbox/src/relay.rs", &[(22, 24)]));
        assert_eq!(ids(&c.nodes()), ["rust:relay::Relay::drain"]);
        assert_eq!(c.symbol_count(), 1);
    }

    #[test]
    fn a_hunk_over_the_impl_header_and_a_method_names_both() {
        let g = graph();
        let c = changed(&g, &set("crates/outbox/src/relay.rs", &[(14, 17)]));
        assert_eq!(
            ids(&c.files[0].symbols),
            ["rust:relay::Relay#impl", "rust:relay::Relay::new"]
        );
    }

    #[test]
    fn lines_outside_every_declaration_fall_back_to_the_file_node() {
        let g = graph();
        let c = changed(&g, &set("crates/outbox/src/relay.rs", &[(1, 2)]));
        assert!(c.files[0].symbols.is_empty());
        assert_eq!(ids(&c.nodes()), ["file:crates/outbox/src/relay.rs"]);
        let m = changed(&g, &set("Cargo.toml", &[(1, 3)]));
        assert_eq!(ids(&m.nodes()), ["file:Cargo.toml"]);
        let u = changed(&g, &set("README.md", &[(1, 1)]));
        assert!(u.nodes().is_empty());
        assert_eq!(
            u.unindexed().map(ToString::to_string).collect::<Vec<_>>(),
            ["README.md"]
        );
    }

    #[test]
    fn impact_merges_paths_and_keeps_the_strongest() {
        let g = graph();
        let drain = g.node(&NodeId::new("rust:relay::Relay::drain")).unwrap();
        let hits = impact(&g, &[drain], Reach::depth(3), &EdgeFilter::impact());
        let rows: Vec<(String, usize, f32)> = hits
            .iter()
            .map(|h| (h.node.id.to_string(), h.depth, h.confidence))
            .collect();
        // `report` is reached at depth 1 by a 0.5 reference and at depth 2 through
        // `main` at 0.5; the shallower path wins the tie.
        assert_eq!(
            rows,
            vec![
                ("rust:api::main".to_owned(), 1, 1.0),
                ("rust:api::report".to_owned(), 1, 0.5),
            ]
        );
        assert_eq!(hits[0].from.id, drain.id);
        assert_eq!(hits[0].group(), ImpactGroup::Code);
        // A changed node is never its own impact.
        let main = g.node(&NodeId::new("rust:api::main")).unwrap();
        let both = impact(&g, &[drain, main], Reach::depth(3), &EdgeFilter::impact());
        assert_eq!(
            both.iter()
                .map(|h| h.node.id.to_string())
                .collect::<Vec<_>>(),
            ["rust:api::report"]
        );
    }

    #[test]
    fn groups_cover_every_kind_and_report_the_absent_ones() {
        assert_eq!(ImpactGroup::of(&NodeKind::Method), ImpactGroup::Code);
        assert_eq!(ImpactGroup::of(&NodeKind::View), ImpactGroup::Data);
        assert_eq!(ImpactGroup::of(&NodeKind::Rpc), ImpactGroup::Contracts);
        assert_eq!(ImpactGroup::of(&NodeKind::Workload), ImpactGroup::Workloads);
        assert_eq!(ImpactGroup::of(&NodeKind::Task), ImpactGroup::Tasks);
        assert_eq!(ImpactGroup::of(&NodeKind::Runbook), ImpactGroup::Documents);
        assert_eq!(ImpactGroup::of(&NodeKind::Package), ImpactGroup::Files);
        assert_eq!(ImpactGroup::of(&NodeKind::Commit), ImpactGroup::Other);
        let absent = ImpactGroup::absent_from(&graph());
        assert!(absent.contains(&ImpactGroup::Contracts));
        assert!(!absent.contains(&ImpactGroup::Code));
        assert!(!absent.contains(&ImpactGroup::Files));
    }
}
