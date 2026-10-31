//! Command definitions and their handlers.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, bail};
use clap::{Args, Parser, Subcommand};
use surveyor_extract::{Profile, Repository, extract_repository};
use surveyor_graph::{CodeGraph, EdgeFilter};
use surveyor_link::{FileEntry, LinkInput, link};
use surveyor_model::{EdgeKind, NodeKind, RepoId};
use surveyor_store::{FileRecord, LanceStore};

use crate::render::{self, Output};

/// Directory under the repository root that holds the dataset.
const STORE_DIR: &str = ".surveyor";

/// Surveyor: a code intelligence graph with provenance.
#[derive(Debug, Parser)]
#[command(name = "surveyor", version, about)]
pub struct Cli {
    /// Repository root (defaults to the current directory).
    #[arg(long, global = true, env = "SURVEYOR_REPO")]
    pub repo_root: Option<PathBuf>,
    /// Dataset location (defaults to `<repo-root>/.surveyor`).
    #[arg(long, global = true, env = "SURVEYOR_STORE")]
    pub store: Option<PathBuf>,
    /// Repository identifier inside the dataset (defaults to the profile or directory name).
    #[arg(long, global = true, env = "SURVEYOR_REPO_ID")]
    pub repo_id: Option<String>,
    /// Emit JSON instead of text.
    #[arg(long, global = true)]
    pub json: bool,
    /// Log progress to stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Index the repository into the dataset.
    Index,
    /// Show dataset and repository counts.
    Stats,
    /// Describe one symbol: where it is, what it touches, who touches it.
    Explain(Query),
    /// What may break when a symbol, table or file changes.
    Affected(AffectedArgs),
    /// Shortest dependency path between two symbols.
    Path(PathArgs),
    /// Find nodes by name.
    Query(Query),
    /// Most connected nodes by `PageRank`.
    Hubs(TopArgs),
    /// Clusters of nodes that reference each other.
    Communities(TopArgs),
    /// References that found no declaration.
    Unresolved(TopArgs),
    /// Print the source window a node or citation points at.
    Source(SourceArgs),
    /// Serve the graph over MCP on stdio.
    Mcp,
}

#[derive(Debug, Args)]
pub struct Query {
    /// A name, qualified name or node identifier.
    pub query: String,
    /// Restrict to a node kind, such as `Function` or `Table`.
    #[arg(long)]
    pub kind: Option<NodeKind>,
    /// Maximum results.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct AffectedArgs {
    #[command(flatten)]
    pub query: Query,
    /// How many hops to follow backwards.
    #[arg(long, default_value_t = 3)]
    pub depth: usize,
    /// Comma-separated edge kinds to follow (defaults to the impact kinds).
    #[arg(long, value_delimiter = ',')]
    pub kinds: Vec<EdgeKind>,
    /// Include edges a model inferred or a probe observed.
    #[arg(long)]
    pub all_derivations: bool,
}

#[derive(Debug, Args)]
pub struct PathArgs {
    /// Start symbol.
    pub from: String,
    /// End symbol.
    pub to: String,
    /// Follow edges in their direction only.
    #[arg(long)]
    pub directed: bool,
}

#[derive(Debug, Args)]
pub struct TopArgs {
    /// Maximum results.
    #[arg(long, default_value_t = 20)]
    pub top: usize,
}

#[derive(Debug, Args)]
pub struct SourceArgs {
    /// A node identifier, or `path:line` / `path:start-end`.
    pub target: String,
    /// Lines of context around the span.
    #[arg(long, default_value_t = 2)]
    pub context: u32,
}

struct Paths {
    root: PathBuf,
    store: PathBuf,
}

