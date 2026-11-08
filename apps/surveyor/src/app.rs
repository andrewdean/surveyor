//! Command definitions and their handlers.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, bail};
use clap::{Args, Parser, Subcommand};
use surveyor_extract::{
    Extraction, Profile, Repository, extract_paths_using, extract_repository_with, extractors_with,
};
use surveyor_graph::{CodeGraph, EdgeFilter, Lookup, Partition, PathMode, Reach};
use surveyor_link::{
    ChangedFiles, FileEntry, LinkInput, RepoPackages, federate, link, plan_update, relink,
};
use surveyor_model::{EdgeKind, Fact, GraphStats, LiteralRole, NodeKind, RepoId, RepoPath};
use surveyor_store::{FileRecord, LanceStore, RepositorySnapshot, StoreUpdate};

use crate::eval;
use crate::harness;
use crate::pr;
use crate::render::{self, Output, Updated};

/// Directory under the repository root that holds the dataset.
const STORE_DIR: &str = ".surveyor";

/// The engine that writes a dataset: crate version, query packs, extractor and linker
/// revisions, the effective profile and the precise indexes it merges.
///
/// A dataset written by a different engine, under a different profile or from
/// different SCIP indexes, is re-indexed in full on `update`, because facts from two
/// engines, two profiles or two indexes cannot be mixed file by file. The profile
/// counts as parsed, so an edit to a comment in `surveyor.toml` changes nothing,
/// while a changed glob, dialect or rule directory forces the full index it needs.
///
/// # Errors
/// Returns an error when the profile cannot be serialized, which no profile that
/// parsed can cause.
fn engine_version(profile: &Profile, inputs: &[PathBuf]) -> anyhow::Result<String> {
    let profile_json = serde_json::to_string(profile).context("serialize profile")?;
    let index_hashes: Vec<String> = inputs
        .iter()
        .map(|path| {
            std::fs::read(path).map_or_else(
                |_| String::from("absent"),
                |bytes| surveyor_model::ContentHash::of(bytes).to_hex(),
            )
        })
        .collect();
    let parts = std::iter::once(surveyor_extract::query_pack_fingerprint().to_hex())
        .chain(std::iter::once(
            surveyor_extract::EXTRACT_REVISION.to_string(),
        ))
        .chain(std::iter::once(surveyor_link::LINK_REVISION.to_string()))
        .chain(std::iter::once(profile_json))
        .chain(index_hashes)
        .collect::<Vec<_>>();
    let fingerprint = surveyor_model::ContentHash::of_parts(parts.iter().map(String::as_str));
    Ok(format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        &fingerprint.short()[..8]
    ))
}

/// Opens the dataset at `path` for writing, stamping new snapshots with this build's
/// engine, the `profile` in force and the precise `inputs` it merges.
async fn open_store(
    path: &Path,
    profile: &Profile,
    inputs: &[PathBuf],
) -> anyhow::Result<LanceStore> {
    Ok(open_dataset(path)
        .await?
        .with_engine_version(engine_version(profile, inputs)?))
}

/// Opens the dataset at `path` for reading; a reader writes no snapshot, so it needs
/// no engine stamp.
async fn open_dataset(path: &Path) -> anyhow::Result<LanceStore> {
    LanceStore::open(path).await.context("open dataset")
}

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
    /// Load every repository in the dataset, not only the one asked about, so a walk
    /// crosses the federated edges into the others.
    #[arg(long, global = true)]
    pub all_repos: bool,
    /// Profile file to use instead of `<repo-root>/surveyor.toml`, for a repository
    /// that cannot carry one.
    #[arg(long, global = true, env = "SURVEYOR_PROFILE")]
    pub profile: Option<PathBuf>,
    /// Session ledger to record every answer in (`<store>/sessions/<id>.jsonl`).
    #[arg(long, global = true, env = "SURVEYOR_SESSION")]
    pub session: Option<String>,
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
    Index(IndexArgs),
    /// Reindex only what changed since the dataset's recorded commit.
    Update(UpdateArgs),
    /// Show dataset and repository counts.
    Stats,
    /// Describe one symbol: where it is, what it touches, who touches it.
    Explain(Query),
    /// One screen per symbol: identity, signature, doc, counts with citations, rules.
    Card(CardArgs),
    /// The context an agent needs for a task, inside a token budget, with evidence.
    Context(ContextArgs),
    /// What may break when a symbol, table or file changes.
    Affected(AffectedArgs),
    /// Shortest dependency path between two symbols.
    Path(PathArgs),
    /// Find nodes by name.
    Query(Query),
    /// Nodes the most files depend on.
    Hubs(TopArgs),
    /// List nodes by kind, location and the edges they have or lack.
    Nodes(NodesArgs),
    /// Clusters of nodes that reference each other.
    Communities(TopArgs),
    /// References that found no declaration.
    Unresolved(TopArgs),
    /// Print the source window a node or citation points at.
    Source(SourceArgs),
    /// Run the repository's architectural rules against the graph.
    Check(CheckArgs),
    /// What a change touches, what it may break, whether the invariants hold: a
    /// pull-request comment and a gate (exit 1 on a failed rule).
    Pr(pr::PrArgs),
    /// Recompute the cross-repository package edges for every repository in the dataset.
    Federate,
    /// Serve the graph over MCP on stdio.
    Mcp,
    /// Open, annotate or summarize a session ledger.
    Session(harness::SessionArgs),
    /// Act on an agent host's hook payload (stdin).
    Hook(harness::HookArgs),
    /// Score plain-English questions against the starting nodes they should find.
    Eval(EvalArgs),
    /// Rank nodes for a plain-English question: what `query_graph` starts from.
    Search(SearchArgs),
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// A question in plain English, or a name.
    pub question: String,
    /// Maximum results.
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct EvalArgs {
    /// A questions file (`[[question]]` entries with `ask`, `expect` and `note`) or a
    /// tasks file (`[[task]]` entries with `ask`, `expect`, `files` and `budget`).
    pub questions: PathBuf,
    /// How many starting nodes count: recall and hit rate are measured at this cut.
    #[arg(short = 'k', long, default_value_t = 5)]
    pub limit: usize,
    /// For tasks: a directory holding one store per base commit, named by the first
    /// eight characters of the task's `base`; a task without one uses `--store`.
    #[arg(long)]
    pub stores: Option<PathBuf>,
    /// For tasks: score only the task with this id.
    #[arg(long)]
    pub only: Option<String>,
    /// For tasks: what answers them.
    #[arg(long, value_enum, default_value_t = AnswerSource::ContextPack)]
    pub source: AnswerSource,
    /// For tasks answered by a context pack: write each pack to this directory as
    /// `<task id>.md` and `<task id>.json`.
    #[arg(long)]
    pub dump: Option<PathBuf>,
    /// For tasks with `--stores`: a directory holding one checkout per base commit,
    /// named like the stores, that source windows are read from; defaults to the
    /// parent of `--stores`, then to the repository root.
    #[arg(long)]
    pub trees: Option<PathBuf>,
}

