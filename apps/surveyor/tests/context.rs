//! Context packs over the `mini` fixture: the pack fits its budget, names what the
//! task is about, shows source that matches the checkout, and says what it left out.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mini")
        .canonicalize()
        .expect("fixture exists")
}

fn surveyor(store: &Path, json: bool, args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_surveyor"));
    cmd.arg("--repo-root")
        .arg(fixture())
        .arg("--store")
        .arg(store);
    if json {
        cmd.arg("--json");
    }
    let output = cmd.args(args).output().expect("runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

const TASK: &str = "The outbox relay drains pending rows one at a time; make Relay::drain take a batch size and send rows in batches.";

#[test]
fn a_pack_fits_its_budget_and_cites_source_that_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    let (ok, _, err) = surveyor(&store, true, &["index"]);
    assert!(ok, "{err}");

    let (ok, out, err) = surveyor(&store, true, &["context", TASK, "--budget", "5000"]);
    assert!(ok, "{err}");
    let pack: Value = serde_json::from_str(&out).expect("json pack");
    assert_eq!(pack["budget"], 5000);
    let used = pack["used"]["tokens"].as_u64().expect("tokens");
    assert!(used > 1000 && used <= 5000, "used {used}");

    // The identifier in the task is a seed at full score; text seeds follow.
    let seeds = pack["seeds"].as_array().expect("seeds");
    assert_eq!(
        seeds[0]["node"],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    assert_eq!(seeds[0]["source"], "identifier");
    assert!((seeds[0]["score"].as_f64().unwrap() - 1.0).abs() < 1e-6);
    assert!(seeds.iter().any(|s| s["source"] == "text"), "{seeds:?}");

    let items = pack["items"].as_array().expect("items");
    assert!(!items.is_empty());
    assert_eq!(
        items[0]["card"]["id"],
        "rust:crates/outbox/src/relay.rs::Relay::drain"
    );
    assert_eq!(items[0]["hops"], 0);
    let text = fixture().join("crates/outbox/src/relay.rs");
    let text = std::fs::read_to_string(text).expect("relay.rs");
    let window = &items[0]["window"];
    assert_eq!(window["file"], "crates/outbox/src/relay.rs");
    let lines: Vec<&str> = text.lines().collect();
    let start = window["start_line"].as_u64().unwrap() as usize;
    let end = window["end_line"].as_u64().unwrap() as usize;
    assert_eq!(
        window["text"].as_str().unwrap(),
        lines[start - 1..end].join("\n"),
        "the window is the file's own lines"
    );
    assert!(
        window["text"]
            .as_str()
            .unwrap()
            .contains("self.publish(row)")
    );
    // A node reached by expansion says how.
    let via = items
        .iter()
        .find(|i| i["hops"].as_u64() == Some(1))
        .expect("a one-hop item");
    assert!(
        via["via"]["citation"].as_str().unwrap().contains(':'),
        "{via}"
    );
    // Items are ranked, scores fall, tokens add up under the budget.
    let mut last = f64::MAX;
    let mut sum = 0u64;
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item["rank"], i as u64 + 1);
        let score = item["score"].as_f64().unwrap();
        assert!(score <= last);
        last = score;
        sum += item["tokens"].as_u64().unwrap();
    }
    assert!(sum <= used, "items {sum} inside the whole {used}");
    // The budget leaves candidates out, and the pack says which.
    let candidates = pack["candidates"].as_u64().unwrap();
    assert!(candidates > items.len() as u64, "{pack}");
    assert_eq!(
        pack["omitted_total"].as_u64().unwrap(),
        candidates - items.len() as u64
    );
    assert!(
        pack["omitted"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["reason"] == "over_budget")
    );

    // The Markdown is the text output and holds the sections an agent reads.
    let (ok, md, _) = surveyor(&store, false, &["context", TASK, "--budget", "5000"]);
    assert!(ok);
    assert_eq!(md, pack["markdown"].as_str().unwrap());
    for heading in ["# Context pack", "## Seeds", "## Context", "## Left out"] {
        assert!(md.contains(heading), "missing {heading}");
    }
    assert!(
        md.contains(&format!("used {used}")),
        "the budget line says what was counted"
    );

    // A seed passed by hand leads the pack.
    let (ok, out, err) = surveyor(
        &store,
        true,
        &[
            "context",
            "tidy up",
            "--seed",
            "sql:cw_outbox.outbox",
            "--budget",
            "1500",
        ],
    );
    assert!(ok, "{err}");
    let pack: Value = serde_json::from_str(&out).expect("json pack");
    assert_eq!(pack["seeds"][0]["source"], "given");
    assert_eq!(pack["items"][0]["card"]["id"], "sql:cw_outbox.outbox");
}
