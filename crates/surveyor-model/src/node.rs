//! Nodes: the things in a codebase that relationships connect.

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    ByteSpan, ContentHash, Language, LineSpan, Location, ModelError, NodeId, RepoId, RepoPath,
    SourceLocation, TextSpan,
};

/// The kind of thing a node represents.
///
/// The built-in variants are language- and organization-neutral. An adapter adds its
/// own kinds through [`NodeKind::Adapter`], namespaced by the adapter name, such as
/// `causeway:OutboxRelay`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
#[non_exhaustive]
pub enum NodeKind {
    // Structure
    Repository,
    Workspace,
    Package,
    Module,
    File,
    // Code symbols
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Impl,
    TypeAlias,
    Const,
    Macro,
    Test,
    Variable,
    // Data
    Schema,
    Table,
    Column,
    Index,
    Constraint,
    Migration,
    Query,
    // Contracts
    ProtoFile,
    Service,
    Rpc,
    Message,
    OpenApiOperation,
    GeneratedBinding,
    // Messaging
    Topic,
    Queue,
    Producer,
    Consumer,
    // Platform
    Workload,
    Image,
    Dockerfile,
    ConfigMap,
    Secret,
    Flag,
    Policy,
    Namespace,
    Environment,
    // Tasks
    Task,
    Script,
    Workflow,
    HostModule,
    // Knowledge
    Design,
    Runbook,
    Skill,
    Plan,
    // History
    Commit,
    Author,
    PullRequest,
    // Inferred
    Capability,
    Summary,
    Community,
    /// An adapter-defined kind, written as `adapter:Kind`.
    Adapter(String),
}

impl NodeKind {
    /// The canonical spelling, such as `Function` or `causeway:OutboxRelay`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Repository => "Repository",
            Self::Workspace => "Workspace",
            Self::Package => "Package",
            Self::Module => "Module",
            Self::File => "File",
            Self::Function => "Function",
            Self::Method => "Method",
            Self::Class => "Class",
            Self::Struct => "Struct",
            Self::Enum => "Enum",
            Self::Interface => "Interface",
            Self::Trait => "Trait",
            Self::Impl => "Impl",
            Self::TypeAlias => "TypeAlias",
            Self::Const => "Const",
            Self::Macro => "Macro",
            Self::Test => "Test",
            Self::Variable => "Variable",
            Self::Schema => "Schema",
            Self::Table => "Table",
            Self::Column => "Column",
            Self::Index => "Index",
            Self::Constraint => "Constraint",
            Self::Migration => "Migration",
            Self::Query => "Query",
            Self::ProtoFile => "ProtoFile",
            Self::Service => "Service",
            Self::Rpc => "Rpc",
            Self::Message => "Message",
            Self::OpenApiOperation => "OpenApiOperation",
            Self::GeneratedBinding => "GeneratedBinding",
            Self::Topic => "Topic",
            Self::Queue => "Queue",
            Self::Producer => "Producer",
            Self::Consumer => "Consumer",
            Self::Workload => "Workload",
            Self::Image => "Image",
            Self::Dockerfile => "Dockerfile",
            Self::ConfigMap => "ConfigMap",
            Self::Secret => "Secret",
            Self::Flag => "Flag",
            Self::Policy => "Policy",
            Self::Namespace => "Namespace",
            Self::Environment => "Environment",
            Self::Task => "Task",
            Self::Script => "Script",
            Self::Workflow => "Workflow",
            Self::HostModule => "HostModule",
            Self::Design => "Design",
            Self::Runbook => "Runbook",
            Self::Skill => "Skill",
            Self::Plan => "Plan",
            Self::Commit => "Commit",
            Self::Author => "Author",
            Self::PullRequest => "PullRequest",
            Self::Capability => "Capability",
            Self::Summary => "Summary",
            Self::Community => "Community",
            Self::Adapter(name) => name,
        }
    }

    /// True for kinds that name a code symbol a developer would search for.
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::Method
                | Self::Class
                | Self::Struct
                | Self::Enum
                | Self::Interface
                | Self::Trait
                | Self::Impl
                | Self::TypeAlias
                | Self::Const
                | Self::Macro
                | Self::Test
                | Self::Variable
        )
    }

    /// True for kinds that can be a call target.
    #[must_use]
    pub fn is_callable(&self) -> bool {
        matches!(
            self,
            Self::Function | Self::Method | Self::Macro | Self::Test
        )
    }

    /// True for kinds that name a type.
    #[must_use]
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Self::Class
                | Self::Struct
                | Self::Enum
                | Self::Interface
                | Self::Trait
                | Self::TypeAlias
        )
    }

    /// Every built-in kind, in declaration order.
    #[must_use]
    pub fn builtin() -> &'static [NodeKind] {
        use NodeKind as K;
        &[
            K::Repository,
            K::Workspace,
            K::Package,
            K::Module,
            K::File,
            K::Function,
            K::Method,
            K::Class,
            K::Struct,
            K::Enum,
            K::Interface,
            K::Trait,
            K::Impl,
            K::TypeAlias,
            K::Const,
            K::Macro,
            K::Test,
            K::Variable,
            K::Schema,
            K::Table,
            K::Column,
            K::Index,
            K::Constraint,
            K::Migration,
            K::Query,
            K::ProtoFile,
            K::Service,
            K::Rpc,
            K::Message,
            K::OpenApiOperation,
            K::GeneratedBinding,
            K::Topic,
            K::Queue,
            K::Producer,
            K::Consumer,
            K::Workload,
            K::Image,
            K::Dockerfile,
            K::ConfigMap,
            K::Secret,
            K::Flag,
            K::Policy,
            K::Namespace,
            K::Environment,
            K::Task,
            K::Script,
            K::Workflow,
            K::HostModule,
            K::Design,
            K::Runbook,
            K::Skill,
            K::Plan,
            K::Commit,
            K::Author,
            K::PullRequest,
            K::Capability,
            K::Summary,
            K::Community,
        ]
    }
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for NodeKind {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(kind) = Self::builtin().iter().find(|k| k.as_str() == s) {
            return Ok(kind.clone());
        }
        match s.split_once(':') {
            Some((adapter, kind)) if !adapter.is_empty() && !kind.is_empty() => {
                Ok(Self::Adapter(s.to_owned()))
            }
            _ => Err(ModelError::invalid("NodeKind", s)),
        }
    }
}

