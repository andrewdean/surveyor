//! The Markdown a pack is read as, which is also what its budget is counted on.

use std::fmt::Write as _;

use surveyor_graph::RuleStatus;

use crate::pack::{ContextPack, Item, Omission, Seed, SeedSource, Window};

/// Longest attribute value shown on an item, in bytes.
const MAX_ATTRIBUTE_BYTES: usize = 400;

/// Declaring spans listed on an item before the rest are counted; a view that
/// twenty-nine migrations recreate is a fact, not a list to read.
const MAX_DECLARED_LISTED: usize = 4;

/// A node identifier without its language and path: `Relay::drain` for
/// `rust:crates/outbox/src/relay.rs::Relay::drain`, the path for a file, the
/// qualified name for a table or task. The citation beside it names the file.
#[must_use]
pub fn short(id: &str) -> &str {
    let Some((prefix, rest)) = id.split_once(':') else {
        return id;
    };
    match prefix {
        "file" | "doc" | "pkg" | "task" | "sql" => rest,
        _ => rest.split_once("::").map_or(rest, |(_, name)| name),
    }
}

/// The whole pack as Markdown.
#[must_use]
pub fn markdown(pack: &ContextPack) -> String {
    let mut out = String::new();
    out.push_str(&header(&pack.task, &pack.seeds));
    let _ = writeln!(
        out,
        "Budget {} tokens, used {}. Seeds {}, candidates {}, included {}, left out {}.\n",
        pack.budget,
        pack.used.tokens,
        pack.seeds.len(),
        pack.candidates,
        pack.items.len(),
        pack.omitted_total
    );
    out.push_str("## Context\n\n");
    for item in &pack.items {
        out.push_str(&item_markdown(item));
        if let Some(w) = &item.window {
            out.push_str(&window_markdown(w));
        }
        out.push('\n');
    }
    out.push_str(&omitted_markdown(pack));
    if !pack.gaps.is_empty() {
        out.push_str("## Unresolved references in the seeds' files\n\n");
        for g in &pack.gaps {
            let _ = writeln!(out, "- `{}` at {} ({})", g.name, g.citation, g.reason);
        }
        out.push('\n');
    }
    out
}

/// The frame around the items, sized before the items are chosen: the header as it
/// will print, with the budget line's numbers left to fill.
#[must_use]
pub fn frame_estimate(task: &str, seeds: &[Seed]) -> String {
    let mut out = header(task, seeds);
    out.push_str("Budget 00000 tokens, used 00000. Seeds 00, candidates 0000, included 000, left out 0000.\n\n## Context\n\n## Left out\n\n");
    out
}

fn header(task: &str, seeds: &[Seed]) -> String {
    let mut out = String::new();
    out.push_str("# Context pack\n\n");
    let _ = writeln!(out, "Task: {task}\n");
    out.push_str("## Seeds\n\n");
    for s in seeds {
        let source = match s.source {
            SeedSource::Given => "given",
            SeedSource::Identifier => "identifier",
            SeedSource::Text => "text",
        };
        let term = match s.source {
            SeedSource::Text => String::new(),
            _ => format!(" `{}`", s.term),
        };
        let _ = writeln!(
            out,
            "- `{}` {} {:.2} ({source}{term})",
            s.node, s.kind, s.score
        );
    }
    out.push('\n');
    out
}

/// One item without its window.
#[must_use]
pub fn item_markdown(item: &Item) -> String {
    let c = &item.card;
    let mut out = String::new();
    let _ = writeln!(out, "### {}. {} {}", item.rank, c.kind, c.qualified_name);
    let _ = writeln!(out, "{}  ", place_line(item).join("  "));
    if c.declared_at.len() > 1 {
        let listed = c
            .declared_at
            .iter()
            .take(MAX_DECLARED_LISTED)
            .cloned()
            .collect::<Vec<_>>();
        let more = c.declared_at.len().saturating_sub(listed.len());
        let more = if more > 0 {
            format!(" and {more} more")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "declared in {} places: {}{more}  ",
            c.declared_at.len(),
            listed.join(", ")
        );
    }
    if let Some(sig) = &c.signature {
        let _ = writeln!(out, "`{sig}`  ");
    }
    if let Some(doc) = &c.doc
        && !doc.is_empty()
    {
        if c.doc_bytes > doc.len() {
            let _ = writeln!(out, "{doc} (doc continues, {} bytes)  ", c.doc_bytes);
        } else {
            let _ = writeln!(out, "{doc}  ");
        }
    }
    for (k, v) in &item.attributes {
        let v = v.trim();
        if v.is_empty() {
            continue;
        }
        let shown: String = v.chars().take(MAX_ATTRIBUTE_BYTES).collect();
        let cut = if shown.len() < v.len() { " …" } else { "" };
        let _ = writeln!(out, "{k}: {shown}{cut}  ");
    }
    if c.members > 0 {
        let _ = writeln!(out, "members {}  ", c.members);
    }
    out.push_str(&counts_and_rules(c));
    out
}

