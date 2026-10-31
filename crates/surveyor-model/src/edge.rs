//! Edges: typed, provenance-bearing relationships between nodes.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{ContentHash, ModelError, NodeId, Provenance};

/// The kind of relationship an edge records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
#[non_exhaustive]
pub enum EdgeKind {
    Contains,
    Defines,
    Imports,
    DependsOn,
    Calls,
    References,
    Implements,
    Extends,
    UsesType,
    Accepts,
    Returns,
    Reads,
    Writes,
    Migrates,
    Exposes,
    CallsRemote,
    Publishes,
    Consumes,
    ConfiguredBy,
    GatedBy,
    Builds,
    Deploys,
    Runs,
    TestedBy,
    DocumentedBy,
    DesignedBy,
    ChangedIn,
    GovernedBy,
    OwnedBy,
    SimilarTo,
    SupportsCapability,
    SummarizedAs,
    ObservedCalling,
    ObservedQuerying,
    /// An adapter-defined kind, written as `adapter:KIND`.
    Adapter(String),
}

impl EdgeKind {
    /// The canonical upper-snake spelling, such as `CALLS`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Contains => "CONTAINS",
            Self::Defines => "DEFINES",
            Self::Imports => "IMPORTS",
            Self::DependsOn => "DEPENDS_ON",
            Self::Calls => "CALLS",
            Self::References => "REFERENCES",
            Self::Implements => "IMPLEMENTS",
            Self::Extends => "EXTENDS",
            Self::UsesType => "USES_TYPE",
            Self::Accepts => "ACCEPTS",
            Self::Returns => "RETURNS",
            Self::Reads => "READS",
            Self::Writes => "WRITES",
            Self::Migrates => "MIGRATES",
            Self::Exposes => "EXPOSES",
            Self::CallsRemote => "CALLS_REMOTE",
            Self::Publishes => "PUBLISHES",
            Self::Consumes => "CONSUMES",
            Self::ConfiguredBy => "CONFIGURED_BY",
            Self::GatedBy => "GATED_BY",
            Self::Builds => "BUILDS",
            Self::Deploys => "DEPLOYS",
            Self::Runs => "RUNS",
            Self::TestedBy => "TESTED_BY",
            Self::DocumentedBy => "DOCUMENTED_BY",
            Self::DesignedBy => "DESIGNED_BY",
            Self::ChangedIn => "CHANGED_IN",
            Self::GovernedBy => "GOVERNED_BY",
            Self::OwnedBy => "OWNED_BY",
            Self::SimilarTo => "SIMILAR_TO",
            Self::SupportsCapability => "SUPPORTS_CAPABILITY",
            Self::SummarizedAs => "SUMMARIZED_AS",
            Self::ObservedCalling => "OBSERVED_CALLING",
            Self::ObservedQuerying => "OBSERVED_QUERYING",
            Self::Adapter(name) => name,
        }
    }

    /// Every built-in kind, in declaration order.
    #[must_use]
    pub fn builtin() -> &'static [EdgeKind] {
        use EdgeKind as E;
        &[
            E::Contains,
            E::Defines,
            E::Imports,
            E::DependsOn,
            E::Calls,
            E::References,
            E::Implements,
            E::Extends,
            E::UsesType,
            E::Accepts,
            E::Returns,
            E::Reads,
            E::Writes,
            E::Migrates,
            E::Exposes,
            E::CallsRemote,
            E::Publishes,
            E::Consumes,
            E::ConfiguredBy,
            E::GatedBy,
            E::Builds,
            E::Deploys,
            E::Runs,
            E::TestedBy,
            E::DocumentedBy,
            E::DesignedBy,
            E::ChangedIn,
            E::GovernedBy,
            E::OwnedBy,
            E::SimilarTo,
            E::SupportsCapability,
            E::SummarizedAs,
            E::ObservedCalling,
            E::ObservedQuerying,
        ]
    }

    /// Kinds along which a change to the target can break the source.
    ///
    /// Reverse impact ("what breaks if X changes") follows these kinds backwards.
    #[must_use]
    pub fn impact_kinds() -> &'static [EdgeKind] {
        use EdgeKind as E;
        &[
            E::Calls,
            E::References,
            E::Implements,
            E::Extends,
            E::UsesType,
            E::Accepts,
            E::Returns,
            E::Reads,
            E::Writes,
            E::Imports,
            E::DependsOn,
            E::Exposes,
            E::CallsRemote,
            E::Consumes,
            E::Runs,
            E::Deploys,
            E::Builds,
        ]
    }

    /// True for structural nesting edges, which traversal usually skips.
    #[must_use]
    pub fn is_structural(&self) -> bool {
        matches!(self, Self::Contains | Self::Defines)
    }
}