/// What produces the answer a task is scored on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AnswerSource {
    /// The context pack for the task text, within the task's budget.
    ContextPack,
    /// Today's `query_graph` answer to the task text, as the MCP tool returns it.
    QueryGraph,
}

impl AnswerSource {
    fn name(self) -> &'static str {
        match self {
            Self::ContextPack => "context-pack",
            Self::QueryGraph => "query-graph",
        }
    }
}

#[derive(Debug, Args)]
pub struct IndexArgs {
    /// A SCIP index to merge as the precise tier (repeatable), besides any the
    /// profile's `[scip] indexes` names.
    #[arg(long, value_name = "index.scip")]
    pub scip: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Fall back to a full index when more than this share of files would be relinked.
    #[arg(long, default_value_t = UpdateArgs::DEFAULT_FULL_THRESHOLD)]
    pub full_threshold: f64,
}

impl UpdateArgs {
    /// Above this share of relinked files a full index is cheaper than a delta.
    const DEFAULT_FULL_THRESHOLD: f64 = 0.5;
}

impl Default for UpdateArgs {
    fn default() -> Self {
        Self {
            full_threshold: Self::DEFAULT_FULL_THRESHOLD,
        }
    }
}

#[derive(Debug, Args)]
pub struct Query {
    /// A name, qualified name or node identifier.
    pub query: String,
    /// Restrict to a node kind, such as `Function`, `Table`, `View`, `Task` or `Document`.
    #[arg(long)]
    pub kind: Option<NodeKind>,
    /// Maximum results.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct CardArgs {
    /// Names, qualified names or node identifiers; one card each.
    #[arg(required = true)]
    pub queries: Vec<String>,
    /// Restrict to a node kind, such as `Function`, `Table`, `View`, `Task` or `Document`.
    #[arg(long)]
    pub kind: Option<NodeKind>,
}

#[derive(Debug, Args)]
pub struct ContextArgs {
    /// The task, as an agent would receive it.
    pub task: String,
    /// Token budget for the pack (`cl100k_base` tokens of the Markdown).
    #[arg(long, default_value_t = surveyor_context::DEFAULT_BUDGET)]
    pub budget: usize,
    /// Identifiers to seed on besides what the task names; repeatable.
    #[arg(long = "seed")]
    pub seeds: Vec<String>,
}

#[derive(Debug, Args)]
pub struct AffectedArgs {
    #[command(flatten)]
    pub query: Query,
    /// How many hops to follow backwards.
    #[arg(long, default_value_t = 3)]
    pub depth: usize,
    /// Drop paths whose confidence product falls below this; 0 keeps every path.
    #[arg(long, default_value_t = Reach::DEFAULT_MIN_CONFIDENCE)]
    pub min_confidence: f32,
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
    /// Follow edges in their direction only (the default tries that first, then
    /// searches against direction and labels the result a lead).
    #[arg(long, conflicts_with = "undirected")]
    pub directed: bool,
    /// Ignore edge direction from the start.
    #[arg(long)]
    pub undirected: bool,
}

impl PathArgs {
    fn mode(&self) -> PathMode {
        match (self.directed, self.undirected) {
            (true, _) => PathMode::Directed,
            (_, true) => PathMode::Undirected,
            _ => PathMode::DirectedFirst,
        }
    }
}

#[derive(Debug, Args)]
pub struct TopArgs {
    /// Maximum results.
    #[arg(long, default_value_t = 20)]
    pub top: usize,
}

#[derive(Debug, Args)]
pub struct NodesArgs {
    /// Restrict to a node kind, such as `Function` or `Table`.
    #[arg(long)]
    pub kind: Option<NodeKind>,
    /// Restrict to nodes whose file starts with this path.
    #[arg(long = "in")]
    pub in_path: Option<String>,
    /// Keep only nodes that lack every one of these edge kinds, in either direction.
    #[arg(long, value_delimiter = ',')]
    pub without: Vec<EdgeKind>,
    /// Keep only nodes that have every one of these edge kinds, in either direction.
    #[arg(long = "with", value_delimiter = ',')]
    pub with_kinds: Vec<EdgeKind>,
    /// Maximum results.
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
}

/// Nodes matching the listing filters, in identifier order.
fn list_nodes<'a>(graph: &'a CodeGraph, a: &NodesArgs) -> Vec<&'a surveyor_model::Node> {
    let has = |node: &surveyor_model::Node, kind: &EdgeKind| {
        let filter = EdgeFilter::default().with_kinds(vec![kind.clone()]);
        !graph.all_neighbors(&node.id, &filter).is_empty()
    };
    let under = |node: &surveyor_model::Node| {
        a.in_path.as_deref().is_none_or(|p| {
            node.file()
                .is_some_and(|f| f.starts_with_dir(p) || f.as_str().starts_with(p))
        })
    };
    let mut found: Vec<&surveyor_model::Node> = graph
        .nodes()
        .iter()
        .filter(|n| a.kind.as_ref().is_none_or(|k| &n.kind == k))
        .filter(|n| under(n))
        .filter(|n| a.without.iter().all(|k| !has(n, k)))
        .filter(|n| a.with_kinds.iter().all(|k| has(n, k)))
        .collect();
    found.sort_by(|x, y| x.id.cmp(&y.id));
    found.truncate(a.limit);
    found
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Extra rule files or directories, besides the profile's `[rules] dirs`.
    #[arg(long = "rules", value_name = "PATH")]
    pub rules: Vec<PathBuf>,
}

/// Rule directories from the profile plus the extra paths, split into dirs and files.
pub(crate) fn rule_sources(
    root: &Path,
    profile: &Profile,
    extra: &[PathBuf],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (dirs, files): (Vec<PathBuf>, Vec<PathBuf>) =
        extra.iter().cloned().partition(|p| p.is_dir());
    let profile_dirs = profile.rules.dirs.iter().map(|d| root.join(d));
    (profile_dirs.chain(dirs).collect(), files)
}

#[derive(Debug, Args)]
pub struct SourceArgs {
    /// A node identifier, or `path:line` / `path:start-end`.
    pub target: String,
    /// Lines of context around the span.
    #[arg(long, default_value_t = 2)]
    pub context: u32,
}

