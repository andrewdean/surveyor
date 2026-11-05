//! `surveyor eval`: the retrieval eval runs against a store and reports its scores.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

fn surveyor(store: &Path, args: &[&str]) -> (bool, Value, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(store)
        .arg("--json")
        .args(args)
        .output()
        .expect("runs");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let json = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), json, stderr)
}

#[test]
fn eval_scores_questions_and_reports_unknown_identifiers() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    let (ok, _, err) = surveyor(&store, &["index"]);
    assert!(ok, "{err}");

    let questions = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/mini-eval.toml");
    let (ok, summary, err) = surveyor(&store, &["eval", questions.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert_eq!(summary["questions"], 2, "{summary}");
    assert_eq!(summary["k"], 5);
    assert_eq!(summary["unknown_ids"], 1, "{summary}");
    let scored = summary["scored"].as_array().expect("scored");
    assert_eq!(
        scored[0]["unknown"],
        serde_json::json!(["rust:crates/outbox/src/relay.rs::Nope"])
    );
    for s in scored {
        let recall = s["recall"].as_f64().expect("recall");
        assert!((0.0..=1.0).contains(&recall), "{s}");
        assert!(s["returned"].as_array().expect("returned").len() <= 5);
    }
    // `backlog` names the function; the baseline matcher and every later ranker find it.
    assert_eq!(scored[1]["hit"], true, "{}", scored[1]);
    assert!(summary["mrr"].as_f64().expect("mrr") > 0.0);

    let (ok, _, err) = surveyor(&store, &["eval", "/nonexistent/questions.toml"]);
    assert!(!ok);
    assert!(err.contains("read questions"), "{err}");
}
