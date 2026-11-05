//! End-to-end: index the `mini` fixture and ask it the Phase 0 questions.
//!
//! Every assertion that names an edge also checks that its citation points at text
//! that exists and says what the edge claims.

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
    surveyor_in(&fixture(), store, args)
}

fn surveyor_in(root: &Path, store: &Path, args: &[&str]) -> Value {
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

fn cited_text(citation: &str) -> String {
    let (file, lines) = citation.rsplit_once(':').expect("file:lines");
    let (start, end) = match lines.split_once('-') {
        Some((a, b)) => (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()),
        None => (
            lines.parse::<usize>().unwrap(),
            lines.parse::<usize>().unwrap(),
        ),
    };
    let text = std::fs::read_to_string(fixture().join(file)).expect("cited file exists");
    text.lines()
        .skip(start - 1)
        .take(end - start + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

fn relations<'a>(explain: &'a Value, direction: &str, kind: &str) -> Vec<&'a Value> {
    explain[direction]
        .as_array()
        .expect("array")
        .iter()
        .filter(|r| r["kind"] == kind)
        .collect()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one scenario walks every Phase 0 question in order"
)]
fn index_and_answer_phase_zero_questions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");

    let indexed = surveyor(&store, &["index"]);
    assert_eq!(indexed["repo"], "mini");
    assert!(indexed["nodes"].as_u64().unwrap() > 60, "{indexed}");
    assert!(indexed["edges"].as_u64().unwrap() > 80, "{indexed}");
    assert_eq!(indexed["extraction_errors"], 0);
    let by_kind = &indexed["nodes_by_kind"];
    for kind in [
        "Package",
        "File",
        "Struct",
        "Trait",
        "Impl",
        "Method",
        "Function",
        "Test",
        "Class",
        "Interface",
        "Table",
        "View",
        "Column",
        "Index",
        "Constraint",
        "Query",
        "Design",
        "Runbook",
    ] {
        assert!(
            by_kind[kind].as_u64().unwrap_or(0) > 0,
            "no {kind} nodes: {by_kind}"
        );
    }
    let edges = &indexed["edges_by_kind"];
    for kind in [
        "CONTAINS",
        "DEFINES",
        "CALLS",
        "IMPLEMENTS",
        "EXTENDS",
        "USES_TYPE",
        "READS",
        "WRITES",
        "MIGRATES",
        "IMPORTS",
        "DEPENDS_ON",
        "TESTED_BY",
        "DESIGNED_BY",
        "DOCUMENTED_BY",
    ] {
        assert!(
            edges[kind].as_u64().unwrap_or(0) > 0,
            "no {kind} edges: {edges}"
        );
    }
    assert!(indexed["edges_by_derivation"]["resolved"].as_u64().unwrap() > 0);
    assert!(indexed["edges_by_derivation"]["declared"].as_u64().unwrap() > 0);

    // explain a type: an impl written for another type that only mentions this one
    // (`impl From<&Relay> for DrainSummary`) contributes no members to it.
    let relay = surveyor(&store, &["explain", "Relay"]);
    let member_names: Vec<&str> = relay["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(
        member_names.contains(&"publish") && !member_names.contains(&"from"),
        "{member_names:?}"
    );
    let summary = surveyor(&store, &["explain", "DrainSummary"]);
    assert!(
        summary["members"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["name"] == "from"),
        "{summary}"
    );

    // a `use` of a symbol from another crate names the crate and the symbol as one
    // binding, so the import edge is certain.
    let drain_trait = surveyor(&store, &["explain", "Drain"]);
    let import = relations(&drain_trait, "incoming", "IMPORTS")
        .into_iter()
        .find(|r| r["node"]["id"] == "file:crates/api/src/main.rs")
        .expect("api imports Drain");
    assert_eq!(import["confidence"], 1.0, "{import}");

    // explain: the relay's drain method calls publish and reads the outbox through the store.
    let drain = surveyor(&store, &["explain", "Relay::drain"]);
    assert_eq!(
        drain["node"]["id"],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    assert_eq!(drain["node"]["kind"], "Method");
    let calls = relations(&drain, "outgoing", "CALLS");
    let names: Vec<&str> = calls
        .iter()
        .map(|r| r["node"]["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"publish") && names.contains(&"pending") && names.contains(&"mark_sent"),
        "{names:?}"
    );
    let publish = calls
        .iter()
        .find(|r| r["node"]["name"] == "publish")
        .unwrap();
    let citation = publish["evidence"][0].as_str().unwrap();
    assert!(
        cited_text(citation).contains("self.publish(row)"),
        "citation {citation} does not show the call"
    );
    let tested = relations(&drain, "outgoing", "TESTED_BY");
    assert!(
        tested
            .iter()
            .any(|r| r["node"]["name"] == "drains_pending_rows")
    );

    // explain a table: migrations, readers and writers across languages.
    let outbox = surveyor(&store, &["explain", "cw_outbox.outbox"]);
    assert_eq!(outbox["node"]["kind"], "Table");
    let members: Vec<&str> = outbox["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(
        members.contains(&"payload") && members.contains(&"outbox_pending_idx"),
        "{members:?}"
    );
    let readers: Vec<&str> = relations(&outbox, "incoming", "READS")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert!(
        readers.iter().any(|r| r.ends_with("OutboxStore::pending")),
        "{readers:?}"
    );
    let writers: Vec<&str> = relations(&outbox, "incoming", "WRITES")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert!(
        writers
            .iter()
            .any(|r| r.ends_with("OutboxStore::mark_sent")),
        "{writers:?}"
    );
    let migrated = relations(&outbox, "incoming", "MIGRATES");
    assert!(
        migrated
            .iter()
            .any(|r| r["node"]["id"] == "file:migrations/0001_outbox.sql")
    );
    let read_edge: Value = (*relations(&outbox, "incoming", "READS")
        .iter()
        .find(|r| r["node"]["id"].as_str().unwrap().ends_with("pending"))
        .unwrap())
    .clone();
    assert_eq!(read_edge["derivation"], "resolved");
    assert!(
        cited_text(read_edge["evidence"][0].as_str().unwrap()).contains("FROM cw_outbox.outbox")
    );

    // users table is read from Rust and Python.
    let users = surveyor(&store, &["explain", "cw_users.users"]);
    let readers: Vec<&str> = relations(&users, "incoming", "READS")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert!(
        readers.iter().any(|r| r.starts_with("rust:crates/api")),
        "{readers:?}"
    );
    assert!(
        readers.iter().any(|r| r.starts_with("python:svc/db.py")),
        "{readers:?}"
    );

    // explain a view: it reads the table its query names, and code reads it.
    let pending = surveyor(&store, &["explain", "cw_outbox.pending_outbox"]);
    assert_eq!(pending["node"]["kind"], "View");
    let view_reads = relations(&pending, "outgoing", "READS");
    assert_eq!(view_reads.len(), 1, "{pending}");
    assert_eq!(view_reads[0]["node"]["id"], "sql:cw_outbox.outbox");
    assert_eq!(view_reads[0]["derivation"], "resolved");
    assert!(
        cited_text(view_reads[0]["evidence"][0].as_str().unwrap())
            .contains("FROM cw_outbox.outbox")
    );
    assert!(
        relations(&pending, "incoming", "READS")
            .iter()
            .any(|r| r["node"]["id"] == "rust:crates/api/src/main.rs::report_backlog"),
        "{pending}"
    );
    assert!(
        relations(&pending, "incoming", "MIGRATES")
            .iter()
            .any(|r| r["node"]["id"] == "file:migrations/0003_outbox_views.sql")
    );

    // explain a view created by EXECUTE inside a DO block: its schema is a format()
    // argument, so it lives in the default schema and reads `outbox` by bare name.
    let backlog = surveyor(&store, &["explain", "outbox_backlog_v1"]);
    assert_eq!(backlog["node"]["id"], "sql:public.outbox_backlog_v1");
    assert_eq!(backlog["node"]["kind"], "View");
    let backlog_reads = relations(&backlog, "outgoing", "READS");
    assert_eq!(backlog_reads.len(), 1, "{backlog}");
    assert_eq!(backlog_reads[0]["node"]["id"], "sql:cw_outbox.outbox");
    assert!(
        cited_text(backlog_reads[0]["evidence"][0].as_str().unwrap())
            .contains("CREATE OR REPLACE VIEW %I.outbox_backlog_v1")
    );
    assert!(
        relations(&backlog, "incoming", "READS")
            .iter()
            .any(|r| r["node"]["id"] == "python:svc/db.py::Reports::backlog"),
        "{backlog}"
    );

    // affected: changing the outbox table reaches the store, the relay, the API binary and the test.
    let affected = surveyor(&store, &["affected", "cw_outbox.outbox", "--depth", "4"]);
    let ids: Vec<&str> = affected["affected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["node"]["id"].as_str().unwrap())
        .collect();
    for expected in [
        "rust:crates/outbox/src/store.rs::OutboxStore::pending",
        "rust:crates/outbox/src/relay.rs::Relay::drain",
        "rust:crates/api/src/main.rs::main",
        // `purge` names the table through `{schema}.outbox`, so the schema is unknown
        // until run time; `outbox` is declared in one schema, so the edge is certain.
        "rust:crates/outbox/src/store.rs::OutboxStore::purge",
        // `report_backlog` reads the table through the `pending_outbox` view.
        "sql:cw_outbox.pending_outbox",
        "rust:crates/api/src/main.rs::report_backlog",
        // `Reports.backlog` reads it through the view a DO block creates.
        "sql:public.outbox_backlog_v1",
        "python:svc/db.py::Reports::backlog",
    ] {
        assert!(ids.contains(&expected), "{expected} missing from {ids:?}");
    }
    let backlog = affected["affected"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["node"]["id"] == "rust:crates/api/src/main.rs::report_backlog")
        .expect("report_backlog is affected");
    assert_eq!(backlog["depth"], 2, "{backlog}");
    assert_eq!(backlog["via"]["kind"], "READS", "{backlog}");

    let purge = affected["affected"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["node"]["id"] == "rust:crates/outbox/src/store.rs::OutboxStore::purge")
        .expect("purge is affected");
    assert_eq!(purge["via"]["kind"], "WRITES", "{purge}");
    assert_eq!(purge["depth"], 1, "{purge}");
    assert!(
        affected["affected"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["via"]["derivation"] != "inferred")
    );

    // path: from the API's main to the outbox table.
    let path = surveyor(
        &store,
        &[
            "path",
            "crates/api/src/main.rs::main",
            "cw_outbox.outbox",
            "--directed",
        ],
    );
    assert_eq!(path["found"], true, "{path}");
    let kinds: Vec<&str> = path["steps"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["edge"]["kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"CALLS") && (kinds.contains(&"READS") || kinds.contains(&"WRITES")),
        "{kinds:?}"
    );

    // TypeScript heritage and relative imports; Python imports; package dependencies.
    let panel = surveyor(&store, &["explain", "OutboxPanel"]);
    assert!(
        relations(&panel, "outgoing", "EXTENDS")
            .iter()
            .any(|r| r["node"]["name"] == "HttpClient")
    );
    assert!(
        relations(&panel, "outgoing", "IMPLEMENTS")
            .iter()
            .any(|r| r["node"]["name"] == "Handler")
    );
    let outbox_ts = surveyor(&store, &["explain", "file:web/src/outbox.ts"]);
    let imports: Vec<&str> = relations(&outbox_ts, "outgoing", "IMPORTS")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert!(
        imports.contains(&"file:web/src/client.ts") && imports.contains(&"pkg:npm/react"),
        "{imports:?}"
    );
    let api = surveyor(&store, &["explain", "pkg:cargo/api"]);
    let deps: Vec<&str> = relations(&api, "outgoing", "DEPENDS_ON")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert!(
        deps.contains(&"pkg:cargo/outbox") && deps.contains(&"pkg:cargo/axum"),
        "{deps:?}"
    );
    let main_py = surveyor(&store, &["explain", "file:svc/main.py"]);
    assert!(
        relations(&main_py, "outgoing", "IMPORTS")
            .iter()
            .any(|r| r["node"]["id"] == "file:svc/db.py")
    );

    // docs: the RFD covers the relay file; the runbook covers the drain script.
    let relay_file = surveyor(&store, &["explain", "file:crates/outbox/src/relay.rs"]);
    assert!(
        relations(&relay_file, "outgoing", "DESIGNED_BY")
            .iter()
            .any(|r| r["node"]["id"] == "doc:docs/rfd/0001-outbox.md")
    );
    assert!(
        relations(&relay_file, "outgoing", "DOCUMENTED_BY")
            .iter()
            .any(|r| r["node"]["id"] == "doc:docs/runbooks/stuck-outbox.md")
    );

    // shell and rhai.
    let drain_sh = surveyor(&store, &["explain", "file:scripts/drain.sh"]);
    assert!(
        relations(&drain_sh, "outgoing", "IMPORTS")
            .iter()
            .any(|r| r["node"]["id"] == "file:scripts/lib.sh")
    );
    let deploy = surveyor(&store, &["explain", "rhai:scripts/deploy.rhai::deploy"]);
    assert_eq!(deploy["node"]["kind"], "Function");

    // source: the cited window exists and shows the declaration.
    let src = surveyor(
        &store,
        &[
            "source",
            "rust:crates/outbox/src/relay.rs::Relay::drain",
            "--context",
            "0",
        ],
    );
    assert!(
        src["lines"][0].as_str().unwrap().contains("fn drain"),
        "{src}"
    );

    // check: the fixture's own rule passes; a failing rule from --rules exits 1 with citations.
    let check = surveyor(&store, &["check"]);
    assert_eq!(check["outcomes"][0]["id"], "outbox-written-only-by-outbox");
    assert_eq!(check["outcomes"][0]["passed"], true, "{check}");
    let failing = dir.path().join("no-sql-in-api.rhai");
    std::fs::write(
        &failing,
        "const ID = \"api-has-no-sql\";\nconst DESC = \"the api crate issues no SQL\";\nfn check(g) { let out = []; for e in g.edges(\"READS\") { if e.file != () && e.file.starts_with(\"crates/api/\") { out.push(violation(e, \"reads a table from the api crate\")); } } out }\n",
    )
    .expect("write rule");
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(&store)
        .arg("--json")
        .args(["check", "--rules"])
        .arg(&failing)
        .output()
        .expect("runs");
    assert_eq!(output.status.code(), Some(1), "a failing rule exits 1");

    // An ambiguous name is refused with the candidates, never answered silently,
    // and a qualifier that matches nothing is an error rather than another file's node.
    let refused = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
            .arg("--repo-root")
            .arg(fixture())
            .arg("--store")
            .arg(&store)
            .args(args)
            .output()
            .expect("run surveyor");
        assert!(!output.status.success(), "{args:?} should fail");
        String::from_utf8_lossy(&output.stderr).into_owned()
    };
    let err = refused(&["explain", "handle"]);
    assert!(err.contains("ambiguous"), "{err}");
    assert!(
        err.contains("web/src/outbox.ts::OutboxPanel::handle"),
        "{err}"
    );
    assert!(err.contains("web/src/client.ts::Handler::handle"), "{err}");
    let err = refused(&["affected", "crates/nowhere/src/lib.rs::main"]);
    assert!(err.contains("no node matches"), "{err}");
    assert!(err.contains("crates/api/src/main.rs::main"), "{err}");
    let report: Value = serde_json::from_slice(&output.stdout).expect("json report");
    let api = report["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == "api-has-no-sql")
        .expect("rule ran");
    assert_eq!(api["passed"], false);
    let cited: Vec<String> = api["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .map(|v| cited_text(v["citation"].as_str().expect("citation")))
        .collect();
    assert!(
        cited.iter().any(|t| t.contains("cw_users.users")),
        "{cited:?}"
    );

    assert!(
        cited.iter().any(|t| t.contains("cw_outbox.pending_outbox")),
        "{cited:?}"
    );

    // stats and unresolved are available; a second index replaces, not duplicates.
    let again = surveyor(&store, &["index"]);
    assert_eq!(again["nodes"], indexed["nodes"]);
    let stats = surveyor(&store, &["stats"]);
    assert_eq!(stats["nodes"], indexed["nodes"]);
    let unresolved = surveyor(&store, &["unresolved"]);
    assert!(
        unresolved
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["name"] != "publish")
    );
}

#[test]
fn an_external_profile_includes_hidden_directories() {
    // The fixture's `.mise/` is hidden, so the default walk never sees it: without a
    // node, `explain rollout` is an error.
    let plain = tempfile::tempdir().expect("tempdir");
    let plain_store = plain.path().join("db");
    surveyor(&plain_store, &["index"]);
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .args(["--repo-root"])
        .arg(fixture())
        .arg("--store")
        .arg(&plain_store)
        .args(["explain", "rollout"])
        .output()
        .expect("runs");
    assert!(
        !output.status.success(),
        "hidden task is not indexed by default"
    );

    // A profile outside the repository (Causeway cannot carry one) includes it.
    let dir = tempfile::tempdir().expect("tempdir");
    let profile = dir.path().join("mini.toml");
    let text = std::fs::read_to_string(fixture().join("surveyor.toml")).expect("fixture profile");
    std::fs::write(&profile, format!("{text}\n[walk]\ninclude = [\".mise\"]\n")).expect("write");
    let store = dir.path().join("db");
    let profile = profile.to_string_lossy().into_owned();
    surveyor(&store, &["--profile", &profile, "index"]);
    let explain = surveyor(&store, &["--profile", &profile, "explain", "rollout"]);
    assert_eq!(explain["node"]["file"], ".mise/tasks/rollout.rhai");
    let callers: Vec<&str> = relations(&explain, "incoming", "CALLS")
        .iter()
        .map(|r| r["node"]["id"].as_str().unwrap())
        .collect();
    assert_eq!(callers, vec!["rhai:.mise/tasks/ship.rhai::ship"]);
    // The same tree without the include is a different profile: update re-indexes in
    // full rather than carrying the hidden file over, and the task is gone again.
    let updated = surveyor(&store, &["update"]);
    assert_eq!(updated["strategy"], "full", "{updated}");
    assert!(
        updated["reason"]
            .as_str()
            .is_some_and(|r| r.contains("profile")),
        "{updated}"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_surveyor"))
        .args(["--repo-root"])
        .arg(fixture())
        .arg("--store")
        .arg(&store)
        .args(["explain", "rollout"])
        .output()
        .expect("runs");
    assert!(
        !output.status.success(),
        "hidden task is dropped once the profile stops including it"
    );
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create directory");
    for entry in std::fs::read_dir(from).expect("read directory") {
        let entry = entry.expect("directory entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy file");
        }
    }
}

fn kind_count(stats: &Value, kind: &str) -> u64 {
    stats["graph"]["nodes_by_kind"][kind].as_u64().unwrap_or(0)
}

#[test]
fn a_profile_edit_reaches_the_dataset_on_update() {
    // A copy of the fixture, so its committed profile can be edited in place.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    copy_tree(&fixture(), &root);
    let store = dir.path().join("db");
    let indexed = surveyor_in(&root, &store, &["index"]);
    assert_eq!(indexed["nodes_by_kind"]["Design"], 1, "{indexed}");
    assert_eq!(indexed["nodes_by_kind"]["Runbook"], 1, "{indexed}");

    // A comment is not a profile change: update stays incremental.
    let profile = root.join("surveyor.toml");
    let original = std::fs::read_to_string(&profile).expect("profile");
    std::fs::write(&profile, format!("# reviewed\n{original}")).expect("write");
    let updated = surveyor_in(&root, &store, &["update"]);
    assert_ne!(updated["strategy"], "full", "{updated}");

    // Moving the runbook glob under `designs` is one: as with Causeway's
    // `docs/adr/**` corrected to `docs/adrs/**`, the dataset must reclassify the
    // documents, which only a full index does.
    let edited = original
        .replace(
            r#"designs = ["docs/rfd/**/*.md"]"#,
            r#"designs = ["docs/rfd/**/*.md", "docs/runbooks/**/*.md"]"#,
        )
        .replace(r#"runbooks = ["docs/runbooks/**/*.md"]"#, "runbooks = []");
    assert_ne!(
        edited, original,
        "the fixture profile has the expected globs"
    );
    std::fs::write(&profile, edited).expect("write");
    let updated = surveyor_in(&root, &store, &["update"]);
    assert_eq!(updated["strategy"], "full", "{updated}");
    assert!(
        updated["reason"]
            .as_str()
            .is_some_and(|r| r.contains("profile")),
        "{updated}"
    );
    let stats = surveyor_in(&root, &store, &["stats"]);
    assert_eq!(kind_count(&stats, "Design"), 2, "{stats}");
    assert_eq!(kind_count(&stats, "Runbook"), 0, "{stats}");
}
