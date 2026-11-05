//! Cards over the `mini` fixture: every citation a card carries points at a span that
//! exists, and the declaring span holds the node's name.

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
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "json for {args:?}: {e}\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

/// A citation split into its file and line range; a whole-file citation has none.
fn parse_citation(citation: &str) -> (String, Option<(usize, usize)>) {
    let Some((file, lines)) = citation.rsplit_once(':') else {
        return (citation.to_owned(), None);
    };
    let range = match lines.split_once('-') {
        Some((a, b)) => a.parse::<usize>().ok().zip(b.parse::<usize>().ok()),
        None => lines.parse::<usize>().ok().map(|l| (l, l)),
    };
    match range {
        Some(range) => (file.to_owned(), Some(range)),
        None => (citation.to_owned(), None),
    }
}

/// The text a citation points at, asserting the file and lines exist.
fn cited_text(citation: &str) -> String {
    let (file, range) = parse_citation(citation);
    let path = fixture().join(&file);
    assert!(path.is_file(), "citation {citation}: {file} does not exist");
    let text = std::fs::read_to_string(&path).expect("readable");
    let Some((start, end)) = range else {
        return text;
    };
    let total = text.lines().count();
    assert!(
        start >= 1 && start <= end && end <= total,
        "citation {citation}: lines outside the file's {total}"
    );
    text.lines()
        .skip(start - 1)
        .take(end - start + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

/// What of a node's name must appear in its declaring span: the bare name for most
/// kinds, the self type for an impl block, nothing for a query (named by hash) or a
/// column (named inside a table's statement, which the table's span holds).
fn name_to_find(card: &Value) -> Option<String> {
    let name = card["name"].as_str().expect("name");
    match card["kind"].as_str().expect("kind") {
        "Query" | "Column" | "File" => None,
        "Impl" => name.split("#impl").next().map(str::to_owned),
        _ => Some(name.split('#').next().unwrap_or(name).to_owned()),
    }
}

#[test]
fn every_card_field_cites_a_span_that_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("ds");
    surveyor(&store, &["index"]);

    let nodes = surveyor(&store, &["nodes", "--limit", "100000"]);
    let ids: Vec<String> = nodes
        .as_array()
        .expect("nodes")
        .iter()
        .map(|n| n["id"].as_str().expect("id").to_owned())
        .collect();
    assert!(ids.len() > 50, "the fixture has {} nodes", ids.len());

    let mut args = vec!["card"];
    args.extend(ids.iter().map(String::as_str));
    let cards = surveyor(&store, &args);
    let cards = cards.as_array().expect("cards");
    assert_eq!(cards.len(), ids.len());

    let mut citations = 0usize;
    let mut counts = 0usize;
    for card in cards {
        let id = card["id"].as_str().expect("id");
        let declared = card["declared_at"].as_array().expect("declared_at");
        if card["file"].is_string() {
            assert!(
                !declared.is_empty(),
                "{id} has a file but no declaring span"
            );
        }
        for (i, c) in declared.iter().enumerate() {
            let text = cited_text(c.as_str().expect("citation"));
            citations += 1;
            if i == 0
                && card["lines"].is_string()
                && let Some(name) = name_to_find(card)
            {
                assert!(
                    text.contains(&name),
                    "{id}: declaring span {c} does not hold {name:?}:\n{text}"
                );
            }
        }
        for count in card["counts"].as_array().expect("counts") {
            counts += 1;
            let total = count["total"].as_u64().expect("total") as usize;
            let samples = count["samples"].as_array().expect("samples");
            assert!(total >= 1 && samples.len() <= 3 && samples.len() <= total);
            for s in samples {
                cited_text(s["citation"].as_str().expect("sample citation"));
                citations += 1;
                assert!(
                    ids.contains(&s["node"].as_str().expect("node").to_owned()),
                    "{id}: sample names unknown node {}",
                    s["node"]
                );
            }
        }
    }
    assert!(citations > 200, "{citations} citations checked");
    assert!(counts > 100, "{counts} counts checked");

    // One card in full: the relay's drain, with its rule in scope.
    let drain = cards
        .iter()
        .find(|c| c["id"] == "rust:crates/outbox/src/relay.rs::Relay::drain")
        .expect("drain card");
    assert_eq!(drain["package"], "pkg:cargo/outbox");
    assert_eq!(drain["language"], "rust");
    let relation = |name: &str| {
        drain["counts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["relation"] == name)
            .map(|c| c["total"].as_u64().unwrap())
    };
    assert_eq!(relation("callers"), Some(2), "{drain}");
    assert_eq!(relation("calls"), Some(3), "{drain}");
    assert_eq!(relation("tests"), Some(1), "{drain}");
    assert_eq!(
        drain["rules"],
        serde_json::json!([{
            "rule": "outbox-written-only-by-outbox",
            "severity": "fail",
            "status": "scope"
        }]),
        "{drain}"
    );
    // A node outside the rule's scope carries no rule.
    let main = cards
        .iter()
        .find(|c| c["id"] == "rust:crates/api/src/main.rs::main")
        .expect("main card");
    assert_eq!(main["rules"], serde_json::json!([]));

    // The same card rides inside `explain --json`.
    let explained = surveyor(
        &store,
        &["explain", "rust:crates/outbox/src/relay.rs::Relay::drain"],
    );
    assert_eq!(&explained["card"], drain);
}
