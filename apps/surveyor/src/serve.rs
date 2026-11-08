//! `surveyor serve`: the existing answers behind a port, for a team.
//!
//! The server owns no new state. It opens the dataset `--store` names, loads one
//! graph per repository the dataset holds (with the federated edges that touch it,
//! or every repository with `--all-repos`), builds the text index and loads the rules
//! exactly as `surveyor mcp` does, and shares that state between two fronts: a JSON
//! API under `/v1` whose every route returns the structure the CLI's `--json` prints
//! for the same command, and MCP over streamable HTTP at `/mcp/<repo>`, one service
//! per repository, with the stdio server's tools unchanged.
//!
//! A task polls the dataset's generation (the sum of its tables' Lance versions)
//! every `--reload-every` seconds and swaps a repository's graph, index and rule run
//! when it changed, so an `update` into the shared dataset reaches readers without a
//! restart. The listener binds loopback unless `--listen` says otherwise, `--token`
//! requires a bearer token on every request, and TLS is a reverse proxy's job.

// Rust guideline compliant 2026-02-21

use std::collections::BTreeMap;
use std::io::Write as _;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use clap::Args;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surveyor_extract::Profile;
use surveyor_graph::{EdgeFilter, Reach};
use surveyor_mcp::{Live, Slot, Surveyor};
use surveyor_model::{NodeKind, RepoId};
use surveyor_store::LanceStore;

use crate::app::{self, Cli, Paths};
use crate::render::{self, Output};

/// Arguments of `surveyor serve`.
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Address to listen on. Loopback by default; a team server behind a proxy
    /// listens on the proxy's network and lets the proxy terminate TLS.
    #[arg(long, default_value = "127.0.0.1:7700", env = "SURVEYOR_LISTEN")]
    pub listen: String,
    /// Bearer token every request must carry as `Authorization: Bearer <token>`.
    #[arg(long, env = "SURVEYOR_TOKEN", hide_env_values = true)]
    pub token: Option<String>,
    /// A repository's checkout, as `<repo>=<path>`, for source windows and the source
    /// in context packs; repeatable. A dataset of one repository takes `--repo-root`.
    /// A repository without one serves every answer but `source` and
    /// `read_project_source`.
    #[arg(long = "root", value_name = "REPO=PATH")]
    pub roots: Vec<String>,
    /// Seconds between checks of the dataset for a new write; 0 never reloads.
    #[arg(long, default_value_t = 30, value_name = "SECONDS")]
    pub reload_every: u64,
    /// Host names the MCP endpoint accepts in the `Host` header besides loopback,
    /// such as the name a proxy forwards; repeatable.
    #[arg(long = "host", value_name = "NAME")]
    pub hosts: Vec<String>,
}

/// One repository as the server holds it.
struct Repo {
    slot: Arc<Slot>,
    root: Option<PathBuf>,
    /// Where the rules come from: the profile at the root, or `--profile`.
    profile: Option<PathBuf>,
    /// Generation the slot was loaded at, and when.
    loaded_generation: AtomicU64,
    loaded_at_ms: AtomicI64,
}

/// What every handler sees.
struct Shared {
    store_path: PathBuf,
    store: LanceStore,
    repos: BTreeMap<RepoId, Repo>,
    all_repos: bool,
    token: Option<String>,
    generation: AtomicU64,
    started: Instant,
}

type AppState = Arc<Shared>;

/// A refused request: the status and one line saying why, as `{"error": ...}`.
#[derive(Debug)]
struct ApiError(StatusCode, String);

impl ApiError {
    fn bad_request(e: impl std::fmt::Display) -> Self {
        Self(StatusCode::BAD_REQUEST, e.to_string())
    }

    fn not_found(e: impl std::fmt::Display) -> Self {
        Self(StatusCode::NOT_FOUND, e.to_string())
    }