pub struct Paths {
    pub root: PathBuf,
    pub store: PathBuf,
}

/// The profile: `--profile` when given, else `<repo-root>/surveyor.toml` or the defaults.
pub fn load_profile(cli: &Cli, p: &Paths) -> anyhow::Result<Profile> {
    match &cli.profile {
        Some(file) => {
            Profile::load_file(file).with_context(|| format!("load profile {}", file.display()))
        }
        None => Profile::load(&p.root).context("load surveyor.toml"),
    }
}

fn paths(cli: &Cli) -> anyhow::Result<Paths> {
    paths_from(cli, None)
}

/// The root and store: `--repo-root` and `--store` when given, else `fallback_root`
/// (a hook payload's working directory), else the current directory.
pub fn paths_from(cli: &Cli, fallback_root: Option<&Path>) -> anyhow::Result<Paths> {
    let root = match (&cli.repo_root, fallback_root) {
        (Some(r), _) => r.clone(),
        (None, Some(r)) => r.to_path_buf(),
        (None, None) => std::env::current_dir().context("current directory")?,
    };
    let root = root
        .canonicalize()
        .with_context(|| format!("repository root {}", root.display()))?;
    let store = cli.store.clone().unwrap_or_else(|| root.join(STORE_DIR));
    Ok(Paths { root, store })
}

pub async fn open_graph(cli: &Cli, p: &Paths) -> anyhow::Result<(LanceStore, CodeGraph, RepoId)> {
    open_graph_at(cli, p, &p.store).await
}