/// The line under an item's heading: identifier, place, package, score and the hop
/// that reached it.
fn place_line(item: &Item) -> Vec<String> {
    let c = &item.card;
    let mut place: Vec<String> = vec![format!("`{}`", c.id)];
    if let Some(file) = &c.file {
        match &c.lines {
            Some(lines) => place.push(format!("{file}:{lines}")),
            None => place.push(file.clone()),
        }
    }
    if let Some(p) = &c.package {
        place.push(p.clone());
    }
    place.push(format!("score {:.2}", item.score));
    if let Some(v) = &item.via {
        let arrow = match v.direction {
            surveyor_graph::Direction::Outgoing => "->",
            surveyor_graph::Direction::Incoming => "<-",
        };
        place.push(format!(
            "via `{}` {arrow} {} ({}, {:.2})",
            short(&v.from),
            v.edge,
            v.citation,
            v.confidence
        ));
    }
    place
}

/// A card's relation counts with their samples, and its rule notes.
fn counts_and_rules(c: &surveyor_graph::Card) -> String {
    let mut out = String::new();
    for count in &c.counts {
        let samples = count
            .samples
            .iter()
            .map(|s| format!("`{}` ({})", short(&s.node), s.citation))
            .collect::<Vec<_>>()
            .join(", ");
        let more = count.total.saturating_sub(count.samples.len());
        let more = if more > 0 {
            format!(" … {more} more")
        } else {
            String::new()
        };
        let _ = writeln!(out, "{} {}: {samples}{more}  ", count.relation, count.total);
    }
    for r in &c.rules {
        match r.status {
            RuleStatus::Finding => {
                let _ = writeln!(
                    out,
                    "rule `{}` ({}): {}{}  ",
                    r.rule,
                    r.severity,
                    r.message.as_deref().unwrap_or_default(),
                    r.citation
                        .as_deref()
                        .map(|c| format!(" at {c}"))
                        .unwrap_or_default()
                );
            }
            RuleStatus::Scope => {
                let _ = writeln!(
                    out,
                    "rule `{}` ({}): governs this file  ",
                    r.rule, r.severity
                );
            }
        }
    }
    out
}

/// A source window as a fenced block.
#[must_use]
pub fn window_markdown(w: &Window) -> String {
    let mut out = String::new();
    let cut = if w.truncated { ", cut" } else { "" };
    let _ = writeln!(out, "{}:{}-{}{cut}", w.file, w.start_line, w.end_line);
    let _ = writeln!(out, "```{}", w.language.as_deref().unwrap_or_default());
    out.push_str(&w.text);
    out.push_str("\n```\n");
    out
}

