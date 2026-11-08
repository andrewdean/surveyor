//! The agent harness: session ledgers and the Claude Code hook adapter.
//!
//! `surveyor session start|note|summary` works the ledger directly, for a harness
//! script or an agent host without hooks. `surveyor hook claude-code` reads the
//! payload Claude Code writes to a hook's stdin and does the session's bookkeeping
//! for it: `update` and one line of orientation at `SessionStart`, a context pack
//! for the first prompt at `UserPromptSubmit`, a ledger record for every Surveyor
//! tool call at `PostToolUse`, and the summary at `Stop`. The adapter is the one
//! place that knows Claude Code's field names; a hook exits 0 whatever happens,
//! because the ledger must never block an agent.

// Rust guideline compliant 2026-02-21

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surveyor_session::{Consulted, Event, Ledger, SessionId, Source};

use crate::app::{Cli, Paths};
use crate::render::{self, Output};

/// Prefix of a Surveyor tool name as Claude Code reports it for the `surveyor`
/// MCP server; the tool's own name follows.
const CLAUDE_CODE_TOOL_PREFIX: &str = "mcp__surveyor__";

#[derive(Debug, Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommand,
}

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// Open a session: update the dataset, plan the pack for a task and record it.
    Start(StartArgs),
    /// Record nodes or `file:lines` the agent read outside Surveyor.
    Note(NoteArgs),
    /// The consulted set of a session, as Markdown or JSON.
    Summary(SummaryArgs),
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// The task, as the agent received it; a pack is planned for it.
    #[arg(long)]
    pub task: Option<String>,
    /// Session identifier; defaults to `--session`, then to a generated one.
    #[arg(long)]
    pub id: Option<String>,
    /// Skip `surveyor update` before opening the session.
    #[arg(long)]
    pub no_update: bool,
    /// Token budget for the pack; defaults to `surveyor context`'s.
    #[arg(long, default_value_t = surveyor_context::DEFAULT_BUDGET)]
    pub budget: usize,
}

