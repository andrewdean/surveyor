//! The retrieval eval: plain-English questions scored against the nodes they should
//! start from.
//!
//! A questions file lists `[[question]]` entries, each a question as a user types it
//! and the identifiers of the nodes an engineer would open first. The eval asks the
//! graph for each question's starting nodes and scores the answer by recall at `k`
//! (the share of expected nodes in the top `k`), hit rate (any expected node in the
//! top `k`) and mean reciprocal rank (the rank of the first expected node within the
//! top [`DEEP_RANK`]). Expected identifiers the graph does not hold are reported, so
//! an expectation never silently names a node that does not exist.

// Rust guideline compliant 2026-02-21

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// How deep the ranking is read for the reciprocal rank.
///
/// Twenty is what an agent would page through before rephrasing; a node below it
/// counts as missed.
pub const DEEP_RANK: usize = 20;

/// A questions file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Questions {
    /// The questions, in file order.
    #[serde(default)]
    pub question: Vec<Question>,
}

/// One question and the starting nodes it should find.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Question {
    /// The question as a user would type it.
    pub ask: String,
    /// Identifiers of the nodes that are right starting points.
    pub expect: Vec<String>,
    /// Why the question is in the set, such as the words the user uses that the
    /// repository does not.
    #[serde(default)]
    pub note: Option<String>,
}

/// Reads a questions file.
///
/// # Errors
/// Returns an error when the file cannot be read or is not a questions file.
pub fn load(path: &Path) -> anyhow::Result<Questions> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read questions {}", path.display()))?;
    let questions: Questions =
        toml::from_str(&text).with_context(|| format!("parse questions {}", path.display()))?;
    anyhow::ensure!(
        !questions.question.is_empty(),
        "{} holds no [[question]] entries",
        path.display()
    );
    Ok(questions)
}

/// How one question scored.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scored {
    /// The question.
    pub ask: String,
    /// The expected identifiers.
    pub expect: Vec<String>,
    /// Expected identifiers the graph does not hold.
    pub unknown: Vec<String>,
    /// The identifiers returned, top `k` first.
    pub returned: Vec<String>,
    /// True when any expected node is in the top `k`.
    pub hit: bool,
    /// Share of the expected nodes in the top `k`.
    pub recall: f64,
    /// One-based rank of the first expected node within the deep ranking.
    pub first_rank: Option<usize>,
    /// `1 / first_rank`, or 0 when no expected node ranks.
    pub reciprocal_rank: f64,
    /// The note from the questions file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Scores one question: `top` is the ranking cut at `k`, `deep` the same ranking read
/// to [`DEEP_RANK`], `known` says whether the graph holds an identifier.
#[must_use]
pub fn score(
    question: &Question,
    top: &[String],
    deep: &[String],
    known: impl Fn(&str) -> bool,
) -> Scored {
    let expected: BTreeSet<&str> = question.expect.iter().map(String::as_str).collect();
    let found = top
        .iter()
        .filter(|id| expected.contains(id.as_str()))
        .count();
    let first_rank = deep
        .iter()
        .position(|id| expected.contains(id.as_str()))
        .map(|i| i + 1);
    Scored {
        ask: question.ask.clone(),
        expect: question.expect.clone(),
        unknown: question
            .expect
            .iter()
            .filter(|id| !known(id))
            .cloned()
            .collect(),
        returned: top.to_vec(),
        hit: found > 0,
        recall: if expected.is_empty() {
            0.0
        } else {
            found as f64 / expected.len() as f64
        },
        first_rank,
        reciprocal_rank: first_rank.map_or(0.0, |r| 1.0 / r as f64),
        note: question.note.clone(),
    }
}

/// The scores over a whole questions file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Summary {
    /// Number of questions.
    pub questions: usize,
    /// The cut for recall and hit rate.
    pub k: usize,
    /// The depth read for the reciprocal rank.
    pub deep: usize,
    /// Questions with an expected node in the top `k`.
    pub hits: usize,
    /// `hits / questions`.
    pub hit_rate: f64,
    /// Mean recall at `k`.
    pub mean_recall: f64,
    /// Mean reciprocal rank.
    pub mrr: f64,
    /// Expected identifiers the graph does not hold, across all questions.
    pub unknown_ids: usize,
    /// Every question's score, in file order.
    pub scored: Vec<Scored>,
}

