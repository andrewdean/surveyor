//! The planner: the minimum sufficient context for one task, inside a budget.
//!
//! The pack is built the way an engineer investigates. Seeds come from the
//! identifiers the task names and from the BM25F ranking of its text; expansion
//! walks the graph one hop in both directions and two hops of reverse impact, with
//! the same edges `explain` and `affected` cite; ranking multiplies the seed's score
//! by the confidence along the path, a decay per hop and the kind prior the search
//! uses; selection fills the budget greedily by score, a card for every node that
//! fits and a source window for the best of them; and the pack ends with what was
//! considered and left out, so the agent can ask for more.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use surveyor_graph::{Card, CodeGraph, Direction, EdgeFilter, Lookup, Reach, RuleNote};
use surveyor_model::{Edge, EdgeKind, Node, NodeId, NodeKind, RepoPath, Unresolved};
use surveyor_search::TextIndex;

use crate::render;
use crate::text::identifiers_in;
use crate::tokens::{Measured, Tokenizer};

/// The budget when the caller names none, in tokens.
///
/// Thirty thousand is the figure the RFD's Phase 2 exit test states; it leaves most
/// of a 200 thousand token context to the agent.
pub const DEFAULT_BUDGET: usize = 30_000;

/// Text seeds taken from the BM25F ranking.
pub const SEED_LIMIT: usize = 10;

/// What a hop costs: a neighbor of a seed scores half the seed, two hops a quarter.
pub const HOP_DECAY: f32 = 0.5;

/// What a hop up the structure costs: the impl block, type or file that owns a seed
/// is nearly as relevant as the seed, so climbing loses little.
pub const OWNER_DECAY: f32 = 0.8;

/// Owner levels climbed from a seed: the impl or class that defines a method, then
/// the type the impl is written for or the file.
pub const OWNER_LEVELS: usize = 2;

/// One-hop neighbors of a seed that are expanded one hop further, best first; the
/// second ring is where a task's "who calls the caller" and "what the owner also
/// does" live, and the ranking keeps it below the first.
pub const SECOND_RING_FROM: usize = 40;

/// How much a candidate's own match against the task text lifts it: a node whose
/// name, path or doc says the task's words outranks an equally connected one that
/// does not, by up to this factor over its graph score.
pub const LEXICAL_WEIGHT: f32 = 1.0;

/// Candidates whose text match is read from the index; the rest count as no match.
const LEXICAL_HITS: usize = 2_000;

/// Neighbors of one kind and direction that share a hop's score in full; past this
/// many, each gets its share, since a node with a thousand callers says little about
/// any one of them and a file with two hundred members even less.
pub const FAN_OUT: usize = 12;

/// Reverse-impact nodes at two hops that share a seed's score in full; past this
/// many, each gets its share.
pub const IMPACT_FAN_OUT: usize = 40;

/// A source window is worth this share of its node's card when the budget is filled,
/// so a well-ranked node's body yields to a better-ranked node's card.
pub const WINDOW_WEIGHT: f32 = 0.6;

/// Longest source window, in lines; a longer span is cut and says so.
pub const MAX_WINDOW_LINES: usize = 80;

/// Candidates considered after ranking; the rest are reported as beyond the cap.
pub const MAX_CANDIDATES: usize = 1_500;

/// Candidates scoring below this are not considered; a quarter-confidence path from
/// a weak seed is where the graph stops saying anything about the task.
pub const SCORE_FLOOR: f32 = 0.01;

/// Omitted candidates listed in the pack; the rest are counted.
pub const OMITTED_LISTED: usize = 30;

/// Tokens reserved per listed omission before the items are chosen; a long
/// identifier with its kind, score and reason runs to about this.
const OMITTED_LINE_TOKENS: usize = 40;

/// Omissions reserved for before the items are chosen; the list is cut first when
/// the whole runs over, so a small budget goes to items, not to the list of what
/// they displaced.
const OMITTED_RESERVED: usize = 10;

