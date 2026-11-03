//! Text and JSON rendering of answers, always with citations.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use surveyor_model::RepoPath;

use anyhow::{Context, bail};
use serde::Serialize;
use surveyor_graph::{
    Affected, CodeGraph, Community, Direction, EdgeFilter, FoundPath, Hub, Neighbor, PathDirection,
    PathTier,
};
use surveyor_model::{Edge, EdgeKind, GraphStats, Node, NodeKind, Unresolved};
use surveyor_rules::{Report, Severity};
use surveyor_store::{RepositorySnapshot, StoreStats};

/// The shape answers are rendered in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    /// Human-readable text with citations.
    Text,
    /// Pretty-printed JSON.
    Json,
}

#[derive(Debug, Serialize)]
pub struct NodeView {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub qualified_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

impl NodeView {
    pub fn of(n: &Node) -> Self {
        Self {
            id: n.id.to_string(),
            kind: n.kind.to_string(),
            name: n.name.clone(),
            qualified_name: n.qualified_name.clone(),
            file: n.file().map(ToString::to_string),
            lines: n.lines().map(|l| l.to_string()),
            signature: n.signature.clone(),
            doc: n.doc.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EdgeView {
    pub kind: String,
    pub src: String,
    pub dst: String,
    pub derivation: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

impl EdgeView {
    pub fn of(e: &Edge) -> Self {
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

/// What an `update` run did.
#[derive(Debug)]
pub struct Updated {
    /// The snapshot after the write.
    pub snapshot: RepositorySnapshot,
    /// `git`, `hashes` or `full`.
    pub strategy: String,
    /// Why a full index ran instead, when it did.
    pub reason: Option<String>,
    /// Files whose content changed.
    pub modified: Vec<RepoPath>,
    /// Files that are new.
    pub added: Vec<RepoPath>,
    /// Files that are gone.
    pub deleted: Vec<RepoPath>,
    /// Every relinked file with the reason.
    pub relinked: BTreeMap<RepoPath, String>,
    /// Per-file extraction failures.
    pub extraction_errors: usize,
    /// Wall-clock time.
    pub took: Duration,
}

fn location(n: &Node) -> String {
    n.location
        .as_ref()
        .map(surveyor_model::Location::citation)
        .unwrap_or_default()
}

fn cite(e: &Edge) -> String {
    let mut s = e
        .provenance
        .evidence
        .first()
        .map(surveyor_model::Evidence::citation)
        .unwrap_or_default();
    let d = e.provenance.derivation.kind();
    if e.provenance.confidence.get() < 1.0 {
        s = format!("{s} [{d} {:.2}]", e.provenance.confidence.get());
    } else if d != surveyor_model::DerivationKind::Syntactic {
        s = format!("{s} [{d}]");
    }
    if !e.attributes.is_empty() {
        let attrs: Vec<String> = e
            .attributes
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        s = format!("{s} {{{}}}", attrs.join(", "));
    }
    s
}

fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("json: {e}"),
    }
}

fn first_paragraph(doc: &str) -> String {
    let text = doc
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    match text.char_indices().nth(400) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text,
    }
}

impl Output {
    pub fn indexed(
        self,
        snap: &RepositorySnapshot,
        stats: &GraphStats,
        store: &Path,
        errors: usize,
        took: Duration,
    ) {
        if self == Self::Json {
            #[derive(Serialize)]
            struct View<'a> {
                repo: &'a str,
                commit: &'a str,
                store: String,
                nodes: u64,
                edges: u64,
                files: u64,
                unresolved: u64,
                extraction_errors: usize,
                seconds: f64,
                nodes_by_kind: &'a BTreeMap<String, usize>,
                edges_by_kind: &'a BTreeMap<String, usize>,
                edges_by_derivation: &'a BTreeMap<String, usize>,
            }
            print_json(&View {
                repo: snap.repo.as_str(),
                commit: snap.commit.as_str(),
                store: store.display().to_string(),
                nodes: snap.nodes,
                edges: snap.edges,
                files: snap.files,
                unresolved: snap.unresolved,
                extraction_errors: errors,
                seconds: took.as_secs_f64(),
                nodes_by_kind: &stats.nodes_by_kind,
                edges_by_kind: &stats.edges_by_kind,
                edges_by_derivation: &stats.edges_by_derivation,
            });
            return;
        }
        println!(
            "indexed {} at {} in {:.1}s",
            snap.repo,
            short(snap.commit.as_str()),
            took.as_secs_f64()
        );
        println!(
            "  files {}  nodes {}  edges {}  unresolved {}  extraction errors {}",
            snap.files, snap.nodes, snap.edges, snap.unresolved, errors
        );
        println!("  store {}", store.display());
        println!("  nodes by kind: {}", top_counts(&stats.nodes_by_kind, 12));
        println!("  edges by kind: {}", top_counts(&stats.edges_by_kind, 12));
        println!(
            "  edges by derivation: {}",
            top_counts(&stats.edges_by_derivation, 5)
        );
    }

    pub fn updated(self, u: &Updated, store: &Path) {
        if self == Self::Json {
            #[derive(Serialize)]
            struct View<'a> {
                repo: &'a str,
                commit: &'a str,
                store: String,
                strategy: &'a str,
                #[serde(skip_serializing_if = "Option::is_none")]
                reason: Option<&'a str>,
                modified: &'a [RepoPath],
                added: &'a [RepoPath],
                deleted: &'a [RepoPath],
                relinked: &'a BTreeMap<RepoPath, String>,
                nodes: u64,
                edges: u64,
                files: u64,
                unresolved: u64,
                extraction_errors: usize,
                seconds: f64,
            }
            print_json(&View {
                repo: u.snapshot.repo.as_str(),
                commit: u.snapshot.commit.as_str(),
                store: store.display().to_string(),
                strategy: &u.strategy,
                reason: u.reason.as_deref(),
                modified: &u.modified,
                added: &u.added,
                deleted: &u.deleted,
                relinked: &u.relinked,
                nodes: u.snapshot.nodes,
                edges: u.snapshot.edges,
                files: u.snapshot.files,
                unresolved: u.snapshot.unresolved,
                extraction_errors: u.extraction_errors,
                seconds: u.took.as_secs_f64(),
            });
            return;
        }
        println!(
            "updated {} to {} in {:.1}s ({}{})",
            u.snapshot.repo,
            short(u.snapshot.commit.as_str()),
            u.took.as_secs_f64(),
            u.strategy,
            u.reason
                .as_deref()
                .map(|r| format!(": {r}"))
                .unwrap_or_default()
        );
        println!(
            "  changed: {} modified, {} added, {} deleted; relinked {} files",
            u.modified.len(),
            u.added.len(),
            u.deleted.len(),
            u.relinked.len()
        );
        for (path, reason) in u.relinked.iter().take(20) {
            println!("    {path}  ({reason})");
        }
        if u.relinked.len() > 20 {
            println!("    … and {} more", u.relinked.len() - 20);
        }
        println!(
            "  files {}  nodes {}  edges {}  unresolved {}  extraction errors {}",
            u.snapshot.files,
            u.snapshot.nodes,
            u.snapshot.edges,
            u.snapshot.unresolved,
            u.extraction_errors
        );
        println!("  store {}", store.display());
    }

    pub fn stats(self, store: &Path, stats: &StoreStats, per_repo: Option<&GraphStats>) {
        if self == Self::Json {
            #[derive(Serialize)]
            struct Repo<'a> {
                repo: &'a str,
                commit: &'a str,
                nodes: u64,
                edges: u64,
                files: u64,
                unresolved: u64,
                indexed_at_ms: i64,
                surveyor_version: &'a str,
            }
            #[derive(Serialize)]
            struct View<'a> {
                store: String,
                version: u64,
                nodes: usize,
                edges: usize,
                repositories: Vec<Repo<'a>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                graph: Option<&'a GraphStats>,
            }
            print_json(&View {
                store: store.display().to_string(),
                version: stats.version,
                nodes: stats.nodes,
                edges: stats.edges,
                repositories: stats
                    .repositories
                    .iter()
                    .map(|s| Repo {
                        repo: s.repo.as_str(),
                        commit: s.commit.as_str(),
                        nodes: s.nodes,
                        edges: s.edges,
                        files: s.files,
                        unresolved: s.unresolved,
                        indexed_at_ms: s.indexed_at.as_unix_millis(),
                        surveyor_version: &s.surveyor_version,
                    })
                    .collect(),
                graph: per_repo,
            });
            return;
        }
        println!(
            "store {} (lance version {})",
            store.display(),
            stats.version
        );
        println!("  nodes {}  edges {}", stats.nodes, stats.edges);
        for s in &stats.repositories {
            println!(
                "  {}: commit {}  files {}  nodes {}  edges {}  unresolved {}  (surveyor {})",
                s.repo,
                short(s.commit.as_str()),
                s.files,
                s.nodes,
                s.edges,
                s.unresolved,
                s.surveyor_version
            );
        }
        if let Some(g) = per_repo {
            println!("  nodes by kind: {}", top_counts(&g.nodes_by_kind, 40));
            println!("  edges by kind: {}", top_counts(&g.edges_by_kind, 40));
            println!(
                "  edges by derivation: {}",
                top_counts(&g.edges_by_derivation, 5)
            );
        }
    }

