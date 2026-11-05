//! Cards: the one-screen description of a node, derived from the graph on request.
//!
//! A card is never stored. Every field is a function of the node and the edges around
//! it, so a card is exactly as current as the dataset and cannot drift from it, and
//! every count carries sample citations so a reader can open what it summarizes.
//! The rule notes are the one part the graph cannot derive: the layer that owns the
//! rule engine fills them in.

use serde::Serialize;
use surveyor_model::{Edge, EdgeKind, Node, NodeKind};

use crate::{CodeGraph, Direction, EdgeFilter};

/// How many citations a count carries as samples.
///
/// Three is enough to open the most certain cases without turning a count of a
/// hundred callers back into a list; `explain` lists them all.
pub const SAMPLES_PER_COUNT: usize = 3;

/// A node's card.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Card {
    /// Stable identifier.
    pub id: String,
    /// Node kind.
    pub kind: String,
    /// Short name.
    pub name: String,
    /// Scoped name.
    pub qualified_name: String,
    /// Source language, when the node comes from source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The package that owns the node's file, by identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// The file the node lives in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// The lines of the node, when it is narrower than a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<String>,
    /// Signature line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// First paragraph of the doc comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Bytes of the whole doc comment, so a reader knows what the paragraph cut.
    pub doc_bytes: usize,
    /// Every span that declares the node: its own, and for a relation every migration
    /// that creates or recreates it. The signature and doc are read from the first.
    pub declared_at: Vec<String>,
    /// Members: the nodes it defines or contains, plus the methods of its impls.
    pub members: usize,
    /// Relations by kind and direction, each with a count and sample citations.
    pub counts: Vec<Count>,
    /// The rules the node is subject to; filled by the rule layer.
    pub rules: Vec<RuleNote>,
}

/// One relation of a card: how many edges of a kind point one way, with samples.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Count {
    /// A readable name for the relation, such as `callers` or `tests`.
    pub relation: String,
    /// The edge kind, such as `CALLS`.
    pub edge: String,
    /// Which way the edges point relative to the node.
    pub direction: Direction,
    /// How many such edges there are.
    pub total: usize,
    /// Up to [`SAMPLES_PER_COUNT`] edges, most certain first.
    pub samples: Vec<Sample>,
}

/// One sampled edge of a count.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sample {
    /// The node at the other end.
    pub node: String,
    /// `file:lines` of the edge's first evidence.
    pub citation: String,
    /// The edge's confidence.
    pub confidence: f32,
}

/// Why a rule is on a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleStatus {
    /// The last rule run reported a finding on this node or inside its span.
    Finding,
    /// The rule declares a scope that covers the node's file.
    Scope,
}

/// A rule the node is subject to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleNote {
    /// Rule identifier.
    pub rule: String,
    /// `fail`, `warn` or `budget`.
    pub severity: String,
    /// Why the rule is listed.
    pub status: RuleStatus,
    /// The finding's message, for a finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The finding's citation, for a finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
}