/// Seeds an ambiguous identifier contributes, and the score each gets: a name
/// several declarations share points at all of them, weakly.
pub const AMBIGUOUS_SEEDS: usize = 5;
/// See [`AMBIGUOUS_SEEDS`].
pub const AMBIGUOUS_SCORE: f32 = 0.4;

/// The best text seed scores this: an identifier the task spells out outranks a
/// text match, however strong.
pub const TEXT_SEED_CEILING: f32 = 0.9;

/// Unresolved references listed from the seeds' files.
pub const GAPS_LISTED: usize = 10;

/// Attributes rendered on an item, in this order; the rest are not prose.
const SHOWN_ATTRIBUTES: &[&str] = &["description", "run", "depends", "headings"];

/// Where a seed came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedSource {
    /// An identifier the caller passed.
    Given,
    /// An identifier found in the task text.
    Identifier,
    /// The BM25F ranking of the task text.
    Text,
}

/// A starting node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Seed {
    /// Node identifier.
    pub node: String,
    /// Node kind.
    pub kind: String,
    /// Scoped name.
    pub qualified_name: String,
    /// Seed score in `(0, 1]`.
    pub score: f32,
    /// Where it came from.
    pub source: SeedSource,
    /// The identifier or text that produced it.
    pub term: String,
}

/// How a node was reached: the last hop of its best path.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Via {
    /// The node one hop closer to a seed.
    pub from: String,
    /// Edge kind.
    pub edge: String,
    /// Direction of the edge relative to `from`.
    pub direction: Direction,
    /// Edge confidence.
    pub confidence: f32,
    /// `file:lines` of the edge's first evidence.
    pub citation: String,
}

/// A source window of a node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Window {
    /// File.
    pub file: String,
    /// First line, one-based.
    pub start_line: u32,
    /// Last line, one-based.
    pub end_line: u32,
    /// True when the span was cut at [`MAX_WINDOW_LINES`].
    pub truncated: bool,
    /// Language for the code fence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The lines.
    pub text: String,
}

/// One node in the pack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Item {
    /// Position in the pack, from one.
    pub rank: usize,
    /// Final score.
    pub score: f32,
    /// Hops from the nearest seed; zero for a seed.
    pub hops: usize,
    /// The last hop of the best path, for a node that is not a seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<Via>,
    /// The node's card.
    pub card: Card,
    /// Descriptive attributes: a task's description and command, a document's headings.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
    /// Source window, when one fit the budget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
    /// Tokens the item takes in the Markdown rendering.
    pub tokens: usize,
}

/// Why a candidate is not in the pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Omission {
    /// Its card did not fit the budget.
    OverBudget,
    /// Its score fell below [`SCORE_FLOOR`].
    BelowFloor,
    /// It ranked beyond [`MAX_CANDIDATES`].
    BeyondCap,
}

/// A candidate that is not in the pack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Omitted {
    /// Node identifier.
    pub node: String,
    /// Node kind.
    pub kind: String,
    /// Scoped name.
    pub qualified_name: String,
    /// Final score.
    pub score: f32,
    /// Tokens its card would have taken; zero when it was never measured.
    pub tokens: usize,
    /// Why it was left out.
    pub reason: Omission,
}

/// An unresolved reference in a seed's file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// The name that found no declaration.
    pub name: String,
    /// `file:lines` of the use.
    pub citation: String,
    /// Why it stayed unresolved.
    pub reason: String,
}

/// The pack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContextPack {
    /// The task as given.
    pub task: String,
    /// The budget in tokens.
    pub budget: usize,
    /// Tokens and bytes of the Markdown rendering.
    pub used: Measured,
    /// The seeds, best first.
    pub seeds: Vec<Seed>,
    /// Candidates considered after the floor and the cap.
    pub candidates: usize,
    /// What is in the pack, best first.
    pub items: Vec<Item>,
    /// Nodes whose window did not fit, among the items.
    pub windows_omitted: usize,
    /// Candidates left out, best first, at most [`OMITTED_LISTED`].
    pub omitted: Vec<Omitted>,
    /// Candidates left out in all, the listed ones included.
    pub omitted_total: usize,
    /// Unresolved references in the seeds' files, at most [`GAPS_LISTED`].
    pub gaps: Vec<Gap>,
    /// The Markdown rendering, what an agent reads.
    pub markdown: String,
}

