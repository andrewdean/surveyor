//! The invariant gate on the `mini` fixture: a clean tree passes `surveyor check`
//! with exit 0, and a planted violation of the fixture's own rule fails it with exit
//! 1 and a citation of the planted line, through `update` as a CI step would run it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

/// Copies the fixture into `into`, so the planted edit never touches the checkout.
fn copy_fixture(into: &Path) {
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("mkdir");
        for entry in std::fs::read_dir(from).expect("readdir") {
            let entry = entry.expect("entry");
            let target = to.join(entry.file_name());
            if entry.file_type().expect("type").is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).expect("copy");
            }
        }
    }
    copy_dir(&fixture(), into);
}

fn surveyor(root: &Path, store: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(root)
        .arg("--store")
        .arg(store)
        .args(args)
        .output()
        .expect("runs")
}

#[test]
fn a_planted_violation_fails_the_gate_and_the_clean_tree_passes_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_fixture(&root);
    let store = dir.path().join("ds");

    let indexed = surveyor(&root, &store, &["index"]);
    assert!(
        indexed.status.success(),
        "{}",
        String::from_utf8_lossy(&indexed.stderr)
    );

    // Clean: the fixture's rule (only crates/outbox writes the outbox table) passes.
    let clean = surveyor(&root, &store, &["check"]);
    assert_eq!(
        clean.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&clean.stdout)
    );
    let text = String::from_utf8_lossy(&clean.stdout);
    assert!(text.contains("1 rules, 0 violations, 0 failed"), "{text}");

    // Plant: the api crate writes the outbox table directly.
    let main_rs = root.join("crates/api/src/main.rs");
    let mut source = std::fs::read_to_string(&main_rs).expect("main.rs");
    source.push_str(
        "\nfn purge_sent_rows() {\n    let _ = sqlx::query(\"DELETE FROM cw_outbox.outbox WHERE sent\");\n}\n",
    );
    std::fs::write(&main_rs, &source).expect("write");
    let planted_line = source.lines().count() - 1;

    let updated = surveyor(&root, &store, &["--json", "update"]);
    assert!(
        updated.status.success(),
        "{}",
        String::from_utf8_lossy(&updated.stderr)
    );
    let updated: Value = serde_json::from_slice(&updated.stdout).expect("json");
    assert!(
        updated["modified"]
            .as_array()
            .expect("modified")
            .iter()
            .any(|f| f == "crates/api/src/main.rs"),
        "{updated}"
    );

    // The write is recorded twice, from the function and from its query literal
    // (the same citation); both findings name the planted line.
    let planted = surveyor(&root, &store, &["check"]);
    let text = String::from_utf8_lossy(&planted.stdout);
    assert_eq!(planted.status.code(), Some(1), "{text}");
    assert!(
        text.contains("FAIL [outbox-written-only-by-outbox]"),
        "{text}"
    );
    assert!(text.contains("1 rules, 2 violations, 1 failed"), "{text}");

    // The same in JSON, as `check_invariants` returns it.
    let json = surveyor(&root, &store, &["--json", "check"]);
    assert_eq!(json.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&json.stdout).expect("json report");
    let outcome = &report["outcomes"][0];
    assert_eq!(outcome["passed"], false);
    let violations = outcome["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 2, "{outcome}");
    for v in violations {
        assert_eq!(
            v["citation"],
            format!("crates/api/src/main.rs:{planted_line}"),
            "the finding cites the planted line: {v}"
        );
        assert!(
            v["subject"]
                .as_str()
                .unwrap()
                .starts_with("rust:crates/api/src/main.rs::purge_sent_rows"),
            "{v}"
        );
    }

    // Removing the plant restores the gate.
    std::fs::copy(fixture().join("crates/api/src/main.rs"), &main_rs).expect("restore");
    assert!(surveyor(&root, &store, &["update"]).status.success());
    assert_eq!(surveyor(&root, &store, &["check"]).status.code(), Some(0));
}
