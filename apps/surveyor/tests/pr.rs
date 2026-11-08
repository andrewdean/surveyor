//! `surveyor pr` on the `mini` fixture: a diff maps to the declarations its lines fall
//! in, a planted violation makes the gate exit 1 with the planted line cited, the
//! gate can be switched off for a comment-only job, removed symbols are named from
//! the base version, and an unknown base exits 2.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

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

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
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

fn text(o: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn ok(o: &Output) {
    assert!(o.status.success(), "{}", text(o));
}

const PLANT: &str = "\nfn purge_sent_rows() {\n    let _ = sqlx::query(\"DELETE FROM cw_outbox.outbox WHERE sent\");\n}\n";

#[test]
#[expect(clippy::too_many_lines, reason = "one fixture, every exit code")]
fn a_change_maps_to_symbols_and_the_gate_honors_the_rules() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_fixture(&root);
    let store = dir.path().join("ds");
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "fixture"]);
    ok(&surveyor(&root, &store, &["index"]));

    // 1. A unified diff maps its lines onto the innermost declaration: line 27 of
    //    relay.rs is inside `Drain::drain` for `Relay`; the gate passes on the clean tree.
    let diff = dir.path().join("change.diff");
    std::fs::write(
        &diff,
        "--- a/crates/outbox/src/relay.rs\n+++ b/crates/outbox/src/relay.rs\n@@ -27,1 +27,2 @@\n         let rows = self.store.pending(limit);\n+        // batch\n",
    )
    .expect("write diff");
    let from_diff = surveyor(&root, &store, &["pr", "--diff", diff.to_str().unwrap()]);
    let md = String::from_utf8_lossy(&from_diff.stdout).to_string();
    assert_eq!(from_diff.status.code(), Some(0), "{}", text(&from_diff));
    assert!(md.starts_with("<!-- surveyor-pr -->\n"), "{md}");
    assert!(md.contains("**gate passes**"), "{md}");
    assert!(md.contains("Changed symbols (1)"), "{md}");
    assert!(md.contains("`crates/outbox/src/relay.rs:"), "{md}");
    assert!(
        md.contains("Removed symbols are not listed: a diff file names no base commit"),
        "{md}"
    );
    let json = surveyor(
        &root,
        &store,
        &["--json", "pr", "--diff", diff.to_str().unwrap()],
    );
    let report: Value = serde_json::from_slice(&json.stdout).expect("json report");
    assert_eq!(report["change"]["source"], "diff");
    assert_eq!(report["change"]["symbols_changed"], 1);
    let changed = &report["change"]["nodes_changed"][0];
    assert_eq!(changed["name"], "drain", "{changed}");
    assert!(
        changed["id"]
            .as_str()
            .unwrap()
            .starts_with("rust:crates/outbox/src/relay.rs::"),
        "{changed}"
    );
    assert_eq!(report["files"][0]["hunks"][0], "27-28");
    // `main` in the api crate calls the relay's drain: it is affected at depth 1 with
    // the call site cited.
    let rows: Vec<&Value> = report["impact"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["rows"].as_array().unwrap())
        .collect();
    assert!(
        rows.iter().any(|r| r["node"]["name"] == "main"
            && r["depth"] == 1
            && r["citation"]
                .as_str()
                .unwrap()
                .starts_with("crates/api/src/main.rs:")),
        "{}",
        report["impact"]
    );
    assert_eq!(report["gate"]["exit"], 0);
    assert_eq!(report["rules"]["report"]["outcomes"][0]["passed"], true);

    // 2. An uncommitted plant: the api crate writes the outbox table. The working tree
    //    counts, so `--base HEAD` sees it once `update` has relinked the file.
    let main_rs = root.join("crates/api/src/main.rs");
    let original = std::fs::read_to_string(&main_rs).expect("main.rs");
    std::fs::write(&main_rs, format!("{original}{PLANT}")).expect("write");
    let planted_line = format!("{original}{PLANT}").lines().count() - 1;
    ok(&surveyor(&root, &store, &["update"]));

    let gated = surveyor(&root, &store, &["pr", "--base", "HEAD"]);
    let md = String::from_utf8_lossy(&gated.stdout).to_string();
    assert_eq!(gated.status.code(), Some(1), "{}", text(&gated));
    assert!(md.contains("**gate fails** (exit 1)"), "{md}");
    assert!(md.contains("| `purge_sent_rows` | Function |"), "{md}");
    assert!(
        md.contains("| `outbox-written-only-by-outbox` | FAIL | 2 |"),
        "{md}"
    );
    assert!(
        md.contains(&format!(
            "- `crates/api/src/main.rs:{planted_line}` writes the outbox table"
        )),
        "{md}"
    );
    assert!(md.contains("Measured against `HEAD` (`"), "{md}");
    assert!(!md.contains("may be stale"), "{md}");

    // The same with the gate off: exit 0, the failure still in the text.
    let ungated = surveyor(&root, &store, &["pr", "--base", "HEAD", "--no-gate"]);
    assert_eq!(ungated.status.code(), Some(0), "{}", text(&ungated));
    assert!(
        String::from_utf8_lossy(&ungated.stdout).contains("gate not enforced (`--no-gate`)"),
        "{}",
        text(&ungated)
    );

    // `--out` writes both files and prints nothing on stdout; the exit code stands.
    let out = dir.path().join("out");
    let written = surveyor(
        &root,
        &store,
        &["pr", "--base", "HEAD", "--out", out.to_str().unwrap()],
    );
    assert_eq!(written.status.code(), Some(1), "{}", text(&written));
    assert!(written.stdout.is_empty());
    let file_md = std::fs::read_to_string(out.join("pr.md")).expect("pr.md");
    assert!(file_md.starts_with("<!-- surveyor-pr -->\n"));
    let file_json: Value =
        serde_json::from_slice(&std::fs::read(out.join("pr.json")).expect("pr.json")).unwrap();
    assert_eq!(file_json["gate"]["exit"], 1);
    assert_eq!(file_json["gate"]["enforced"], true);
    assert_eq!(file_json["change"]["source"], "git");
    assert_eq!(file_json["files"][0]["path"], "crates/api/src/main.rs");

    // 3. Committed, measured against the parent: the same change, and the base is the
    //    ref itself because it is an ancestor.
    git(&root, &["commit", "-q", "-am", "plant"]);
    ok(&surveyor(&root, &store, &["update"]));
    let parent = surveyor(&root, &store, &["--json", "pr", "--base", "HEAD~1"]);
    assert_eq!(parent.status.code(), Some(1), "{}", text(&parent));
    let report: Value = serde_json::from_slice(&parent.stdout).expect("json");
    // The function and the query literal inside it: each changed line maps to the
    // innermost declaration around it, and the literal is a node of its own.
    let kinds: Vec<(String, String)> = report["change"]["nodes_changed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["kind"].as_str().unwrap().to_owned(),
                n["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(report["change"]["symbols_changed"], 2, "{kinds:?}");
    assert!(
        kinds.contains(&("Function".to_owned(), "purge_sent_rows".to_owned())),
        "{kinds:?}"
    );
    assert!(kinds.iter().any(|(k, _)| k == "Query"), "{kinds:?}");
    assert!(
        report["change"]["measured_against"]
            .as_str()
            .unwrap()
            .starts_with("`HEAD~1` (`"),
        "{}",
        report["change"]
    );
    assert_eq!(report["removed"]["listed"], true);
    assert_eq!(report["removed"]["symbols"].as_array().unwrap().len(), 0);

    // 4. Removing a function names it from the base version of the file.
    let without_backlog = original.replace(
        "fn report_backlog() {\n    let _ = sqlx::query_as::<_, (i64,)>(\"SELECT count(*) FROM cw_outbox.pending_outbox\");\n}\n",
        "",
    );
    assert_ne!(
        without_backlog, original,
        "the fixture still has report_backlog"
    );
    std::fs::write(&main_rs, &without_backlog).expect("write");
    ok(&surveyor(&root, &store, &["update"]));
    let removed = surveyor(&root, &store, &["--json", "pr", "--base", "HEAD"]);
    let report: Value = serde_json::from_slice(&removed.stdout).expect("json");
    let names: Vec<&str> = report["removed"]["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["qualified_name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"report_backlog"), "{}", report["removed"]);
    assert!(names.contains(&"purge_sent_rows"), "{}", report["removed"]);
    assert_eq!(
        report["gate"]["exit"], 0,
        "the plant is gone: {}",
        report["rules"]
    );
    let md = String::from_utf8_lossy(&surveyor(&root, &store, &["pr", "--base", "HEAD"]).stdout)
        .to_string();
    assert!(md.contains("Removed symbols: "), "{md}");
    assert!(
        md.contains("`report_backlog` (Function, `crates/api/src/main.rs:"),
        "{md}"
    );

    // 5. A base that does not resolve, or a diff that cannot be read, is exit 2.
    let unknown = surveyor(&root, &store, &["pr", "--base", "no-such-ref"]);
    assert_eq!(unknown.status.code(), Some(2), "{}", text(&unknown));
    assert!(text(&unknown).contains("no-such-ref"));
    let missing = surveyor(&root, &store, &["pr", "--diff", "/nonexistent.diff"]);
    assert_eq!(missing.status.code(), Some(2), "{}", text(&missing));
    let no_dataset = surveyor(
        &root,
        &dir.path().join("nowhere"),
        &["pr", "--base", "HEAD"],
    );
    assert_eq!(no_dataset.status.code(), Some(2), "{}", text(&no_dataset));
}