    fn internal(e: impl std::fmt::Display) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

type ApiResult = Result<Json<Value>, ApiError>;

// ---- loading ------------------------------------------------------------------------------

/// `<repo>=<path>` pairs, each path canonical.
fn parse_roots(raw: &[String]) -> anyhow::Result<BTreeMap<RepoId, PathBuf>> {
    raw.iter()
        .map(|entry| {
            let (repo, path) = entry
                .split_once('=')
                .with_context(|| format!("--root {entry:?} is not <repo>=<path>"))?;
            let path = PathBuf::from(path)
                .canonicalize()
                .with_context(|| format!("--root {entry:?}: checkout"))?;
            Ok((RepoId::new(repo), path))
        })
        .collect()
}

/// The rules for a repository: the profile's `[rules] dirs` under its root, or none
/// when it has no root. A rule that does not load is reported and leaves the
/// repository without rules, as the MCP server does.
fn load_rules(
    repo: &RepoId,
    root: Option<&Path>,
    profile: Option<&Path>,
) -> surveyor_rules::RuleSet {
    let Some(root) = root else {
        return surveyor_rules::RuleSet::default();
    };
    let profile = match profile {
        Some(file) => Profile::load_file(file),
        None => Profile::load(root),
    };
    let profile = match profile {
        Ok(profile) => profile,
        Err(e) => {
            tracing::warn!(repo = %repo, error = %e, "profile not loaded; no rules");
            return surveyor_rules::RuleSet::default();
        }
    };
    let (dirs, files) = app::rule_sources(root, &profile, &[]);
    match surveyor_rules::RuleSet::load(&dirs, &files) {
        Ok(rules) => rules,
        Err(e) => {
            tracing::warn!(repo = %repo, error = %e, "rules not loaded");
            surveyor_rules::RuleSet::default()
        }
    }
}

/// Loads one repository's state: graph, text index, rules.
async fn load_repo(
    store: &LanceStore,
    repo: &RepoId,
    root: Option<&Path>,
    profile: Option<&Path>,
    all_repos: bool,
) -> anyhow::Result<Surveyor> {
    let started = Instant::now();
    let graph = app::load_graph_for(store, repo, all_repos).await?;
    let rules = load_rules(repo, root, profile);
    let server = Surveyor::new(
        store.clone(),
        graph,
        repo.clone(),
        root.map(Path::to_path_buf),
        rules,
    );
    tracing::info!(
        repo = %repo,
        nodes = server.graph().node_count(),
        edges = server.graph().edge_count(),
        rules = server.rules().rules().len(),
        ms = started.elapsed().as_millis() as u64,
        "loaded"
    );
    Ok(server)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// Reloads every repository when the dataset's generation moved.
async fn reload_if_changed(shared: &Shared) -> anyhow::Result<bool> {
    let generation = shared
        .store
        .generation()
        .await
        .context("dataset generation")?;
    if generation == shared.generation.load(Ordering::Acquire) {
        return Ok(false);
    }
    let started = Instant::now();
    for (id, repo) in &shared.repos {
        let server = load_repo(
            &shared.store,
            id,
            repo.root.as_deref(),
            repo.profile.as_deref(),
            shared.all_repos,
        )
        .await
        .with_context(|| format!("reload {id}"))?;
        repo.slot.swap(server);
        repo.loaded_generation.store(generation, Ordering::Release);
        repo.loaded_at_ms.store(now_ms(), Ordering::Release);
    }
    let known = shared.repos.len();
    let present = shared.store.snapshots().await?.len();
    if present > known {
        tracing::warn!(
            serving = known,
            in_dataset = present,
            "the dataset gained a repository; restart to serve it"
        );
    }
    shared.generation.store(generation, Ordering::Release);
    tracing::info!(
        generation,
        repositories = known,
        ms = started.elapsed().as_millis() as u64,
        "reloaded"
    );
    Ok(true)
}

// ---- handlers -----------------------------------------------------------------------------

/// The repository a path names, as the state holds it now.
fn repo_of(shared: &Shared, repo: &str) -> Result<Arc<Surveyor>, ApiError> {
    shared
        .repos
        .get(&RepoId::new(repo))
        .map(|r| r.slot.current())
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "no repository {repo:?} on this server; it serves {}",
                shared
                    .repos
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}

/// Runs graph work off the async runtime so a long walk never stalls another request.
async fn blocking<F>(f: F) -> ApiResult
where
    F: FnOnce() -> Result<Value, ApiError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ApiError::internal(format!("worker: {e}")))?
        .map(Json)
}