fn paths(cli: &Cli) -> anyhow::Result<Paths> {
    let root = match &cli.repo_root {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let root = root
        .canonicalize()
        .with_context(|| format!("repository root {}", root.display()))?;
    let store = cli.store.clone().unwrap_or_else(|| root.join(STORE_DIR));
    Ok(Paths { root, store })
}

async fn open_graph(cli: &Cli, p: &Paths) -> anyhow::Result<(LanceStore, CodeGraph, RepoId)> {
    if !p.store.exists() {
        bail!(
            "no dataset at {}; run `surveyor index` first",
            p.store.display()
        );
    }
    let store = LanceStore::open(&p.store).await.context("open dataset")?;
    let repo = if let Some(id) = &cli.repo_id {
        RepoId::new(id)
    } else {
        let snapshots = store.snapshots().await?;
        match snapshots.len() {
            0 => bail!("dataset at {} has no indexed repository", p.store.display()),
            1 => snapshots[0].repo.clone(),
            _ => {
                let guess = Repository::open(&p.root, None).map(|r| r.id().clone()).ok();
                match guess.filter(|g| snapshots.iter().any(|s| &s.repo == g)) {
                    Some(g) => g,
                    None => bail!(
                        "dataset holds {} repositories; pass --repo-id (one of: {})",
                        snapshots.len(),
                        snapshots
                            .iter()
                            .map(|s| s.repo.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                }
            }
        }
    };
    let started = Instant::now();
    let (nodes, edges) = store.load_graph(Some(&repo)).await.context("load graph")?;
    let graph = CodeGraph::new(nodes, edges);
    tracing::info!(
        nodes = graph.node_count(),
        edges = graph.edge_count(),
        ms = started.elapsed().as_millis() as u64,
        "graph loaded"
    );
    Ok((store, graph, repo))
}

/// Resolves a user query to exactly one node, or explains the ambiguity.
fn resolve_one<'a>(
    graph: &'a CodeGraph,
    query: &str,
    kind: Option<&NodeKind>,
) -> anyhow::Result<&'a surveyor_model::Node> {
    let found = graph.find(query, kind, 10);
    match found.first() {
        None => bail!("no node matches {query:?}"),
        Some(first) => {
            if found.len() > 1
                && found[1].name.eq_ignore_ascii_case(&first.name)
                && found[1].kind == first.kind
                && first.id.as_str() != query
            {
                tracing::warn!(
                    chosen = %first.id,
                    alternatives = %found.iter().skip(1).take(5).map(|n| n.id.to_string()).collect::<Vec<_>>().join(", "),
                    "query is ambiguous; using the closest match"
                );
            }
            Ok(first)
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one match arm per subcommand")]
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let p = paths(&cli)?;
    let out = if cli.json { Output::Json } else { Output::Text };
    match &cli.command {
        Command::Index => index(&cli, &p, &out).await,
        Command::Stats => {
            let store = LanceStore::open(&p.store).await.context("open dataset")?;
            let stats = store.stats().await?;
            let per_repo = match cli.repo_id.as_deref().map(RepoId::new) {
                Some(repo) => Some(store.graph_stats(&repo).await?),
                None if stats.repositories.len() == 1 => {
                    Some(store.graph_stats(&stats.repositories[0].repo).await?)
                }
                None => None,
            };
            out.stats(&p.store, &stats, per_repo.as_ref());
            Ok(())
        }
        Command::Explain(q) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let node = resolve_one(&graph, &q.query, q.kind.as_ref())?;
            out.explain(&graph, node, q.limit);
            Ok(())
        }
        Command::Affected(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let node = resolve_one(&graph, &a.query.query, a.query.kind.as_ref())?;
            let mut filter = EdgeFilter::impact();
            if !a.kinds.is_empty() {
                filter = filter.with_kinds(a.kinds.clone());
            }
            if a.all_derivations {
                filter.deterministic_only = false;
            }
            let hits = graph.affected(&node.id, a.depth, &filter);
            out.affected(node, &hits, a.depth);
            Ok(())
        }
        Command::Path(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let from = resolve_one(&graph, &a.from, None)?;
            let to = resolve_one(&graph, &a.to, None)?;
            let filter = EdgeFilter {
                include_structural: false,
                ..EdgeFilter::default()
            };
            let path = graph
                .shortest_path(&from.id, &to.id, &filter, a.directed)
                .or_else(|| {
                    graph.shortest_path(&from.id, &to.id, &EdgeFilter::default(), a.directed)
                });
            out.path(from, to, path.as_deref());
            Ok(())
        }
        Command::Query(q) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let found = graph.find(&q.query, q.kind.as_ref(), q.limit);
            out.nodes(&found);
            Ok(())
        }
        Command::Hubs(t) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let filter = EdgeFilter {
                include_structural: false,
                ..EdgeFilter::default()
            };
            let ranked = graph.page_rank(t.top, &filter);
            out.hubs(&graph, &ranked);
            Ok(())
        }
        Command::Communities(t) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let filter = EdgeFilter {
                include_structural: false,
                ..EdgeFilter::default()
            };
            let comms = graph.communities(&filter);
            out.communities(&graph, &comms, t.top);
            Ok(())
        }
        Command::Unresolved(t) => {
            let (store, _, repo) = open_graph(&cli, &p).await?;
            let items = store.unresolved(&repo).await?;
            out.unresolved(&items, t.top);
            Ok(())
        }
        Command::Source(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let window = render::source_window(&graph, &p.root, &a.target, a.context)?;
            out.source(&window);
            Ok(())
        }
        Command::Mcp => {
            let (store, graph, repo) = open_graph(&cli, &p).await?;
            surveyor_mcp::serve_stdio(store, graph, repo, p.root.clone())
                .await
                .context("mcp server")?;
            Ok(())
        }
    }
}