/// Opens the dataset at `store` (not necessarily the one `--store` names) and loads
/// its graph.
async fn open_graph_at(
    cli: &Cli,
    p: &Paths,
    store: &Path,
) -> anyhow::Result<(LanceStore, CodeGraph, RepoId)> {
    if !store.exists() {
        bail!(
            "no dataset at {}; run `surveyor index` first",
            store.display()
        );
    }
    let store = open_dataset(store).await?;
    let repo = if let Some(id) = &cli.repo_id {
        RepoId::new(id)
    } else {
        let snapshots = store.snapshots().await?;
        match snapshots.len() {
            0 => bail!("dataset at {} has no indexed repository", p.store.display()),
            1 => snapshots[0].repo.clone(),
            _ => {
                let guess = load_profile(cli, p)
                    .and_then(|profile| Ok(Repository::open_with(&p.root, None, &profile)?))
                    .map(|r| r.id().clone())
                    .ok();
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
    let graph = load_graph_for(&store, &repo, cli.all_repos).await?;
    tracing::info!(
        nodes = graph.node_count(),
        edges = graph.edge_count(),
        repositories = graph.repos().len(),
        ms = started.elapsed().as_millis() as u64,
        "graph loaded"
    );
    Ok((store, graph, repo))
}

/// Loads `repo`'s partition and, when the dataset holds other repositories, the
/// federated edges that touch it with their foreign endpoints, or every partition with
/// `all_repos`. A dataset of one repository loads as it always did and reads no other
/// table.
async fn load_graph_for(
    store: &LanceStore,
    repo: &RepoId,
    all_repos: bool,
) -> anyhow::Result<CodeGraph> {
    let (nodes, edges) = store.load_graph(Some(repo)).await.context("load graph")?;
    let others: Vec<RepoId> = store
        .snapshots()
        .await?
        .into_iter()
        .map(|s| s.repo)
        .filter(|r| r != repo)
        .collect();
    if others.is_empty() {
        return Ok(CodeGraph::new(nodes, edges));
    }
    let mut partitions = vec![Partition {
        repo: repo.clone(),
        nodes,
        edges,
    }];
    let federated = store
        .load_federated(repo)
        .await
        .context("load federated edges")?;
    if all_repos {
        for other in &others {
            let (nodes, edges) = store.load_graph(Some(other)).await.context("load graph")?;
            partitions.push(Partition {
                repo: other.clone(),
                nodes,
                edges,
            });
        }
    } else {
        // Only the far ends of the federated edges, each under its own repository.
        let mut wanted: BTreeMap<RepoId, BTreeSet<surveyor_model::NodeId>> = BTreeMap::new();
        for f in &federated {
            if f.src_repo != *repo {
                wanted
                    .entry(f.src_repo.clone())
                    .or_default()
                    .insert(f.edge.src.clone());
            }
            if f.dst_repo != *repo {
                wanted
                    .entry(f.dst_repo.clone())
                    .or_default()
                    .insert(f.edge.dst.clone());
            }
        }
        for (other, ids) in wanted {
            let ids: Vec<surveyor_model::NodeId> = ids.into_iter().collect();
            let nodes = store
                .nodes_in(&other, &ids)
                .await
                .context("load foreign nodes")?;
            partitions.push(Partition {
                repo: other,
                nodes,
                edges: Vec::new(),
            });
        }
    }
    Ok(CodeGraph::federated(repo.clone(), partitions, federated))
}

/// Runs the federate pass when the dataset holds other repositories and rewrites the
/// `federated` table from the whole store; a dataset of one repository is left as it
/// is. The table is rewritten rather than patched for the repository just written,
/// because a new declarer of a coordinate changes the confidence of every consumer's
/// row, whichever repository they are in; the rows are few.
async fn federate_after_write(store: &LanceStore, repo: &RepoId) -> anyhow::Result<usize> {
    let snapshots = store.snapshots().await?;
    if snapshots.len() < 2 {
        return Ok(0);
    }
    let started = Instant::now();
    let edges = federated_edges(store, &snapshots).await?;
    store
        .rewrite_federated(&edges)
        .await
        .context("write federated edges")?;
    let touching = edges
        .iter()
        .filter(|f| f.src_repo == *repo || f.dst_repo == *repo)
        .count();
    tracing::info!(
        federated = edges.len(),
        touching_this_repository = touching,
        repositories = snapshots.len(),
        ms = started.elapsed().as_millis() as u64,
        "federated"
    );
    Ok(touching)
}

/// The federate pass over every repository's packages and dependency edges.
async fn federated_edges(
    store: &LanceStore,
    snapshots: &[RepositorySnapshot],
) -> anyhow::Result<Vec<surveyor_model::FederatedEdge>> {
    let mut loaded = Vec::with_capacity(snapshots.len());
    for s in snapshots {
        let (nodes, edges) = store
            .load_packages(&s.repo)
            .await
            .context("load packages")?;
        loaded.push((s.repo.clone(), nodes, edges));
    }
    let input: Vec<RepoPackages<'_>> = loaded
        .iter()
        .map(|(repo, nodes, edges)| RepoPackages {
            repo: repo.clone(),
            packages: nodes,
            depends: edges,
        })
        .collect();
    Ok(federate(&input))
}

/// Resolves a user query to exactly one node, or explains why it cannot.
fn resolve_one<'a>(
    graph: &'a CodeGraph,
    query: &str,
    kind: Option<&NodeKind>,
) -> anyhow::Result<&'a surveyor_model::Node> {
    let listing = |nodes: &[&surveyor_model::Node]| {
        nodes
            .iter()
            .map(|n| format!("  {:<10} {}\n             {}", n.kind, location(n), n.id))
            .collect::<Vec<_>>()
            .join("\n")
    };
    match graph.lookup(query, kind) {
        Lookup::Exact(node) | Lookup::Unique(node) => Ok(node),
        Lookup::Ambiguous(nodes) => bail!(
            "{query:?} is ambiguous: {} nodes match it equally well. Repeat with one of their identifiers:\n{}",
            nodes.len(),
            listing(&nodes)
        ),
        Lookup::NotFound { similar } if similar.is_empty() => bail!("no node matches {query:?}"),
        Lookup::NotFound { similar } => bail!(
            "no node matches {query:?}; the qualifier matches none of the {} nodes with that name:\n{}",
            similar.len(),
            listing(&similar)
        ),
    }
}

fn location(node: &surveyor_model::Node) -> String {
    match (node.file(), node.lines()) {
        (Some(file), Some(lines)) => format!("{file}:{}-{}", lines.start, lines.end),
        (Some(file), None) => file.to_string(),
        _ => String::new(),
    }
}

#[expect(clippy::too_many_lines, reason = "one match arm per subcommand")]
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    if let Command::Hook(a) = &cli.command {
        // A hook finds its root in the payload and must exit 0 whatever happens.
        harness::hook(&cli, a).await;
        return Ok(());
    }
    let p = paths(&cli)?;
    let out = if cli.json { Output::Json } else { Output::Text };
    let ledger = harness::ledger(&cli, &p.store)?;
    let record = |event: surveyor_session::Event| harness::record(ledger.as_ref(), event);
    match &cli.command {
        Command::Index(a) => run_index(&cli, &p, &a.scip, &out).await,
        Command::Update(a) => {
            let updated = update(&cli, &p, a).await?;
            out.updated(&updated, &p.store);
            Ok(())
        }
        Command::Stats => {
            let store = open_dataset(&p.store).await?;
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
            let graph = std::sync::Arc::new(graph);
            let node = resolve_one(&graph, &q.query, q.kind.as_ref())?;
            let rules = rule_report(&cli, &p, &graph);
            let card = card_with_rules(&graph, node, rules.as_ref());
            out.explain(&graph, node, &card, q.limit);
            record(harness::answer(
                "explain",
                serde_json::json!({ "query": q.query }),
                harness::consulted_card(node, &card),
            ));
            Ok(())
        }
        Command::Card(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let graph = std::sync::Arc::new(graph);
            let nodes = a
                .queries
                .iter()
                .map(|q| resolve_one(&graph, q, a.kind.as_ref()))
                .collect::<anyhow::Result<Vec<_>>>()?;
            let rules = rule_report(&cli, &p, &graph);
            let started = Instant::now();
            let cards: Vec<surveyor_graph::Card> = nodes
                .iter()
                .map(|n| card_with_rules(&graph, n, rules.as_ref()))
                .collect();
            tracing::info!(
                cards = cards.len(),
                ms = started.elapsed().as_millis() as u64,
                "cards built"
            );
            out.cards(&cards);
            record(harness::answer(
                "card",
                serde_json::json!({ "queries": a.queries }),
                nodes
                    .iter()
                    .zip(&cards)
                    .flat_map(|(n, c)| harness::consulted_card(n, c))
                    .collect(),
            ));
            Ok(())
        }
        Command::Context(a) => {
            let pack = pack_for(&cli, &p, &a.task, a.budget, &a.seeds).await?;
            match out {
                Output::Json => render::print_json(&pack),
                Output::Text => print!("{}", pack.markdown),
            }
            record(surveyor_session::Event::pack_of(&pack));
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
            let reach = Reach::depth(a.depth).with_min_confidence(a.min_confidence);
            let hits = graph.affected(&node.id, reach, &filter);
            out.affected(node, &hits, a.depth);
            record(harness::answer(
                "affected",
                serde_json::json!({ "query": a.query.query, "depth": a.depth, "min_confidence": a.min_confidence }),
                std::iter::once(surveyor_session::Consulted::of_node(node))
                    .chain(
                        hits.iter()
                            .map(|h| surveyor_session::Consulted::of_node_via(h.node, h.via)),
                    )
                    .collect(),
            ));
            Ok(())
        }
        Command::Path(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let from = resolve_one(&graph, &a.from, None)?;
            let to = resolve_one(&graph, &a.to, None)?;
            let found = graph.best_path(&from.id, &to.id, a.mode());
            out.path(from, to, found.as_ref());
            record(harness::answer(
                "path",
                serde_json::json!({ "from": a.from, "to": a.to }),
                found.as_ref().map_or_else(
                    || {
                        vec![
                            surveyor_session::Consulted::of_node(from),
                            surveyor_session::Consulted::of_node(to),
                        ]
                    },
                    |path| {
                        path.steps
                            .iter()
                            .map(|s| match s.edge {
                                Some(e) => surveyor_session::Consulted::of_node_via(s.node, e),
                                None => surveyor_session::Consulted::of_node(s.node),
                            })
                            .collect()
                    },
                ),
            ));
            Ok(())
        }
        Command::Query(q) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let found = graph.find(&q.query, q.kind.as_ref(), q.limit);
            out.nodes(&found);
            record(harness::answer(
                "query",
                serde_json::json!({ "query": q.query }),
                found
                    .iter()
                    .map(|n| surveyor_session::Consulted::of_node(n))
                    .collect(),
            ));
            Ok(())
        }
        Command::Hubs(t) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let filter = EdgeFilter {
                include_structural: false,
                ..EdgeFilter::default()
            };
            let hubs = graph.hubs(t.top, &filter);
            out.hubs(&hubs);
            Ok(())
        }
        Command::Nodes(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let found = list_nodes(&graph, a);
            out.nodes(&found);
            record(harness::answer(
                "nodes",
                serde_json::json!({ "kind": a.kind.as_ref().map(ToString::to_string), "in": a.in_path }),
                found
                    .iter()
                    .map(|n| surveyor_session::Consulted::of_node(n))
                    .collect(),
            ));
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
            let citation = format!(
                "{}:{}-{}",
                window.file, window.span_start_line, window.span_end_line
            );
            record(harness::answer(
                "source",
                serde_json::json!({ "target": a.target }),
                vec![surveyor_session::Consulted::new(
                    if graph
                        .node(&surveyor_model::NodeId::new(&a.target))
                        .is_some()
                    {
                        a.target.clone()
                    } else {
                        citation.clone()
                    },
                    Some(citation),
                )],
            ));
            Ok(())
        }
        Command::Check(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let profile = load_profile(&cli, &p)?;
            let (dirs, files) = rule_sources(&p.root, &profile, &a.rules);
            let rules = match surveyor_rules::RuleSet::load(&dirs, &files) {
                Ok(rules) => rules,
                Err(e) => {
                    eprintln!("surveyor check: {e}");
                    std::process::exit(2);
                }
            };
            if rules.rules().is_empty() {
                bail!(
                    "no rules found under {} (profile `[rules] dirs`) or --rules",
                    dirs.iter()
                        .map(|d| d.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let report = match rules.run(std::sync::Arc::new(graph)) {
                Ok(report) => report,
                Err(e) => {
                    eprintln!("surveyor check: {e}");
                    std::process::exit(2);
                }
            };
            out.check(&report);
            record(harness::answer(
                "check",
                serde_json::Value::Null,
                report
                    .outcomes
                    .iter()
                    .flat_map(|o| o.violations.iter())
                    .map(|v| {
                        surveyor_session::Consulted::new(v.subject.clone(), v.citation.clone())
                    })
                    .collect(),
            ));
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Command::Eval(a) => match eval::load(&a.questions)? {
            eval::Suite::Questions(questions) => {
                let (_, graph, _) = open_graph(&cli, &p).await?;
                let started = Instant::now();
                let index = surveyor_search::TextIndex::build(graph.nodes());
                tracing::info!(
                    documents = index.len(),
                    ms = started.elapsed().as_millis() as u64,
                    "text index built"
                );
                let ids = |nodes: Vec<&surveyor_model::Node>| -> Vec<String> {
                    nodes.iter().map(|n| n.id.to_string()).collect()
                };
                let seeds =
                    |ask: &str, limit: usize| surveyor_search::seeds(&graph, &index, ask, limit);
                let scored = questions
                    .question
                    .iter()
                    .map(|q| {
                        let top = ids(seeds(&q.ask, a.limit.max(1)));
                        let deep = ids(seeds(&q.ask, eval::DEEP_RANK));
                        eval::score(q, &top, &deep, |id| {
                            graph.node(&surveyor_model::NodeId::new(id)).is_some()
                        })
                    })
                    .collect();
                let summary = eval::summarize(scored, a.limit.max(1));
                match out {
                    Output::Json => render::print_json(&summary),
                    Output::Text => eval::print_text(&summary, &a.questions),
                }
                Ok(())
            }
            eval::Suite::Tasks(tasks) => {
                let summary = eval_tasks(&cli, &p, a, &tasks).await?;
                match out {
                    Output::Json => render::print_json(&summary),
                    Output::Text => eval::print_tasks_text(&summary, &a.questions),
                }
                Ok(())
            }
        },
        Command::Search(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let index = surveyor_search::TextIndex::build(graph.nodes());
            let found = surveyor_search::ranked(&graph, &index, &a.question, a.limit.max(1));
            out.ranked(&found);
            record(harness::answer(
                "search",
                serde_json::json!({ "question": a.question }),
                found
                    .iter()
                    .map(|(_, n)| surveyor_session::Consulted::of_node(n))
                    .collect(),
            ));
            Ok(())
        }
        Command::Mcp => {
            let (store, graph, repo) = open_graph(&cli, &p).await?;
            let profile = load_profile(&cli, &p)?;
            let (dirs, files) = rule_sources(&p.root, &profile, &[]);
            let rules = surveyor_rules::RuleSet::load(&dirs, &files).context("load rules")?;
            let server = surveyor_mcp::Surveyor::new(store, graph, repo, p.root.clone(), rules);
            let server = match ledger {
                Some(ledger) => server.with_ledger(ledger),
                None => server,
            };
            surveyor_mcp::serve_stdio(server)
                .await
                .context("mcp server")?;
            Ok(())
        }
        Command::Federate => {
            let store = open_dataset(&p.store).await?;
            let snapshots = store.snapshots().await?;
            let edges = federated_edges(&store, &snapshots).await?;
            // A dataset of one repository gets no table unless it already has one.
            if snapshots.len() > 1 || store.federated_count().await? > 0 {
                store
                    .rewrite_federated(&edges)
                    .await
                    .context("write federated edges")?;
            }
            out.federated(&snapshots, &edges);
            Ok(())
        }
        Command::Pr(a) => pr::run(&cli, &p, a, out).await,
        Command::Session(a) => harness::session(&cli, &p, a, &out).await,
        Command::Hook(_) => unreachable!("handled before the dataset is opened"),
    }
}

/// The context pack for `task` inside `budget`, as `surveyor context` builds it.
pub async fn pack_for(
    cli: &Cli,
    p: &Paths,
    task: &str,
    budget: usize,
    seeds: &[String],
) -> anyhow::Result<surveyor_context::ContextPack> {
    let (store, graph, repo) = open_graph(cli, p).await?;
    let graph = std::sync::Arc::new(graph);
    let index = surveyor_search::TextIndex::build(graph.nodes());
    let tokenizer = surveyor_context::Tokenizer::cl100k();
    let rules = rule_report(cli, p, &graph);
    let unresolved = store.unresolved(&repo).await.unwrap_or_default();
    let planner = planner(
        &graph,
        &index,
        &tokenizer,
        &p.root,
        rules.as_ref(),
        unresolved,
    );
    let started = Instant::now();
    let pack = planner.pack(task, budget, seeds);
    tracing::info!(
        tokens = pack.used.tokens,
        items = pack.items.len(),
        ms = started.elapsed().as_millis() as u64,
        "context pack built"
    );
    Ok(pack)
}

/// How many rules the profile's directories hold; zero when they cannot be loaded.
pub fn rule_count(cli: &Cli, p: &Paths) -> usize {
    load_profile(cli, p)
        .ok()
        .and_then(|profile| {
            let (dirs, files) = rule_sources(&p.root, &profile, &[]);
            surveyor_rules::RuleSet::load(&dirs, &files).ok()
        })
        .map_or(0, |rules| rules.rules().len())
}

/// `surveyor update` with the defaults and nothing printed, for a session start.
pub async fn update_quietly(cli: &Cli, p: &Paths) -> anyhow::Result<()> {
    let updated = update(cli, p, &UpdateArgs::default()).await?;
    tracing::info!(
        repo = %updated.snapshot.repo,
        commit = %updated.snapshot.commit,
        strategy = %updated.strategy,
        ms = updated.took.as_millis() as u64,
        "dataset updated"
    );
    Ok(())
}

/// Scores every task of a tasks file: each against the store of its base commit
/// when `--stores` is given, else against the one dataset.
async fn eval_tasks(
    cli: &Cli,
    p: &Paths,
    a: &EvalArgs,
    tasks: &eval::Tasks,
) -> anyhow::Result<eval::TaskSummary> {
    let tokenizer = surveyor_context::Tokenizer::cl100k();
    let trees = a.trees.clone().or_else(|| {
        a.stores
            .as_ref()
            .and_then(|s| s.parent().map(Path::to_path_buf))
    });
    let selected = tasks
        .task
        .iter()
        .filter(|t| a.only.as_ref().is_none_or(|only| &t.id == only));
    let mut scored = Vec::new();
    // Tasks sharing a base share its graph; the file lists them in base order, so one
    // open per run of equal bases is enough.
    struct Loaded {
        store: PathBuf,
        root: PathBuf,
        graph: std::sync::Arc<CodeGraph>,
        repo: RepoId,
        index: surveyor_search::TextIndex,
        rules: Option<(surveyor_rules::RuleSet, surveyor_rules::Report)>,
        unresolved: Vec<surveyor_model::Unresolved>,
    }
    let mut loaded: Option<Loaded> = None;
    for task in selected {
        let (store, root) = match (&a.stores, task.store_name()) {
            (Some(dir), Some(name)) => {
                let tree = trees.as_ref().map(|t| t.join(&name)).filter(|t| t.is_dir());
                (dir.join(name), tree.unwrap_or_else(|| p.root.clone()))
            }
            _ => (p.store.clone(), p.root.clone()),
        };
        if loaded.as_ref().is_none_or(|l| l.store != store) {
            let (lance, graph, repo) = open_graph_at(cli, p, &store)
                .await
                .with_context(|| format!("task {}", task.id))?;
            let graph = std::sync::Arc::new(graph);
            let index = surveyor_search::TextIndex::build(graph.nodes());
            let rules = rule_report(cli, p, &graph);
            let unresolved = lance.unresolved(&repo).await.unwrap_or_default();
            loaded = Some(Loaded {
                store,
                root,
                graph,
                repo,
                index,
                rules,
                unresolved,
            });
        }
        let Some(l) = loaded.as_ref() else {
            unreachable!("loaded just above");
        };
        let (graph, index) = (&l.graph, &l.index);
        let answer = match a.source {
            AnswerSource::QueryGraph => {
                let view = surveyor_mcp::query_graph(graph, index, &l.repo, &task.ask, 5, 1);
                let text = serde_json::to_string_pretty(&view).context("serialize answer")?;
                answer_of(graph, view.node_ids(), &tokenizer.measure(&text))
            }
            AnswerSource::ContextPack => {
                let planner = planner(
                    graph,
                    index,
                    &tokenizer,
                    &l.root,
                    l.rules.as_ref(),
                    l.unresolved.clone(),
                );
                let pack = planner.pack(&task.ask, task.budget(), &[]);
                if let Some(dir) = &a.dump {
                    std::fs::create_dir_all(dir)
                        .with_context(|| format!("create {}", dir.display()))?;
                    std::fs::write(dir.join(format!("{}.md", task.id)), &pack.markdown)
                        .context("write pack markdown")?;
                    let json = serde_json::to_string_pretty(&pack).context("serialize pack")?;
                    std::fs::write(dir.join(format!("{}.json", task.id)), json)
                        .context("write pack json")?;
                }
                answer_of(
                    graph,
                    pack.items.iter().map(|i| i.card.id.as_str()),
                    &pack.used,
                )
            }
        };
        scored.push(eval::score_task(task, &answer, |id| {
            graph.node(&surveyor_model::NodeId::new(id)).is_some()
        }));
    }
    anyhow::ensure!(!scored.is_empty(), "no task matched --only");
    Ok(eval::summarize_tasks(a.source.name(), scored))
}

/// The key an answer carries: the ids it names and the files those nodes lie in.
fn answer_of<'a>(
    graph: &CodeGraph,
    ids: impl Iterator<Item = &'a str>,
    size: &surveyor_context::Measured,
) -> eval::Answer {
    let ids: BTreeSet<String> = ids.map(str::to_owned).collect();
    let files = ids
        .iter()
        .filter_map(|id| graph.node(&surveyor_model::NodeId::new(id)))
        .filter_map(|n| n.file().map(ToString::to_string))
        .collect();
    eval::Answer {
        ids,
        files,
        size: *size,
    }
}

/// Loads the repository's rules and runs them, for the rule notes on a card.
///
/// A card is a reading command, so a rule that fails to load or run is logged and
/// the card goes out without rule notes rather than failing the command; `check` is
/// where rule errors are reported.
fn rule_report(
    cli: &Cli,
    p: &Paths,
    graph: &std::sync::Arc<CodeGraph>,
) -> Option<(surveyor_rules::RuleSet, surveyor_rules::Report)> {
    let profile = match load_profile(cli, p) {
        Ok(profile) => profile,
        Err(e) => {
            tracing::warn!(error = %e, "profile not loaded; cards carry no rules");
            return None;
        }
    };
    let (dirs, files) = rule_sources(&p.root, &profile, &[]);
    let rules = match surveyor_rules::RuleSet::load(&dirs, &files) {
        Ok(rules) => rules,
        Err(e) => {
            tracing::warn!(error = %e, "rules not loaded; cards carry no rules");
            return None;
        }
    };
    if rules.rules().is_empty() {
        return None;
    }
    match rules.run(std::sync::Arc::clone(graph)) {
        Ok(report) => Some((rules, report)),
        Err(e) => {
            tracing::warn!(error = %e, "rules did not run; cards carry no rules");
            None
        }
    }
}

/// A node's card with the rule notes the last rule run gives it.
fn card_with_rules(
    graph: &CodeGraph,
    node: &surveyor_model::Node,
    rules: Option<&(surveyor_rules::RuleSet, surveyor_rules::Report)>,
) -> surveyor_graph::Card {
    let mut card = graph.card(node);
    if let Some((rules, report)) = rules {
        card.rules = surveyor_rules::notes_for(rules, report, node);
    }
    card
}

/// A planner over `graph` that reads source under `root`, notes rules from the last
/// run and lists the repository's unresolved references near the seeds.
fn planner<'g>(
    graph: &'g CodeGraph,
    index: &'g surveyor_search::TextIndex,
    tokenizer: &'g surveyor_context::Tokenizer,
    root: &'g Path,
    rules: Option<&'g (surveyor_rules::RuleSet, surveyor_rules::Report)>,
    unresolved: Vec<surveyor_model::Unresolved>,
) -> surveyor_context::Planner<'g> {
    surveyor_context::Planner::new(graph, index, tokenizer)
        .with_source(move |path: &RepoPath| std::fs::read_to_string(root.join(path.as_str())).ok())
        .with_rules(move |node| {
            rules.map_or_else(Vec::new, |(rules, report)| {
                surveyor_rules::notes_for(rules, report, node)
            })
        })
        .with_unresolved(unresolved)
}