fn kind_of(kind: Option<&str>) -> Result<Option<NodeKind>, ApiError> {
    kind.map(|k| k.parse::<NodeKind>().map_err(ApiError::bad_request))
        .transpose()
}

#[derive(Debug, Serialize)]
struct RepoView<'a> {
    repo: &'a str,
    commit: &'a str,
    nodes: u64,
    edges: u64,
    files: u64,
    unresolved: u64,
    indexed_at_ms: i64,
    surveyor_version: &'a str,
    /// The checkout source is read from, when the server has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    root: Option<String>,
    rules: usize,
    loaded_generation: u64,
    loaded_at_ms: i64,
    /// Where the repository's MCP service is mounted.
    mcp: String,
}

/// `GET /v1/repos`: every repository the server holds, with what it loaded and when.
async fn repos(State(shared): State<AppState>) -> ApiResult {
    let snapshots = shared.store.snapshots().await.map_err(ApiError::internal)?;
    let repositories: Vec<RepoView<'_>> = snapshots
        .iter()
        .filter_map(|s| shared.repos.get(&s.repo).map(|r| (s, r)))
        .map(|(s, r)| RepoView {
            repo: s.repo.as_str(),
            commit: s.commit.as_str(),
            nodes: s.nodes,
            edges: s.edges,
            files: s.files,
            unresolved: s.unresolved,
            indexed_at_ms: s.indexed_at.as_unix_millis(),
            surveyor_version: &s.surveyor_version,
            root: r.root.as_ref().map(|p| p.display().to_string()),
            rules: r.slot.current().rules().rules().len(),
            loaded_generation: r.loaded_generation.load(Ordering::Acquire),
            loaded_at_ms: r.loaded_at_ms.load(Ordering::Acquire),
            mcp: format!("/mcp/{}", s.repo),
        })
        .collect();
    Ok(Json(serde_json::json!({
        "store": shared.store_path.display().to_string(),
        "generation": shared.generation.load(Ordering::Acquire),
        "uptime_s": shared.started.elapsed().as_secs(),
        "surveyor": env!("CARGO_PKG_VERSION"),
        "repositories": repositories,
    })))
}

/// `GET /v1/{repo}/stats`: what `surveyor --repo-id <repo> stats --json` prints.
async fn stats(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    let stats = shared.store.stats().await.map_err(ApiError::internal)?;
    let per_repo = shared
        .store
        .graph_stats(server.repo())
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(render::stats_json(
        &shared.store_path,
        &stats,
        Some(&per_repo),
    )))
}

#[derive(Debug, Deserialize)]
struct NodeQuery {
    /// A node identifier or a name.
    q: String,
    /// Restrict the lookup to a node kind.
    kind: Option<String>,
}

