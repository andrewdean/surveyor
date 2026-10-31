//! End-to-end: `surveyor index --scip` replaces name-matched edges with resolved ones
//! where the index covers a site and leaves the syntactic graph elsewhere.
//!
//! The index is built here rather than by `rust-analyzer scip`, so the test needs no
//! toolchain; it covers two sites of the `mini` fixture.

use std::path::{Path, PathBuf};
use std::process::Command;

use scip::types::{Document, Index, Metadata, Occurrence, ToolInfo};
use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

fn surveyor(store: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(store)
        .arg("--json")
        .args(args)
        .output()
        .expect("runs");
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(serde_json::from_slice(&output.stdout).expect("json"))
}

fn occurrence(symbol: &str, range: &[i32], roles: i32) -> Occurrence {
    Occurrence {
        range: range.to_vec(),
        symbol: symbol.to_owned(),
        symbol_roles: roles,
        ..Default::default()
    }
}

/// An index that knows `Relay::publish`, `Relay::drain` (the impl's method) and the
/// two call sites that name them: one inside `relay.rs`, one in the API's `main`.
fn write_index(path: &Path) {
    let publish = "rust-analyzer cargo outbox 0.1.0 relay/impl#[Relay]publish().";
    let drain = "rust-analyzer cargo outbox 0.1.0 relay/impl#[Relay][Drain]drain().";
    let index = Index {
        metadata: Some(Metadata {
            tool_info: Some(ToolInfo {
                name: "rust-analyzer".into(),
                ..Default::default()
            })
            .into(),
            project_root: "file:///mini".into(),
            ..Default::default()
        })
        .into(),
        documents: vec![
            Document {
                relative_path: "crates/outbox/src/relay.rs".into(),
                occurrences: vec![
                    occurrence(publish, &[21, 7, 14], 1),
                    occurrence(drain, &[25, 7, 12], 1),
                    occurrence(publish, &[28, 17, 24], 0),
                ],
                ..Default::default()
            },
            Document {
                relative_path: "crates/api/src/main.rs".into(),
                occurrences: vec![occurrence(drain, &[5, 21, 26], 0)],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    scip::write_message_to_file(path, index).expect("writes index");
}

fn calls<'a>(explain: &'a Value, name: &str) -> Vec<&'a Value> {
    explain["outgoing"]
        .as_array()
        .expect("array")
        .iter()
        .filter(|r| r["kind"] == "CALLS" && r["node"]["name"] == name)
        .collect()
}

#[test]
fn scip_index_replaces_name_matches_where_it_covers_them() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("plain");
    let precise = dir.path().join("precise");
    let index_path = dir.path().join("index.scip");
    write_index(&index_path);

    let before = surveyor(&plain, &["index"]).expect("plain index");
    let after = surveyor(&precise, &["index", "--scip", index_path.to_str().unwrap()])
        .expect("precise index");
    assert_eq!(before["nodes"], after["nodes"], "the index adds no nodes");
    assert_eq!(
        after["edges_by_derivation"]["resolved"].as_u64().unwrap(),
        before["edges_by_derivation"]["resolved"].as_u64().unwrap() + 2,
        "{after}"
    );

    // Without the index, `relay.drain(100)` in main matches the trait method and the
    // impl method at 0.5 each; with it, one certain edge to the impl's method.
    let main_before = surveyor(&plain, &["explain", "crates/api/src/main.rs::main"]).unwrap();
    let drains = calls(&main_before, "drain");
    assert_eq!(drains.len(), 2, "{main_before}");
    assert!(drains.iter().all(|d| d["derivation"] == "syntactic"));
    let main_after = surveyor(&precise, &["explain", "crates/api/src/main.rs::main"]).unwrap();
    let drains = calls(&main_after, "drain");
    assert_eq!(drains.len(), 1, "{main_after}");
    assert_eq!(drains[0]["derivation"], "resolved");
    assert_eq!(drains[0]["confidence"], 1.0);
    assert_eq!(
        drains[0]["node"]["id"],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    assert_eq!(drains[0]["evidence"][0], "crates/api/src/main.rs:6");

    // Inside relay.rs the covered call is resolved and the uncovered ones stand.
    let drain = surveyor(&precise, &["explain", "Relay::drain"]).unwrap();
    assert_eq!(calls(&drain, "publish")[0]["derivation"], "resolved");
    assert_eq!(calls(&drain, "pending")[0]["derivation"], "syntactic");
    assert_eq!(calls(&drain, "mark_sent")[0]["derivation"], "syntactic");

    // A missing index is an error, not a silent downgrade.
    let err = surveyor(&precise, &["index", "--scip", "/nonexistent/index.scip"]).unwrap_err();
    assert!(err.contains("does not exist"), "{err}");
}