/// What a full index produced.
struct Indexed {
    snapshot: RepositorySnapshot,
    stats: GraphStats,
    errors: usize,
}

/// The file rows and link entries for an extraction.
fn file_entries(extraction: &Extraction) -> (Vec<FileEntry>, Vec<FileRecord>) {
    let names = referenced_names(extraction);
    let files: Vec<FileEntry> = extraction
        .units
        .iter()
        .map(|u| FileEntry {
            path: u.path.clone(),
            language: u.language.clone(),
            content_hash: u.content_hash,
            bytes: u.bytes,
            lines: u.lines,
        })
        .collect();
    let records = files
        .iter()
        .map(|f| FileRecord {
            path: f.path.clone(),
            language: f.language.clone(),
            content_hash: f.content_hash,
            bytes: f.bytes,
            lines: f.lines,
            names: names.get(&f.path).cloned().unwrap_or_default(),
        })
        .collect();
    (files, records)
}

/// The names each file references: symbols and tables by short name, plus the paths
/// documentation cites. Incremental update uses them to find files to relink.
fn referenced_names(extraction: &Extraction) -> BTreeMap<RepoPath, Vec<String>> {
    let mut names: BTreeMap<RepoPath, BTreeSet<String>> = BTreeMap::new();
    for set in &extraction.fact_sets {
        let entry = names.entry(set.path.clone()).or_default();
        for fact in &set.facts {
            match fact {
                Fact::Reference(r) => {
                    entry.insert(r.name.clone());
                }
                Fact::Literal(l) if l.role == LiteralRole::Path => {
                    let cited = l.value.trim_start_matches("./").trim_end_matches('/');
                    entry.insert(cited.to_owned());
                }
                Fact::Literal(l) if l.role == LiteralRole::Command => {
                    entry.extend(surveyor_link::command_names(&l.value));
                }
                _ => {}
            }
        }
    }
    names
        .into_iter()
        .map(|(path, set)| (path, set.into_iter().collect()))
        .collect()
}