/// `GET /v1/{repo}/explain?q=`: what `surveyor explain <q> --json` prints.
async fn explain(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<NodeQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let kind = kind_of(q.kind.as_deref())?;
        let node =
            app::resolve_one(server.graph(), &q.q, kind.as_ref()).map_err(ApiError::bad_request)?;
        let card = server.card(node);
        Ok(render::explain_json(server.graph(), node, &card))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct CardQuery {
    /// Node identifiers or names, comma-separated.
    q: String,
    kind: Option<String>,
}

/// `GET /v1/{repo}/card?q=a,b`: what `surveyor card a b --json` prints.
async fn card(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<CardQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let kind = kind_of(q.kind.as_deref())?;
        let cards =
            q.q.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|name| {
                    app::resolve_one(server.graph(), name, kind.as_ref())
                        .map(|node| server.card(node))
                        .map_err(ApiError::bad_request)
                })
                .collect::<Result<Vec<_>, _>>()?;
        if cards.is_empty() {
            return Err(ApiError::bad_request("q names no node"));
        }
        Ok(render::cards_json(&cards))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct AffectedQuery {
    q: String,
    kind: Option<String>,
    /// Hops to follow backwards (default 3).
    depth: Option<usize>,
    /// Smallest path confidence still reported (default 0.25).
    min_confidence: Option<f32>,
    /// Edge kinds to follow, comma-separated; default is the impact set.
    kinds: Option<String>,
    /// Include inferred and observed edges.
    all_derivations: Option<bool>,
}

/// `GET /v1/{repo}/affected?q=&depth=&min_confidence=`: what `surveyor affected`
/// prints with `--json`.
async fn affected(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<AffectedQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let kind = kind_of(q.kind.as_deref())?;
        let node =
            app::resolve_one(server.graph(), &q.q, kind.as_ref()).map_err(ApiError::bad_request)?;
        let mut filter = EdgeFilter::impact();
        if let Some(kinds) = q.kinds.as_deref().filter(|k| !k.is_empty()) {
            let kinds = kinds
                .split(',')
                .map(|k| k.trim().parse().map_err(ApiError::bad_request))
                .collect::<Result<Vec<_>, _>>()?;
            filter = filter.with_kinds(kinds);
        }
        if q.all_derivations.unwrap_or(false) {
            filter.deterministic_only = false;
        }
        let depth = q.depth.unwrap_or(3).clamp(1, 8);
        let reach = Reach::depth(depth)
            .with_min_confidence(q.min_confidence.unwrap_or(0.25).clamp(0.0, 1.0));
        let hits = server.graph().affected(&node.id, reach, &filter);
        Ok(render::affected_json(node, &hits, depth))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct PathQuery {
    from: String,
    to: String,
    /// `directed`, `undirected` or `auto` (default: directed first).
    mode: Option<String>,
}

/// `GET /v1/{repo}/path?from=&to=`: what `surveyor path <from> <to> --json` prints.
async fn path(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<PathQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let mode = match q.mode.as_deref() {
            None | Some("auto") => surveyor_graph::PathMode::DirectedFirst,
            Some("directed") => surveyor_graph::PathMode::Directed,
            Some("undirected") => surveyor_graph::PathMode::Undirected,
            Some(other) => {
                return Err(ApiError::bad_request(format!(
                    "mode {other:?} is not directed, undirected or auto"
                )));
            }
        };
        let from =
            app::resolve_one(server.graph(), &q.from, None).map_err(ApiError::bad_request)?;
        let to = app::resolve_one(server.graph(), &q.to, None).map_err(ApiError::bad_request)?;
        let found = server.graph().best_path(&from.id, &to.id, mode);
        Ok(render::path_json(from, to, found.as_ref()))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct FindQuery {
    q: String,
    kind: Option<String>,
    /// Maximum results (default 20).
    limit: Option<usize>,
}

/// `GET /v1/{repo}/query?q=&kind=`: what `surveyor query <q> --json` prints.
async fn query(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<FindQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let kind = kind_of(q.kind.as_deref())?;
        let found = server
            .graph()
            .find(&q.q, kind.as_ref(), q.limit.unwrap_or(20).clamp(1, 500));
        Ok(render::nodes_json(&found))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    /// Maximum results (default 10).
    limit: Option<usize>,
}

/// `GET /v1/{repo}/search?q=&limit=`: what `surveyor search <q> --json` prints.
async fn search(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<SearchQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        let found = surveyor_search::ranked(
            server.graph(),
            server.index(),
            &q.q,
            q.limit.unwrap_or(10).clamp(1, 200),
        );
        Ok(render::ranked_json(&found))
    })
    .await
}

#[derive(Debug, Deserialize)]
struct ContextQuery {
    task: String,
    /// Token budget (default 30000).
    budget: Option<usize>,
    /// Identifiers to seed on besides what the task names, comma-separated.
    seeds: Option<String>,
}

/// `GET /v1/{repo}/context?task=&budget=`: what `surveyor context <task> --json`
/// prints. Source windows come from the repository's checkout when the server has one.
async fn context(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<ContextQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    let budget = q
        .budget
        .unwrap_or(surveyor_context::DEFAULT_BUDGET)
        .clamp(500, 200_000);
    let seeds: Vec<String> = q
        .seeds
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let pack = server.pack(&q.task, budget, &seeds).await;
    Ok(Json(render::json_value(&pack)))
}

#[derive(Debug, Deserialize)]
struct SourceQuery {
    /// A node identifier, or `path:line` / `path:start-end`.
    target: String,
    /// Lines of context around the span (default 2).
    context: Option<u32>,
}

/// `GET /v1/{repo}/source?target=&context=`: what `surveyor source <target> --json`
/// prints; refused for a repository the server has no checkout of.
async fn source(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
    Query(q): Query<SourceQuery>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    let Some(root) = server.root().map(Path::to_path_buf) else {
        return Err(ApiError(
            StatusCode::CONFLICT,
            format!(
                "this server has no checkout of repository {}; start it with --root {}=<path> to serve source",
                server.repo(),
                server.repo()
            ),
        ));
    };
    blocking(move || {
        let window =
            render::source_window(server.graph(), &root, &q.target, q.context.unwrap_or(2))
                .map_err(ApiError::bad_request)?;
        Ok(render::source_json(&window))
    })
    .await
}

/// `GET /v1/{repo}/check`: what `surveyor check --json` prints, from the rule run
/// the server keeps for the loaded graph.
async fn check(
    State(shared): State<AppState>,
    axum::extract::Path(repo): axum::extract::Path<String>,
) -> ApiResult {
    let server = repo_of(&shared, &repo)?;
    blocking(move || {
        if server.rules().rules().is_empty() {
            return Err(ApiError::not_found(format!(
                "repository {} has no rules on this server (a profile `[rules] dirs` under its root)",
                server.repo()
            )));
        }
        server
            .report()
            .map(render::check_json)
            .ok_or_else(|| ApiError::internal("the rules did not run; see the server log"))
    })
    .await
}

/// Refuses a request without the bearer token the server was started with.
async fn require_token(
    State(shared): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let Some(expected) = &shared.token else {
        return next.run(request).await;
    };
    let given = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    if given.is_some_and(|given| same_secret(given.as_bytes(), expected.as_bytes())) {
        next.run(request).await
    } else {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "a bearer token is required: Authorization: Bearer <token>".to_owned(),
        )
        .into_response()
    }
}

