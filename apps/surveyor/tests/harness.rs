//! The session ledger and the Claude Code hook adapter on the `mini` fixture: a
//! session replayed from recorded hook payloads leaves a ledger whose records name
//! the nodes the answers named, the first prompt gets a pack and the second does
//! not, `--session` records CLI answers, and the summary cites what was consulted.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

fn surveyor(store: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(store)
        .args(args)
        .output()
        .expect("runs")
}

/// Runs `surveyor hook claude-code` with `payload` on stdin, with no `--repo-root`
/// so the adapter takes the root from the payload's `cwd`, as Claude Code's hook does.
fn hook(store: &Path, payload: &Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--store")
        .arg(store)
        .args(["hook", "claude-code"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload.to_string().as_bytes())
        .expect("writes payload");
    child.wait_with_output().expect("hook finishes")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn records(store: &Path, id: &str) -> Vec<Value> {
    let path = store.join("sessions").join(format!("{id}.jsonl"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("ledger {}: {e}", path.display()))
        .lines()
        .map(|l| serde_json::from_str(l).expect("record is json"))
        .collect()
}

const SESSION: &str = "0f6b6a2e-3c1d-4e5f-9a8b-7c6d5e4f3a2b";
const PROMPT: &str = "The outbox relay drains pending rows one at a time; make Relay::drain take a batch size and send rows in batches.";

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one session, replayed event by event"
)]
fn a_claude_code_session_replayed_through_the_adapter_leaves_a_cited_ledger() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    let cwd = fixture();
    let base = |event: &str| {
        json!({
            "session_id": SESSION,
            "cwd": cwd,
            "transcript_path": "/dev/null",
            "hook_event_name": event,
        })
    };

    // SessionStart indexes the fixture (no dataset yet) and prints one line of orientation.
    let mut start = base("SessionStart");
    start["source"] = json!("startup");
    let out = hook(&store, &start);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let orientation = text(&out.stdout);
    assert_eq!(orientation.lines().count(), 1, "{orientation}");
    assert!(
        orientation.starts_with("Surveyor: mini at "),
        "{orientation}"
    );
    assert!(orientation.contains("1 rules"), "{orientation}");
    assert!(orientation.contains(SESSION), "{orientation}");
    let started = records(&store, SESSION);
    assert_eq!(started.len(), 1);
    assert_eq!(started[0]["kind"], "started");
    assert_eq!(started[0]["repo"], "mini");

    // The first prompt gets a pack inside the profile's budget; a second prompt gets nothing.
    let mut prompt = base("UserPromptSubmit");
    prompt["prompt"] = json!(PROMPT);
    let out = hook(&store, &prompt);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let pack_text = text(&out.stdout);
    assert!(
        pack_text.starts_with("Surveyor context pack for this prompt ("),
        "{pack_text}"
    );
    assert!(pack_text.contains("Relay::drain"), "{pack_text}");
    let packed = records(&store, SESSION);
    assert_eq!(packed.len(), 2);
    assert_eq!(packed[1]["kind"], "pack");
    assert_eq!(packed[1]["budget"], 12_000);
    assert_eq!(
        packed[1]["seeds"][0],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    let consulted = packed[1]["consulted"].as_array().expect("consulted");
    assert!(consulted.iter().any(|c| {
        c["id"] == "rust:crates/outbox/src/relay.rs::Relay::drain"
            && c["citation"]
                .as_str()
                .is_some_and(|s| s.starts_with("crates/outbox/src/relay.rs:"))
    }));
    prompt["prompt"] = json!("thanks, now also add a test");
    let out = hook(&store, &prompt);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty(), "{}", text(&out.stdout));
    assert_eq!(records(&store, SESSION).len(), 2);

    // A Surveyor tool call is recorded from its response; another server's is not.
    let node_view = json!({
        "node": {"id": "rust:crates/outbox/src/relay.rs::Relay::drain", "kind": "Method",
                 "name": "drain", "qualified_name": "Relay::drain",
                 "file": "crates/outbox/src/relay.rs", "lines": "20-40"},
        "card": {"id": "rust:crates/outbox/src/relay.rs::Relay::drain", "kind": "Method"}
    });
    let mut used = base("PostToolUse");
    used["tool_name"] = json!("mcp__surveyor__get_node");
    used["tool_input"] = json!({"node_id": "Relay::drain"});
    used["tool_response"] = json!([{"type": "text", "text": node_view.to_string()}]);
    let out = hook(&store, &used);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let mut other = base("PostToolUse");
    other["tool_name"] = json!("mcp__github__get_pull_request");
    other["tool_response"] = json!({"id": "x", "kind": "pr"});
    assert_eq!(hook(&store, &other).status.code(), Some(0));
    let after_tool = records(&store, SESSION);
    assert_eq!(after_tool.len(), 3, "{after_tool:?}");
    assert_eq!(after_tool[2]["kind"], "answer");
    assert_eq!(after_tool[2]["source"], "hook");
    assert_eq!(after_tool[2]["op"], "get_node");
    assert_eq!(after_tool[2]["args"]["node_id"], "Relay::drain");
    assert_eq!(
        after_tool[2]["consulted"][0]["citation"],
        "crates/outbox/src/relay.rs:20-40"
    );

    // A CLI command under --session records what it answered.
    let out = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(&cwd)
        .arg("--store")
        .arg(&store)
        .env("SURVEYOR_SESSION", SESSION)
        .args(["affected", "cw_outbox.outbox", "--depth", "2"])
        .output()
        .expect("runs");
    assert!(out.status.success(), "{}", text(&out.stderr));
    let after_cli = records(&store, SESSION);
    assert_eq!(after_cli.len(), 4);
    assert_eq!(after_cli[3]["source"], "cli");
    assert_eq!(after_cli[3]["op"], "affected");
    assert!(
        after_cli[3]["consulted"]
            .as_array()
            .expect("consulted")
            .len()
            > 1
    );

    // Stop writes the summary beside the ledger; it names the pack and the consulted set.
    let out = hook(&store, &base("Stop"));
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let summary_md =
        std::fs::read_to_string(store.join("sessions").join(format!("{SESSION}.summary.md")))
            .expect("summary written");
    assert!(summary_md.contains(&format!("## Surveyor session {SESSION}")));
    assert!(summary_md.contains("Context pack: "), "{summary_md}");
    assert!(summary_md.contains("| `rust:crates/outbox/src/relay.rs::Relay::drain` |"));
    let summary_json: Value = serde_json::from_str(
        &std::fs::read_to_string(
            store
                .join("sessions")
                .join(format!("{SESSION}.summary.json")),
        )
        .expect("json summary"),
    )
    .expect("json");
    assert_eq!(summary_json["records"], 4);
    assert_eq!(summary_json["answers_by_source"]["hook"], 1);
    assert_eq!(summary_json["answers_by_source"]["cli"], 1);

    // `session summary` prints the same, and `--json` is the same structure.
    let out = surveyor(&store, &["--session", SESSION, "session", "summary"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), summary_md);
}

