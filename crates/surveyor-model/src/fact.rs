//! Facts: what an extractor finds in one source unit before linking.
//!
//! A [`FactSet`] is flat and local to a file. Declarations name things, references
//! name uses of things, literals capture strings with a syntactic role (an SQL string,
//! a configuration key), and declared relations are relationships a manifest states
//! outright. Turning facts into nodes and edges is the linker's job.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ByteSpan, CommitId, ContentHash, DeclaredSource, EdgeKind, Evidence, ExtractorId, GrammarId,
    Language, LineSpan, NodeKind, RepoId, RepoPath, ResolverId, RuleId, SourceLocation, TextSpan,
};

/// One file's bytes plus the coordinates needed to cite it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnit {
    /// Owning repository.
    pub repo: RepoId,
    /// Repository-relative path.
    pub path: RepoPath,
    /// Detected language, if any.
    pub language: Option<Language>,
    /// Commit the content was read at (or a working-tree marker).
    pub commit: CommitId,
    /// Raw file bytes.
    pub content: Vec<u8>,
    /// Hash of the whole file.
    pub content_hash: ContentHash,
}

impl SourceUnit {
    /// Creates a unit, detecting the language from the path and hashing the content.
    #[must_use]
    pub fn new(repo: RepoId, path: RepoPath, commit: CommitId, content: Vec<u8>) -> Self {
        let language = Language::from_path(&path);
        let content_hash = ContentHash::of(&content);
        Self {
            repo,
            path,
            language,
            commit,
            content,
            content_hash,
        }
    }

    /// Overrides the detected language and returns the unit.
    #[must_use]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// The content as text, or `None` when it is not valid UTF-8.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.content).ok()
    }

    /// The text inside `span`, or an empty string when the span is out of range.
    #[must_use]
    pub fn slice(&self, span: ByteSpan) -> &str {
        span.slice(&self.content)
            .and_then(|b| std::str::from_utf8(b).ok())
            .unwrap_or("")
    }

    /// One-based line span covering `span`, computed by counting newlines.
    #[must_use]
    #[expect(
        clippy::naive_bytecount,
        reason = "spans are short; not worth a dependency"
    )]
    pub fn lines_of(&self, span: ByteSpan) -> LineSpan {
        let count = |end: u64| -> u32 {
            let end = usize::try_from(end)
                .unwrap_or(usize::MAX)
                .min(self.content.len());
            let newlines = self.content[..end].iter().filter(|b| **b == b'\n').count();
            u32::try_from(newlines)
                .unwrap_or(u32::MAX - 1)
                .saturating_add(1)
        };
        let start = count(span.start);
        let end = count(span.end.saturating_sub(1).max(span.start));
        LineSpan::new(start, end.max(start))
    }

    /// Pairs `bytes` with the lines it covers in this unit.
    #[must_use]
    pub fn text_span(&self, bytes: ByteSpan) -> TextSpan {
        TextSpan::new(bytes, self.lines_of(bytes))
    }

    /// Hash of the text inside `bytes`.
    #[must_use]
    pub fn hash_of(&self, bytes: ByteSpan) -> ContentHash {
        ContentHash::of(bytes.slice(&self.content).unwrap_or_default())
    }

    /// Builds the evidence record for `bytes` inside this unit.
    #[must_use]
    pub fn evidence(&self, bytes: ByteSpan) -> Evidence {
        Evidence {
            repo: self.repo.clone(),
            location: SourceLocation::new(self.path.clone(), self.text_span(bytes)),
            commit: self.commit.clone(),
            content_hash: self.hash_of(bytes),
        }
    }
}

/// How a call names its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallForm {
    /// `f(...)` or `a::b::f(...)`: the path names the function.
    Path,
    /// `x.f(...)`: the target depends on the receiver's type.
    Method,
}

/// How a reference uses the thing it names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ReferenceRole {
    /// A call expression.
    Call(CallForm),
    /// An import or `use`.
    Import,
    /// A type named in a signature, field or annotation.
    TypeUse,
    /// Implementation of an interface or trait.
    Implements,
    /// Inheritance.
    Extends,
    /// A statement reads the named table or column.
    Read,
    /// A statement writes the named table or column.
    Write,
    /// Any other mention.
    Mention,
}