/// Reads a repository file's text, or nothing when it cannot be read.
pub type SourceReader<'a> = Box<dyn Fn(&RepoPath) -> Option<String> + 'a>;
/// The rule notes of a node.
pub type RuleNotes<'a> = Box<dyn Fn(&Node) -> Vec<RuleNote> + 'a>;

/// Builds packs over one loaded graph.
pub struct Planner<'g> {
    graph: &'g CodeGraph,
    index: &'g TextIndex,
    tokenizer: &'g Tokenizer,
    source: SourceReader<'g>,
    rules: RuleNotes<'g>,
    unresolved: Vec<Unresolved>,
}

impl std::fmt::Debug for Planner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Planner")
            .field("nodes", &self.graph.node_count())
            .field("unresolved", &self.unresolved.len())
            .finish_non_exhaustive()
    }
}

/// What a second, third and further path to the same node adds: a share of its
/// score, so a hub reached from many seeds rises without burying the seed itself.
pub const EXTRA_PATH_WEIGHT: f32 = 0.25;

/// The candidates found so far, by node.
#[derive(Default)]
struct Candidates<'g> {
    found: HashMap<&'g NodeId, Candidate<'g>>,
}

impl<'g> Candidates<'g> {
    /// Offers `node` a path of `score` at `hops` from a seed, reached by `via`.
    /// `adds` says whether this path counts beside the others: a relation does, an
    /// owner does not, or the file that owns ten seeds would outrank all ten.
    fn offer(&mut self, node: &'g Node, score: f32, hops: usize, via: Option<Via>, adds: bool) {
        let entry = self.found.entry(&node.id).or_insert_with(|| Candidate {
            node,
            best: 0.0,
            sum: 0.0,
            lexical: 0.0,
            hops,
            via: via.clone(),
        });
        // The best path names how the node was reached; a seed is its own best.
        if hops == 0 || (entry.hops > 0 && score > entry.best) {
            entry.hops = hops;
            entry.via = via;
        }
        entry.best = entry.best.max(score);
        if adds {
            entry.sum += score;
        } else {
            entry.sum = entry.sum.max(entry.best);
        }
    }
}

/// A node on its way into the pack.
struct Candidate<'g> {
    node: &'g Node,
    /// The best single path's score, or the seed score.
    best: f32,
    /// Every path's score added up, the best included.
    sum: f32,
    /// The node's own match against the task text, in `[0, 1]`.
    lexical: f32,
    hops: usize,
    via: Option<Via>,
}

impl Candidate<'_> {
    /// The final score: the best path, plus [`EXTRA_PATH_WEIGHT`] of the rest, lifted
    /// by the node's own match against the task.
    fn score(&self) -> f32 {
        (self.best + EXTRA_PATH_WEIGHT * (self.sum - self.best))
            * (1.0 + LEXICAL_WEIGHT * self.lexical)
    }
}

impl<'g> Planner<'g> {
    /// A planner that reads no source and knows no rules.
    #[must_use]
    pub fn new(graph: &'g CodeGraph, index: &'g TextIndex, tokenizer: &'g Tokenizer) -> Self {
        Self {
            graph,
            index,
            tokenizer,
            source: Box::new(|_| None),
            rules: Box::new(|_| Vec::new()),
            unresolved: Vec::new(),
        }
    }

    /// Reads source windows through `source`.
    #[must_use]
    pub fn with_source(mut self, source: impl Fn(&RepoPath) -> Option<String> + 'g) -> Self {
        self.source = Box::new(source);
        self
    }

