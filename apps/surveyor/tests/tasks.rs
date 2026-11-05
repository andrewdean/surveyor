//! Tasks and the commands that run them: the edges from a task to its script, from a
//! script to the task it runs and to the binary it starts.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

fn surveyor(store: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(store)
        .arg("--json")
        .args(args)
        .output()
        .expect("runs");
    assert!(
        output.status.success(),
        "surveyor {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
}

fn relation<'a>(explain: &'a Value, direction: &str, kind: &str, id: &str) -> &'a Value {
    explain[direction]
        .as_array()
        .expect("relations")
        .iter()
        .find(|r| r["kind"] == kind && r["node"]["id"] == id)
        .unwrap_or_else(|| panic!("no {direction} {kind} to {id} in {explain}"))
}

#[test]
fn a_task_runs_its_script_which_runs_a_task_and_a_binary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    surveyor(&store, &["index"]);

    let task = surveyor(&store, &["explain", "task:mise/bootstrap"]);
    assert_eq!(task["node"]["kind"], "Task");
    let script = relation(&task, "outgoing", "RUNS", "file:scripts/bootstrap.rhai");
    assert_eq!(script["derivation"], "declared");
    assert_eq!(script["evidence"][0], "mise.toml:15");

    let script = surveyor(&store, &["explain", "file:scripts/bootstrap.rhai"]);
    let migrate = relation(&script, "outgoing", "RUNS", "task:mise/db:migrate");
    assert_eq!(migrate["derivation"], "syntactic");
    assert_eq!(migrate["confidence"], 1.0);
    assert_eq!(migrate["evidence"][0], "scripts/bootstrap.rhai:2");

    let seed = surveyor(&store, &["explain", "rhai:scripts/bootstrap.rhai::seed"]);
    let binary = relation(
        &seed,
        "outgoing",
        "RUNS",
        "rust:crates/api/src/main.rs::main",
    );
    assert_eq!(binary["derivation"], "syntactic");
    assert_eq!(binary["confidence"], 0.75);
    assert_eq!(binary["evidence"][0], "scripts/bootstrap.rhai:5");

    // The dependency edges between tasks, declared by `depends`.
    let ship = surveyor(&store, &["explain", "task:mise/ship"]);
    relation(&ship, "outgoing", "DEPENDS_ON", "task:mise/db:migrate");
    relation(&ship, "outgoing", "DEPENDS_ON", "task:mise/deploy");
}
