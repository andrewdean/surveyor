//! Command definitions and their handlers.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, bail};
use clap::{Args, Parser, Subcommand};
use surveyor_extract::{
    Extraction, Profile, Repository, extract_paths_using, extract_repository_with, extractors_with,
};
use surveyor_graph::{CodeGraph, EdgeFilter, Lookup, PathMode, Reach};
use surveyor_link::{ChangedFiles, FileEntry, LinkInput, link, plan_update, relink};
use surveyor_model::{EdgeKind, Fact, GraphStats, LiteralRole, NodeKind, RepoId, RepoPath};
use surveyor_store::{FileRecord, LanceStore, RepositorySnapshot, StoreUpdate};

use crate::eval;
use crate::render::{self, Output, Updated};

/// Directory under the repository root that holds the dataset.
const STORE_DIR: &str = ".surveyor";

/// The engine that writes a dataset: crate version, query packs, extractor and linker
/// revisions and the precise indexes it merges.
///
/// A dataset written by a different engine, or from different SCIP indexes, is
/// re-indexed in full on `update`, because facts from two engines or two indexes
/// cannot be mixed file by file.
fn engine_version(inputs: &[PathBuf]) -> String {
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
        .chain(index_hashes)
        .collect::<Vec<_>>();
    let fingerprint = surveyor_model::ContentHash::of_parts(parts.iter().map(String::as_str));
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        &fingerprint.short()[..8]
    )
}