#[test]
fn a_hook_with_a_bad_payload_still_exits_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    let out = hook(&store, &json!({"hook_event_name": "Stop"}));
    assert_eq!(out.status.code(), Some(0));
    assert!(text(&out.stderr).contains("surveyor hook:"));
    // A session id that is not a plain file name is refused, still at exit 0.
    let out = hook(
        &store,
        &json!({"session_id": "../escape", "hook_event_name": "Stop", "cwd": fixture()}),
    );
    assert_eq!(out.status.code(), Some(0));
    assert!(
        text(&out.stderr).contains("not a plain file name"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.path().join("escape.jsonl").exists());
}

#[test]
fn session_start_plans_the_pack_and_note_records_outside_reading() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    let out = surveyor(
        &store,
        &[
            "--json", "session", "start", "--id", "t-start", "--task", PROMPT, "--budget", "6000",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let view: Value = serde_json::from_str(&text(&out.stdout)).expect("json");
    assert_eq!(view["id"], "t-start");
    assert_eq!(view["repo"], "mini");
    assert_eq!(view["rules"], 1);
    assert_eq!(view["pack"]["budget"], 6000);
    let used = view["pack"]["used"]["tokens"].as_u64().expect("tokens");
    assert!(used > 0 && used <= 6000, "{used}");

    let out = surveyor(
        &store,
        &[
            "--session",
            "t-start",
            "session",
            "note",
            "Relay::drain",
            "crates/api/src/main.rs:1-5",
            "a note",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let recs = records(&store, "t-start");
    assert_eq!(recs.len(), 3, "{recs:?}");
    assert_eq!(recs[2]["kind"], "note");
    let noted = recs[2]["consulted"].as_array().expect("consulted");
    assert_eq!(
        noted[0]["id"],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    assert_eq!(noted[1]["id"], "crates/api/src/main.rs:1-5");
    assert_eq!(noted[1]["citation"], "crates/api/src/main.rs:1-5");
    assert_eq!(noted[2]["id"], "a note");
    assert!(noted[2].get("citation").is_none());

    // Text mode prints the id first, for a harness script to capture.
    let out = surveyor(
        &store,
        &["session", "start", "--id", "t-text", "--no-update"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("SURVEYOR_SESSION=t-text\n"));

    // An invalid session id is refused before anything is opened.
    let out = surveyor(&store, &["--session", "no/slash", "query", "drain"]);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("not a plain file name"));
}