/// Averages the scores.
#[must_use]
pub fn summarize(scored: Vec<Scored>, k: usize) -> Summary {
    let n = scored.len().max(1) as f64;
    let hits = scored.iter().filter(|s| s.hit).count();
    Summary {
        questions: scored.len(),
        k,
        deep: DEEP_RANK,
        hits,
        hit_rate: hits as f64 / n,
        mean_recall: scored.iter().map(|s| s.recall).sum::<f64>() / n,
        mrr: scored.iter().map(|s| s.reciprocal_rank).sum::<f64>() / n,
        unknown_ids: scored.iter().map(|s| s.unknown.len()).sum(),
        scored,
    }
}

/// Prints the summary as text: one line of totals, then one line per question with
/// what came back for the misses.
pub fn print_text(summary: &Summary, file: &Path) {
    println!(
        "eval {}: {} questions, k={}",
        file.display(),
        summary.questions,
        summary.k
    );
    println!(
        "  hit@{k} {}/{} ({:.0}%)   recall@{k} {:.2}   MRR@{} {:.2}   unknown ids {}",
        summary.hits,
        summary.questions,
        summary.hit_rate * 100.0,
        summary.mean_recall,
        summary.deep,
        summary.mrr,
        summary.unknown_ids,
        k = summary.k,
    );
    for s in &summary.scored {
        let mark = if s.hit { "hit " } else { "miss" };
        let rank = s
            .first_rank
            .map_or_else(|| "  --".to_owned(), |r| format!("#{r:<3}"));
        println!("  {mark} {:.2} {rank} {}", s.recall, s.ask);
        if !s.hit {
            let got = if s.returned.is_empty() {
                "nothing".to_owned()
            } else {
                s.returned.join(", ")
            };
            println!("        got: {got}");
        }
        if !s.unknown.is_empty() {
            println!("        unknown: {}", s.unknown.join(", "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question(expect: &[&str]) -> Question {
        Question {
            ask: "q".into(),
            expect: expect.iter().map(|s| (*s).to_owned()).collect(),
            note: None,
        }
    }

    fn ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn recall_counts_expected_nodes_in_the_top_k_and_rank_reads_deeper() {
        let q = question(&["a", "b", "zzz"]);
        let top = ids(&["x", "a"]);
        let deep = ids(&["x", "a", "b"]);
        let s = score(&q, &top, &deep, |id| id != "zzz");
        assert!(s.hit);
        assert!((s.recall - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(s.first_rank, Some(2));
        assert!((s.reciprocal_rank - 0.5).abs() < 1e-9);
        assert_eq!(s.unknown, vec!["zzz".to_owned()]);

        let miss = score(&q, &ids(&["x"]), &ids(&["x", "y"]), |_| true);
        assert!(!miss.hit);
        assert_eq!(miss.first_rank, None);
        assert!(miss.reciprocal_rank.abs() < f64::EPSILON);
        assert!(miss.recall.abs() < f64::EPSILON);

        let summary = summarize(vec![s, miss], 5);
        assert_eq!(summary.questions, 2);
        assert_eq!(summary.hits, 1);
        assert!((summary.hit_rate - 0.5).abs() < 1e-9);
        assert!((summary.mean_recall - 1.0 / 6.0).abs() < 1e-9);
        assert!((summary.mrr - 0.25).abs() < 1e-9);
        assert_eq!(summary.unknown_ids, 1);
    }

    #[test]
    fn questions_parse_from_toml() {
        let parsed: Questions = toml::from_str(
            "[[question]]\nask = \"where\"\nexpect = [\"a\"]\nnote = \"n\"\n\n[[question]]\nask = \"what\"\nexpect = []\n",
        )
        .expect("parses");
        assert_eq!(parsed.question.len(), 2);
        assert_eq!(parsed.question[0].note.as_deref(), Some("n"));
        assert!(parsed.question[1].expect.is_empty());
    }
}