/// The first paragraph of a doc comment.
#[must_use]
pub fn first_paragraph(doc: &str) -> String {
    doc.lines()
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A readable name for edges of `kind` pointing `direction` relative to the node.
fn relation_name(kind: &EdgeKind, direction: Direction) -> String {
    let out = direction == Direction::Outgoing;
    let named = match (kind, out) {
        (EdgeKind::Calls, true) => "calls",
        (EdgeKind::Calls, false) => "callers",
        (EdgeKind::TestedBy, true) => "tests",
        (EdgeKind::TestedBy, false) => "tests_for",
        (EdgeKind::Reads, true) => "reads",
        (EdgeKind::Reads, false) => "readers",
        (EdgeKind::Writes, true) => "writes",
        (EdgeKind::Writes, false) => "writers",
        (EdgeKind::Implements, true) => "implements",
        (EdgeKind::Implements, false) => "implementations",
        (EdgeKind::DispatchesTo, true) => "dispatches_to",
        (EdgeKind::DispatchesTo, false) => "dispatched_from",
        (EdgeKind::Extends, true) => "extends",
        (EdgeKind::Extends, false) => "extended_by",
        (EdgeKind::UsesType, true) => "uses_types",
        (EdgeKind::UsesType, false) => "used_by",
        (EdgeKind::References, true) => "references",
        (EdgeKind::References, false) => "referenced_by",
        (EdgeKind::Imports, true) => "imports",
        (EdgeKind::Imports, false) => "importers",
        (EdgeKind::DependsOn, true) => "dependencies",
        (EdgeKind::DependsOn, false) => "dependents",
        (EdgeKind::Migrates, true) => "migrates",
        (EdgeKind::Migrates, false) => "migrations",
        (EdgeKind::Runs, true) => "runs",
        (EdgeKind::Runs, false) => "run_by",
        (EdgeKind::DocumentedBy, true) => "documents",
        (EdgeKind::DocumentedBy, false) => "documents_for",
        (EdgeKind::DesignedBy, true) => "designs",
        (EdgeKind::DesignedBy, false) => "designs_for",
        _ => "",
    };
    if named.is_empty() {
        let suffix = if out { "out" } else { "in" };
        format!("{}_{suffix}", kind.as_str().to_lowercase())
    } else {
        named.to_owned()
    }
}

fn first_citation(edge: &Edge) -> Option<String> {
    edge.provenance
        .evidence
        .first()
        .map(surveyor_model::Evidence::citation)
}

/// Whether an impl block is written for `ty`. An impl's name is its self type
/// followed by `#impl`, so `impl From<&Report> for Summary` belongs to `Summary`
/// although it also uses `Report`.
fn impl_is_for(imp: &Node, ty: &Node) -> bool {
    [imp.name.as_str(), imp.qualified_name.as_str()]
        .into_iter()
        .filter_map(|name| name.split("#impl").next())
        .filter_map(|self_type| self_type.rsplit("::").next())
        .any(|self_type| self_type == ty.name)
}

impl CodeGraph {
    /// The package that owns a node: itself for a package, the package containing
    /// its file otherwise.
    #[must_use]
    pub fn package_of<'a>(&'a self, node: &'a Node) -> Option<&'a Node> {
        let contains = EdgeFilter::default().with_kinds(vec![EdgeKind::Contains]);
        let owner = |file_id: &str| {
            self.neighbors(
                &surveyor_model::NodeId::new(file_id),
                Direction::Incoming,
                &contains,
            )
            .into_iter()
            .find(|n| n.node.kind == NodeKind::Package)
            .map(|n| n.node)
        };
        match node.kind {
            NodeKind::Package => Some(node),
            NodeKind::File => owner(node.id.as_str()),
            _ => node.file().and_then(|f| owner(&format!("file:{f}"))),
        }
    }

    /// The members of a node: what it defines or contains, in source order, plus
    /// the members of the impl blocks written for a type.
    #[must_use]
    pub fn members(&self, node: &Node) -> Vec<&Node> {
        let structural =
            EdgeFilter::default().with_kinds(vec![EdgeKind::Defines, EdgeKind::Contains]);
        let mut out: Vec<&Node> = self
            .neighbors(&node.id, Direction::Outgoing, &structural)
            .into_iter()
            .map(|n| n.node)
            .collect();
        if node.kind.is_type() {
            let uses = EdgeFilter::default().with_kinds(vec![EdgeKind::UsesType]);
            for imp in self.neighbors(&node.id, Direction::Incoming, &uses) {
                if imp.node.kind == NodeKind::Impl && impl_is_for(imp.node, node) {
                    out.extend(
                        self.neighbors(&imp.node.id, Direction::Outgoing, &structural)
                            .into_iter()
                            .map(|n| n.node),
                    );
                }
            }
        }
        out.sort_by_key(|a| a.lines());
        out.dedup_by(|a, b| a.id == b.id);
        out
    }

    /// The card of `node`, with no rule notes; the rule layer adds those.
    #[must_use]
    pub fn card(&self, node: &Node) -> Card {
        let all = EdgeFilter::default();
        let mut declared_at: Vec<String> = node
            .location
            .as_ref()
            .map(surveyor_model::Location::citation)
            .into_iter()
            .collect();
        // Grouped counts, structural edges left to `members`.
        let mut groups: std::collections::BTreeMap<(String, bool), Vec<&Edge>> =
            std::collections::BTreeMap::new();
        for n in self.all_neighbors(&node.id, &all) {
            match n.edge.kind {
                EdgeKind::Contains | EdgeKind::Defines => continue,
                EdgeKind::Migrates if !n.outgoing => {
                    declared_at.extend(first_citation(n.edge));
                }
                _ => {}
            }
            groups
                .entry((n.edge.kind.to_string(), n.outgoing))
                .or_default()
                .push(n.edge);
        }
        declared_at.dedup();
        let counts = groups
            .into_iter()
            .map(|((kind, outgoing), mut edges)| {
                let direction = if outgoing {
                    Direction::Outgoing
                } else {
                    Direction::Incoming
                };
                edges.sort_by(|a, b| {
                    b.provenance
                        .confidence
                        .get()
                        .total_cmp(&a.provenance.confidence.get())
                        .then_with(|| a.id.cmp(&b.id))
                });
                let edge_kind = edges[0].kind.clone();
                Count {
                    relation: relation_name(&edge_kind, direction),
                    edge: kind,
                    direction,
                    total: edges.len(),
                    samples: edges
                        .iter()
                        .filter_map(|e| {
                            first_citation(e).map(|citation| Sample {
                                node: if outgoing { &e.dst } else { &e.src }.to_string(),
                                citation,
                                confidence: e.provenance.confidence.get(),
                            })
                        })
                        .take(SAMPLES_PER_COUNT)
                        .collect(),
                }
            })
            .collect();
        Card {
            id: node.id.to_string(),
            kind: node.kind.to_string(),
            name: node.name.clone(),
            qualified_name: node.qualified_name.clone(),
            language: node.language.as_ref().map(ToString::to_string),
            package: self.package_of(node).map(|p| p.id.to_string()),
            file: node.file().map(ToString::to_string),
            lines: node.lines().map(|l| l.to_string()),
            signature: node.signature.clone(),
            doc: node.doc.as_deref().map(first_paragraph),
            doc_bytes: node.doc.as_ref().map_or(0, String::len),
            declared_at,
            members: self.members(node).len(),
            counts,
            rules: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relations_read_as_words() {
        assert_eq!(
            relation_name(&EdgeKind::Calls, Direction::Incoming),
            "callers"
        );
        assert_eq!(
            relation_name(&EdgeKind::Calls, Direction::Outgoing),
            "calls"
        );
        assert_eq!(
            relation_name(&EdgeKind::TestedBy, Direction::Outgoing),
            "tests"
        );
        assert_eq!(
            relation_name(&EdgeKind::Writes, Direction::Incoming),
            "writers"
        );
        assert_eq!(
            relation_name(&EdgeKind::GatedBy, Direction::Outgoing),
            "gated_by_out"
        );
    }

    #[test]
    fn first_paragraph_stops_at_the_blank_line() {
        assert_eq!(first_paragraph("one\ntwo\n\nthree"), "one two");
        assert_eq!(first_paragraph(""), "");
    }
}