/// The SCIP indexes to merge: the profile's that exist, plus `extra`, which must.
fn scip_indexes(
    profile: &Profile,
    root: &std::path::Path,
    extra: &[PathBuf],
) -> anyhow::Result<Vec<PathBuf>> {
    let from_profile = profile.scip.indexes.iter().filter_map(|rel| {
        let path = root.join(rel);
        if path.is_file() {
            Some(path)
        } else {
            tracing::info!(index = %path.display(), "profile names a SCIP index that is absent; skipping");
            None
        }
    });
    let mut all: Vec<PathBuf> = from_profile.collect();
    for path in extra {
        if !path.is_file() {
            bail!("SCIP index {} does not exist", path.display());
        }
        all.push(path.clone());
    }
    Ok(all)
}

async fn run_index(cli: &Cli, p: &Paths, scip: &[PathBuf], out: &Output) -> anyhow::Result<()> {
    let started = Instant::now();
    let indexed = index(cli, p, scip).await?;
    out.indexed(
        &indexed.snapshot,
        &indexed.stats,
        &p.store,
        indexed.errors,
        started.elapsed(),
    );
    Ok(())
}

async fn index(cli: &Cli, p: &Paths, scip: &[PathBuf]) -> anyhow::Result<Indexed> {
    let started = Instant::now();
    let profile = load_profile(cli, p)?;
    let repo = Repository::open_with(&p.root, cli.repo_id.as_deref().map(RepoId::new), &profile)
        .context("open repository")?;
    let indexes = scip_indexes(&profile, &p.root, scip)?;
    tracing::info!(repo = %repo.id(), commit = %repo.commit(), root = %p.root.display(), scip = indexes.len(), "indexing");

    let extraction = extract_repository_with(&repo, &profile, &indexes).context("extract")?;
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

    let (files, records) = file_entries(&extraction);
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
        ms = linked_at.saturating_sub(extracted_at).as_millis() as u64,
        "linked"
    );

    let store = open_store(&p.store, &profile, &indexes).await?;
    let snapshot = store
        .replace_repository(repo.id(), repo.commit(), repo.is_dirty(), &delta, &records)
        .await
        .context("write dataset")?;
    let total = started.elapsed();
    tracing::info!(ms = total.saturating_sub(linked_at).as_millis() as u64, store = %p.store.display(), "written");
    federate_after_write(&store, repo.id()).await?;
    Ok(Indexed {
        snapshot,
        stats,
        errors: extraction.errors.len(),
    })
}

