//! The linker's output: nodes, edges and what could not be resolved.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Edge, ModelError, Node, ReferenceRole, SourceLocation};

/// Why a reference stayed unresolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnresolvedReason {
    /// No declaration with that name exists in the repository.
    NoDeclaration,
    /// More declarations than the linker will guess between.
    Ambiguous,
    /// A method call whose receiver type is unknown matched several remote declarations.
    AmbiguousReceiver,
    /// No table with that name exists in the declared schema.
    NoTable,
}

impl UnresolvedReason {
    /// The kebab-case name used in storage and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoDeclaration => "no-declaration",
            Self::Ambiguous => "ambiguous",
            Self::AmbiguousReceiver => "ambiguous-receiver",
            Self::NoTable => "no-table",
        }
    }
}

impl std::fmt::Display for UnresolvedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for UnresolvedReason {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "no-declaration" => Ok(Self::NoDeclaration),
            "ambiguous" => Ok(Self::Ambiguous),
            "ambiguous-receiver" => Ok(Self::AmbiguousReceiver),
            "no-table" => Ok(Self::NoTable),
            other => Err(ModelError::invalid("UnresolvedReason", other)),
        }
    }
}

/// A reference or literal the linker could not attach to a node.
///
/// Unresolved facts are kept, never guessed into edges; they are the gaps an agent is
/// told about and the first thing a precise resolver fixes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unresolved {
    /// How the name was used.
    pub role: ReferenceRole,
    /// The name that found no declaration.
    pub name: String,
    /// The use site.
    pub site: SourceLocation,
    /// Why it stayed unresolved.
    pub reason: UnresolvedReason,
}

/// Counts by kind for a set of nodes and edges.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphStats {
    /// Total nodes.
    pub nodes: usize,
    /// Total edges.
    pub edges: usize,
    /// Unresolved references.
    pub unresolved: usize,
    /// Nodes by kind name.
    pub nodes_by_kind: BTreeMap<String, usize>,
    /// Edges by kind name.
    pub edges_by_kind: BTreeMap<String, usize>,
    /// Edges by derivation kind name.
    pub edges_by_derivation: BTreeMap<String, usize>,
}

/// Nodes and edges produced for a repository or a changed subset of it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphDelta {
    /// Nodes to upsert.
    pub nodes: Vec<Node>,
    /// Edges to upsert.
    pub edges: Vec<Edge>,
    /// References that found no target.
    pub unresolved: Vec<Unresolved>,
}

impl GraphDelta {
    /// An empty delta.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends another delta's contents.
    pub fn merge(&mut self, other: GraphDelta) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
        self.unresolved.extend(other.unresolved);
    }

    /// Counts nodes and edges by kind.
    #[must_use]
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            nodes: self.nodes.len(),
            edges: self.edges.len(),
            unresolved: self.unresolved.len(),
            nodes_by_kind: count_by(self.nodes.iter().map(|n| n.kind.as_str())),
            edges_by_kind: count_by(self.edges.iter().map(|e| e.kind.as_str())),
            edges_by_derivation: count_by(
                self.edges
                    .iter()
                    .map(|e| e.provenance.derivation.kind().as_str()),
            ),
        }
    }
}

/// Counts how often each key occurs.
fn count_by<'a>(keys: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    keys.fold(BTreeMap::new(), |mut acc, key| {
        *acc.entry(key.to_owned()).or_default() += 1;
        acc
    })
}
