//! The MCP server: Surveyor's graph as tools for coding and operations agents.
//!
//! Tool names in the first group match what graphicle and Graphify publish
//! (`query_graph`, `get_node`, `get_neighbors`, `shortest_path`, `god_nodes`,
//! `get_community`, `read_project_source`), so an adopter already using them switches
//! servers without rewriting agent instructions. The second group (`affected`,
//! `graph_stats`, `find_nodes`) exposes what those tools lack: reverse impact with
//! provenance filters, and counts.
//!
//! Every answer that names an edge cites its evidence as `file:lines`, and
//! `read_project_source` returns the exact cited window, so an agent can verify
//! instead of trust.

// Rust guideline compliant 2026-02-21

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surveyor_graph::{CodeGraph, Direction, EdgeFilter, PathTier};
use surveyor_model::{Edge, EdgeKind, Node, NodeId, NodeKind, RepoId};
use surveyor_store::LanceStore;

/// Shared state behind the tool handlers.
#[derive(Debug)]
pub struct Surveyor {
    store: LanceStore,
    graph: Arc<CodeGraph>,
    repo: RepoId,
    root: PathBuf,
    #[expect(dead_code, reason = "the tool_handler macro reads the router field")]
    tool_router: ToolRouter<Self>,
}

/// Largest source window one call returns, in lines.
///
/// Enough for any function body an agent would cite; larger reads should be split so
/// the agent states what it is looking for.
const MAX_SOURCE_LINES: u32 = 400;

#[derive(Debug, Serialize)]
struct NodeView {
    id: String,
    kind: String,
    name: String,
    qualified_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
}

impl NodeView {
    fn of(n: &Node) -> Self {
        Self {
            id: n.id.to_string(),
            kind: n.kind.to_string(),
            name: n.name.clone(),
            qualified_name: n.qualified_name.clone(),
            file: n.file().map(ToString::to_string),
            lines: n.lines().map(|l| l.to_string()),
            signature: n.signature.clone(),
            doc: n.doc.as_deref().map(first_paragraph),
        }
    }
}

#[derive(Debug, Serialize)]
struct EdgeView {
    kind: String,
    src: String,
    dst: String,
    derivation: String,
    confidence: f32,
    evidence: Vec<String>,
}

