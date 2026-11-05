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
//!
//! A tasks file lists `[[task]]` entries instead: a task as an agent would receive
//! it, the identifiers and files the change that resolved it touched, and a token
//! budget. A task is scored by how much of that key an answer (today's `query_graph`
//! output, a context pack) carries and how many tokens the answer costs.

// Rust guideline compliant 2026-02-21

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use surveyor_context::Measured;

/// How deep the ranking is read for the reciprocal rank.
///
/// Twenty is what an agent would page through before rephrasing; a node below it
/// counts as missed.
pub const DEEP_RANK: usize = 20;

/// The token budget a task's answer is held to when the task names none.
///
/// Thirty thousand is the figure the RFD's Phase 2 exit test states; it leaves most
/// of a 200 thousand token context for the agent's own work.
pub const DEFAULT_BUDGET: usize = 30_000;

/// A questions file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Questions {
    /// The questions, in file order.
    #[serde(default)]
    pub question: Vec<Question>,
}

/// A tasks file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tasks {
    /// The tasks, in file order.
    #[serde(default)]
    pub task: Vec<Task>,
}

/// One task and the answer key the change that resolved it left behind.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Task {
    /// A short name for the task, used to select it and to label its answer.
    pub id: String,
    /// The pull request the task is drawn from, if any.
    #[serde(default)]
    pub pr: Option<u64>,
    /// The commit the task is answered from: the tree before the change.
    #[serde(default)]
    pub base: Option<String>,
    /// The commit that resolved the task.
    #[serde(default)]
    pub merge: Option<String>,
    /// The task as an agent would receive it.
    pub ask: String,
    /// Identifiers of the nodes the change edited and that exist at `base`.
    #[serde(default)]
    pub expect: Vec<String>,
    /// Files the change touched.
    #[serde(default)]
    pub files: Vec<String>,
    /// The token budget the answer is held to; [`DEFAULT_BUDGET`] when absent.
    #[serde(default)]
    pub budget: Option<usize>,
    /// Why the task is in the set.
    #[serde(default)]
    pub note: Option<String>,
}

impl Task {
    /// The budget the answer is held to.
    #[must_use]
    pub fn budget(&self) -> usize {
        self.budget.unwrap_or(DEFAULT_BUDGET)
    }

    /// The directory name of the task's store under a `--stores` directory: the
    /// first eight characters of `base`.
    #[must_use]
    pub fn store_name(&self) -> Option<String> {
        self.base.as_ref().map(|b| b.chars().take(8).collect())
    }
}

/// What an eval file holds: questions or tasks, never both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suite {
    /// Plain-English questions scored by rank.
    Questions(Questions),
    /// Tasks scored by answer key and token cost.
    Tasks(Tasks),
}

#[derive(Debug, Deserialize)]
struct AnySuite {
    #[serde(default)]
    question: Vec<Question>,
    #[serde(default)]
    task: Vec<Task>,
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

/// Reads a questions or tasks file.
///
/// # Errors
/// Returns an error when the file cannot be read, is not an eval file, holds no
/// entries, or mixes `[[question]]` and `[[task]]` entries.
pub fn load(path: &Path) -> anyhow::Result<Suite> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read questions {}", path.display()))?;
    let parsed: AnySuite =
        toml::from_str(&text).with_context(|| format!("parse questions {}", path.display()))?;
    match (parsed.question.is_empty(), parsed.task.is_empty()) {
        (false, true) => Ok(Suite::Questions(Questions {
            question: parsed.question,
        })),
        (true, false) => Ok(Suite::Tasks(Tasks { task: parsed.task })),
        (true, true) => anyhow::bail!(
            "{} holds no [[question]] or [[task]] entries",
            path.display()
        ),
        (false, false) => anyhow::bail!(
            "{} mixes [[question]] and [[task]] entries; one file holds one kind",
            path.display()
        ),
    }
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

/// What an answer to a task carried: the nodes it named and the files they lie in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Answer {
    /// Identifiers of every node the answer names.
    pub ids: BTreeSet<String>,
    /// Files of those nodes, plus any file the answer names outright.
    pub files: BTreeSet<String>,
    /// The answer's size as the agent would receive it.
    pub size: Measured,
}