impl ReferenceRole {
    /// True for calls of either form.
    #[must_use]
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::Call(_))
    }

    /// The edge kind a reference of this role becomes once resolved.
    #[must_use]
    pub fn edge_kind(&self) -> EdgeKind {
        match self {
            Self::Call(_) => EdgeKind::Calls,
            Self::Import => EdgeKind::Imports,
            Self::TypeUse => EdgeKind::UsesType,
            Self::Implements => EdgeKind::Implements,
            Self::Extends => EdgeKind::Extends,
            Self::Read => EdgeKind::Reads,
            Self::Write => EdgeKind::Writes,
            Self::Mention => EdgeKind::References,
        }
    }
}

/// The syntactic role of a string literal worth keeping.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum LiteralRole {
    /// An SQL statement.
    Sql,
    /// A file path.
    Path,
    /// A URL or route.
    Url,
    /// A configuration or environment key.
    ConfigKey,
    /// A feature flag key.
    FlagKey,
    /// A topic or queue name.
    Topic,
    /// A citation of a design document.
    Citation,
    /// A shell command or binary name.
    Command,
    /// Any other literal an extractor chose to keep.
    Other(String),
}

/// Something declared in the unit: a function, a table, a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclarationFact {
    /// What is declared.
    pub kind: NodeKind,
    /// Short name.
    pub name: String,
    /// Enclosing declaration names, outermost first.
    pub scope: Vec<String>,
    /// The whole declaration.
    pub span: TextSpan,
    /// Byte range of the name token.
    pub name_span: ByteSpan,
    /// Signature line, when the language has one.
    pub signature: Option<String>,
    /// Documentation comment, when present.
    pub doc: Option<String>,
    /// The query rule or resolver rule that produced the fact.
    pub rule: RuleId,
    /// Open-ended attributes, such as a column type.
    pub attributes: BTreeMap<String, String>,
    /// Hash of the declaration text, so an unchanged declaration is recognized.
    pub hash: ContentHash,
}

impl DeclarationFact {
    /// The scope joined with `::` plus the name, such as `Relay::drain`.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        if self.scope.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.scope.join("::"), self.name)
        }
    }
}

/// A use of a name at a site inside the unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceFact {
    /// How the name is used.
    pub role: ReferenceRole,
    /// Last segment of the referenced path, such as `drain`.
    pub name: String,
    /// Full referenced path segments, such as `["outbox", "Relay", "drain"]`.
    pub path: Vec<String>,
    /// The reference.
    pub span: TextSpan,
    /// Enclosing declaration names at the reference site, outermost first.
    pub scope: Vec<String>,
    /// The query rule that produced the fact.
    pub rule: RuleId,
    /// Hash of the referenced text, for evidence.
    pub hash: ContentHash,
}

/// A string literal with a syntactic role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralFact {
    /// Why the literal matters.
    pub role: LiteralRole,
    /// The literal's value without quotes.
    pub value: String,
    /// The literal.
    pub span: TextSpan,
    /// Enclosing declaration names at the site, outermost first.
    pub scope: Vec<String>,
    /// The query rule that produced the fact.
    pub rule: RuleId,
    /// Hash of the literal text, for evidence.
    pub hash: ContentHash,
}

/// A relationship a manifest or configuration states outright.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredRelation {
    /// Relationship kind.
    pub kind: EdgeKind,
    /// Kind of the source node.
    pub src_kind: NodeKind,
    /// Name of the source node within its namespace.
    pub src_name: String,
    /// Kind of the target node.
    pub dst_kind: NodeKind,
    /// Name of the target node within its namespace.
    pub dst_name: String,
    /// What kind of file declared it.
    pub source: DeclaredSource,
    /// The declaring text.
    pub span: TextSpan,
    /// Hash of the declaring text, for evidence.
    pub hash: ContentHash,
}

/// Where a precise resolver says a reference points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "kebab-case")]
pub enum SymbolTarget {
    /// A declaration in this repository: the file and the one-based line of its name,
    /// plus the name, which together pick out one Tree-sitter declaration.
    Declared {
        file: RepoPath,
        line: u32,
        name: String,
    },
    /// A symbol outside the repository (the standard library, a dependency), as the
    /// resolver spells it.
    External { symbol: String },
    /// A local binding (a variable or parameter); never a node.
    Local,
}

/// A reference site a precise resolver has bound to its target.
///
/// The linker pairs it with the Tree-sitter reference at the same site, so the edge
/// kind comes from syntax and the target from the resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedReference {
    /// Short name of the symbol at the site, such as `drain`.
    pub name: String,
    /// The reference token.
    pub span: TextSpan,
    /// What it refers to.
    pub target: SymbolTarget,
    /// The resolver that said so, such as `scip/rust-analyzer`.
    pub resolver: ResolverId,
    /// Hash of the referenced text, for evidence.
    pub hash: ContentHash,
}