impl EdgeView {
    fn of(e: &Edge) -> Self {
        Self {
            kind: e.kind.to_string(),
            src: e.src.to_string(),
            dst: e.dst.to_string(),
            derivation: e.provenance.derivation.kind().to_string(),
            confidence: e.provenance.confidence.get(),
            evidence: e
                .provenance
                .evidence
                .iter()
                .map(surveyor_model::Evidence::citation)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Relation {
    kind: String,
    direction: Direction,
    node: NodeView,
    evidence: Vec<String>,
    derivation: String,
    confidence: f32,
}

fn first_paragraph(doc: &str) -> String {
    doc.lines()
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn json_result<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

fn parse_kinds(kinds: &[String]) -> Result<Vec<EdgeKind>, McpError> {
    kinds
        .iter()
        .map(|k| {
            k.parse::<EdgeKind>()
                .map_err(|e| McpError::invalid_params(e.to_string(), None))
        })
        .collect()
}

// ---- parameter types --------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryGraphParams {
    /// A question or a symbol name; names are matched exactly, then by suffix, then as substrings.
    pub query: String,
    /// Maximum seed nodes to expand (default 5).
    #[serde(default)]
    pub limit: Option<usize>,
    /// Hops to expand around each seed (default 1).
    #[serde(default)]
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NodeParams {
    /// A node identifier such as `rust:crates/a/src/lib.rs::Relay::drain`, or a name.
    pub node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NeighborsParams {
    /// A node identifier or name.
    pub node_id: String,
    /// `outgoing`, `incoming` or `both` (default).
    #[serde(default)]
    pub direction: Option<String>,
    /// Edge kinds to include, such as `CALLS`; empty means all.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Only edges with deterministic provenance (syntactic, resolved, declared).
    #[serde(default)]
    pub deterministic_only: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    /// Start node identifier or name.
    pub from: String,
    /// End node identifier or name.
    pub to: String,
    /// Follow edges in their direction only (default false).
    #[serde(default)]
    pub directed: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopParams {
    /// Maximum results (default 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommunityParams {
    /// A node whose community to return; omit for the largest communities.
    #[serde(default)]
    pub node_id: Option<String>,
    /// Maximum communities when no node is given (default 10).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceParams {
    /// Repository-relative file path, or a node identifier.
    pub path: String,
    /// First line, one-based (default: the node's span or 1).
    #[serde(default)]
    pub start_line: Option<u32>,
    /// Last line, inclusive (default: the node's span or start + 60).
    #[serde(default)]
    pub end_line: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AffectedParams {
    /// The symbol, table, file or package that changes.
    pub node_id: String,
    /// Hops to follow backwards (default 3).
    #[serde(default)]
    pub depth: Option<usize>,
    /// Edge kinds to follow; default is the impact set (CALLS, READS, WRITES, IMPLEMENTS, ...).
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Include inferred and observed edges (default false).
    #[serde(default)]
    pub all_derivations: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindParams {
    /// Name, qualified name or identifier fragment.
    pub query: String,
    /// Restrict to a node kind such as `Function`, `Table`, `File`.
    #[serde(default)]
    pub kind: Option<String>,
    /// Maximum results (default 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

// ---- tools -----------------------------------------------------------------------------

#[tool_router]
impl Surveyor {
    /// Creates the server state over a loaded graph.
    #[must_use]
    pub fn new(store: LanceStore, graph: CodeGraph, repo: RepoId, root: PathBuf) -> Self {
        Self {
            store,
            graph: Arc::new(graph),
            repo,
            root,
            tool_router: Self::tool_router(),
        }
    }

    fn resolve(&self, query: &str) -> Result<&Node, McpError> {
        self.graph
            .find(query, None, 1)
            .first()
            .copied()
            .ok_or_else(|| McpError::invalid_params(format!("no node matches {query:?}"), None))
    }

    fn relations(
        &self,
        node: &Node,
        direction: Option<Direction>,
        filter: &EdgeFilter,
    ) -> Vec<Relation> {
        let mut out = Vec::new();
        let dirs: Vec<Direction> = match direction {
            Some(d) => vec![d],
            None => vec![Direction::Outgoing, Direction::Incoming],
        };
        for d in dirs {
            for n in self.graph.neighbors(&node.id, d, filter) {
                out.push(Relation {
                    kind: n.edge.kind.to_string(),
                    direction: d,
                    node: NodeView::of(n.node),
                    evidence: n
                        .edge
                        .provenance
                        .evidence
                        .iter()
                        .map(surveyor_model::Evidence::citation)
                        .collect(),
                    derivation: n.edge.provenance.derivation.kind().to_string(),
                    confidence: n.edge.provenance.confidence.get(),
                });
            }
        }
        out
    }

    #[tool(
        description = "Scoped context for a question: the best-matching nodes and their relations, with citations. Use for 'where is X handled', 'what touches table Y'."
    )]
    async fn query_graph(
        &self,
        Parameters(p): Parameters<QueryGraphParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = p.limit.unwrap_or(5).clamp(1, 50);
        let depth = p.depth.unwrap_or(1).clamp(0, 3);
        let seeds = self.graph.find(&p.query, None, limit);
        #[derive(Serialize)]
        struct Seed {
            node: NodeView,
            relations: Vec<Relation>,
        }
        #[derive(Serialize)]
        struct View {
            repo: String,
            query: String,
            seeds: Vec<Seed>,
        }
        let filter = EdgeFilter {
            include_structural: false,
            ..EdgeFilter::default()
        };
        let seeds = seeds
            .into_iter()
            .map(|n| Seed {
                node: NodeView::of(n),
                relations: if depth == 0 {
                    Vec::new()
                } else {
                    self.relations(n, None, &filter)
                },
            })
            .collect();
        json_result(&View {
            repo: self.repo.to_string(),
            query: p.query,
            seeds,
        })
    }

    #[tool(description = "One node: kind, location, signature, documentation and attributes.")]
    async fn get_node(
        &self,
        Parameters(p): Parameters<NodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.resolve(&p.node_id)?;
        #[derive(Serialize)]
        struct View<'a> {
            node: NodeView,
            attributes: &'a std::collections::BTreeMap<String, String>,
            content_hash: String,
        }
        json_result(&View {
            node: NodeView::of(node),
            attributes: &node.attributes,
            content_hash: node.content_hash.to_hex(),
        })
    }

    #[tool(
        description = "Direct relations of a node: what it calls, reads, implements, and what calls, tests or deploys it. Each relation cites file:lines."
    )]
    async fn get_neighbors(
        &self,
        Parameters(p): Parameters<NeighborsParams>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.resolve(&p.node_id)?;
        let direction = match p.direction.as_deref() {
            Some("outgoing") => Some(Direction::Outgoing),
            Some("incoming") => Some(Direction::Incoming),
            None | Some("both") => None,
            Some(other) => {
                return Err(McpError::invalid_params(
                    format!("direction {other:?}"),
                    None,
                ));
            }
        };
        let mut filter = EdgeFilter {
            deterministic_only: p.deterministic_only,
            ..EdgeFilter::default()
        };
        if !p.kinds.is_empty() {
            filter = filter.with_kinds(parse_kinds(&p.kinds)?);
        }
        #[derive(Serialize)]
        struct View {
            node: NodeView,
            relations: Vec<Relation>,
        }
        json_result(&View {
            node: NodeView::of(node),
            relations: self.relations(node, direction, &filter),
        })
    }

    #[tool(
        description = "Shortest dependency path between two nodes, with the edge and citation for each hop and a tier: precise (every hop certain), loose (some hops resolved by name) or any (through containment or tests)."
    )]
    async fn shortest_path(
        &self,
        Parameters(p): Parameters<PathParams>,
    ) -> Result<CallToolResult, McpError> {
        let from = self.resolve(&p.from)?;
        let to = self.resolve(&p.to)?;
        let path = self.graph.best_path(&from.id, &to.id, p.directed);
        #[derive(Serialize)]
        struct Step {
            node: NodeView,
            #[serde(skip_serializing_if = "Option::is_none")]
            edge: Option<EdgeView>,
        }
        #[derive(Serialize)]
        struct View {
            found: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            tier: Option<PathTier>,
            steps: Vec<Step>,
        }
        json_result(&View {
            found: path.is_some(),
            tier: path.as_ref().map(|(tier, _)| *tier),
            steps: path
                .as_ref()
                .map(|(_, steps)| steps.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|s| Step {
                    node: NodeView::of(s.node),
                    edge: s.edge.map(EdgeView::of),
                })
                .collect(),
        })
    }

    #[tool(
        description = "The hubs a change ripples through: nodes ranked by how many distinct files depend on them, with in and out degree and PageRank for reference."
    )]
    async fn god_nodes(
        &self,
        Parameters(p): Parameters<TopParams>,
    ) -> Result<CallToolResult, McpError> {
        let filter = EdgeFilter {
            include_structural: false,
            ..EdgeFilter::default()
        };
        let hubs = self
            .graph
            .hubs(p.limit.unwrap_or(20).clamp(1, 200), &filter);
        #[derive(Serialize)]
        struct View {
            dependent_files: usize,
            in_degree: usize,
            out_degree: usize,
            rank: f64,
            node: NodeView,
        }
        json_result(
            &hubs
                .iter()
                .map(|h| View {
                    dependent_files: h.dependent_files,
                    in_degree: h.in_degree,
                    out_degree: h.out_degree,
                    rank: h.rank,
                    node: NodeView::of(h.node),
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tool(
        description = "Clusters of nodes that reference each other more than the rest; pass node_id for one node's cluster."
    )]
    async fn get_community(
        &self,
        Parameters(p): Parameters<CommunityParams>,
    ) -> Result<CallToolResult, McpError> {
        let filter = EdgeFilter {
            include_structural: false,
            ..EdgeFilter::default()
        };
        let comms = self.graph.communities(&filter);
        #[derive(Serialize)]
        struct View {
            id: String,
            size: usize,
            members: Vec<NodeView>,
        }
        let render = |c: &surveyor_graph::Community| View {
            id: c.id.clone(),
            size: c.members.len(),
            members: c
                .members
                .iter()
                .filter_map(|m| self.graph.node(m))
                .take(50)
                .map(NodeView::of)
                .collect(),
        };
        match p.node_id {
            Some(id) => {
                let node = self.resolve(&id)?;
                let found = comms
                    .iter()
                    .find(|c| c.members.contains(&node.id))
                    .map(render);
                json_result(&found)
            }
            None => json_result(
                &comms
                    .iter()
                    .take(p.limit.unwrap_or(10).clamp(1, 100))
                    .map(render)
                    .collect::<Vec<_>>(),
            ),
        }
    }

    #[tool(
        description = "Read an exact window of a project file by path and lines, or by node id. Use it to verify a citation."
    )]
    async fn read_project_source(
        &self,
        Parameters(p): Parameters<SourceParams>,
    ) -> Result<CallToolResult, McpError> {
        // A node id names its own window; a path takes the requested lines, or the first 60.
        let window = self.graph.node(&NodeId::new(&p.path)).map_or_else(
            || {
                let start = p.start_line.unwrap_or(1);
                Ok((p.path.clone(), start, p.end_line.unwrap_or(start + 60)))
            },
            |n| {
                let loc = n
                    .source()
                    .ok_or_else(|| McpError::invalid_params("node has no source span", None))?;
                Ok((
                    loc.file.to_string(),
                    loc.span.lines.start,
                    loc.span.lines.end,
                ))
            },
        )?;
        let (file, start, end) = (
            window.0,
            p.start_line.unwrap_or(window.1),
            p.end_line.unwrap_or(window.2),
        );
        if file.contains("..") {
            return Err(McpError::invalid_params(
                "path must stay inside the repository",
                None,
            ));
        }
        let text = std::fs::read_to_string(self.root.join(&file))
            .map_err(|e| McpError::invalid_params(format!("read {file}: {e}"), None))?;
        let lines: Vec<&str> = text.lines().collect();
        let start = start.max(1);
        let end = end
            .max(start)
            .min(start + MAX_SOURCE_LINES - 1)
            .min(lines.len() as u32);
        let body: Vec<String> = if start as usize > lines.len() {
            Vec::new()
        } else {
            lines[(start as usize - 1)..(end as usize)]
                .iter()
                .enumerate()
                .map(|(i, l)| format!("{:>5} | {l}", start as usize + i))
                .collect()
        };
        #[derive(Serialize)]
        struct View {
            file: String,
            start_line: u32,
            end_line: u32,
            total_lines: usize,
            text: String,
        }
        json_result(&View {
            file,
            start_line: start,
            end_line: end,
            total_lines: lines.len(),
            text: body.join("\n"),
        })
    }

    #[tool(
        description = "Reverse impact: everything that may break when a symbol, table, file or package changes, to a depth, following only deterministic edges unless asked otherwise."
    )]
    async fn affected(
        &self,
        Parameters(p): Parameters<AffectedParams>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.resolve(&p.node_id)?;
        let mut filter = EdgeFilter::impact();
        if !p.kinds.is_empty() {
            filter = filter.with_kinds(parse_kinds(&p.kinds)?);
        }
        if p.all_derivations {
            filter.deterministic_only = false;
        }
        let depth = p.depth.unwrap_or(3).clamp(1, 8);
        let hits = self.graph.affected(&node.id, depth, &filter);
        #[derive(Serialize)]
        struct Hit {
            depth: usize,
            confidence: f32,
            node: NodeView,
            via: EdgeView,
        }
        #[derive(Serialize)]
        struct View {
            node: NodeView,
            depth: usize,
            count: usize,
            affected: Vec<Hit>,
        }
        json_result(&View {
            node: NodeView::of(node),
            depth,
            count: hits.len(),
            affected: hits
                .iter()
                .take(500)
                .map(|h| Hit {
                    depth: h.depth,
                    confidence: h.confidence,
                    node: NodeView::of(h.node),
                    via: EdgeView::of(h.via),
                })
                .collect(),
        })
    }

    #[tool(
        description = "Find nodes by name or identifier fragment, optionally restricted to a kind."
    )]
    async fn find_nodes(
        &self,
        Parameters(p): Parameters<FindParams>,
    ) -> Result<CallToolResult, McpError> {
        let kind = match &p.kind {
            Some(k) => Some(
                k.parse::<NodeKind>()
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?,
            ),
            None => None,
        };
        let found = self
            .graph
            .find(&p.query, kind.as_ref(), p.limit.unwrap_or(20).clamp(1, 200));
        json_result(&found.iter().map(|n| NodeView::of(n)).collect::<Vec<_>>())
    }

    #[tool(
        description = "Counts of nodes and edges by kind and derivation for the indexed repository, plus the commit it was indexed at."
    )]
    async fn graph_stats(&self) -> Result<CallToolResult, McpError> {
        let snapshot = self
            .store
            .snapshot(&self.repo)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let delta = surveyor_model::GraphDelta {
            nodes: self.graph.nodes().to_vec(),
            edges: self.graph.edges().to_vec(),
            unresolved: Vec::new(),
        };
        #[derive(Serialize)]
        struct View {
            repo: String,
            commit: Option<String>,
            indexed_at_ms: Option<i64>,
            stats: surveyor_model::GraphStats,
        }
        json_result(&View {
            repo: self.repo.to_string(),
            commit: snapshot.as_ref().map(|s| s.commit.to_string()),
            indexed_at_ms: snapshot.as_ref().map(|s| s.indexed_at.as_unix_millis()),
            stats: delta.stats(),
        })
    }
}