/// Byte equality in time that depends only on the lengths, not on where they differ.
fn same_secret(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// ---- the server ---------------------------------------------------------------------------

/// The repositories to serve: `--repo-id` when given, else every one in the dataset.
async fn repositories_to_serve(cli: &Cli, store: &LanceStore) -> anyhow::Result<Vec<RepoId>> {
    let snapshots = store.snapshots().await?;
    if snapshots.is_empty() {
        bail!("the dataset has no indexed repository");
    }
    match &cli.repo_id {
        Some(id) => {
            let id = RepoId::new(id);
            if !snapshots.iter().any(|s| s.repo == id) {
                bail!(
                    "no repository {id} in the dataset (it holds: {})",
                    snapshots
                        .iter()
                        .map(|s| s.repo.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            Ok(vec![id])
        }
        None => Ok(snapshots.into_iter().map(|s| s.repo).collect()),
    }
}

/// Builds the router: the JSON API, one MCP service per repository, the token check.
fn router(shared: &AppState, hosts: &[String]) -> Router {
    let api = Router::new()
        .route("/v1/repos", get(repos))
        .route("/v1/{repo}/stats", get(stats))
        .route("/v1/{repo}/explain", get(explain))
        .route("/v1/{repo}/card", get(card))
        .route("/v1/{repo}/affected", get(affected))
        .route("/v1/{repo}/path", get(path))
        .route("/v1/{repo}/query", get(query))
        .route("/v1/{repo}/search", get(search))
        .route("/v1/{repo}/context", get(context))
        .route("/v1/{repo}/source", get(source))
        .route("/v1/{repo}/check", get(check));
    let mut router = api;
    for (id, repo) in &shared.repos {
        let slot = Arc::clone(&repo.slot);
        let mut config =
            rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default();
        config.allowed_hosts.extend(hosts.iter().cloned());
        let service = StreamableHttpService::new(
            move || Ok(Live::new(Arc::clone(&slot))),
            Arc::new(LocalSessionManager::default()),
            config,
        );
        router = router.nest_service(&format!("/mcp/{id}"), service);
    }
    router
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(shared),
            require_token,
        ))
        .with_state(Arc::clone(shared))
}

/// Runs `surveyor serve` until interrupted.
pub async fn run(cli: &Cli, p: &Paths, args: &ServeArgs, out: Output) -> anyhow::Result<()> {
    let store = app::open_dataset(&p.store).await?;
    let ids = repositories_to_serve(cli, &store).await?;
    let mut roots = parse_roots(&args.roots)?;
    // A dataset of one repository is the common case: its checkout is --repo-root.
    if let [only] = ids.as_slice()
        && !roots.contains_key(only)
    {
        roots.insert(only.clone(), p.root.clone());
    }
    let generation = store.generation().await.context("dataset generation")?;
    let mut repos = BTreeMap::new();
    for id in &ids {
        let root = roots.get(id).cloned();
        // --profile names one repository's profile; with several, each root has its own.
        let profile = cli.profile.clone().filter(|_| ids.len() == 1);
        let server = load_repo(
            &store,
            id,
            root.as_deref(),
            profile.as_deref(),
            cli.all_repos,
        )
        .await
        .with_context(|| format!("load {id}"))?;
        repos.insert(
            id.clone(),
            Repo {
                slot: Arc::new(Slot::new(server)),
                root,
                profile,
                loaded_generation: AtomicU64::new(generation),
                loaded_at_ms: AtomicI64::new(now_ms()),
            },
        );
    }
    let shared: AppState = Arc::new(Shared {
        store_path: p.store.clone(),
        store,
        repos,
        all_repos: cli.all_repos,
        token: args.token.clone(),
        generation: AtomicU64::new(generation),
        started: Instant::now(),
    });
    let listener = tokio::net::TcpListener::bind(&args.listen)
        .await
        .with_context(|| format!("listen on {}", args.listen))?;
    let addr: SocketAddr = listener.local_addr().context("listener address")?;
    announce(out, addr, &shared, args);

    if args.reload_every > 0 {
        let every = Duration::from_secs(args.reload_every);
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(every);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            tick.tick().await;
            loop {
                tick.tick().await;
                if let Err(e) = reload_if_changed(&shared).await {
                    tracing::warn!(error = %e, "reload failed; serving the previous graph");
                }
            }
        });
    }

    axum::serve(listener, router(&shared, &args.hosts))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("stopping");
        })
        .await
        .context("serve")?;
    Ok(())
}