/// One extracted fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fact", rename_all = "kebab-case")]
pub enum Fact {
    Declaration(DeclarationFact),
    Reference(ReferenceFact),
    Literal(LiteralFact),
    Relation(DeclaredRelation),
    Resolved(ResolvedReference),
}

/// Everything one extractor found in one source unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactSet {
    /// The extractor that produced the facts.
    pub extractor: ExtractorId,
    /// The grammar used, when the extractor is Tree-sitter based.
    pub grammar: Option<GrammarId>,
    /// Owning repository.
    pub repo: RepoId,
    /// File the facts came from.
    pub path: RepoPath,
    /// Language of the file.
    pub language: Option<Language>,
    /// Commit the file was read at.
    pub commit: CommitId,
    /// Hash of the whole file.
    pub content_hash: ContentHash,
    /// The facts, in source order where it matters.
    pub facts: Vec<Fact>,
}

impl FactSet {
    /// Creates an empty fact set for `unit`.
    #[must_use]
    pub fn empty(extractor: ExtractorId, unit: &SourceUnit) -> Self {
        Self {
            extractor,
            grammar: None,
            repo: unit.repo.clone(),
            path: unit.path.clone(),
            language: unit.language.clone(),
            commit: unit.commit.clone(),
            content_hash: unit.content_hash,
            facts: Vec::new(),
        }
    }

    /// Iterates over the declarations only.
    pub fn declarations(&self) -> impl Iterator<Item = &DeclarationFact> {
        self.facts.iter().filter_map(|f| match f {
            Fact::Declaration(d) => Some(d),
            _ => None,
        })
    }

    /// Iterates over the references only.
    pub fn references(&self) -> impl Iterator<Item = &ReferenceFact> {
        self.facts.iter().filter_map(|f| match f {
            Fact::Reference(r) => Some(r),
            _ => None,
        })
    }

    /// Iterates over the literals only.
    pub fn literals(&self) -> impl Iterator<Item = &LiteralFact> {
        self.facts.iter().filter_map(|f| match f {
            Fact::Literal(l) => Some(l),
            _ => None,
        })
    }

    /// Iterates over the declared relations only.
    pub fn relations(&self) -> impl Iterator<Item = &DeclaredRelation> {
        self.facts.iter().filter_map(|f| match f {
            Fact::Relation(r) => Some(r),
            _ => None,
        })
    }

    /// Iterates over the precisely resolved references only.
    pub fn resolved(&self) -> impl Iterator<Item = &ResolvedReference> {
        self.facts.iter().filter_map(|f| match f {
            Fact::Resolved(r) => Some(r),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("a.rs"),
            CommitId::new("c"),
            text.into(),
        )
    }

    #[test]
    fn lines_are_one_based_and_inclusive() {
        let u = unit("fn a() {}\nfn b() {\n}\n");
        assert_eq!(u.lines_of(ByteSpan::new(0, 9)), LineSpan::new(1, 1));
        assert_eq!(u.lines_of(ByteSpan::new(10, 21)), LineSpan::new(2, 3));
        let ev = u.evidence(ByteSpan::new(10, 21));
        assert_eq!(ev.citation(), "a.rs:2-3");
        assert_eq!(
            u.text_span(ByteSpan::new(10, 21)).lines,
            LineSpan::new(2, 3)
        );
        assert_eq!(ev.content_hash, ContentHash::of(b"fn b() {\n}\n"));
    }

    #[test]
    fn qualified_name_joins_scope() {
        let d = DeclarationFact {
            kind: NodeKind::Method,
            name: "drain".into(),
            scope: vec!["outbox".into(), "Relay".into()],
            span: TextSpan::new(ByteSpan::new(0, 1), LineSpan::new(1, 1)),
            name_span: ByteSpan::new(0, 1),
            signature: None,
            doc: None,
            rule: RuleId::new("definition.method"),
            attributes: BTreeMap::new(),
            hash: ContentHash::of(b"fn drain"),
        };
        assert_eq!(d.qualified_name(), "outbox::Relay::drain");
        assert_eq!(
            ReferenceRole::Call(CallForm::Method).edge_kind(),
            EdgeKind::Calls
        );
    }
}