/// Reindexes what changed since the dataset's recorded commit.
///
/// Falls back to a full index when there is no index yet, when the engine, the profile
/// or a precise index changed, when a manifest changed, or when the plan would relink
/// more than `--full-threshold` of the files.
/// Reindexes what changed since the dataset's recorded commit, or everything when
/// the delta cannot be trusted, and says what it did.
async fn update(cli: &Cli, p: &Paths, args: &UpdateArgs) -> anyhow::Result<Updated> {
    let started = Instant::now();
    let profile = load_profile(cli, p)?;
    let repo = Repository::open_with(&p.root, cli.repo_id.as_deref().map(RepoId::new), &profile)
        .context("open repository")?;
    if !p.store.exists() {
        return full(cli, p, "no dataset", started).await;
    }
    let indexes = scip_indexes(&profile, &p.root, &[])?;
    let store = open_store(&p.store, &profile, &indexes).await?;
    let Some(snapshot) = store.snapshot(repo.id()).await? else {
        return full(cli, p, "repository not indexed", started).await;
    };
    if snapshot.surveyor_version != store.engine_version() {
        let reason = format!(
            "the dataset was written by engine {}; this build, its profile and its indexes are {}",
            snapshot.surveyor_version,
            store.engine_version()
        );
        return full(cli, p, &reason, started).await;
    }
    let known = store.files(repo.id()).await.context("read file records")?;
    let hashes = known
        .iter()
        .map(|f| (f.path.clone(), f.content_hash))
        .collect();
    let changes = repo
        .changes_since(&profile, Some(&snapshot.commit), snapshot.dirty, &hashes)
        .context("detect changes")?;
    tracing::info!(
        source = changes.source.as_str(),
        modified = changes.modified.len(),
        added = changes.added.len(),
        deleted = changes.deleted.len(),
        unchanged = changes.unchanged,
        ms = started.elapsed().as_millis() as u64,
        "changes"
    );
    let changed = ChangedFiles {
        modified: changes.modified.iter().cloned().collect(),
        added: changes.added.iter().cloned().collect(),
        deleted: changes.deleted.iter().cloned().collect(),
    };
    if changed.touches_manifest() {
        return full(cli, p, "a package manifest changed", started).await;
    }
    if changes.is_empty() && snapshot.commit == *repo.commit() && snapshot.dirty == repo.is_dirty()
    {
        return Ok(Updated {
            snapshot,
            strategy: changes.source.as_str().to_owned(),
            reason: None,
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            relinked: BTreeMap::new(),
            extraction_errors: 0,
            took: started.elapsed(),
        });
    }

    match incremental(&repo, &profile, &store, &known, &changed, args, &indexes).await? {
        Incremental::Applied {
            snapshot,
            relinked,
            extraction_errors,
        } => {
            federate_after_write(&store, repo.id()).await?;
            Ok(Updated {
                snapshot,
                strategy: changes.source.as_str().to_owned(),
                reason: None,
                modified: changes.modified,
                added: changes.added,
                deleted: changes.deleted,
                relinked,
                extraction_errors,
                took: started.elapsed(),
            })
        }
        Incremental::TooLarge(reason) => full(cli, p, &reason, started).await,
    }
}