/// Says where the server listens and what it serves, once, on stdout.
fn announce(out: Output, addr: SocketAddr, shared: &Shared, args: &ServeArgs) {
    let repos: Vec<Value> = shared
        .repos
        .iter()
        .map(|(id, r)| {
            serde_json::json!({
                "repo": id.to_string(),
                "root": r.root.as_ref().map(|p| p.display().to_string()),
                "mcp": format!("http://{addr}/mcp/{id}"),
            })
        })
        .collect();
    match out {
        Output::Json => render::print_json(&serde_json::json!({
            "listen": addr.to_string(),
            "api": format!("http://{addr}/v1"),
            "token": shared.token.is_some(),
            "reload_every_s": args.reload_every,
            "repositories": repos,
        })),
        Output::Text => {
            println!(
                "serving http://{addr}/v1 and {} ({}{}{})",
                if shared.repos.len() == 1 {
                    format!(
                        "http://{addr}/mcp/{}",
                        shared.repos.keys().next().map_or("", RepoId::as_str)
                    )
                } else {
                    format!("http://{addr}/mcp/<repo>")
                },
                shared
                    .repos
                    .iter()
                    .map(|(id, r)| match &r.root {
                        Some(root) => format!("{id} at {}", root.display()),
                        None => format!("{id}, no checkout"),
                    })
                    .collect::<Vec<_>>()
                    .join("; "),
                if shared.token.is_some() {
                    "; bearer token required"
                } else {
                    ""
                },
                if args.reload_every > 0 {
                    format!("; reload every {} s", args.reload_every)
                } else {
                    String::new()
                }
            );
        }
    }
    let _ = std::io::stdout().flush();
}
