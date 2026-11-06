//! Text retrieval over the graph's nodes: the starting points of a plain-English
//! question.
//!
//! A [`TextIndex`] holds one document per node, built from its name, scope, file
//! path, signature, doc comment and descriptive attributes, tokenized the way code is
//! read: identifiers split at case changes and underscores (`AdvisoryLockKey` is also
//! `advisory`, `lock`, `key`), words stemmed, stopwords dropped. Questions are ranked
//! by BM25F, so a rare word outweighs a common one and a match in a name outweighs one
//! in a paragraph. [`seeds`] is what `query_graph` seeds on: a bare name resolves as
//! `find` always has, a sentence goes to the index. The search only picks starting
//! nodes; the answer is still the graph around them.
//!
//! # Examples
//!
//! ```
//! use surveyor_search::tokenize;
//!
//! assert_eq!(
//!     tokenize("AdvisoryLockKey::for_workspace"),
//!     vec!["advisorylockkey", "advisori", "lock", "key", "forworkspace", "workspac"]
//! );
//! ```

// Rust guideline compliant 2026-02-21

use std::collections::HashMap;

use rust_stemmers::{Algorithm, Stemmer};
use surveyor_graph::CodeGraph;
use surveyor_model::{Node, NodeKind};

mod stopwords;

pub use stopwords::STOPWORDS;

/// BM25 term-frequency saturation. 1.2 is the textbook value; the documents here are
/// short and the field weights already favor names, so no tuning was needed.
const K1: f32 = 1.2;

/// The fields a node's text is split into, each normalized against its own average
/// length (BM25F). One document-wide length would let a long doc comment sink the
/// very nodes that explain themselves best.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    /// The short name, split at case changes and underscores.
    Name,
    /// The scope in front of the name: type, module, schema.
    Scope,
    /// The segments of the file path.
    Path,
    /// The signature line.
    Signature,
    /// The doc comment, a task description, a document's headings.
    Doc,
    /// The node kind, so `view` or `task` in a question counts a little.
    Kind,
}

impl Field {
    /// How much a match in the field counts. A name is what a question is usually
    /// about; a doc paragraph says what the node does; a path segment places it; a
    /// signature repeats the name and its types.
    fn weight(self) -> f32 {
        match self {
            Self::Name => 3.0,
            Self::Scope => 1.5,
            Self::Path | Self::Doc => 1.0,
            Self::Signature | Self::Kind => 0.5,
        }
    }

    /// Length normalization per field: prose is normalized at the textbook 0.75, so a
    /// paragraph that mentions a word once in passing counts less than a sentence
    /// about it; names and paths are short and nearly fixed in length, so 0.3 keeps a
    /// three-word name from outranking a one-word one by length alone.
    fn b(self) -> f32 {
        match self {
            Self::Doc => 0.75,
            Self::Kind => 0.0,
            _ => 0.3,
        }
    }
}

/// A prior per node kind, multiplied into the score. An impl block repeats its type's
/// name and a query repeats its function's words, so each yields to the node it is
/// part of; a test named as a sentence matches many words of a question about the
/// code it tests, so it yields a little to that code; a document's title and headings
/// repeat its subject, so when a document and the code it describes both match, the
/// code comes first and the document right behind it, still a seed when nothing in
/// the code says the words.
#[must_use]
pub fn kind_prior(kind: &NodeKind) -> f32 {
    match kind {
        NodeKind::Impl => 0.6,
        NodeKind::Query => 0.5,
        NodeKind::Test => 0.8,
        NodeKind::Document | NodeKind::Design | NodeKind::Runbook => 0.85,
        NodeKind::Column | NodeKind::File => 0.9,
        _ => 1.0,
    }
}

/// Longest doc comment or attribute text indexed per node, in bytes; a generated
/// reference keeps its opening.
const MAX_FIELD_BYTES: usize = 4000;

/// Attributes whose text describes the node in words, indexed like a doc comment.
const TEXT_ATTRIBUTES: &[&str] = &["description", "headings", "run", "depends"];

/// Shortest token kept, in bytes: `v1`, `id` and `s3` are terms; `a` is not.
const MIN_TOKEN_BYTES: usize = 2;

/// One ranked node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// Position of the node in the slice the index was built from.
    pub node: usize,
    /// BM25F score; comparable within one search only.
    pub score: f32,
}

/// An inverted index over node text.
#[derive(Debug, Clone)]
pub struct TextIndex {
    /// Term to `(document, field-normalized weighted term frequency)` postings,
    /// documents ascending. Normalization happens at build time, when every field's
    /// average length is known, so a search only saturates and sums.
    postings: HashMap<String, Vec<(u32, f32)>>,
    /// The kind prior of each document.
    priors: Vec<f32>,
}

