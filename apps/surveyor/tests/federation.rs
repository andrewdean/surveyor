//! Federation on the `mini` fixture: a second repository whose manifest depends on
//! `@mini/web` shares a store with it. The federate pass pairs the consumer's stub with
//! the declared package; `explain` from either side shows the other repository with a
//! citation into its manifest; a third repository declaring the same coordinate keeps
//! the stub and splits the confidence; a store with one repository never gets the
//! `federated` table.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
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

fn json(root: &Path, store: &Path, args: &[&str]) -> Value {
    let o = surveyor(root, store, &[&["--json"], args].concat());
    assert!(
        o.status.success(),
        "{args:?}: {}\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| {
        panic!(
            "json for {args:?}: {e}\n{}",
            String::from_utf8_lossy(&o.stdout)
        )
    })
}

fn text(root: &Path, store: &Path, args: &[&str]) -> String {
    let o = surveyor(root, store, args);
    assert!(
        o.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// A repository of one manifest that depends on `@mini/web`.
fn consumer(dir: &Path, name: &str) -> PathBuf {
    let root = dir.join(name);
    std::fs::create_dir_all(root.join("src")).expect("mkdir");
    std::fs::write(
        root.join("package.json"),
        format!(
            "{{\n  \"name\": \"{name}\",\n  \"version\": \"0.1.0\",\n  \"dependencies\": {{\n    \"@mini/web\": \"file:../mini/web\",\n    \"react\": \"^18.0.0\"\n  }}\n}}\n"
        ),
    )
    .expect("package.json");
    std::fs::write(
        root.join("src/index.ts"),
        "import { drain } from \"@mini/web\";\nexport const run = () => drain();\n",
    )
    .expect("index.ts");
    std::fs::write(
        root.join("surveyor.toml"),
        format!("[repository]\nid = \"{name}\"\n"),
    )
    .expect("profile");
    root
}

/// A repository that declares `@mini/web` too.
fn declarer(dir: &Path, name: &str) -> PathBuf {
    let root = dir.join(name);
    std::fs::create_dir_all(root.join("web")).expect("mkdir");
    std::fs::write(
        root.join("web/package.json"),
        "{\n  \"name\": \"@mini/web\",\n  \"version\": \"2.0.0\"\n}\n",
    )
    .expect("package.json");
    std::fs::write(
        root.join("surveyor.toml"),
        format!("[repository]\nid = \"{name}\"\n"),
    )
    .expect("profile");
    root
}

#[test]
#[expect(clippy::too_many_lines, reason = "one store, every federation path")]
fn two_repositories_share_a_federated_edge_and_a_third_splits_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mini = fixture();
    let store = dir.path().join("ds");
    let consumer = consumer(dir.path(), "consumer");

    // One repository: no federated table is created or read.
    let o = surveyor(&mini, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!store.join("federated.lance").exists());
    let stats = json(&mini, &store, &["stats"]);
    assert_eq!(stats["federated"], 0);

    // Two: indexing the consumer runs the pass and writes one row.
    let o = surveyor(&consumer, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(store.join("federated.lance").exists());
    let stats = json(&mini, &store, &["--repo-id", "mini", "stats"]);
    assert_eq!(stats["federated"], 1, "{stats}");

    // From the declaring side: the consumer appears under its repository with the
    // citation into its manifest.
    let explain = json(
        &mini,
        &store,
        &["--repo-id", "mini", "explain", "pkg:npm/@mini/web"],
    );
    assert_eq!(explain["node"]["repo"], "mini");
    let incoming = explain["incoming"].as_array().unwrap();
    let from_consumer = incoming
        .iter()
        .find(|r| r["node"]["repo"] == "consumer")
        .unwrap_or_else(|| panic!("consumer among incoming: {incoming:?}"));
    assert_eq!(from_consumer["kind"], "DEPENDS_ON");
    assert_eq!(from_consumer["node"]["id"], "pkg:npm/consumer");
    assert_eq!(from_consumer["confidence"], 1.0);
    assert!(
        from_consumer["evidence"][0]
            .as_str()
            .unwrap()
            .starts_with("package.json:"),
        "{from_consumer}"
    );
    let shown = text(
        &mini,
        &store,
        &["--repo-id", "mini", "explain", "pkg:npm/@mini/web"],
    );
    assert!(shown.contains("[consumer]"), "{shown}");

    // From the consuming side: its stub is folded into mini's declared package, which
    // is shown as foreign with its own location; the local dependency edge reaches it.
    let explain = json(
        &consumer,
        &store,
        &["--repo-id", "consumer", "explain", "pkg:npm/@mini/web"],
    );
    assert_eq!(explain["node"]["repo"], "mini", "{}", explain["node"]);
    assert_eq!(explain["node"]["file"], "web/package.json");
    let incoming = explain["incoming"].as_array().unwrap();
    assert!(
        incoming
            .iter()
            .any(|r| r["node"]["id"] == "pkg:npm/consumer" && r["kind"] == "DEPENDS_ON"),
        "{incoming:?}"
    );
    assert!(
        incoming
            .iter()
            .any(|r| r["node"]["id"] == "file:src/index.ts" && r["kind"] == "IMPORTS"),
        "the import edge was re-pointed too: {incoming:?}"
    );

    // Across both: affected from the declared package reaches the consumer and its file.
    let affected = json(
        &mini,
        &store,
        &[
            "--repo-id",
            "mini",
            "--all-repos",
            "affected",
            "pkg:npm/@mini/web",
            "--depth",
            "2",
        ],
    );
    let reached: Vec<&str> = affected["affected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["node"]["id"].as_str().unwrap())
        .collect();
    assert!(reached.contains(&"pkg:npm/consumer"), "{reached:?}");
    assert!(reached.contains(&"file:src/index.ts"), "{reached:?}");
    assert!(
        affected["affected"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["node"]["repo"].is_string()),
        "every rendered node names its repository"
    );

    // `surveyor federate` rewrites the table for every repository and lists the edges.
    let listed = json(&mini, &store, &["federate"]);
    assert_eq!(listed["edges"].as_array().unwrap().len(), 1, "{listed}");
    assert_eq!(listed["edges"][0]["src_repo"], "consumer");
    assert_eq!(listed["edges"][0]["dst_repo"], "mini");
    let shown = text(&mini, &store, &["federate"]);
    assert!(
        shown.contains("consumer") && shown.contains("pkg:npm/@mini/web"),
        "{shown}"
    );

    // Three: a second declarer. The pass rewrites the consumer's rows at 0.5 each, the
    // stub stays as the hub, and both declarers are reached.
    let other = declarer(dir.path(), "other");
    let o = surveyor(&other, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let stats = json(&mini, &store, &["--repo-id", "mini", "stats"]);
    assert_eq!(stats["federated"], 2, "{stats}");
    let explain = json(
        &consumer,
        &store,
        &["--repo-id", "consumer", "explain", "pkg:npm/consumer"],
    );
    let mut deps: Vec<(String, f64)> = explain["outgoing"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "DEPENDS_ON" && r["node"]["id"] == "pkg:npm/@mini/web")
        .map(|r| {
            (
                r["node"]["repo"].as_str().unwrap().to_owned(),
                r["confidence"].as_f64().unwrap(),
            )
        })
        .collect();
    deps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        deps,
        vec![
            ("consumer".to_owned(), 1.0),
            ("mini".to_owned(), 0.5),
            ("other".to_owned(), 0.5),
        ],
        "{}",
        explain["outgoing"]
    );

    // Reindexing mini alone leaves the consumer's row to `other` intact and rewrites its own.
    let o = surveyor(&mini, &store, &["index"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let stats = json(&mini, &store, &["--repo-id", "mini", "stats"]);
    assert_eq!(stats["federated"], 2, "{stats}");
}
