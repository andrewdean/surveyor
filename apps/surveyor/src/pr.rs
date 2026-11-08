//! `surveyor pr`: what a change touches, what it may break, and whether the
//! invariants hold, as a pull-request comment and a gate.
//!
//! One command, two consumers. CI posts the Markdown as a comment and reads the exit
//! code; a harness script runs it before `gh pr create` and refuses to continue on
//! exit 1. Surveyor never calls GitHub: `examples/ci/github-pr-comment.yml` and
//! `examples/harness/open-pr.sh` show the shell around it.

// Rust guideline compliant 2026-02-21

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Context;
use clap::Args;
use serde::Serialize;
use surveyor_extract::{Base, Profile, Repository};
use surveyor_graph::{Changed, CodeGraph, EdgeFilter, Impact, ImpactGroup, Reach};
use surveyor_model::{ChangeSet, CommitId, NodeKind, RepoId, SourceUnit};
use surveyor_rules::{Report, RuleSet, Severity};
use surveyor_session::Summary;

use crate::app::{Cli, Paths};
use crate::render::{EdgeView, NodeView, Output, cite, print_json};
use crate::{app, harness};

/// The first line of the Markdown, so a CI step can find and replace its comment.
pub const MARKER: &str = "<!-- surveyor-pr -->";

/// Exit when every `fail` rule passed, or the gate is off.
pub const EXIT_PASS: i32 = 0;
/// Exit when a rule failed and the gate is on.
pub const EXIT_FAIL: i32 = 1;
/// Exit when the report could not be made: no dataset, a rule that does not load, an
/// unknown base, an unreadable diff.
pub const EXIT_ERROR: i32 = 2;

#[derive(Debug, Args)]
pub struct PrArgs {
    /// Measure against this ref: the merge base of HEAD and it. The default is the
    /// merge base of HEAD and the default branch (origin/HEAD, then origin/main,
    /// origin/master, main, master).
    #[arg(long, conflicts_with = "diff")]
    pub base: Option<String>,
    /// Read the change from a unified diff instead of git (`-` for stdin).
    #[arg(long, value_name = "FILE")]
    pub diff: Option<PathBuf>,
    /// Hops to follow backwards from each changed node.
    #[arg(long, default_value_t = 3)]
    pub depth: usize,
    /// Drop impact paths whose confidence product falls below this.
    #[arg(long, default_value_t = Reach::DEFAULT_MIN_CONFIDENCE)]
    pub min_confidence: f32,
    /// Rows per group in the Markdown; the JSON holds every row.
    #[arg(long, default_value_t = 40)]
    pub rows: usize,
    /// Extra rule files or directories, besides the profile's `[rules] dirs`.
    #[arg(long = "rules", value_name = "PATH")]
    pub rules: Vec<PathBuf>,
    /// Write `pr.md` and `pr.json` into this directory instead of printing.
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
    /// Exit 0 whatever the rules say, for a job that only comments.
    #[arg(long)]
    pub no_gate: bool,
}