/// How one task scored.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskScored {
    /// The task's short name.
    pub id: String,
    /// The pull request, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,
    /// The task text.
    pub ask: String,
    /// Expected identifiers the store does not hold.
    pub unknown: Vec<String>,
    /// Expected identifiers the answer names.
    pub found_ids: Vec<String>,
    /// Expected files the answer reaches.
    pub found_files: Vec<String>,
    /// Expected identifiers and files the answer misses, the unknown ones included.
    pub missing: Vec<String>,
    /// Share of the key (identifiers and files together) the answer carries.
    pub recall: f64,
    /// True when any part of the key is carried.
    pub hit: bool,
    /// Tokens the answer costs.
    pub tokens: usize,
    /// Bytes the answer takes.
    pub bytes: usize,
    /// The budget the answer was held to.
    pub budget: usize,
    /// True when the answer fits the budget.
    pub within_budget: bool,
    /// The note from the tasks file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Scores one task's answer against its key; `known` says whether the store holds
/// an identifier.
#[must_use]
pub fn score_task(task: &Task, answer: &Answer, known: impl Fn(&str) -> bool) -> TaskScored {
    let found_ids: Vec<String> = task
        .expect
        .iter()
        .filter(|id| answer.ids.contains(id.as_str()))
        .cloned()
        .collect();
    let found_files: Vec<String> = task
        .files
        .iter()
        .filter(|f| answer.files.contains(f.as_str()))
        .cloned()
        .collect();
    let missing: Vec<String> = task
        .expect
        .iter()
        .filter(|id| !answer.ids.contains(id.as_str()))
        .chain(
            task.files
                .iter()
                .filter(|f| !answer.files.contains(f.as_str())),
        )
        .cloned()
        .collect();
    let total = task.expect.len() + task.files.len();
    let found = found_ids.len() + found_files.len();
    let budget = task.budget();
    TaskScored {
        id: task.id.clone(),
        pr: task.pr,
        ask: task.ask.clone(),
        unknown: task
            .expect
            .iter()
            .filter(|id| !known(id))
            .cloned()
            .collect(),
        found_ids,
        found_files,
        missing,
        recall: if total == 0 {
            0.0
        } else {
            found as f64 / total as f64
        },
        hit: found > 0,
        tokens: answer.size.tokens,
        bytes: answer.size.bytes,
        budget,
        within_budget: answer.size.tokens <= budget,
        note: task.note.clone(),
    }
}

/// The scores over a whole tasks file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskSummary {
    /// What produced the answers, such as `query-graph`.
    pub source: String,
    /// Number of tasks.
    pub tasks: usize,
    /// Tasks whose answer carries any of the key.
    pub hits: usize,
    /// Mean recall of the key.
    pub mean_recall: f64,
    /// Key entries (identifiers and files) carried, over all tasks.
    pub keys_found: usize,
    /// Key entries over all tasks.
    pub keys_total: usize,
    /// Median tokens of an answer.
    pub median_tokens: usize,
    /// Largest answer in tokens.
    pub max_tokens: usize,
    /// Answers over their budget.
    pub over_budget: usize,
    /// Expected identifiers the stores do not hold, across all tasks.
    pub unknown_ids: usize,
    /// Every task's score, in file order.
    pub scored: Vec<TaskScored>,
}

/// Averages the task scores.
#[must_use]
pub fn summarize_tasks(source: &str, scored: Vec<TaskScored>) -> TaskSummary {
    let n = scored.len().max(1) as f64;
    let mut tokens: Vec<usize> = scored.iter().map(|s| s.tokens).collect();
    tokens.sort_unstable();
    let median_tokens = tokens.get(tokens.len() / 2).copied().unwrap_or(0);
    TaskSummary {
        source: source.to_owned(),
        tasks: scored.len(),
        hits: scored.iter().filter(|s| s.hit).count(),
        mean_recall: scored.iter().map(|s| s.recall).sum::<f64>() / n,
        keys_found: scored
            .iter()
            .map(|s| s.found_ids.len() + s.found_files.len())
            .sum(),
        keys_total: scored
            .iter()
            .map(|s| s.found_ids.len() + s.found_files.len() + s.missing.len())
            .sum(),
        median_tokens,
        max_tokens: tokens.last().copied().unwrap_or(0),
        over_budget: scored.iter().filter(|s| !s.within_budget).count(),
        unknown_ids: scored.iter().map(|s| s.unknown.len()).sum(),
        scored,
    }
}