    pub fn nodes(self, nodes: &[&Node]) {
        if self == Self::Json {
            print_json(&nodes.iter().map(|n| NodeView::of(n)).collect::<Vec<_>>());
            return;
        }
        if nodes.is_empty() {
            println!("no matches");
        } else {
            println!("{} nodes", nodes.len());
        }
        for n in nodes {
            println!("{:<10} {:<50} {}", n.kind, n.qualified_name, location(n));
            println!("           {}", n.id);
        }
    }

    pub fn explain(self, graph: &CodeGraph, node: &Node, limit: usize) {
        let all = EdgeFilter::default();
        let out = graph.neighbors(&node.id, Direction::Outgoing, &all);
        let inc = graph.neighbors(&node.id, Direction::Incoming, &all);
        let members = members_of(graph, node);
        if self == Self::Json {
            #[derive(Serialize)]
            struct Rel {
                kind: String,
                node: NodeView,
                evidence: Vec<String>,
                derivation: String,
                confidence: f32,
            }
            #[derive(Serialize)]
            struct View {
                node: NodeView,
                outgoing: Vec<Rel>,
                incoming: Vec<Rel>,
                members: Vec<NodeView>,
            }
            let rel = |n: &Neighbor<'_>| Rel {
                kind: n.edge.kind.to_string(),
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
            };
            print_json(&View {
                node: NodeView::of(node),
                outgoing: out.iter().map(rel).collect(),
                incoming: inc.iter().map(rel).collect(),
                members: members.iter().map(|n| NodeView::of(n)).collect(),
            });
            return;
        }
        println!("{} {}", node.kind, node.qualified_name);
        println!("  id        {}", node.id);
        if !location(node).is_empty() {
            println!("  location  {}", location(node));
        }
        if let Some(sig) = &node.signature {
            println!("  signature {sig}");
        }
        if let Some(doc) = &node.doc {
            println!("  doc       {}", first_paragraph(doc));
        }
        for (k, v) in &node.attributes {
            if k != "syntax" && k != "sql" {
                println!("  {k:<9} {v}");
            }
        }
        if !members.is_empty() {
            println!("members ({}):", members.len());
            for m in members.iter().take(limit) {
                println!("  {:<8} {:<40} {}", m.kind, m.name, location(m));
            }
            if members.len() > limit {
                println!("  … {} more", members.len() - limit);
            }
        }
        print_groups("outgoing", &out, limit, true);
        print_groups("incoming", &inc, limit, false);
    }

    pub fn affected(self, node: &Node, hits: &[Affected<'_>], depth: usize) {
        if self == Self::Json {
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
                affected: Vec<Hit>,
            }
            print_json(&View {
                node: NodeView::of(node),
                depth,
                affected: hits
                    .iter()
                    .map(|h| Hit {
                        depth: h.depth,
                        confidence: h.confidence,
                        node: NodeView::of(h.node),
                        via: EdgeView::of(h.via),
                    })
                    .collect(),
            });
            return;
        }
        println!(
            "affected by a change to {} {} (depth {}): {} nodes",
            node.kind,
            node.qualified_name,
            depth,
            hits.len()
        );
        let mut by_depth: BTreeMap<usize, Vec<&Affected<'_>>> = BTreeMap::new();
        for h in hits {
            by_depth.entry(h.depth).or_default().push(h);
        }
        for (d, items) in by_depth {
            println!("depth {d} ({}):", items.len());
            for h in items {
                let confidence = if h.confidence < 1.0 {
                    format!("{:.2}", h.confidence)
                } else {
                    String::new()
                };
                println!(
                    "  {:<8} {:<48} {:<4} via {:<12} {}",
                    h.node.kind,
                    truncate(&h.node.qualified_name, 48),
                    confidence,
                    h.via.kind,
                    cite(h.via)
                );
            }
        }
    }

    pub fn path(self, from: &Node, to: &Node, path: Option<&FoundPath<'_>>) {
        if self == Self::Json {
            #[derive(Serialize)]
            struct Step {
                node: NodeView,
                #[serde(skip_serializing_if = "Option::is_none")]
                edge: Option<EdgeView>,
            }
            #[derive(Serialize)]
            struct View {
                from: NodeView,
                to: NodeView,
                found: bool,
                #[serde(skip_serializing_if = "Option::is_none")]
                tier: Option<PathTier>,
                #[serde(skip_serializing_if = "Option::is_none")]
                direction: Option<PathDirection>,
                steps: Vec<Step>,
            }
            print_json(&View {
                from: NodeView::of(from),
                to: NodeView::of(to),
                found: path.is_some(),
                tier: path.map(|p| p.tier),
                direction: path.map(|p| p.direction),
                steps: path
                    .map(|p| p.steps.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .map(|s| Step {
                        node: NodeView::of(s.node),
                        edge: s.edge.map(EdgeView::of),
                    })
                    .collect(),
            });
            return;
        }
        match path {
            None => println!(
                "no path from {} to {}",
                from.qualified_name, to.qualified_name
            ),
            Some(found) => {
                let tier = match found.tier {
                    PathTier::Precise => "every hop certain",
                    PathTier::Loose => "some hops resolved by name; check the confidences",
                    PathTier::Any => "only through containment or test coverage; treat as a lead",
                };
                let direction = match found.direction {
                    PathDirection::Directed => "",
                    PathDirection::Undirected => {
                        "; no directed path exists, so hops marked <- run against their edge: a lead, not a dependency chain"
                    }
                };
                println!(
                    "path from {} to {} ({} hops, {tier}{direction}):",
                    from.qualified_name,
                    to.qualified_name,
                    found.steps.len().saturating_sub(1)
                );
                for s in &found.steps {
                    match s.edge {
                        None => println!(
                            "  {} {}  ({})",
                            s.node.kind,
                            s.node.qualified_name,
                            location(s.node)
                        ),
                        Some(e) => {
                            let arrow = if e.src == s.node.id { "<-" } else { "->" };
                            println!(
                                "  {arrow} {:<12} {} {}  ({})",
                                e.kind,
                                s.node.kind,
                                s.node.qualified_name,
                                cite(e)
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn hubs(self, hubs: &[Hub<'_>]) {
        if self == Self::Json {
            #[derive(Serialize)]
            struct View {
                dependent_files: usize,
                in_degree: usize,
                out_degree: usize,
                rank: f64,
                node: NodeView,
            }
            print_json(
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
            );
            return;
        }
        println!(
            "{:<6} {:<6} {:<6} {:<8} {:<10} node",
            "files", "in", "out", "rank", "kind"
        );
        for h in hubs {
            println!(
                "{:<6} {:<6} {:<6} {:<8.4} {:<10} {}  ({})",
                h.dependent_files,
                h.in_degree,
                h.out_degree,
                h.rank,
                h.node.kind,
                h.node.qualified_name,
                location(h.node)
            );
        }
    }

    pub fn communities(self, graph: &CodeGraph, comms: &[Community], top: usize) {
        if self == Self::Json {
            print_json(&comms.iter().take(top).collect::<Vec<_>>());
            return;
        }
        println!(
            "{} communities (showing {})",
            comms.len(),
            comms.len().min(top)
        );
        for c in comms.iter().take(top) {
            println!("community {} ({} members):", c.id, c.members.len());
            for m in c.members.iter().take(12) {
                if let Some(n) = graph.node(m) {
                    println!("  {:<8} {}", n.kind, n.qualified_name);
                }
            }
            if c.members.len() > 12 {
                println!("  … {} more", c.members.len() - 12);
            }
        }
    }

    pub fn unresolved(self, items: &[Unresolved], top: usize) {
        if self == Self::Json {
            print_json(&items.iter().take(top).collect::<Vec<_>>());
            return;
        }
        let counts = items
            .iter()
            .fold(BTreeMap::<(String, &str), usize>::new(), |mut acc, u| {
                *acc.entry((u.reason.to_string(), u.name.as_str()))
                    .or_default() += 1;
                acc
            });
        let mut rows: Vec<((String, &str), usize)> = counts.into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        println!(
            "{} unresolved references, {} distinct names",
            items.len(),
            rows.len()
        );
        for ((reason, name), count) in rows.into_iter().take(top) {
            println!("  {count:>6}  {reason:<18} {name}");
        }
    }

    pub fn check(self, report: &Report) {
        if self == Self::Json {
            print_json(report);
            return;
        }
        for o in &report.outcomes {
            let verdict = if o.passed { "ok  " } else { "FAIL" };
            let judged = match (o.severity, o.budget) {
                (Severity::Budget, Some(b)) => format!("{} of {b} allowed", o.violations.len()),
                (Severity::Warn, _) => format!("{} (warning)", o.violations.len()),
                _ => o.violations.len().to_string(),
            };
            println!("{verdict} [{}] {} — {judged}", o.id, o.desc);
            for v in &o.violations {
                match &v.citation {
                    Some(c) => println!("      {c}  {}  ({})", v.message, v.subject),
                    None => println!("      {}  ({})", v.message, v.subject),
                }
            }
        }
        let failed = report.outcomes.iter().filter(|o| !o.passed).count();
        println!(
            "{} rules, {} violations, {failed} failed",
            report.outcomes.len(),
            report.violation_count()
        );
    }

    pub fn source(self, window: &SourceWindow) {
        if self == Self::Json {
            print_json(window);
            return;
        }
        println!("{}:{}-{}", window.file, window.start_line, window.end_line);
        for (i, line) in window.lines.iter().enumerate() {
            let no = window.start_line as usize + i;
            let marker =
                if no >= window.span_start_line as usize && no <= window.span_end_line as usize {
                    ">"
                } else {
                    " "
                };
            println!("{marker}{no:>5} | {line}");
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SourceWindow {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub span_start_line: u32,
    pub span_end_line: u32,
    pub lines: Vec<String>,
}

/// Reads the lines around a node's span or a `path:line[-line]` citation.
pub fn source_window(
    graph: &CodeGraph,
    root: &Path,
    target: &str,
    context: u32,
) -> anyhow::Result<SourceWindow> {
    let (file, start, end) = resolve_target(graph, target)?;
    let text = std::fs::read_to_string(root.join(&file)).with_context(|| format!("read {file}"))?;
    let all: Vec<&str> = text.lines().collect();
    if all.is_empty() {
        bail!("{file} is empty");
    }
    let first = start.saturating_sub(context).max(1);
    let last = (end + context).min(all.len() as u32);
    let lines = all[(first as usize - 1)..(last as usize)]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    Ok(SourceWindow {
        file,
        start_line: first,
        end_line: last,
        span_start_line: start,
        span_end_line: end,
        lines,
    })
}

/// A node id, a `path:line[-line]` citation, or a name, as file and line range.
fn resolve_target(graph: &CodeGraph, target: &str) -> anyhow::Result<(String, u32, u32)> {
    let of_node = |node: &Node| -> anyhow::Result<(String, u32, u32)> {
        let loc = node.source().context("node has no source span")?;
        Ok((
            loc.file.to_string(),
            loc.span.lines.start,
            loc.span.lines.end,
        ))
    };
    if let Some(node) = graph.node(&surveyor_model::NodeId::new(target)) {
        return of_node(node);
    }
    if let Some((path, range)) = target.rsplit_once(':')
        && let Ok(lines) = parse_line_range(range)
    {
        return Ok((path.to_owned(), lines.0, lines.1));
    }
    graph
        .find(target, None, 1)
        .first()
        .context("no such node or citation")
        .and_then(|n| of_node(n))
}

/// `12` or `12-20` as a closed line range.
fn parse_line_range(range: &str) -> anyhow::Result<(u32, u32)> {
    match range.split_once('-') {
        Some((a, b)) => Ok((a.parse()?, b.parse()?)),
        None => range.parse().map(|n| (n, n)).map_err(Into::into),
    }
}

/// Members: `DEFINES` and `CONTAINS` children, plus those of impl blocks for a type.
fn members_of<'a>(graph: &'a CodeGraph, node: &Node) -> Vec<&'a Node> {
    let structural = EdgeFilter::default().with_kinds(vec![EdgeKind::Defines, EdgeKind::Contains]);
    let mut out: Vec<&Node> = graph
        .neighbors(&node.id, Direction::Outgoing, &structural)
        .into_iter()
        .map(|n| n.node)
        .collect();
    if node.kind.is_type() {
        let uses = EdgeFilter::default().with_kinds(vec![EdgeKind::UsesType]);
        for imp in graph.neighbors(&node.id, Direction::Incoming, &uses) {
            if imp.node.kind == NodeKind::Impl && impl_is_for(imp.node, node) {
                out.extend(
                    graph
                        .neighbors(&imp.node.id, Direction::Outgoing, &structural)
                        .into_iter()
                        .map(|n| n.node),
                );
            }
        }
    }
    out.sort_by_key(|a| a.lines());
    out.dedup_by(|a, b| a.id == b.id);
    out
}

/// Whether an impl block is written for `ty`. An impl's name is its self type
/// followed by `#impl`, so `impl From<&Report> for Summary` belongs to `Summary`
/// although it also uses `Report`.
fn impl_is_for(imp: &Node, ty: &Node) -> bool {
    [imp.name.as_str(), imp.qualified_name.as_str()]
        .into_iter()
        .filter_map(|name| name.split("#impl").next())
        .filter_map(|self_type| self_type.rsplit("::").next())
        .any(|self_type| self_type == ty.name)
}

fn print_groups(label: &str, items: &[Neighbor<'_>], limit: usize, outgoing: bool) {
    if items.is_empty() {
        return;
    }
    let mut groups: BTreeMap<String, Vec<&Neighbor<'_>>> = BTreeMap::new();
    for n in items {
        groups.entry(n.edge.kind.to_string()).or_default().push(n);
    }
    println!("{label}:");
    for (kind, list) in groups {
        let arrow = if outgoing { "->" } else { "<-" };
        println!("  {arrow} {kind} ({})", list.len());
        for n in list.iter().take(limit) {
            println!(
                "     {:<8} {:<44} {}",
                n.node.kind,
                truncate(&n.node.qualified_name, 44),
                cite(n.edge)
            );
        }
        if list.len() > limit {
            println!("     … {} more", list.len() - limit);
        }
    }
}

fn top_counts(map: &BTreeMap<String, usize>, n: usize) -> String {
    let mut v: Vec<(&String, &usize)> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    v.into_iter()
        .take(n)
        .map(|(k, c)| format!("{k} {c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn short(commit: &str) -> &str {
    if commit.len() > 12 {
        &commit[..12]
    } else {
        commit
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let cut: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