#[tool_handler]
impl ServerHandler for Surveyor {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions(format!(
                "Surveyor code graph for repository {}. Start with query_graph or find_nodes to locate symbols, \
                 get_neighbors for relations, affected before changing anything, and read_project_source to verify \
                 any citation before relying on it. Every relation carries derivation and confidence; prefer \
                 deterministic edges for decisions.",
                self.repo
            ))
    }
}

/// Raised when the MCP session cannot start or ends abnormally.
#[derive(Debug)]
pub struct ServeError {
    message: String,
    backtrace: std::backtrace::Backtrace,
}

impl ServeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }

    /// The backtrace captured when the error was created.
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.backtrace
    }
}

impl std::fmt::Display for ServeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mcp session: {}", self.message)
    }
}

impl std::error::Error for ServeError {}

/// Serves the graph over MCP on stdin and stdout until the client disconnects.
///
/// # Errors
/// Returns an error when the transport fails to start or the session ends abnormally.
pub async fn serve_stdio(
    store: LanceStore,
    graph: CodeGraph,
    repo: RepoId,
    root: PathBuf,
) -> Result<(), ServeError> {
    let server = Surveyor::new(store, graph, repo, root);
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| ServeError::new(e.to_string()))?;
    service
        .waiting()
        .await
        .map_err(|e| ServeError::new(e.to_string()))?;
    Ok(())
}