/// Prints the task summary as text: one line of totals, then one line per task with
/// what the answer missed.
pub fn print_tasks_text(summary: &TaskSummary, file: &Path) {
    println!(
        "eval {}: {} tasks, source {}",
        file.display(),
        summary.tasks,
        summary.source
    );
    println!(
        "  keys {}/{} ({:.0}%)   mean recall {:.2}   hits {}/{}   tokens median {} max {}   over budget {}   unknown ids {}",
        summary.keys_found,
        summary.keys_total,
        if summary.keys_total == 0 {
            0.0
        } else {
            summary.keys_found as f64 / summary.keys_total as f64 * 100.0
        },
        summary.mean_recall,
        summary.hits,
        summary.tasks,
        summary.median_tokens,
        summary.max_tokens,
        summary.over_budget,
        summary.unknown_ids,
    );
    for s in &summary.scored {
        let mark = if s.hit { "hit " } else { "miss" };
        let fit = if s.within_budget { " " } else { "!" };
        println!(
            "  {mark} {:.2} {:>6}{fit}tok {:>7} B  {}",
            s.recall, s.tokens, s.bytes, s.id
        );
        if !s.missing.is_empty() {
            println!("        missing: {}", s.missing.join(", "));
        }
        if !s.unknown.is_empty() {
            println!("        unknown: {}", s.unknown.join(", "));
        }
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

    #[test]
    fn a_file_holds_questions_or_tasks_never_both() {
        let dir = tempfile::tempdir().expect("tempdir");
        let write = |name: &str, text: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, text).expect("writes");
            path
        };
        let tasks = write(
            "t.toml",
            "[[task]]\nid = \"a\"\nask = \"do\"\nbase = \"0123456789abcdef\"\nexpect = [\"x\"]\nfiles = [\"f.rs\"]\n",
        );
        let Suite::Tasks(t) = load(&tasks).expect("loads") else {
            panic!("tasks expected");
        };
        assert_eq!(t.task[0].budget(), DEFAULT_BUDGET);
        assert_eq!(t.task[0].store_name().as_deref(), Some("01234567"));
        let mixed = write(
            "m.toml",
            "[[task]]\nid = \"a\"\nask = \"do\"\n\n[[question]]\nask = \"q\"\nexpect = []\n",
        );
        assert!(load(&mixed).is_err());
        let empty = write("e.toml", "");
        assert!(load(&empty).is_err());
    }

    #[test]
    fn a_task_scores_its_key_and_its_budget() {
        let task = Task {
            id: "t".into(),
            pr: Some(1),
            base: None,
            merge: None,
            ask: "fix it".into(),
            expect: vec!["a".into(), "b".into(), "zzz".into()],
            files: vec!["f.rs".into(), "g.rs".into()],
            budget: Some(10),
            note: None,
        };
        let answer = Answer {
            ids: ["a", "q"].into_iter().map(String::from).collect(),
            files: ["f.rs"].into_iter().map(String::from).collect(),
            size: Measured {
                tokens: 12,
                bytes: 40,
            },
        };
        let s = score_task(&task, &answer, |id| id != "zzz");
        assert_eq!(s.found_ids, vec!["a".to_owned()]);
        assert_eq!(s.found_files, vec!["f.rs".to_owned()]);
        assert_eq!(s.missing, vec!["b", "zzz", "g.rs"]);
        assert_eq!(s.unknown, vec!["zzz".to_owned()]);
        assert!((s.recall - 0.4).abs() < 1e-9);
        assert!(s.hit);
        assert!(!s.within_budget);

        let summary = summarize_tasks("query-graph", vec![s]);
        assert_eq!(summary.keys_found, 2);
        assert_eq!(summary.keys_total, 5);
        assert_eq!(summary.over_budget, 1);
        assert_eq!(summary.median_tokens, 12);
        assert_eq!(summary.unknown_ids, 1);
    }
}