impl TextIndex {
    /// Builds the index over `nodes`; a hit's `node` is a position in this slice.
    ///
    /// A `File` node of a Markdown file is left out: its `Document` node carries the
    /// same path and the text, and two hits for one file would crowd out a third node.
    #[must_use]
    pub fn build(nodes: &[Node]) -> Self {
        let stemmer = Stemmer::create(Algorithm::English);
        // Pass one: tokens per field per document, and each field's total length.
        let mut tokens: Vec<Vec<(Field, Vec<String>)>> = Vec::with_capacity(nodes.len());
        let mut totals = [0.0f32; 6];
        let mut counts = [0usize; 6];
        for node in nodes {
            let mut per_field = Vec::new();
            if indexed(node) {
                for (field, text) in fields(node) {
                    let terms = tokenize_with(&stemmer, text);
                    if terms.is_empty() {
                        continue;
                    }
                    totals[field as usize] += terms.len() as f32;
                    counts[field as usize] += 1;
                    per_field.push((field, terms));
                }
            }
            tokens.push(per_field);
        }
        let averages: [f32; 6] = std::array::from_fn(|i| {
            if counts[i] == 0 {
                1.0
            } else {
                totals[i] / counts[i] as f32
            }
        });
        // Pass two: each term's weighted frequency, normalized by its field's length
        // against the field's average, summed over fields.
        let mut postings: HashMap<String, Vec<(u32, f32)>> = HashMap::new();
        for (doc, per_field) in tokens.into_iter().enumerate() {
            let mut frequencies: HashMap<String, f32> = HashMap::new();
            for (field, terms) in per_field {
                let b = field.b();
                let norm = 1.0 - b + b * terms.len() as f32 / averages[field as usize];
                let weight = field.weight() / norm;
                for term in terms {
                    *frequencies.entry(term).or_default() += weight;
                }
            }
            for (term, tf) in frequencies {
                postings.entry(term).or_default().push((doc as u32, tf));
            }
        }
        Self {
            postings,
            priors: nodes.iter().map(|n| kind_prior(&n.kind)).collect(),
        }
    }

    /// Number of documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.priors.len()
    }

    /// True when the index holds no document.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.priors.is_empty()
    }

    /// The `limit` best documents for `query`, best first; empty when the query has no
    /// term the index knows. Ties keep the lower document position.
    #[must_use]
    pub fn search(&self, query: &str, limit: usize) -> Vec<Hit> {
        let terms = query_terms(query);
        if terms.is_empty() || self.is_empty() {
            return Vec::new();
        }
        let n = self.priors.len() as f32;
        let mut scores: HashMap<u32, f32> = HashMap::new();
        for (term, query_weight) in terms {
            let Some(list) = self.postings.get(&term) else {
                continue;
            };
            let df = list.len() as f32;
            let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
            for &(doc, tf) in list {
                let saturated = tf * (K1 + 1.0) / (tf + K1);
                *scores.entry(doc).or_default() += query_weight * idf * saturated;
            }
        }
        let mut hits: Vec<Hit> = scores
            .into_iter()
            .map(|(doc, score)| Hit {
                node: doc as usize,
                score: score * self.priors[doc as usize],
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.node.cmp(&b.node))
        });
        hits.truncate(limit);
        hits
    }
}

/// Whether a node gets a document: everything but the `File` node of a Markdown file,
/// which its `Document` node stands for.
fn indexed(node: &Node) -> bool {
    !(node.kind == NodeKind::File && node.language == Some(surveyor_model::Language::Markdown))
}

/// The texts of a node by field; the doc field may appear more than once (doc
/// comment, then each descriptive attribute) and its lengths add up.
fn fields(node: &Node) -> Vec<(Field, &str)> {
    let mut out = vec![(Field::Name, node.name.as_str())];
    let scope = node
        .qualified_name
        .strip_suffix(node.name.as_str())
        .unwrap_or(node.qualified_name.as_str());
    if !scope.is_empty() {
        out.push((Field::Scope, scope));
    }
    if let Some(file) = node.file() {
        out.push((Field::Path, file.as_str()));
    }
    if let Some(signature) = node.signature.as_deref() {
        out.push((Field::Signature, cut(signature)));
    }
    if let Some(doc) = node.doc.as_deref() {
        out.push((Field::Doc, cut(doc)));
    }
    for key in TEXT_ATTRIBUTES {
        // A task's description is already its doc; index it once.
        if *key == "description" && node.doc.is_some() {
            continue;
        }
        if let Some(value) = node.attributes.get(*key) {
            out.push((Field::Doc, cut(value)));
        }
    }
    out.push((Field::Kind, node.kind.as_str()));
    out
}

