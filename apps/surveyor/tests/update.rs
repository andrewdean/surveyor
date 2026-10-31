//! End-to-end: `surveyor update` after an edit produces the graph a fresh `index` of
//! the edited tree produces, by hashing (no git) and by git diff (clean commits).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use surveyor_model::{Edge, Node, RepoId};
use surveyor_store::LanceStore;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

/// Copies the fixture into `into`, so the test can edit it.
fn copy_fixture(into: &Path) {
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("mkdir");
        for entry in std::fs::read_dir(from).expect("read dir") {
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

fn surveyor(root: &Path, store: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(root)
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
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "json for {args:?}: {e}\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
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

/// The comparable shape of a stored graph: node identities, edge triples with their
/// provenance and citation, and unresolved gaps.
async fn shape(store: &Path) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let store = LanceStore::open(store).await.expect("open store");
    let repo = RepoId::new("mini");
    let (nodes, edges) = store.load_graph(Some(&repo)).await.expect("load");
    let node_shape = |n: &Node| {
        format!(
            "{} {} {} {}",
            n.id,
            n.kind,
            n.content_hash.short(),
            n.location
                .as_ref()
                .map(surveyor_model::Location::citation)
                .unwrap_or_default()
        )
    };
    let edge_shape = |e: &Edge| {
        format!(
            "{} -{}-> {} {} {:.2} {} against {:?}",
            e.src,
            e.kind,
            e.dst,
            e.provenance.derivation.kind(),
            e.provenance.confidence.get(),
            e.provenance.evidence[0].citation(),
            e.resolved_against
        )
    };
    let unresolved = store
        .unresolved(&repo)
        .await
        .expect("unresolved")
        .iter()
        .map(|u| format!("{} {} {}", u.name, u.site.citation(), u.reason))
        .collect();
    (
        nodes.iter().map(node_shape).collect(),
        edges.iter().map(edge_shape).collect(),
        unresolved,
    )
}

/// Edits the copy: a new method on the store, a call to it from the relay, a new
/// file in the API crate, and a deleted shell helper.
fn edit(root: &Path) {
    let store_rs = root.join("crates/outbox/src/store.rs");
    let text = std::fs::read_to_string(&store_rs).expect("read");
    std::fs::write(
        &store_rs,
        text.replace(
            "    /// Marks rows as sent.",
            "    /// Counts pending rows.\n    pub fn audit(&self) -> usize { self.pending(1).len() }\n\n    /// Marks rows as sent.",
        ),
    )
    .expect("write");
    let relay_rs = root.join("crates/outbox/src/relay.rs");
    let text = std::fs::read_to_string(&relay_rs).expect("read");
    std::fs::write(
        &relay_rs,
        text.replace(
            "let rows = self.store.pending(limit);",
            "let rows = self.store.pending(limit);\n        let _ = self.store.audit();",
        ),
    )
    .expect("write");
    std::fs::write(
        root.join("crates/api/src/report.rs"),
        "use outbox::Relay;\n\npub fn describe(relay: &Relay) -> String { format!(\"{relay:?}\") }\n",
    )
    .expect("write");
    std::fs::remove_file(root.join("scripts/lib.sh")).expect("remove");
}

async fn assert_update_matches_index(root: &Path, store: &Path, fresh: &Path, strategy: &str) {
    let updated = surveyor(root, store, &["update"]);
    assert_eq!(updated["strategy"], strategy, "{updated}");
    assert_eq!(
        updated["modified"].as_array().unwrap().len(),
        2,
        "{updated}"
    );
    assert_eq!(
        updated["added"],
        serde_json::json!(["crates/api/src/report.rs"])
    );
    assert_eq!(updated["deleted"], serde_json::json!(["scripts/lib.sh"]));
    let relinked = updated["relinked"].as_object().unwrap();
    assert!(
        relinked.contains_key("crates/api/src/main.rs"),
        "main.rs calls drain, whose declaration changed: {relinked:?}"
    );
    assert!(
        relinked.contains_key("scripts/drain.sh"),
        "drain.sh sourced the deleted helper: {relinked:?}"
    );

    let indexed = surveyor(root, fresh, &["index"]);
    assert_eq!(updated["nodes"], indexed["nodes"], "{updated}\n{indexed}");
    assert_eq!(updated["edges"], indexed["edges"]);
    assert_eq!(updated["unresolved"], indexed["unresolved"]);
    assert_eq!(updated["files"], indexed["files"]);

    let (want_n, want_e, want_u) = shape(fresh).await;
    let (got_n, got_e, got_u) = shape(store).await;
    assert_eq!(got_n, want_n);
    assert_eq!(got_e, want_e);
    assert_eq!(got_u, want_u);

    // The dataset stays queryable and answers from the new graph.
    let audit = surveyor(root, store, &["explain", "OutboxStore::audit"]);
    assert_eq!(audit["node"]["kind"], "Method");
    let stats = surveyor(root, store, &["stats"]);
    assert_eq!(stats["nodes"], indexed["nodes"]);
}

#[tokio::test]
async fn update_by_hashing_matches_a_fresh_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_fixture(&root);
    let store = dir.path().join("store");
    let fresh = dir.path().join("fresh");

    let first = surveyor(&root, &store, &["update"]);
    assert_eq!(first["strategy"], "full", "{first}");
    assert_eq!(first["reason"], "no dataset");
    let noop = surveyor(&root, &store, &["update"]);
    assert_eq!(noop["strategy"], "hashes");
    assert_eq!(noop["relinked"].as_object().unwrap().len(), 0);

    edit(&root);
    assert_update_matches_index(&root, &store, &fresh, "hashes").await;
}

#[tokio::test]
async fn update_by_git_diff_matches_a_fresh_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_fixture(&root);
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "base"]);
    let store = dir.path().join("store");
    let fresh = dir.path().join("fresh");

    let indexed = surveyor(&root, &store, &["index"]);
    assert_ne!(indexed["commit"], "worktree");

    edit(&root);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "edit"]);
    assert_update_matches_index(&root, &store, &fresh, "git").await;
    let stats = surveyor(&root, &store, &["stats"]);
    assert_ne!(stats["repositories"][0]["commit"], indexed["commit"]);

    // A manifest change falls back to a full index.
    std::fs::write(
        root.join("crates/api/Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.2.0\"\n\n[dependencies]\noutbox = { path = \"../outbox\" }\naxum = \"0.7\"\n",
    )
    .expect("write");
    let full = surveyor(&root, &store, &["update"]);
    assert_eq!(full["strategy"], "full", "{full}");
}