#[derive(Debug, Args)]
pub struct NoteArgs {
    /// Node identifiers, names or `path:line[-line]` citations.
    #[arg(required = true)]
    pub items: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SummaryArgs {
    /// Also write `<id>.summary.md` and `<id>.summary.json` beside the ledger.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub host: HookHost,
}

#[derive(Debug, Subcommand)]
pub enum HookHost {
    /// Read a Claude Code hook payload on stdin and act on its event.
    ClaudeCode,
}

/// The ledger `--session` names, or none.
///
/// # Errors
/// Returns an error when the identifier is not a plain file name.
pub fn ledger(cli: &Cli, store: &Path) -> anyhow::Result<Option<Ledger>> {
    cli.session
        .as_deref()
        .map(|raw| {
            SessionId::parse(raw)
                .map(|id| Ledger::in_store(store, id))
                .map_err(Into::into)
        })
        .transpose()
}

/// Appends `event` when there is a ledger; a ledger that cannot be written is
/// logged and the command's answer stands.
pub fn record(ledger: Option<&Ledger>, event: Event) {
    if let Some(ledger) = ledger
        && let Err(e) = ledger.append(event)
    {
        tracing::warn!(error = %e, "session ledger not written");
    }
}

/// A command's answer, for the ledger.
pub fn answer(op: &str, args: Value, consulted: Vec<Consulted>) -> Event {
    Event::Answer {
        source: Source::Cli,
        op: op.to_owned(),
        args,
        consulted,
    }
}

/// A node and the sampled relations its card cites.
pub fn consulted_card(node: &surveyor_model::Node, card: &surveyor_graph::Card) -> Vec<Consulted> {
    std::iter::once(Consulted::of_node(node))
        .chain(card.counts.iter().flat_map(|count| {
            count
                .samples
                .iter()
                .map(|s| Consulted::new(s.node.clone(), Some(s.citation.clone())))
        }))
        .collect()
}

/// What `session start` and the `SessionStart` hook report about the dataset.
#[derive(Debug, Serialize)]
struct Opened {
    id: SessionId,
    ledger: PathBuf,
    repo: String,
    commit: String,
    nodes: usize,
    edges: usize,
    rules: usize,
}

impl Opened {
    /// One line for an agent's context.
    fn orientation(&self) -> String {
        format!(
            "Surveyor: {} at {}, {} nodes, {} edges, {} rules; session {}. Use the surveyor tools (query_graph, context_pack, affected, check_invariants) and verify citations with read_project_source; `surveyor --session {} …` records what you consult.",
            self.repo,
            short(&self.commit),
            self.nodes,
            self.edges,
            self.rules,
            self.id,
            self.id
        )
    }
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

/// Updates the dataset unless told not to, opens the ledger and records the start.
async fn open_session(
    cli: &Cli,
    p: &Paths,
    id: SessionId,
    task: Option<&str>,
    update: bool,
) -> anyhow::Result<Opened> {
    if update {
        crate::app::update_quietly(cli, p).await?;
    }
    let (store, graph, repo) = crate::app::open_graph(cli, p).await?;
    let commit = store
        .snapshot(&repo)
        .await?
        .map(|s| s.commit.to_string())
        .unwrap_or_default();
    let rules = crate::app::rule_count(cli, p);
    let ledger = Ledger::in_store(&p.store, id.clone());
    ledger.append(Event::Started {
        task: task.map(str::to_owned),
        repo: repo.to_string(),
        commit: commit.clone(),
    })?;
    Ok(Opened {
        id,
        ledger: ledger.path().to_path_buf(),
        repo: repo.to_string(),
        commit,
        nodes: graph.node_count(),
        edges: graph.edge_count(),
        rules,
    })
}

/// Runs a `surveyor session` subcommand.
pub async fn session(cli: &Cli, p: &Paths, args: &SessionArgs, out: &Output) -> anyhow::Result<()> {
    match &args.command {
        SessionCommand::Start(a) => {
            let id = match a.id.as_deref().or(cli.session.as_deref()) {
                Some(raw) => SessionId::parse(raw)?,
                None => SessionId::generate(),
            };
            let opened = open_session(cli, p, id, a.task.as_deref(), !a.no_update).await?;
            let pack = match &a.task {
                Some(task) => {
                    let pack = crate::app::pack_for(cli, p, task, a.budget, &[]).await?;
                    Ledger::in_store(&p.store, opened.id.clone()).append(Event::pack_of(&pack))?;
                    Some(pack)
                }
                None => None,
            };
            match out {
                Output::Json => {
                    #[derive(Serialize)]
                    struct View<'a> {
                        #[serde(flatten)]
                        opened: &'a Opened,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        pack: Option<&'a surveyor_context::ContextPack>,
                    }
                    render::print_json(&View {
                        opened: &opened,
                        pack: pack.as_ref(),
                    });
                }
                Output::Text => {
                    println!("SURVEYOR_SESSION={}", opened.id);
                    println!("{}", opened.orientation());
                    if let Some(pack) = pack {
                        println!();
                        print!("{}", pack.markdown);
                    }
                }
            }
            Ok(())
        }
        SessionCommand::Note(a) => {
            let Some(ledger) = ledger(cli, &p.store)? else {
                bail!("session note needs --session <id> or SURVEYOR_SESSION");
            };
            let (_, graph, _) = crate::app::open_graph(cli, p).await?;
            let consulted = a
                .items
                .iter()
                .map(|item| match graph.lookup(item, None) {
                    surveyor_graph::Lookup::Exact(n) | surveyor_graph::Lookup::Unique(n) => {
                        Consulted::of_node(n)
                    }
                    _ => Consulted::new(item.clone(), is_citation(item).then(|| item.clone())),
                })
                .collect();
            ledger.append(Event::Note { consulted })?;
            Ok(())
        }
        SessionCommand::Summary(a) => {
            let Some(ledger) = ledger(cli, &p.store)? else {
                bail!("session summary needs --session <id> or SURVEYOR_SESSION");
            };
            if !ledger.exists() {
                bail!("no ledger at {}", ledger.path().display());
            }
            let summary = ledger.summary()?;
            if a.write {
                write_summary(&ledger, &summary)?;
            }
            match out {
                Output::Json => render::print_json(&summary),
                Output::Text => print!("{}", summary.markdown()),
            }
            Ok(())
        }
    }
}

/// `path:line` or `path:start-end`.
fn is_citation(item: &str) -> bool {
    item.rsplit_once(':').is_some_and(|(path, range)| {
        !path.is_empty()
            && range.split_once('-').map_or_else(
                || range.parse::<u32>().is_ok(),
                |(a, b)| a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok(),
            )
    })
}

fn write_summary(ledger: &Ledger, summary: &surveyor_session::Summary) -> anyhow::Result<()> {
    std::fs::write(ledger.summary_path("md"), summary.markdown())
        .with_context(|| format!("write {}", ledger.summary_path("md").display()))?;
    std::fs::write(
        ledger.summary_path("json"),
        serde_json::to_vec_pretty(summary).context("serialize summary")?,
    )
    .with_context(|| format!("write {}", ledger.summary_path("json").display()))?;
    Ok(())
}

/// The fields of a Claude Code hook payload the adapter reads.
///
/// Every event carries `session_id`, `cwd` and `hook_event_name`; the rest depends
/// on the event (code.claude.com/docs/en/hooks, read 2026-11-06).
#[derive(Debug, Deserialize)]
struct ClaudeCodePayload {
    session_id: String,
    #[serde(default)]
    cwd: Option<PathBuf>,
    hook_event_name: String,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: Option<Value>,
    #[serde(default)]
    tool_response: Option<Value>,
}

/// Runs a hook: reads the payload, acts, and exits 0 whatever happened.
///
/// Errors are printed to stderr, which Claude Code shows in its transcript view and
/// never to the model; stdout is what the model sees on `SessionStart` and
/// `UserPromptSubmit`.
pub async fn hook(cli: &Cli, args: &HookArgs) {
    let outcome = match args.host {
        HookHost::ClaudeCode => claude_code(cli).await,
    };
    if let Err(e) = outcome {
        eprintln!("surveyor hook: {e:#}");
    }
}

async fn claude_code(cli: &Cli) -> anyhow::Result<()> {
    let mut raw = String::new();
    std::io::stdin()
        .read_to_string(&mut raw)
        .context("read hook payload")?;
    let payload: ClaudeCodePayload =
        serde_json::from_str(&raw).context("parse Claude Code hook payload")?;
    let p = crate::app::paths_from(cli, payload.cwd.as_deref())?;
    let id = SessionId::parse(&payload.session_id)?;
    let ledger = Ledger::in_store(&p.store, id.clone());
    match payload.hook_event_name.as_str() {
        "SessionStart" => {
            let opened = open_session(cli, &p, id, None, true).await?;
            println!("{}", opened.orientation());
        }
        "UserPromptSubmit" => {
            let prompt = payload.prompt.unwrap_or_default();
            let packed = ledger
                .read()?
                .iter()
                .any(|r| matches!(r.event, Event::Pack { .. }));
            if packed || prompt.trim().is_empty() {
                return Ok(());
            }
            let budget = crate::app::load_profile(cli, &p).map_or_else(
                |_| surveyor_extract::HarnessProfile::default().prompt_budget,
                |profile| profile.harness.prompt_budget,
            );
            let pack = crate::app::pack_for(cli, &p, &prompt, budget, &[]).await?;
            if pack.seeds.is_empty() {
                return Ok(());
            }
            ledger.append(Event::pack_of(&pack))?;
            println!(
                "Surveyor context pack for this prompt ({} of {} tokens; every statement cites file:lines, verify with read_project_source before relying on one):\n",
                pack.used.tokens, pack.budget
            );
            print!("{}", pack.markdown);
        }
        "PostToolUse" => {
            let Some(op) = payload
                .tool_name
                .as_deref()
                .and_then(|name| name.strip_prefix(CLAUDE_CODE_TOOL_PREFIX))
            else {
                return Ok(());
            };
            let consulted = payload
                .tool_response
                .as_ref()
                .map(consulted_in_response)
                .unwrap_or_default();
            ledger.append(Event::Answer {
                source: Source::Hook,
                op: op.to_owned(),
                args: payload.tool_input.unwrap_or(Value::Null),
                consulted,
            })?;
        }
        "Stop" | "SessionEnd" if ledger.exists() => {
            write_summary(&ledger, &ledger.summary()?)?;
        }
        _ => {}
    }
    Ok(())
}

/// Node references in a tool response as Claude Code relays it: the value itself,
/// and every `text` string in it that parses as JSON, since an MCP result arrives
/// as content blocks whose text is the server's JSON.
fn consulted_in_response(response: &Value) -> Vec<Consulted> {
    let mut found = surveyor_session::consulted_in(response);
    let mut texts = Vec::new();
    texts_in(response, &mut texts);
    for text in texts {
        if let Ok(parsed) = serde_json::from_str::<Value>(text) {
            for c in surveyor_session::consulted_in(&parsed) {
                if !found.iter().any(|f| f.id == c.id) {
                    found.push(c);
                }
            }
        }
    }
    found
}

fn texts_in<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get("text") {
                out.push(text);
            }
            map.values().for_each(|v| texts_in(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| texts_in(v, out)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn citations_are_recognized() {
        assert!(is_citation("src/lib.rs:10"));
        assert!(is_citation("src/lib.rs:10-20"));
        assert!(!is_citation("rust:src/lib.rs::f"));
        assert!(!is_citation("src/lib.rs"));
    }

    #[test]
    fn tool_responses_are_read_as_content_blocks_or_plain_json() {
        let inner = json!({"node": {"id": "rust:a.rs::f", "kind": "Function", "file": "a.rs", "lines": "1-2"}});
        let blocks = json!([{"type": "text", "text": inner.to_string()}]);
        let found = consulted_in_response(&blocks);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "rust:a.rs::f");
        assert_eq!(found[0].citation.as_deref(), Some("a.rs:1-2"));
        let plain = consulted_in_response(&inner);
        assert_eq!(plain, found);
    }
}