impl Display for EdgeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(kind) = Self::builtin().iter().find(|k| k.as_str() == s) {
            return Ok(kind.clone());
        }
        match s.split_once(':') {
            Some((adapter, kind)) if !adapter.is_empty() && !kind.is_empty() => {
                Ok(Self::Adapter(s.to_owned()))
            }
            _ => Err(ModelError::invalid("EdgeKind", s)),
        }
    }
}

impl From<EdgeKind> for String {
    fn from(value: EdgeKind) -> Self {
        value.as_str().to_owned()
    }
}

impl TryFrom<String> for EdgeKind {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// Identifies an edge; derived from source, kind, target and the first evidence span.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EdgeId(String);

impl EdgeId {
    /// Derives the identifier for an edge between `src` and `dst` of `kind`.
    ///
    /// `discriminator` distinguishes several edges of the same kind between the same
    /// endpoints, typically the first evidence location.
    #[must_use]
    pub fn derive(src: &NodeId, kind: &EdgeKind, dst: &NodeId, discriminator: &str) -> Self {
        let hash =
            ContentHash::of_parts([src.as_str(), kind.as_str(), dst.as_str(), discriminator]);
        Self(format!("e:{}", hash.short()))
    }

    /// Borrows the identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for EdgeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for EdgeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One relationship between two nodes, with the provenance that justifies it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Identifier derived from the endpoints, kind and first evidence span.
    pub id: EdgeId,
    /// Source node.
    pub src: NodeId,
    /// Target node.
    pub dst: NodeId,
    /// Relationship kind.
    pub kind: EdgeKind,
    /// How the edge was derived and what supports it.
    pub provenance: Provenance,
    /// Declaration nodes this edge was resolved against; a change to any invalidates it.
    pub resolved_against: Vec<NodeId>,
}

impl Edge {
    /// Creates an edge, deriving its identifier from the endpoints and first evidence.
    #[must_use]
    pub fn new(src: NodeId, kind: EdgeKind, dst: NodeId, provenance: Provenance) -> Self {
        let discriminator = provenance
            .evidence
            .first()
            .map(|e| {
                format!(
                    "{}:{}:{}",
                    e.location.file,
                    e.location.span.start(),
                    e.location.span.end()
                )
            })
            .unwrap_or_default();
        let id = EdgeId::derive(&src, &kind, &dst, &discriminator);
        Self {
            id,
            src,
            dst,
            kind,
            provenance,
            resolved_against: Vec::new(),
        }
    }

    /// Records the declarations the edge was resolved against and returns the edge.
    #[must_use]
    pub fn resolved_against(mut self, nodes: impl IntoIterator<Item = NodeId>) -> Self {
        self.resolved_against.extend(nodes);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kinds_round_trip() {
        for kind in EdgeKind::builtin() {
            let parsed: EdgeKind = kind.as_str().parse().expect("builtin parses");
            assert_eq!(&parsed, kind);
        }
        assert_eq!(EdgeKind::DependsOn.as_str(), "DEPENDS_ON");
        assert!("calls".parse::<EdgeKind>().is_err());
    }

    #[test]
    fn edge_id_is_deterministic() {
        let a = NodeId::new("a");
        let b = NodeId::new("b");
        let x = EdgeId::derive(&a, &EdgeKind::Calls, &b, "f:1:2");
        let y = EdgeId::derive(&a, &EdgeKind::Calls, &b, "f:1:2");
        let z = EdgeId::derive(&a, &EdgeKind::Calls, &b, "f:1:3");
        assert_eq!(x, y);
        assert_ne!(x, z);
    }
}