/// The first [`MAX_FIELD_BYTES`] of `text`, on a character boundary.
fn cut(text: &str) -> &str {
    if text.len() <= MAX_FIELD_BYTES {
        return text;
    }
    let end = (0..=MAX_FIELD_BYTES)
        .rev()
        .find(|i| text.is_char_boundary(*i))
        .unwrap_or(0);
    &text[..end]
}

/// The query's distinct terms with a weight each: a term repeated in the question
/// counts once more per repetition.
fn query_terms(query: &str) -> Vec<(String, f32)> {
    let stemmer = Stemmer::create(Algorithm::English);
    let mut weights: Vec<(String, f32)> = Vec::new();
    for token in tokenize_with(&stemmer, query) {
        match weights.iter_mut().find(|(t, _)| *t == token) {
            Some((_, w)) => *w += 1.0,
            None => weights.push((token, 1.0)),
        }
    }
    weights
}

/// Splits `text` into search terms: identifiers at case changes and underscores (the
/// whole identifier kept as a term of its own when it splits), lowercased, stopwords
/// dropped, stemmed.
#[must_use]
pub fn tokenize(text: &str) -> Vec<String> {
    tokenize_with(&Stemmer::create(Algorithm::English), text)
}

fn tokenize_with(stemmer: &Stemmer, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
        if word.is_empty() {
            continue;
        }
        let parts = split_identifier(word);
        let compound = word.replace('_', "").to_lowercase();
        if parts.len() > 1 && compound.len() >= MIN_TOKEN_BYTES && !is_stopword(&compound) {
            out.push(compound);
        }
        for part in parts {
            let lower = part.to_lowercase();
            if lower.len() < MIN_TOKEN_BYTES || is_stopword(&lower) {
                continue;
            }
            out.push(stemmer.stem(&lower).into_owned());
        }
    }
    out
}

fn is_stopword(word: &str) -> bool {
    STOPWORDS.binary_search(&word).is_ok()
}

/// The parts of an identifier: `catalog_search_export_v1` is four, `AdvisoryLockKey`
/// three, `HTTPServer` two (`HTTP`, `Server`), `v1` and `sha256` one each.
fn split_identifier(word: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    for piece in word.split('_').filter(|p| !p.is_empty()) {
        let bytes: Vec<(usize, char)> = piece.char_indices().collect();
        let mut start = 0usize;
        for i in 1..bytes.len() {
            let (at, c) = bytes[i];
            let prev = bytes[i - 1].1;
            let next_lower = bytes.get(i + 1).is_some_and(|(_, n)| n.is_lowercase());
            let boundary = (c.is_uppercase() && prev.is_lowercase())
                || (c.is_uppercase() && prev.is_uppercase() && next_lower);
            if boundary {
                parts.push(&piece[start..at]);
                start = at;
            }
        }
        parts.push(&piece[start..]);
    }
    parts
}

/// Symbols rank above tasks and documents, those above files and structure, when
/// scores tie.
fn kind_rank(kind: &NodeKind) -> u8 {
    match kind {
        k if k.is_symbol() => 0,
        NodeKind::Task | NodeKind::Document | NodeKind::Design | NodeKind::Runbook => 1,
        NodeKind::Table | NodeKind::View | NodeKind::Schema | NodeKind::Column => 2,
        NodeKind::Module | NodeKind::Package => 3,
        NodeKind::File => 4,
        _ => 5,
    }
}

/// The nodes a question starts from.
///
/// A query without spaces is a name: it resolves through [`CodeGraph::find`] exactly
/// as before, so `query_graph("OutboxEventSink")` and `find_nodes` agree. Anything
/// else is ranked by the index; `index` must have been built over `graph.nodes()`.
#[must_use]
pub fn seeds<'g>(
    graph: &'g CodeGraph,
    index: &TextIndex,
    question: &str,
    limit: usize,
) -> Vec<&'g Node> {
    ranked(graph, index, question, limit)
        .into_iter()
        .map(|(_, node)| node)
        .collect()
}