#[test]
fn a_stale_dataset_is_flagged_and_a_session_summary_is_embedded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_fixture(&root);
    let store = dir.path().join("ds");
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "fixture"]);
    ok(&surveyor(&root, &store, &["index"]));

    // A session that consulted one node.
    let noted = surveyor(
        &root,
        &store,
        &[
            "--session",
            "pr-test",
            "explain",
            "rust:crates/outbox/src/relay.rs::Relay",
        ],
    );
    ok(&noted);

    // An edit without `update`: the dataset no longer matches the tree.
    std::fs::write(
        root.join("crates/api/src/main.rs"),
        format!(
            "{}{PLANT}",
            std::fs::read_to_string(root.join("crates/api/src/main.rs")).unwrap()
        ),
    )
    .expect("write");
    let o = surveyor(
        &root,
        &store,
        &["--session", "pr-test", "pr", "--base", "HEAD"],
    );
    let md = String::from_utf8_lossy(&o.stdout).to_string();
    // The dataset predates the plant, so the rule still passes; the report says why
    // it may be wrong.
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(md.contains("**Dataset may be stale:**"), "{md}");
    assert!(
        md.contains("with uncommitted changes. Run `surveyor update` first."),
        "{md}"
    );
    assert!(md.contains("**Surveyor session pr-test**"), "{md}");
    assert!(
        md.contains("rust:crates/outbox/src/relay.rs::Relay"),
        "{md}"
    );
    let json = surveyor(
        &root,
        &store,
        &["--json", "--session", "pr-test", "pr", "--base", "HEAD"],
    );
    let report: Value = serde_json::from_slice(&json.stdout).expect("json");
    assert_eq!(report["session"]["id"], "pr-test");
    assert!(
        report["dataset"]["stale"].is_string(),
        "{}",
        report["dataset"]
    );
}