impl From<NodeKind> for String {
    fn from(value: NodeKind) -> Self {
        value.as_str().to_owned()
    }
}

impl TryFrom<String> for NodeKind {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// One node in the graph.
///
/// `id` is stable across commits for an unchanged declaration; `content_hash` changes
/// whenever the declaration's text changes, which is how incremental update recognizes
/// work to redo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Stable identifier.
    pub id: NodeId,
    /// What the node is.
    pub kind: NodeKind,
    /// Short name, such as `drain`.
    pub name: String,
    /// Scoped name, such as `outbox::Relay::drain`.
    pub qualified_name: String,
    /// Repository that owns the node.
    pub repo: RepoId,
    /// Language, when the node comes from source.
    pub language: Option<Language>,
    /// Where the node lives, when it comes from a file.
    pub location: Option<Location>,
    /// Hash of the declaration text, or of the node's defining attributes.
    pub content_hash: ContentHash,
    /// Signature line, such as `pub fn drain(&mut self) -> Result<usize>`.
    pub signature: Option<String>,
    /// Documentation comment, if any.
    pub doc: Option<String>,
    /// Open-ended attributes an extractor or adapter attaches.
    pub attributes: BTreeMap<String, String>,
}

impl Node {
    /// Creates a node that is not tied to a source span, such as a package or table.
    pub fn synthetic(id: NodeId, kind: NodeKind, name: impl Into<String>, repo: RepoId) -> Self {
        let name = name.into();
        let content_hash = ContentHash::of_parts([id.as_str(), kind.as_str(), name.as_str()]);
        Self {
            id,
            kind,
            qualified_name: name.clone(),
            name,
            repo,
            language: None,
            location: None,
            content_hash,
            signature: None,
            doc: None,
            attributes: BTreeMap::new(),
        }
    }

    /// Sets the qualified name and returns the node.
    #[must_use]
    pub fn with_qualified_name(mut self, qualified: impl Into<String>) -> Self {
        self.qualified_name = qualified.into();
        self
    }

    /// Places the node in a whole file and returns it.
    #[must_use]
    pub fn in_file(mut self, file: RepoPath) -> Self {
        self.location = Some(Location::File { file });
        self
    }

    /// Places the node at a span inside a file and returns it.
    #[must_use]
    pub fn at(mut self, file: RepoPath, span: TextSpan) -> Self {
        self.location = Some(Location::Span(SourceLocation::new(file, span)));
        self
    }

    /// The file the node lives in, if any.
    #[must_use]
    pub fn file(&self) -> Option<&RepoPath> {
        self.location.as_ref().map(Location::file)
    }

    /// The span of the node, when it is narrower than a file.
    #[must_use]
    pub fn span(&self) -> Option<TextSpan> {
        self.location.as_ref().and_then(Location::span)
    }

    /// The byte range of the node, when it is narrower than a file.
    #[must_use]
    pub fn bytes(&self) -> Option<ByteSpan> {
        self.span().map(|s| s.bytes)
    }

    /// The lines of the node, when it is narrower than a file.
    #[must_use]
    pub fn lines(&self) -> Option<LineSpan> {
        self.span().map(|s| s.lines)
    }

    /// The source location, when the node is narrower than a file.
    #[must_use]
    pub fn source(&self) -> Option<&SourceLocation> {
        self.location.as_ref().and_then(Location::source)
    }

    /// Adds an attribute and returns the node.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// A one-line human label: kind, qualified name and location.
    #[must_use]
    pub fn label(&self) -> String {
        self.location.as_ref().map_or_else(
            || format!("{} {}", self.kind, self.qualified_name),
            |loc| format!("{} {} ({})", self.kind, self.qualified_name, loc.citation()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip() {
        for kind in NodeKind::builtin() {
            let parsed: NodeKind = kind.as_str().parse().expect("builtin parses");
            assert_eq!(&parsed, kind);
        }
        let adapter: NodeKind = "causeway:OutboxRelay".parse().expect("adapter kind");
        assert_eq!(adapter, NodeKind::Adapter("causeway:OutboxRelay".into()));
        assert!("Nope".parse::<NodeKind>().is_err());
        let json = serde_json::to_string(&NodeKind::Function).expect("serializes");
        assert_eq!(json, "\"Function\"");
    }

    #[test]
    fn synthetic_node_has_stable_hash() {
        let a = Node::synthetic(
            NodeId::new("sql:s.t"),
            NodeKind::Table,
            "t",
            RepoId::new("r"),
        );
        let b = Node::synthetic(
            NodeId::new("sql:s.t"),
            NodeKind::Table,
            "t",
            RepoId::new("r"),
        );
        assert_eq!(a.content_hash, b.content_hash);
        assert_eq!(a.label(), "Table t");
    }
}