/// [`seeds`] with each node's score: `None` for a name resolved by [`CodeGraph::find`],
/// the BM25F score otherwise.
#[must_use]
pub fn ranked<'g>(
    graph: &'g CodeGraph,
    index: &TextIndex,
    question: &str,
    limit: usize,
) -> Vec<(Option<f32>, &'g Node)> {
    let q = question.trim();
    if !q.contains(char::is_whitespace) {
        let named = graph.find(q, None, limit);
        if !named.is_empty() {
            return named.into_iter().map(|n| (None, n)).collect();
        }
    }
    let nodes = graph.nodes();
    let mut hits = index.search(q, limit.saturating_mul(4).max(limit));
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| kind_rank(&nodes[a.node].kind).cmp(&kind_rank(&nodes[b.node].kind)))
            .then_with(|| nodes[a.node].id.cmp(&nodes[b.node].id))
    });
    hits.into_iter()
        .take(limit)
        .filter_map(|h| nodes.get(h.node).map(|n| (Some(h.score), n)))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use surveyor_model::{ByteSpan, ContentHash, LineSpan, NodeId, RepoId, RepoPath, TextSpan};

    use super::*;

    fn node(id: &str, kind: NodeKind, name: &str, file: &str, doc: Option<&str>) -> Node {
        let qualified = id.rsplit("::").next().unwrap_or(name);
        let mut n = Node::synthetic(NodeId::new(id), kind, name, RepoId::new("r"))
            .with_qualified_name(id.split_once("::").map_or(qualified, |(_, q)| q))
            .at(
                RepoPath::new(file),
                TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1)),
            );
        n.doc = doc.map(str::to_owned);
        n.content_hash = ContentHash::of(id.as_bytes());
        n.attributes = BTreeMap::new();
        n
    }

    fn corpus() -> Vec<Node> {
        vec![
            node(
                "rust:a/lock.rs::AdvisoryLockKey::for_workspace",
                NodeKind::Method,
                "for_workspace",
                "a/lock.rs",
                Some(
                    "The key of a workspace's lock for a purpose, by the derivation in the module documentation.",
                ),
            ),
            node(
                "rust:a/cache.rs::SessionKeyspace::key",
                NodeKind::Method,
                "key",
                "a/cache.rs",
                None,
            ),
            node(
                "rust:a/explorer.rs::ExplorerDatasets::loaded",
                NodeKind::Method,
                "loaded",
                "a/explorer.rs",
                None,
            ),
            {
                let mut t = node(
                    "task:mise/devdata:load",
                    NodeKind::Task,
                    "devdata:load",
                    ".mise/toml/devdata.toml",
                    Some(
                        "Run every development data loader in derived order: validate the manifest, load the corpus, seed reference data.",
                    ),
                );
                t.attributes
                    .insert("description".into(), t.doc.clone().unwrap_or_default());
                t
            },
            node(
                "sql:public.tasks.data",
                NodeKind::Column,
                "data",
                "m.sql",
                None,
            ),
        ]
    }

    #[test]
    fn identifiers_split_and_stem() {
        assert_eq!(
            split_identifier("catalog_search_export_v1"),
            vec!["catalog", "search", "export", "v1"]
        );
        assert_eq!(split_identifier("HTTPServer"), vec!["HTTP", "Server"]);
        assert_eq!(split_identifier("v1"), vec!["v1"]);
        assert_eq!(tokenize("the loaders are loaded"), vec!["loader", "load"]);
        assert_eq!(tokenize("devdata:load"), vec!["devdata", "load"]);
        assert!(tokenize("a an the").is_empty());
    }

    #[test]
    fn rare_words_in_names_and_docs_outrank_common_words() {
        let nodes = corpus();
        let index = TextIndex::build(&nodes);
        assert_eq!(index.len(), 5);
        let top = |q: &str| {
            index
                .search(q, 3)
                .into_iter()
                .map(|h| nodes[h.node].id.to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            top("what breaks if I change the advisory lock key derivation")[0],
            "rust:a/lock.rs::AdvisoryLockKey::for_workspace"
        );
        assert_eq!(
            top("where is development data created and how is it loaded")[0],
            "task:mise/devdata:load",
            "the task description carries the question's words; `loaded` and `data` alone do not win"
        );
        assert!(top("zzz qqq").is_empty());
    }

    #[test]
    fn a_bare_name_still_resolves_by_name() {
        let nodes = corpus();
        let graph = CodeGraph::new(nodes.clone(), Vec::new());
        let index = TextIndex::build(graph.nodes());
        let by_name = seeds(&graph, &index, "for_workspace", 5);
        assert_eq!(by_name[0].name, "for_workspace");
        let by_text = seeds(&graph, &index, "development data loader", 5);
        assert_eq!(by_text[0].id.as_str(), "task:mise/devdata:load");
        assert!(seeds(&graph, &index, "   ", 5).is_empty());
    }
}