/// Opens the dataset at `path`, stamping new snapshots with this build's engine and
/// the precise `inputs` it merges.
async fn open_store(path: &Path, inputs: &[PathBuf]) -> anyhow::Result<LanceStore> {
    Ok(LanceStore::open(path)
        .await
        .context("open dataset")?
        .with_engine_version(engine_version(inputs)))
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
    /// Profile file to use instead of `<repo-root>/surveyor.toml`, for a repository
    /// that cannot carry one.
    #[arg(long, global = true, env = "SURVEYOR_PROFILE")]
    pub profile: Option<PathBuf>,
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
    /// Serve the graph over MCP on stdio.
    Mcp,
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
    /// A questions file: `[[question]]` entries with `ask`, `expect` and `note`.
    pub questions: PathBuf,
    /// How many starting nodes count: recall and hit rate are measured at this cut.
    #[arg(short = 'k', long, default_value_t = 5)]
    pub limit: usize,
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
    #[arg(long, default_value_t = 0.5)]
    pub full_threshold: f64,
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
fn rule_sources(root: &Path, profile: &Profile, extra: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
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

struct Paths {
    root: PathBuf,
    store: PathBuf,
}

/// The profile: `--profile` when given, else `<repo-root>/surveyor.toml` or the defaults.
fn load_profile(cli: &Cli, p: &Paths) -> anyhow::Result<Profile> {
    match &cli.profile {
        Some(file) => {
            Profile::load_file(file).with_context(|| format!("load profile {}", file.display()))
        }
        None => Profile::load(&p.root).context("load surveyor.toml"),
    }
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
    let store = open_store(&p.store, &[]).await?;
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
    let p = paths(&cli)?;
    let out = if cli.json { Output::Json } else { Output::Text };
    match &cli.command {
        Command::Index(a) => run_index(&cli, &p, &a.scip, &out).await,
        Command::Update(a) => update(&cli, &p, a, &out).await,
        Command::Stats => {
            let store = open_store(&p.store, &[]).await?;
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
            let reach = Reach::depth(a.depth).with_min_confidence(a.min_confidence);
            let hits = graph.affected(&node.id, reach, &filter);
            out.affected(node, &hits, a.depth);
            Ok(())
        }
        Command::Path(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let from = resolve_one(&graph, &a.from, None)?;
            let to = resolve_one(&graph, &a.to, None)?;
            let found = graph.best_path(&from.id, &to.id, a.mode());
            out.path(from, to, found.as_ref());
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
            let hubs = graph.hubs(t.top, &filter);
            out.hubs(&hubs);
            Ok(())
        }
        Command::Nodes(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let found = list_nodes(&graph, a);
            out.nodes(&found);
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
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Command::Eval(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let questions = eval::load(&a.questions)?;
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
        Command::Search(a) => {
            let (_, graph, _) = open_graph(&cli, &p).await?;
            let index = surveyor_search::TextIndex::build(graph.nodes());
            let found = surveyor_search::ranked(&graph, &index, &a.question, a.limit.max(1));
            out.ranked(&found);
            Ok(())
        }
        Command::Mcp => {
            let (store, graph, repo) = open_graph(&cli, &p).await?;
            let profile = load_profile(&cli, &p)?;
            let (dirs, files) = rule_sources(&p.root, &profile, &[]);
            let rules = surveyor_rules::RuleSet::load(&dirs, &files).context("load rules")?;
            surveyor_mcp::serve_stdio(store, graph, repo, p.root.clone(), rules)
                .await
                .context("mcp server")?;
            Ok(())
        }
    }
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

    let store = open_store(&p.store, &indexes).await?;
    let snapshot = store
        .replace_repository(repo.id(), repo.commit(), repo.is_dirty(), &delta, &records)
        .await
        .context("write dataset")?;
    let total = started.elapsed();
    tracing::info!(ms = total.saturating_sub(linked_at).as_millis() as u64, store = %p.store.display(), "written");
    Ok(Indexed {
        snapshot,
        stats,
        errors: extraction.errors.len(),
    })
}

/// Reindexes what changed since the dataset's recorded commit.
///
/// Falls back to a full index when there is no index yet, when a manifest changed, or
/// when the plan would relink more than `--full-threshold` of the files.
async fn update(cli: &Cli, p: &Paths, args: &UpdateArgs, out: &Output) -> anyhow::Result<()> {
    let started = Instant::now();
    let profile = load_profile(cli, p)?;
    let repo = Repository::open_with(&p.root, cli.repo_id.as_deref().map(RepoId::new), &profile)
        .context("open repository")?;
    if !p.store.exists() {
        return full(cli, p, out, "no dataset", started).await;
    }
    let indexes = scip_indexes(&profile, &p.root, &[])?;
    let store = open_store(&p.store, &indexes).await?;
    let Some(snapshot) = store.snapshot(repo.id()).await? else {
        return full(cli, p, out, "repository not indexed", started).await;
    };
    if snapshot.surveyor_version != store.engine_version() {
        let reason = format!(
            "the dataset was written by engine {}; this build and its indexes are {}",
            snapshot.surveyor_version,
            store.engine_version()
        );
        return full(cli, p, out, &reason, started).await;
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
        return full(cli, p, out, "a package manifest changed", started).await;
    }
    if changes.is_empty() && snapshot.commit == *repo.commit() && snapshot.dirty == repo.is_dirty()
    {
        out.updated(
            &Updated {
                snapshot,
                strategy: changes.source.as_str().to_owned(),
                reason: None,
                modified: Vec::new(),
                added: Vec::new(),
                deleted: Vec::new(),
                relinked: BTreeMap::new(),
                extraction_errors: 0,
                took: started.elapsed(),
            },
            &p.store,
        );
        return Ok(());
    }

    match incremental(&repo, &profile, &store, &known, &changed, args, &indexes).await? {
        Incremental::Applied {
            snapshot,
            relinked,
            extraction_errors,
        } => {
            out.updated(
                &Updated {
                    snapshot,
                    strategy: changes.source.as_str().to_owned(),
                    reason: None,
                    modified: changes.modified,
                    added: changes.added,
                    deleted: changes.deleted,
                    relinked,
                    extraction_errors,
                    took: started.elapsed(),
                },
                &p.store,
            );
            Ok(())
        }
        Incremental::TooLarge(reason) => full(cli, p, out, &reason, started).await,
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
async fn full(
    cli: &Cli,
    p: &Paths,
    out: &Output,
    reason: &str,
    started: Instant,
) -> anyhow::Result<()> {
    tracing::info!(reason, "updating with a full index");
    let indexed = index(cli, p, &[]).await?;
    out.updated(
        &Updated {
            snapshot: indexed.snapshot,
            strategy: "full".to_owned(),
            reason: Some(reason.to_owned()),
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            relinked: BTreeMap::new(),
            extraction_errors: indexed.errors,
            took: started.elapsed(),
        },
        &p.store,
    );
    Ok(())
}