fn omitted_markdown(pack: &ContextPack) -> String {
    let mut out = String::new();
    out.push_str("## Left out\n\n");
    if pack.windows_omitted > 0 {
        let _ = writeln!(
            out,
            "{} included nodes have no source window for want of budget.  ",
            pack.windows_omitted
        );
    }
    if pack.omitted_total == 0 {
        out.push_str("Nothing: every candidate fit.\n\n");
        return out;
    }
    for o in &pack.omitted {
        let reason = match o.reason {
            Omission::OverBudget => "over budget",
            Omission::BelowFloor => "below the score floor",
            Omission::BeyondCap => "beyond the candidate cap",
        };
        let tokens = if o.tokens > 0 {
            format!(", {} tokens", o.tokens)
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "- `{}` {} {:.2} ({reason}{tokens})",
            o.node, o.kind, o.score
        );
    }
    let rest = pack.omitted_total.saturating_sub(pack.omitted.len());
    if rest > 0 {
        let _ = writeln!(out, "- and {rest} more");
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use surveyor_graph::{Card, Count, Direction, RuleNote, RuleStatus, Sample};

    use super::{frame_estimate, item_markdown, markdown, short, window_markdown};
    use crate::pack::{ContextPack, Gap, Item, Omission, Omitted, Seed, SeedSource, Via, Window};
    use crate::tokens::Measured;

    fn card() -> Card {
        Card {
            id: "rust:crates/outbox/src/relay.rs::Relay::drain".into(),
            kind: "Method".into(),
            name: "drain".into(),
            qualified_name: "Relay::drain".into(),
            language: Some("rust".into()),
            package: Some("pkg:cargo/outbox".into()),
            file: Some("crates/outbox/src/relay.rs".into()),
            lines: Some("26-33".into()),
            signature: Some("fn drain(&mut self, limit: usize) -> usize".into()),
            doc: Some("Drains up to limit rows.".into()),
            doc_bytes: 60,
            declared_at: vec!["crates/outbox/src/relay.rs:26-33".into()],
            members: 0,
            counts: vec![Count {
                relation: "callers".into(),
                edge: "CALLS".into(),
                direction: Direction::Incoming,
                total: 4,
                samples: vec![Sample {
                    node: "rust:crates/api/src/main.rs::main".into(),
                    citation: "crates/api/src/main.rs:6".into(),
                    confidence: 1.0,
                }],
            }],
            rules: vec![RuleNote {
                rule: "outbox-written-only-by-outbox".into(),
                severity: "fail".into(),
                status: RuleStatus::Scope,
                message: None,
                citation: None,
            }],
        }
    }

    fn pack() -> ContextPack {
        let window = Window {
            file: "crates/outbox/src/relay.rs".into(),
            start_line: 26,
            end_line: 33,
            truncated: true,
            language: Some("rust".into()),
            text: "fn drain() {}".into(),
        };
        let item = Item {
            rank: 1,
            score: 1.5,
            hops: 1,
            via: Some(Via {
                from: "rust:crates/api/src/main.rs::main".into(),
                edge: "CALLS".into(),
                direction: Direction::Outgoing,
                confidence: 0.75,
                citation: "crates/api/src/main.rs:6".into(),
            }),
            card: card(),
            attributes: BTreeMap::from([("description".to_owned(), "drains".to_owned())]),
            window: Some(window),
            tokens: 120,
        };
        ContextPack {
            task: "batch the drain".into(),
            budget: 3000,
            used: Measured {
                tokens: 500,
                bytes: 2000,
            },
            seeds: vec![Seed {
                node: "rust:crates/outbox/src/relay.rs::Relay::drain".into(),
                kind: "Method".into(),
                qualified_name: "Relay::drain".into(),
                score: 1.0,
                source: SeedSource::Identifier,
                term: "Relay::drain".into(),
            }],
            candidates: 3,
            items: vec![item],
            windows_omitted: 1,
            omitted: vec![Omitted {
                node: "sql:cw_outbox.outbox".into(),
                kind: "Table".into(),
                qualified_name: "cw_outbox.outbox".into(),
                score: 0.4,
                tokens: 300,
                reason: Omission::OverBudget,
            }],
            omitted_total: 2,
            gaps: vec![Gap {
                name: "nope".into(),
                citation: "crates/outbox/src/relay.rs:30".into(),
                reason: "no-declaration".into(),
            }],
            markdown: String::new(),
        }
    }

    #[test]
    fn the_markdown_holds_every_section_with_short_names_and_citations() {
        let md = markdown(&pack());
        for expected in [
            "# Context pack",
            "Task: batch the drain",
            "## Seeds",
            "(identifier `Relay::drain`)",
            "Budget 3000 tokens, used 500. Seeds 1, candidates 3, included 1, left out 2.",
            "### 1. Method Relay::drain",
            "via `main` -> CALLS (crates/api/src/main.rs:6, 0.75)",
            "callers 4: `main` (crates/api/src/main.rs:6) … 3 more",
            "description: drains",
            "Drains up to limit rows. (doc continues, 60 bytes)",
            "rule `outbox-written-only-by-outbox` (fail): governs this file",
            "crates/outbox/src/relay.rs:26-33, cut
```rust
fn drain() {}
```",
            "## Left out",
            "1 included nodes have no source window",
            "`sql:cw_outbox.outbox` Table 0.40 (over budget, 300 tokens)",
            "- and 1 more",
            "## Unresolved references in the seeds' files",
            "`nope` at crates/outbox/src/relay.rs:30 (no-declaration)",
        ] {
            assert!(
                md.contains(expected),
                "missing {expected:?} in:
{md}"
            );
        }
        let item = item_markdown(&pack().items[0]);
        assert!(md.contains(&item));
        let window = window_markdown(pack().items[0].window.as_ref().unwrap());
        assert!(md.contains(&window));
        let frame = frame_estimate("batch the drain", &pack().seeds);
        assert!(frame.contains("## Seeds") && frame.contains("## Left out"));
        assert!(!frame.contains("### 1."));
    }

    #[test]
    fn short_names_drop_language_and_path() {
        assert_eq!(
            short("rust:crates/outbox/src/relay.rs::Relay::drain"),
            "Relay::drain"
        );
        assert_eq!(
            short("file:crates/outbox/src/relay.rs"),
            "crates/outbox/src/relay.rs"
        );
        assert_eq!(short("sql:cw_outbox.outbox"), "cw_outbox.outbox");
        assert_eq!(short("task:mise/db:migrate"), "mise/db:migrate");
        assert_eq!(short("pkg:cargo/outbox"), "cargo/outbox");
        assert_eq!(short("plain"), "plain");
    }
}