async fn index(cli: &Cli, p: &Paths, out: &Output) -> anyhow::Result<()> {
    let started = Instant::now();
    let repo = Repository::open(&p.root, cli.repo_id.as_deref().map(RepoId::new))
        .context("open repository")?;
    let profile = Profile::load(&p.root).context("load surveyor.toml")?;
    tracing::info!(repo = %repo.id(), commit = %repo.commit(), root = %p.root.display(), "indexing");

    let extraction = extract_repository(&repo, &profile).context("extract")?;
    let extracted_at = started.elapsed();
    tracing::info!(
        files = extraction.units.len(),
        facts = extraction.fact_count(),
        errors = extraction.errors.len(),
        ms = extracted_at.as_millis() as u64,
        "extracted"
    );
    for err in extraction.errors.iter().take(20) {
        tracing::warn!(error = %err, "extraction error");
    }

    let files: Vec<FileEntry> = extraction
        .units
        .iter()
        .map(|u| FileEntry {
            path: u.path.clone(),
            language: u.language.clone(),
            content_hash: u.content_hash,
            bytes: u.bytes,
        })
        .collect();
    let records: Vec<FileRecord> = files
        .iter()
        .map(|f| FileRecord {
            path: f.path.clone(),
            language: f.language.clone(),
            content_hash: f.content_hash,
            bytes: f.bytes,
        })
        .collect();
    let input = LinkInput::new(repo.id().clone(), repo.commit().clone())
        .with_files(files)
        .with_fact_sets(extraction.fact_sets);
    let delta = link(&input);
    let linked_at = started.elapsed();
    let stats = delta.stats();
    tracing::info!(
        nodes = stats.nodes,
        edges = stats.edges,
        unresolved = stats.unresolved,
        ms = linked_at.checked_sub(extracted_at).unwrap().as_millis() as u64,
        "linked"
    );

    let store = LanceStore::open(&p.store).await.context("open dataset")?;
    let snapshot = store
        .replace_repository(repo.id(), repo.commit(), &delta, &records)
        .await
        .context("write dataset")?;
    let total = started.elapsed();
    tracing::info!(ms = total.checked_sub(linked_at).unwrap().as_millis() as u64, store = %p.store.display(), "written");
    out.indexed(&snapshot, &stats, &p.store, extraction.errors.len(), total);
    Ok(())
}