    /// Fills each card's rule notes through `rules`.
    #[must_use]
    pub fn with_rules(mut self, rules: impl Fn(&Node) -> Vec<RuleNote> + 'g) -> Self {
        self.rules = Box::new(rules);
        self
    }

    /// Lists the unresolved references of the seeds' files as gaps.
    #[must_use]
    pub fn with_unresolved(mut self, unresolved: Vec<Unresolved>) -> Self {
        self.unresolved = unresolved;
        self
    }

    /// The pack for `task` inside `budget` tokens, seeded also on `given` identifiers.
    #[must_use]
    pub fn pack(&self, task: &str, budget: usize, given: &[String]) -> ContextPack {
        let seeds = self.seeds(task, given);
        let (mut candidates, below_floor) = self.expand(task, &seeds);
        let beyond_cap = candidates.len().saturating_sub(MAX_CANDIDATES);
        let capped: Vec<Candidate<'g>> = candidates.split_off(candidates.len().min(MAX_CANDIDATES));
        let considered = candidates.len();
        let (items, omitted, windows_omitted) = self.select(task, &seeds, &candidates, budget);
        let mut omitted_all: Vec<Omitted> = omitted;
        omitted_all.extend(capped.iter().map(|c| omitted_of(c, 0, Omission::BeyondCap)));
        omitted_all.extend(
            below_floor
                .iter()
                .map(|c| omitted_of(c, 0, Omission::BelowFloor)),
        );
        omitted_all.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.node.cmp(&b.node))
        });
        let omitted_total = omitted_all.len();
        omitted_all.truncate(OMITTED_LISTED);
        let gaps = self.gaps(&seeds);
        let mut pack = ContextPack {
            task: task.to_owned(),
            budget,
            used: Measured::default(),
            seeds,
            candidates: considered + beyond_cap,
            items,
            windows_omitted,
            omitted: omitted_all,
            omitted_total,
            gaps,
            markdown: String::new(),
        };
        self.settle(&mut pack);
        // The reserve for the frame is an estimate; when the whole still runs over,
        // the list of omissions is halved, then the weakest windows go, then the
        // weakest items, until the count fits the budget.
        while pack.used.tokens > budget && !(pack.items.is_empty() && pack.omitted.is_empty()) {
            if !pack.omitted.is_empty() {
                let keep = pack.omitted.len() / 2;
                pack.omitted.truncate(keep);
            } else if let Some(last) = pack.items.iter_mut().rev().find(|i| i.window.is_some()) {
                last.window = None;
                pack.windows_omitted += 1;
            } else {
                let item = pack.items.pop().expect("not empty");
                pack.omitted.insert(
                    0,
                    Omitted {
                        node: item.card.id,
                        kind: item.card.kind,
                        qualified_name: item.card.qualified_name,
                        score: item.score,
                        tokens: item.tokens,
                        reason: Omission::OverBudget,
                    },
                );
                pack.omitted.truncate(OMITTED_LISTED);
                pack.omitted_total += 1;
            }
            self.settle(&mut pack);
        }
        pack
    }

    /// Renders the pack and counts it until the count the budget line prints is the
    /// count of the text that prints it; one extra round is the usual cost.
    fn settle(&self, pack: &mut ContextPack) {
        for _ in 0..4 {
            pack.markdown = render::markdown(pack);
            let used = self.tokenizer.measure(&pack.markdown);
            let settled = used == pack.used;
            pack.used = used;
            if settled {
                return;
            }
        }
    }

    fn seeds(&self, task: &str, given: &[String]) -> Vec<Seed> {
        let mut seeds: Vec<Seed> = Vec::new();
        let mut add = |node: &Node, score: f32, source: SeedSource, term: &str| {
            if let Some(existing) = seeds.iter_mut().find(|s| s.node == node.id.as_str()) {
                if score > existing.score {
                    existing.score = score;
                    existing.source = source;
                    term.clone_into(&mut existing.term);
                }
                return;
            }
            seeds.push(Seed {
                node: node.id.to_string(),
                kind: node.kind.to_string(),
                qualified_name: node.qualified_name.clone(),
                score,
                source,
                term: term.to_owned(),
            });
        };
        let mut resolve = |term: &str, source: SeedSource| match self.graph.lookup(term, None) {
            Lookup::Exact(n) | Lookup::Unique(n) => add(n, 1.0, source, term),
            Lookup::Ambiguous(nodes) => {
                for n in nodes.into_iter().take(AMBIGUOUS_SEEDS) {
                    add(n, AMBIGUOUS_SCORE, source, term);
                }
            }
            Lookup::NotFound { .. } => {}
        };
        for id in given {
            resolve(id, SeedSource::Given);
        }
        for term in identifiers_in(task) {
            resolve(&term, SeedSource::Identifier);
        }
        let ranked = surveyor_search::ranked(self.graph, self.index, task, SEED_LIMIT);
        let top = ranked
            .iter()
            .filter_map(|(s, _)| *s)
            .fold(0.0_f32, f32::max);
        for (score, node) in ranked {
            // A name resolved by `find` carries no score and counts as the best text
            // seed; a ranked hit scales to the top one; a zero score says nothing.
            let score = match score {
                None => TEXT_SEED_CEILING,
                Some(s) if s > 0.0 && top > 0.0 => s / top * TEXT_SEED_CEILING,
                Some(_) => continue,
            };
            add(node, score, SeedSource::Text, task);
        }
        seeds.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.node.cmp(&b.node))
        });
        seeds
    }

    /// Candidates above the floor, best first, and those below it.
    fn expand(&self, task: &str, seeds: &[Seed]) -> (Vec<Candidate<'g>>, Vec<Candidate<'g>>) {
        let mut found = Candidates::default();
        // The first ring: every neighbor of every seed, remembered for the second ring.
        let mut first_ring: Vec<(f32, &'g Node)> = Vec::new();
        for seed in seeds {
            if let Some(node) = self.graph.node(&NodeId::new(&seed.node)) {
                self.expand_seed(node, seed.score, &mut found, &mut first_ring);
            }
        }
        self.expand_second_ring(first_ring, &mut found);
        self.lift_by_text(task, seeds, &mut found);
        let mut all: Vec<Candidate<'g>> = found.found.into_values().collect();
        all.sort_by(|a, b| {
            b.score()
                .total_cmp(&a.score())
                .then_with(|| a.node.id.cmp(&b.node.id))
        });
        let floor = all.partition_point(|c| c.score() >= SCORE_FLOOR);
        let below = all.split_off(floor);
        (all, below)
    }

    /// One seed: itself, its neighbors, the owners above it and its reverse impact
    /// two hops out.
    fn expand_seed(
        &self,
        node: &'g Node,
        seed_score: f32,
        found: &mut Candidates<'g>,
        first_ring: &mut Vec<(f32, &'g Node)>,
    ) {
        let neighbors = EdgeFilter::default();
        found.offer(node, seed_score, 0, None, true);
        for (n, share) in with_shares(self.graph.all_neighbors(&node.id, &neighbors)) {
            let confidence = n.edge.provenance.confidence.get();
            let score = seed_score
                * confidence
                * HOP_DECAY
                * share
                * surveyor_search::kind_prior(&n.node.kind);
            found.offer(n.node, score, 1, Some(via(node, n.edge, n.outgoing)), true);
            first_ring.push((score, n.node));
        }
        self.climb_owners(node, seed_score, found);
        let impact = EdgeFilter::impact();
        let hits: Vec<_> = self
            .graph
            .affected(&node.id, Reach::depth(2), &impact)
            .into_iter()
            .filter(|hit| hit.depth == 2)
            .collect();
        let share = (IMPACT_FAN_OUT as f32 / hits.len().max(1) as f32).min(1.0);
        for hit in hits {
            let score = seed_score
                * hit.confidence
                * HOP_DECAY.powi(hit.depth as i32)
                * share
                * surveyor_search::kind_prior(&hit.node.kind);
            // An affected node reached us through `via`, whose source it is.
            let from = self.graph.node(&hit.via.dst).unwrap_or(node);
            found.offer(
                hit.node,
                score,
                hit.depth,
                Some(via(from, hit.via, false)),
                true,
            );
        }
    }

    /// Up the structure from `node`: the impl or class defining a method, then the
    /// type the impl is for, or the file; what the node is part of.
    fn climb_owners(&self, node: &'g Node, score: f32, found: &mut Candidates<'g>) {
        let structural =
            EdgeFilter::default().with_kinds(vec![EdgeKind::Contains, EdgeKind::Defines]);
        let uses = EdgeFilter::default().with_kinds(vec![EdgeKind::UsesType]);
        let mut owner_of = node;
        let mut owner_score = score;
        for level in 1..=OWNER_LEVELS {
            owner_score *= OWNER_DECAY;
            let Some(definer) = self.owner(owner_of, &structural) else {
                break;
            };
            found.offer(
                definer.node,
                owner_score,
                level,
                Some(via(owner_of, definer.edge, false)),
                false,
            );
            if definer.node.kind == NodeKind::Impl
                && let Some(ty) = self
                    .graph
                    .neighbors(&definer.node.id, Direction::Outgoing, &uses)
                    .into_iter()
                    .find(|n| n.node.kind.is_type() && definer.node.name.starts_with(&n.node.name))
            {
                found.offer(
                    ty.node,
                    owner_score,
                    level,
                    Some(via(definer.node, ty.edge, true)),
                    false,
                );
            }
            owner_of = definer.node;
        }
    }

    /// The node that defines or contains `node`, short of the package and repository.
    fn owner(
        &self,
        node: &'g Node,
        structural: &EdgeFilter,
    ) -> Option<surveyor_graph::Neighbor<'g>> {
        self.graph
            .neighbors(&node.id, Direction::Incoming, structural)
            .into_iter()
            .find(|n| n.node.kind != NodeKind::Repository && n.node.kind != NodeKind::Package)
    }

    /// The second ring: the best first-ring nodes expanded one more hop, and the
    /// impl, class or file that owns each of them.
    fn expand_second_ring(&self, mut first_ring: Vec<(f32, &'g Node)>, found: &mut Candidates<'g>) {
        let neighbors = EdgeFilter::default();
        let structural =
            EdgeFilter::default().with_kinds(vec![EdgeKind::Contains, EdgeKind::Defines]);
        first_ring.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
        first_ring.dedup_by(|a, b| a.1.id == b.1.id);
        for (score, node) in first_ring.into_iter().take(SECOND_RING_FROM) {
            for (n, share) in with_shares(self.graph.all_neighbors(&node.id, &neighbors)) {
                let confidence = n.edge.provenance.confidence.get();
                let hop = score
                    * confidence
                    * HOP_DECAY
                    * share
                    * surveyor_search::kind_prior(&n.node.kind);
                found.offer(n.node, hop, 2, Some(via(node, n.edge, n.outgoing)), true);
            }
            if let Some(owner) = self.owner(node, &structural) {
                found.offer(
                    owner.node,
                    score * OWNER_DECAY,
                    2,
                    Some(via(node, owner.edge, false)),
                    false,
                );
            }
        }
    }

    /// Each candidate's own match against the task text lifts its graph score; a
    /// seed the task spells out is the task's own word for it and gets the full lift.
    fn lift_by_text(&self, task: &str, seeds: &[Seed], found: &mut Candidates<'g>) {
        let nodes = self.graph.nodes();
        let hits = self.index.search(task, LEXICAL_HITS);
        let top = hits.iter().map(|h| h.score).fold(0.0_f32, f32::max);
        if top > 0.0 {
            for hit in hits {
                if let Some(c) = nodes.get(hit.node).and_then(|n| found.found.get_mut(&n.id)) {
                    c.lexical = hit.score / top;
                }
            }
        }
        for seed in seeds.iter().filter(|s| s.source != SeedSource::Text) {
            if let Some(c) = found.found.get_mut(&NodeId::new(&seed.node)) {
                c.lexical = 1.0;
            }
        }
    }

    /// Fills the budget: a card per candidate and a window for the best, greedily by
    /// score, a window being worth [`WINDOW_WEIGHT`] of its card.
    fn select(
        &self,
        task: &str,
        seeds: &[Seed],
        candidates: &[Candidate<'g>],
        budget: usize,
    ) -> (Vec<Item>, Vec<Omitted>, usize) {
        let files: RefCell<HashMap<RepoPath, Option<String>>> = RefCell::new(HashMap::new());
        let read = |path: &RepoPath| -> Option<String> {
            files
                .borrow_mut()
                .entry(path.clone())
                .or_insert_with(|| (self.source)(path))
                .clone()
        };
        // The frame around the items is paid for first, with room for the omissions.
        let frame = render::frame_estimate(task, seeds);
        let reserve = self.tokenizer.count(&frame) + OMITTED_RESERVED * OMITTED_LINE_TOKENS;
        let mut used = reserve.min(budget);

        // Every candidate's card and its tokens, in rank order.
        let mut cards: Vec<(usize, Item)> = Vec::with_capacity(candidates.len());
        for (i, c) in candidates.iter().enumerate() {
            let mut card = self.graph.card(c.node);
            card.rules = (self.rules)(c.node);
            let attributes = c
                .node
                .attributes
                .iter()
                .filter(|(k, _)| SHOWN_ATTRIBUTES.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let item = Item {
                rank: i + 1,
                score: c.score(),
                hops: c.hops,
                via: c.via.clone(),
                card,
                attributes,
                window: None,
                tokens: 0,
            };
            cards.push((i, item));
        }
        // Pieces: (value, candidate index, is_window), best value first.
        let mut pieces: Vec<(f32, usize, bool)> = Vec::with_capacity(candidates.len() * 2);
        for (i, c) in candidates.iter().enumerate() {
            pieces.push((c.score(), i, false));
            if wants_window(c.node) {
                pieces.push((c.score() * WINDOW_WEIGHT, i, true));
            }
        }
        pieces.sort_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
        });

        let mut included: Vec<bool> = vec![false; candidates.len()];
        let mut windows_omitted = 0usize;
        for (_, i, is_window) in pieces {
            let item = &mut cards[i].1;
            if is_window {
                if !included[i] {
                    continue;
                }
                let Some(window) = window_of(candidates[i].node, &read) else {
                    continue;
                };
                let tokens = self.tokenizer.count(&render::window_markdown(&window));
                if used + tokens <= budget {
                    used += tokens;
                    item.tokens += tokens;
                    item.window = Some(window);
                } else {
                    windows_omitted += 1;
                }
            } else {
                let tokens = self.tokenizer.count(&render::item_markdown(item));
                item.tokens = tokens;
                if used + tokens <= budget {
                    used += tokens;
                    included[i] = true;
                }
            }
        }
        let mut items = Vec::new();
        let mut omitted = Vec::new();
        // Every candidate's card was offered, so one that is not in is over budget.
        for (i, item) in cards {
            if included[i] {
                items.push(item);
            } else {
                omitted.push(omitted_of(
                    &candidates[i],
                    item.tokens,
                    Omission::OverBudget,
                ));
            }
        }
        for (rank, item) in items.iter_mut().enumerate() {
            item.rank = rank + 1;
        }
        (items, omitted, windows_omitted)
    }

    fn gaps(&self, seeds: &[Seed]) -> Vec<Gap> {
        let files: Vec<&RepoPath> = seeds
            .iter()
            .filter_map(|s| self.graph.node(&NodeId::new(&s.node)))
            .filter_map(Node::file)
            .collect();
        let mut gaps: Vec<Gap> = self
            .unresolved
            .iter()
            .filter(|u| files.contains(&&u.site.file))
            .map(|u| Gap {
                name: u.name.clone(),
                citation: u.site.citation(),
                reason: u.reason.to_string(),
            })
            .collect();
        gaps.sort_by(|a, b| {
            a.citation
                .cmp(&b.citation)
                .then_with(|| a.name.cmp(&b.name))
        });
        gaps.dedup();
        gaps.truncate(GAPS_LISTED);
        gaps
    }
}

/// Each neighbor with the share of a hop's score it gets: one, up to [`FAN_OUT`]
/// neighbors of the same edge kind and direction, and `FAN_OUT / n` past that.
fn with_shares(
    neighbors: Vec<surveyor_graph::Neighbor<'_>>,
) -> Vec<(surveyor_graph::Neighbor<'_>, f32)> {
    let mut sizes: HashMap<(&EdgeKind, bool), usize> = HashMap::new();
    for n in &neighbors {
        *sizes.entry((&n.edge.kind, n.outgoing)).or_default() += 1;
    }
    let shares: HashMap<(EdgeKind, bool), f32> = sizes
        .into_iter()
        .map(|((kind, out), size)| ((kind.clone(), out), (FAN_OUT as f32 / size as f32).min(1.0)))
        .collect();
    neighbors
        .into_iter()
        .map(|n| {
            let share = shares
                .get(&(n.edge.kind.clone(), n.outgoing))
                .copied()
                .unwrap_or(1.0);
            (n, share)
        })
        .collect()
}

fn via(from: &Node, edge: &Edge, outgoing: bool) -> Via {
    Via {
        from: from.id.to_string(),
        edge: edge.kind.to_string(),
        direction: if outgoing {
            Direction::Outgoing
        } else {
            Direction::Incoming
        },
        confidence: edge.provenance.confidence.get(),
        citation: edge
            .provenance
            .evidence
            .first()
            .map(surveyor_model::Evidence::citation)
            .unwrap_or_default(),
    }
}

fn omitted_of(c: &Candidate<'_>, tokens: usize, reason: Omission) -> Omitted {
    Omitted {
        node: c.node.id.to_string(),
        kind: c.node.kind.to_string(),
        qualified_name: c.node.qualified_name.clone(),
        score: c.score(),
        tokens,
        reason,
    }
}

/// Whether a node's source is worth a window: a declaration with a body, not a
/// container, a column, an impl header or a query literal its function already shows.
fn wants_window(node: &Node) -> bool {
    node.lines().is_some()
        && !matches!(
            node.kind,
            NodeKind::File
                | NodeKind::Package
                | NodeKind::Module
                | NodeKind::Workspace
                | NodeKind::Repository
                | NodeKind::Schema
                | NodeKind::Column
                | NodeKind::Query
                | NodeKind::Index
                | NodeKind::Constraint
                | NodeKind::Impl
                | NodeKind::Variant
        )
}

fn window_of(node: &Node, read: &dyn Fn(&RepoPath) -> Option<String>) -> Option<Window> {
    let file = node.file()?;
    let lines = node.lines()?;
    let text = read(file)?;
    let start = lines.start as usize;
    let end = lines.end as usize;
    let all: Vec<&str> = text.lines().collect();
    if start == 0 || start > all.len() {
        return None;
    }
    let last = end.min(all.len());
    let cut = last.min(start + MAX_WINDOW_LINES - 1);
    let body = all[start - 1..cut].join("\n");
    Some(Window {
        file: file.to_string(),
        start_line: start as u32,
        end_line: cut as u32,
        truncated: cut < last,
        language: node.language.as_ref().map(ToString::to_string),
        text: body,
    })
}