/// How an incremental attempt ended.
enum Incremental {
    /// The delta was written.
    Applied {
        snapshot: RepositorySnapshot,
        relinked: BTreeMap<RepoPath, String>,
        extraction_errors: usize,
    },
    /// The plan would relink too much of the repository.
    TooLarge(String),
}

/// Probes the changed files, plans the relink, relinks and writes the delta.
async fn incremental(
    repo: &Repository,
    profile: &Profile,
    store: &LanceStore,
    known: &[FileRecord],
    changed: &ChangedFiles,
    args: &UpdateArgs,
    scip: &[PathBuf],
) -> anyhow::Result<Incremental> {
    let started = Instant::now();
    // The same extractors as a full index, precise tier included, so the files
    // re-extracted here keep the bindings the indexes give them.
    let extractors = extractors_with(profile, repo.root(), scip).context("load extractors")?;
    // Probe: link the changed files alone to learn the declarations they now hold.
    let first =
        extract_paths_using(&extractors, repo, profile, changed.to_extract()).context("extract")?;
    let (probe_files, _) = file_entries(&first);
    let probe = link(
        &LinkInput::new(repo.id().clone(), repo.commit().clone())
            .with_files(probe_files)
            .with_fact_sets(first.fact_sets.clone()),
    );
    let (nodes, edges) = store
        .load_graph(Some(repo.id()))
        .await
        .context("load graph")?;
    let loaded_at = started.elapsed();
    let names: HashMap<RepoPath, Vec<String>> = known
        .iter()
        .map(|f| (f.path.clone(), f.names.clone()))
        .collect();
    let unresolved = store
        .unresolved(repo.id())
        .await
        .context("load unresolved references")?;
    let plan = plan_update(&nodes, &edges, &names, &unresolved, changed, &probe);
    tracing::info!(
        relink = plan.relink.len(),
        drop_files = plan.drop_files.len(),
        drop_edges = plan.drop_edges.len(),
        seed_nodes = plan.seed.nodes.len(),
        seed_edges = plan.seed.edges.len(),
        ms = (started.elapsed().saturating_sub(loaded_at)).as_millis() as u64,
        "planned"
    );
    let share = plan.relink.len() as f64 / known.len().max(1) as f64;
    if share > args.full_threshold {
        return Ok(Incremental::TooLarge(format!(
            "{} of {} files would be relinked",
            plan.relink.len(),
            known.len()
        )));
    }

    let second = extract_paths_using(&extractors, repo, profile, plan.relink_unchanged())
        .context("extract")?;
    let mut extraction = first;
    extraction.units.extend(second.units);
    extraction.fact_sets.extend(second.fact_sets);
    extraction.errors.extend(second.errors);
    for err in extraction.errors.iter().take(20) {
        tracing::warn!(error = %err, "extraction error");
    }
    let (files, records) = file_entries(&extraction);
    let input = LinkInput::new(repo.id().clone(), repo.commit().clone())
        .with_files(files)
        .with_fact_sets(extraction.fact_sets);
    let delta = relink(&input, plan.seed);
    let linked_at = started.elapsed();
    tracing::info!(
        nodes = delta.nodes.len(),
        edges = delta.edges.len(),
        unresolved = delta.unresolved.len(),
        "relinked"
    );
    let store_update = StoreUpdate {
        drop_files: plan.drop_files.iter().cloned().collect(),
        drop_edges: plan.drop_edges,
        delta,
        files: records,
    };
    let snapshot = store
        .apply_update(repo.id(), repo.commit(), repo.is_dirty(), &store_update)
        .await
        .context("write dataset")?;
    tracing::info!(
        ms = started.elapsed().saturating_sub(linked_at).as_millis() as u64,
        "written"
    );
    Ok(Incremental::Applied {
        snapshot,
        relinked: plan
            .relink
            .into_iter()
            .map(|(path, reason)| (path, reason.label()))
            .collect(),
        extraction_errors: extraction.errors.len(),
    })
}

/// Runs a full index in place of an update and reports it as such.
async fn full(cli: &Cli, p: &Paths, reason: &str, started: Instant) -> anyhow::Result<Updated> {
    tracing::info!(reason, "updating with a full index");
    let indexed = index(cli, p, &[]).await?;
    Ok(Updated {
        snapshot: indexed.snapshot,
        strategy: "full".to_owned(),
        reason: Some(reason.to_owned()),
        modified: Vec::new(),
        added: Vec::new(),
        deleted: Vec::new(),
        relinked: BTreeMap::new(),
        extraction_errors: indexed.errors,
        took: started.elapsed(),
    })
}