/// The whole report, as the JSON carries it and the Markdown renders it.
#[derive(Debug, Serialize)]
pub struct PrReport {
    pub surveyor: String,
    pub repo: String,
    pub dataset: DatasetView,
    pub change: ChangeView,
    pub files: Vec<FileView>,
    pub removed: RemovedView,
    pub impact: ImpactView,
    pub rules: Option<RulesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Summary>,
    pub gate: GateView,
    pub took_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct DatasetView {
    pub commit: String,
    pub dirty: bool,
    pub nodes: usize,
    pub edges: usize,
    /// Why the dataset may not match the tree, when it may not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangeView {
    /// Where the change came from: `git` or `diff`.
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// How the base was chosen, or which diff file was read.
    pub measured_against: String,
    pub head: String,
    pub files_changed: usize,
    pub symbols_changed: usize,
    pub nodes_changed: Vec<NodeView>,
    pub deleted: Vec<String>,
    pub unindexed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FileView {
    pub path: String,
    pub added: bool,
    pub hunks: Vec<String>,
    pub symbols: Vec<NodeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_node: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RemovedView {
    pub listed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub symbols: Vec<RemovedSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RemovedSymbol {
    pub file: String,
    pub kind: String,
    pub qualified_name: String,
    /// Lines at the base commit.
    pub lines: String,
}

#[derive(Debug, Serialize)]
pub struct ImpactView {
    pub depth: usize,
    pub min_confidence: f32,
    pub total: usize,
    pub groups: Vec<GroupView>,
    /// Groups no node of the dataset falls in: kinds the graph cannot see.
    pub absent: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupView {
    pub group: ImpactGroup,
    pub label: String,
    pub rows: Vec<RowView>,
}

#[derive(Debug, Serialize)]
pub struct RowView {
    pub node: NodeView,
    pub depth: usize,
    pub confidence: f32,
    pub via: EdgeView,
    pub citation: String,
    pub from: String,
}

#[derive(Debug, Serialize)]
pub struct RulesView {
    pub sources: Vec<String>,
    pub report: Report,
}

#[derive(Debug, Serialize)]
pub struct GateView {
    pub enforced: bool,
    pub passed: bool,
    pub exit: i32,
}

/// Runs the command; exits 2 itself when the report cannot be made.
pub async fn run(cli: &Cli, p: &Paths, a: &PrArgs, out: Output) -> anyhow::Result<()> {
    let report = match build(cli, p, a).await {
        Ok(report) => report,
        Err(e) => {
            eprintln!("surveyor pr: {e:#}");
            std::process::exit(EXIT_ERROR);
        }
    };
    let md = markdown(&report, a.rows);
    match &a.out {
        Some(dir) => {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
            std::fs::write(dir.join("pr.md"), &md).context("write pr.md")?;
            let json = serde_json::to_string_pretty(&report).context("serialize report")?;
            std::fs::write(dir.join("pr.json"), json).context("write pr.json")?;
            eprintln!(
                "surveyor pr: {} files, {} changed symbols, {} affected, {}; wrote {} and {}",
                report.change.files_changed,
                report.change.symbols_changed,
                report.impact.total,
                rules_phrase(&report),
                dir.join("pr.md").display(),
                dir.join("pr.json").display()
            );
        }
        None if out == Output::Json => print_json(&report),
        None => print!("{md}"),
    }
    if report.gate.exit != EXIT_PASS {
        std::process::exit(report.gate.exit);
    }
    Ok(())
}

/// Builds the report: the change, its nodes, their impact, the rules, the session.
async fn build(cli: &Cli, p: &Paths, a: &PrArgs) -> anyhow::Result<PrReport> {
    let started = Instant::now();
    let profile = app::load_profile(cli, p)?;
    let repo = Repository::open_with(&p.root, cli.repo_id.as_deref().map(RepoId::new), &profile)
        .context("open repository")?;
    let (store, graph, repo_id) = app::open_graph(cli, p).await?;
    let snapshot = store
        .snapshot(&repo_id)
        .await?
        .with_context(|| format!("repository {repo_id} is not in the dataset"))?;
    let input = change_input(a, &repo)?;

    // Changed nodes and their impact, rendered before the graph moves into the rule run.
    let changed: Changed<'_> = surveyor_graph::changed(&graph, &input.set);
    let nodes = changed.nodes();
    let reach = Reach::depth(a.depth).with_min_confidence(a.min_confidence);
    let hits = surveyor_graph::impact(&graph, &nodes, reach, &EdgeFilter::impact());
    let impact = impact_view(&hits, a.depth, a.min_confidence, absent_groups(&graph));
    let change = ChangeView {
        source: input.source.to_owned(),
        base: input.base.as_ref().map(|b| b.commit.to_string()),
        measured_against: input.measured_against.clone(),
        head: repo.commit().to_string(),
        files_changed: input.set.file_count(),
        symbols_changed: changed.symbol_count(),
        nodes_changed: nodes.iter().map(|n| NodeView::of(n)).collect(),
        deleted: changed.deleted.iter().map(ToString::to_string).collect(),
        unindexed: changed.unindexed().map(ToString::to_string).collect(),
    };
    let files: Vec<FileView> = changed
        .files
        .iter()
        .map(|f| FileView {
            path: f.path.to_string(),
            added: f.added,
            hunks: f.hunks.iter().map(ToString::to_string).collect(),
            symbols: f.symbols.iter().map(|n| NodeView::of(n)).collect(),
            file_node: f.file.map(|n| n.id.to_string()),
        })
        .collect();
    let (node_count, edge_count) = (graph.node_count(), graph.edge_count());
    let removed = removed_view(&repo, &profile, input.base.as_ref(), &input.set);
    let rules = rules_view(p, &profile, a, graph)?;
    // The session, when one is named and has a ledger.
    let session = harness::ledger(cli, &p.store)?
        .filter(surveyor_session::Ledger::exists)
        .map(|l| l.summary())
        .transpose()
        .context("summarize session")?;
    let passed = rules.as_ref().is_none_or(|r| r.report.passed());
    let exit = if passed || a.no_gate {
        EXIT_PASS
    } else {
        EXIT_FAIL
    };
    Ok(PrReport {
        surveyor: env!("CARGO_PKG_VERSION").to_owned(),
        repo: repo_id.to_string(),
        dataset: DatasetView {
            commit: snapshot.commit.to_string(),
            dirty: snapshot.dirty,
            nodes: node_count,
            edges: edge_count,
            stale: stale_note(&snapshot.commit, snapshot.dirty, &repo),
        },
        change,
        files,
        removed,
        impact,
        rules,
        session,
        gate: GateView {
            enforced: !a.no_gate,
            passed,
            exit,
        },
        took_ms: started.elapsed().as_millis() as u64,
    })
}

/// The change to measure and where it came from.
struct ChangeInput {
    set: ChangeSet,
    base: Option<Base>,
    measured_against: String,
    source: &'static str,
}

/// Reads the change from `--diff`, else diffs the base against the working tree.
fn change_input(a: &PrArgs, repo: &Repository) -> anyhow::Result<ChangeInput> {
    if let Some(file) = &a.diff {
        let text = if file.as_os_str() == "-" {
            std::io::read_to_string(std::io::stdin()).context("read diff from stdin")?
        } else {
            std::fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?
        };
        return Ok(ChangeInput {
            set: ChangeSet::parse_unified(&text),
            base: None,
            measured_against: format!("diff `{}`", file.display()),
            source: "diff",
        });
    }
    let base = repo
        .resolve_base(a.base.as_deref())
        .context("resolve base")?;
    let set = repo
        .changes_against(&base.commit)
        .context("diff against base")?;
    Ok(ChangeInput {
        set,
        measured_against: if base.commit.as_str().starts_with(&base.spec) {
            format!("`{}`", short(base.commit.as_str()))
        } else {
            format!("{} (`{}`)", base.describe(), short(base.commit.as_str()))
        },
        base: Some(base),
        source: "git",
    })
}

/// Removed symbols from the base versions, or the reason they are not listed.
fn removed_view(
    repo: &Repository,
    profile: &Profile,
    base: Option<&Base>,
    set: &ChangeSet,
) -> RemovedView {
    let Some(base) = base else {
        return RemovedView {
            listed: false,
            reason: Some(
                "a diff file names no base commit to read the old declarations from".to_owned(),
            ),
            symbols: Vec::new(),
        };
    };
    match removed_symbols(repo, profile, &base.commit, set) {
        Ok(symbols) => RemovedView {
            listed: true,
            reason: None,
            symbols,
        },
        Err(e) => RemovedView {
            listed: false,
            reason: Some(format!("the base versions could not be read: {e:#}")),
            symbols: Vec::new(),
        },
    }
}

/// The rule run, as `check` performs it; `None` when no rule is configured.
fn rules_view(
    p: &Paths,
    profile: &Profile,
    a: &PrArgs,
    graph: CodeGraph,
) -> anyhow::Result<Option<RulesView>> {
    let (dirs, files) = app::rule_sources(&p.root, profile, &a.rules);
    let rules = RuleSet::load(&dirs, &files).map_err(|e| anyhow::anyhow!("{e}"))?;
    if rules.rules().is_empty() {
        return Ok(None);
    }
    let report = rules
        .run(std::sync::Arc::new(graph))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(Some(RulesView {
        sources: dirs
            .iter()
            .chain(files.iter())
            .map(|d| d.display().to_string())
            .collect(),
        report,
    }))
}

/// Why the dataset may not match the tree, when commit or dirtiness differ.
fn stale_note(dataset_commit: &CommitId, dataset_dirty: bool, repo: &Repository) -> Option<String> {
    let dirty = |d: bool| if d { " with uncommitted changes" } else { "" };
    (dataset_commit != repo.commit() || dataset_dirty != repo.is_dirty()).then(|| {
        format!(
            "the dataset is at {}{}; HEAD is {}{}. Run `surveyor update` first.",
            short(dataset_commit.as_str()),
            dirty(dataset_dirty),
            short(repo.commit().as_str()),
            dirty(repo.is_dirty())
        )
    })
}

/// Declarations present in a file at `base` and absent from it now, and every
/// declaration of a deleted file, by running the name-tier extractors on both
/// versions and comparing kind and scoped name.
fn removed_symbols(
    repo: &Repository,
    profile: &Profile,
    base: &CommitId,
    set: &ChangeSet,
) -> anyhow::Result<Vec<RemovedSymbol>> {
    let extractors = surveyor_extract::default_extractors(profile).context("load extractors")?;
    let decls = |unit: &SourceUnit| -> BTreeMap<(String, String), String> {
        extractors
            .run(unit)
            .facts
            .iter()
            .flat_map(surveyor_model::FactSet::declarations)
            .map(|d| {
                (
                    (d.kind.to_string(), d.qualified_name()),
                    d.span.lines.to_string(),
                )
            })
            .collect()
    };
    let mut out: BTreeSet<RemovedSymbol> = BTreeSet::new();
    for f in set.files.iter().filter(|f| !f.added) {
        let Some(before) = repo.unit_at(base, &f.path)? else {
            continue;
        };
        let now = repo.unit(&f.path)?.map(|u| decls(&u)).unwrap_or_default();
        for ((kind, qualified_name), lines) in decls(&before) {
            if !now.contains_key(&(kind.clone(), qualified_name.clone())) {
                out.insert(RemovedSymbol {
                    file: f.path.to_string(),
                    kind,
                    qualified_name,
                    lines,
                });
            }
        }
    }
    for path in &set.deleted {
        if let Some(before) = repo.unit_at(base, path)? {
            for ((kind, qualified_name), lines) in decls(&before) {
                out.insert(RemovedSymbol {
                    file: path.to_string(),
                    kind,
                    qualified_name,
                    lines,
                });
            }
        }
    }
    Ok(out.into_iter().collect())
}

fn impact_view(
    hits: &[Impact<'_>],
    depth: usize,
    min_confidence: f32,
    absent: Vec<String>,
) -> ImpactView {
    let mut by_group: BTreeMap<ImpactGroup, Vec<RowView>> = BTreeMap::new();
    for h in hits {
        by_group.entry(h.group()).or_default().push(RowView {
            node: NodeView::of(h.node),
            depth: h.depth,
            confidence: h.confidence,
            via: EdgeView::of(h.via),
            citation: cite(h.via),
            from: h.from.id.to_string(),
        });
    }
    ImpactView {
        depth,
        min_confidence,
        total: hits.len(),
        groups: ImpactGroup::ALL
            .into_iter()
            .filter_map(|g| {
                by_group.remove(&g).map(|rows| GroupView {
                    group: g,
                    label: g.label().to_owned(),
                    rows,
                })
            })
            .collect(),
        absent,
    }
}

/// The groups no node of `graph` falls in, as labels; `Other` is never reported.
fn absent_groups(graph: &CodeGraph) -> Vec<String> {
    ImpactGroup::absent_from(graph)
        .into_iter()
        .filter(|g| *g != ImpactGroup::Other)
        .map(|g| g.label().to_owned())
        .collect()
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(12)]
}

fn rules_phrase(r: &PrReport) -> String {
    match &r.rules {
        Some(rules) => {
            let failed = rules.report.outcomes.iter().filter(|o| !o.passed).count();
            format!(
                "{} rules, {} violations, {failed} failed",
                rules.report.outcomes.len(),
                rules.report.violation_count()
            )
        }
        None => "no rules".to_owned(),
    }
}

/// Escapes a cell for a Markdown table.
fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// A cell in code font; a backtick inside the text would end the span, so it goes.
fn code(s: &str) -> String {
    format!("`{}`", cell(s).replace('`', ""))
}

/// The comment. Every row carries a `file:lines` citation; the first line is
/// [`MARKER`].
#[expect(clippy::too_many_lines, reason = "one section per part of the report")]
pub fn markdown(r: &PrReport, rows: usize) -> String {
    // Writing to a String cannot fail, so the results are dropped.
    let mut s = String::new();
    let _ = writeln!(s, "{MARKER}");
    let _ = writeln!(s, "### Surveyor: what this change touches\n");
    let gate = match (r.gate.enforced, r.gate.passed, &r.rules) {
        (_, _, None) => "no rules to gate on".to_owned(),
        (true, true, _) => "**gate passes**".to_owned(),
        (true, false, _) => format!("**gate fails** (exit {})", r.gate.exit),
        (false, true, _) => "gate passes (not enforced)".to_owned(),
        (false, false, _) => "**rules fail**, gate not enforced (`--no-gate`)".to_owned(),
    };
    let _ = writeln!(
        s,
        "Measured against {}, dataset at `{}`{}: **{} files, {} changed symbols, {} affected nodes within {} hops, {}**; {gate}.\n",
        r.change.measured_against,
        short(&r.dataset.commit),
        if r.dataset.dirty {
            " with uncommitted changes"
        } else {
            ""
        },
        r.change.files_changed,
        r.change.symbols_changed,
        r.impact.total,
        r.impact.depth,
        rules_phrase(r),
    );
    if let Some(stale) = &r.dataset.stale {
        let _ = writeln!(s, "> **Dataset may be stale:** {stale}\n");
    }

    // Changed symbols.
    let symbols: Vec<&NodeView> = r
        .change
        .nodes_changed
        .iter()
        .filter(|n| n.kind != NodeKind::File.to_string())
        .collect();
    let _ = writeln!(s, "**Changed symbols ({})**\n", symbols.len());
    if symbols.is_empty() {
        let _ = writeln!(s, "No indexed declaration lies on a changed line.\n");
    } else {
        let _ = writeln!(s, "| Symbol | Kind | Where |\n|---|---|---|");
        for n in symbols.iter().take(rows) {
            let _ = writeln!(
                s,
                "| {} | {} | {} |",
                code(&n.qualified_name),
                n.kind,
                code(&where_of(n))
            );
        }
        if symbols.len() > rows {
            let _ = writeln!(s, "\n… and {} more in the JSON.", symbols.len() - rows);
        }
        let _ = writeln!(s);
    }
    let file_only: Vec<&FileView> = r.files.iter().filter(|f| f.symbols.is_empty()).collect();
    let mut notes: Vec<String> = Vec::new();
    let as_files: Vec<String> = file_only
        .iter()
        .filter(|f| f.file_node.is_some())
        .map(|f| code(&f.path))
        .collect();
    if !as_files.is_empty() {
        notes.push(format!(
            "Changed as whole files (no declaration at the changed lines): {}.",
            as_files.join(", ")
        ));
    }
    if !r.change.unindexed.is_empty() {
        notes.push(format!(
            "Not in the dataset: {}.",
            r.change
                .unindexed
                .iter()
                .map(|p| code(p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !r.change.deleted.is_empty() {
        notes.push(format!(
            "Deleted: {}.",
            r.change
                .deleted
                .iter()
                .map(|p| code(p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    match (&r.removed.listed, r.removed.symbols.is_empty()) {
        (true, false) => notes.push(format!(
            "Removed symbols: {}.",
            r.removed
                .symbols
                .iter()
                .map(|x| format!(
                    "{} ({}, `{}:{}` at base)",
                    code(&x.qualified_name),
                    x.kind,
                    x.file,
                    x.lines
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        (true, true) => {}
        (false, _) => notes.push(format!(
            "Removed symbols are not listed: {}.",
            r.removed.reason.as_deref().unwrap_or("unknown")
        )),
    }
    for n in notes {
        let _ = writeln!(s, "{n}");
    }
    let _ = writeln!(s);

    // Affected.
    let _ = writeln!(
        s,
        "**Affected ({} nodes within {} hops; confidence is the product along the path, floor {:.2})**\n",
        r.impact.total, r.impact.depth, r.impact.min_confidence
    );
    if r.impact.groups.is_empty() {
        let _ = writeln!(s, "Nothing in the graph depends on the changed nodes.\n");
    }
    for g in &r.impact.groups {
        let _ = writeln!(s, "_{} ({})_\n", g.label, g.rows.len());
        let _ = writeln!(
            s,
            "| Node | Kind | Where | Depth | Confidence | Via | Evidence |\n|---|---|---|---|---|---|---|"
        );
        for row in g.rows.iter().take(rows) {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {:.2} | {} | {} |",
                code(&row.node.qualified_name),
                row.node.kind,
                code(&where_of(&row.node)),
                row.depth,
                row.confidence,
                row.via.kind,
                code(&row.citation)
            );
        }
        if g.rows.len() > rows {
            let _ = writeln!(s, "\n… and {} more in the JSON.", g.rows.len() - rows);
        }
        let _ = writeln!(s);
    }
    if !r.impact.absent.is_empty() {
        let _ = writeln!(
            s,
            "The dataset holds no nodes of these kinds, so they cannot appear: {}.\n",
            r.impact.absent.join(", ").to_lowercase()
        );
    }

    // Invariants.
    match &r.rules {
        None => {
            let _ = writeln!(
                s,
                "**Invariants**\n\nNo rules were found (profile `[rules] dirs` or `--rules`).\n"
            );
        }
        Some(rules) => {
            let _ = writeln!(s, "**Invariants ({})**\n", rules_phrase(r));
            let _ = writeln!(s, "| Rule | Verdict | Findings |\n|---|---|---|");
            for o in &rules.report.outcomes {
                let verdict = if o.passed { "ok" } else { "FAIL" };
                let judged = match (o.severity, o.budget) {
                    (Severity::Budget, Some(b)) => format!("{} of {b} allowed", o.violations.len()),
                    (Severity::Warn, _) => format!("{} (warning)", o.violations.len()),
                    _ => o.violations.len().to_string(),
                };
                let _ = writeln!(s, "| {} | {verdict} | {judged} |", code(&o.id));
            }
            let _ = writeln!(s);
            for o in rules
                .report
                .outcomes
                .iter()
                .filter(|o| !o.violations.is_empty())
            {
                let _ = writeln!(s, "`{}`: {}", o.id, cell(&o.desc));
                for v in o.violations.iter().take(rows) {
                    let at = v.citation.as_deref().map(code).unwrap_or_default();
                    let _ = writeln!(s, "- {at} {} ({})", cell(&v.message), code(&v.subject));
                }
                if o.violations.len() > rows {
                    let _ = writeln!(s, "- … and {} more in the JSON.", o.violations.len() - rows);
                }
                let _ = writeln!(s);
            }
        }
    }

    // Session.
    if let Some(summary) = &r.session {
        let body = summary.markdown();
        let body = body
            .strip_prefix("## ")
            .map_or(body.clone(), |rest| format!("**{rest}"));
        let body = body.replacen('\n', "**\n", 1);
        let _ = writeln!(s, "{body}");
    }

    let _ = writeln!(
        s,
        "<sub>surveyor {} · repository `{}` · {} nodes, {} edges · {} ms</sub>",
        r.surveyor, r.repo, r.dataset.nodes, r.dataset.edges, r.took_ms
    );
    s
}

fn where_of(n: &NodeView) -> String {
    match (&n.file, &n.lines) {
        (Some(f), Some(l)) => format!("{f}:{l}"),
        (Some(f), None) => f.clone(),
        _ => String::new(),
    }
}
