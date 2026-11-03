//! The Tree-sitter host: one engine, one query pack per language.
//!
//! A query pack (`languages/<name>/surveyor.scm`) tags syntax with capture names the
//! host understands: `definition.<kind>`, `reference.<role>`, `literal.<role>`, plus
//! `name`, `doc` and `attr.<key>` companions inside the same pattern. Adding a language
//! is a grammar crate and a query file; nothing here is language specific except the
//! small import-path expanders and the docstring rules.

use std::collections::BTreeMap;

use streaming_iterator::StreamingIterator;
use surveyor_model::{
    ByteSpan, CallForm, ContentHash, DeclarationFact, ExtractorId, Fact, FactSet, GrammarId,
    Language, LiteralFact, LiteralRole, NodeKind, ReferenceFact, ReferenceRole, RuleId, SourceUnit,
    TextSpan,
};
use tree_sitter::{Node, Parser, Query, QueryCursor};

use crate::{ExtractError, Extractor};

/// The extractor identifier prefix; the grammar name follows a slash.
const EXTRACTOR_PREFIX: &str = "tree-sitter";

/// Longest signature kept, in bytes; longer ones are cut with an ellipsis.
///
/// Signatures feed symbol cards and prompts, where a 300-byte line is already wide.
const MAX_SIGNATURE_BYTES: usize = 300;

/// Longest documentation comment kept, in bytes.
const MAX_DOC_BYTES: usize = 4_000;

/// Prefix that turns a macro's token tree into a parsable call expression.
const MACRO_WRAPPER: &str = "__m(";

/// Macro arguments nested this deep are no longer re-parsed.
const MAX_MACRO_DEPTH: u8 = 3;

/// What a capture name asks the host to record.
#[derive(Debug, Clone)]
enum CaptureRole {
    /// A declaration; the language, when given, is the one that sees it under this
    /// name (`definition.function.rhai` on a Rust `register_fn("name", ..)`).
    Definition(NodeKind, Option<Language>),
    Reference(ReferenceRole),
    Literal(LiteralRole),
    Name,
    Doc,
    Attr(String),
    Receiver,
    Ignore,
}

fn role_for_capture(name: &str) -> Result<CaptureRole, ExtractError> {
    let unknown = |what: &str| ExtractError::query(format!("unknown {what} in capture @{name}"));
    if name.starts_with('_') {
        return Ok(CaptureRole::Ignore);
    }
    if let Some(kind) = name.strip_prefix("definition.") {
        let (kind, exposed) = match kind.split_once('.') {
            Some((kind, language)) => (
                kind,
                Some(
                    language
                        .parse::<Language>()
                        .map_err(|_unparsed| unknown("exposed language"))?,
                ),
            ),
            None => (kind, None),
        };
        return definition_kind(kind)
            .map(|kind| CaptureRole::Definition(kind, exposed))
            .ok_or_else(|| unknown("definition kind"));
    }
    if let Some(role) = name.strip_prefix("reference.") {
        return reference_role(role)
            .map(CaptureRole::Reference)
            .ok_or_else(|| unknown("reference role"));
    }
    if let Some(role) = name.strip_prefix("literal.") {
        return Ok(CaptureRole::Literal(literal_role(role)));
    }
    if let Some(key) = name.strip_prefix("attr.") {
        return Ok(CaptureRole::Attr(key.to_owned()));
    }
    match name {
        "name" => Ok(CaptureRole::Name),
        "doc" => Ok(CaptureRole::Doc),
        "receiver" => Ok(CaptureRole::Receiver),
        _ => Err(unknown("capture")),
    }
}

fn definition_kind(kind: &str) -> Option<NodeKind> {
    Some(match kind {
        "function" => NodeKind::Function,
        "method" => NodeKind::Method,
        "class" => NodeKind::Class,
        "struct" => NodeKind::Struct,
        "enum" => NodeKind::Enum,
        "interface" => NodeKind::Interface,
        "trait" => NodeKind::Trait,
        "impl" => NodeKind::Impl,
        "type" => NodeKind::TypeAlias,
        "const" | "constant" => NodeKind::Const,
        "macro" => NodeKind::Macro,
        "test" => NodeKind::Test,
        "module" => NodeKind::Module,
        "variable" => NodeKind::Variable,
        "variant" => NodeKind::Variant,
        _ => return None,
    })
}

fn reference_role(role: &str) -> Option<ReferenceRole> {
    Some(match role {
        "call" => ReferenceRole::Call(CallForm::Path),
        "call.method" => ReferenceRole::Call(CallForm::Method),
        "call.macro" => ReferenceRole::Call(CallForm::Macro),
        "import" => ReferenceRole::Import,
        "type" => ReferenceRole::TypeUse,
        "implements" => ReferenceRole::Implements,
        "extends" => ReferenceRole::Extends,
        "mention" => ReferenceRole::Mention,
        _ => return None,
    })
}

fn literal_role(role: &str) -> LiteralRole {
    match role {
        "sql" => LiteralRole::Sql,
        "path" => LiteralRole::Path,
        "url" => LiteralRole::Url,
        "config-key" => LiteralRole::ConfigKey,
        "flag-key" => LiteralRole::FlagKey,
        "topic" => LiteralRole::Topic,
        "citation" => LiteralRole::Citation,
        "command" => LiteralRole::Command,
        other => LiteralRole::Other(other.to_owned()),
    }
}

struct Pack {
    grammar: GrammarId,
    languages: Vec<Language>,
    ts: tree_sitter::Language,
    query: Query,
    roles: Vec<CaptureRole>,
    capture_names: Vec<String>,
}

impl Pack {
    fn compile(
        grammar: &str,
        languages: &[Language],
        ts: tree_sitter::Language,
        source: &str,
    ) -> Result<Self, ExtractError> {
        let context = |e: ExtractError| ExtractError::query(format!("{grammar}: {e}"));
        let query =
            Query::new(&ts, source).map_err(|e| ExtractError::query(format!("{grammar}: {e}")))?;
        let capture_names: Vec<String> = query
            .capture_names()
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let roles = capture_names
            .iter()
            .map(|n| role_for_capture(n).map_err(context))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            grammar: GrammarId::new(grammar),
            languages: languages.to_vec(),
            ts,
            query,
            roles,
            capture_names,
        })
    }

    fn extractor_id(&self) -> ExtractorId {
        ExtractorId::new(format!("{EXTRACTOR_PREFIX}/{}", self.grammar))
    }
}

/// Runs the embedded query packs over any file whose language has one.
pub struct TreeSitterExtractor {
    packs: Vec<Pack>,
}

impl std::fmt::Debug for TreeSitterExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeSitterExtractor")
            .field(
                "grammars",
                &self.grammars().map(GrammarId::as_str).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Default for TreeSitterExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterExtractor {
    /// Compiles the embedded packs for Rust, TypeScript, JavaScript, Python and Bash.
    ///
    /// # Panics
    /// Panics if an embedded query pack fails to compile, which is a build defect the
    /// crate's tests catch.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("embedded query packs compile")
    }

    /// Compiles the embedded packs, returning the first pack that fails.
    pub fn try_new() -> Result<Self, ExtractError> {
        let packs = vec![
            Pack::compile(
                "rust",
                &[Language::Rust],
                tree_sitter_rust::LANGUAGE.into(),
                include_str!("../../../languages/rust/surveyor.scm"),
            )?,
            Pack::compile(
                "typescript",
                &[Language::TypeScript],
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                include_str!("../../../languages/typescript/surveyor.scm"),
            )?,
            Pack::compile(
                "tsx",
                &[Language::Tsx],
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                include_str!("../../../languages/typescript/surveyor.scm"),
            )?,
            Pack::compile(
                "javascript",
                &[Language::JavaScript],
                tree_sitter_javascript::LANGUAGE.into(),
                include_str!("../../../languages/javascript/surveyor.scm"),
            )?,
            Pack::compile(
                "python",
                &[Language::Python],
                tree_sitter_python::LANGUAGE.into(),
                include_str!("../../../languages/python/surveyor.scm"),
            )?,
            Pack::compile(
                "bash",
                &[Language::Bash],
                tree_sitter_bash::LANGUAGE.into(),
                include_str!("../../../languages/bash/surveyor.scm"),
            )?,
        ];
        Ok(Self { packs })
    }

    /// The grammars this host can run.
    pub fn grammars(&self) -> impl Iterator<Item = &GrammarId> {
        self.packs.iter().map(|p| &p.grammar)
    }

    fn pack_for(&self, language: &Language) -> Option<&Pack> {
        self.packs.iter().find(|p| p.languages.contains(language))
    }

    /// Extracts facts from `unit` using the pack for its language.
    ///
    /// # Errors
    /// Returns an error when the unit has no language pack or the parser gives up
    /// (a timeout or an invalid language), never for syntax errors in the file.
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let language = unit
            .language
            .as_ref()
            .ok_or_else(|| ExtractError::parse("no language detected"))?;
        let pack = self
            .pack_for(language)
            .ok_or_else(|| ExtractError::parse(format!("no query pack for {language}")))?;
        let mut parser = Parser::new();
        parser
            .set_language(&pack.ts)
            .map_err(|e| ExtractError::parse(format!("{}: {e}", pack.grammar)))?;
        let tree = parser
            .parse(&unit.content, None)
            .ok_or_else(|| ExtractError::parse("parser returned no tree"))?;

        let site = Site {
            unit,
            language,
            delta: 0,
        };
        let mut raw = RawFacts::default();
        collect(
            pack,
            &tree.root_node(),
            &unit.content,
            site,
            &mut raw,
            Pass::Full,
        );
        if pack.grammar.as_str() == "rust" {
            macro_bodies(
                pack,
                &mut parser,
                &tree.root_node(),
                &unit.content,
                site,
                &mut raw,
                0,
            );
        }

        Ok(FactSet {
            grammar: Some(pack.grammar.clone()),
            facts: raw.into_facts(),
            ..FactSet::empty(pack.extractor_id(), unit)
        })
    }
}

impl Extractor for TreeSitterExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_PREFIX)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        unit.language
            .as_ref()
            .is_some_and(|l| self.pack_for(l).is_some())
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

/// Whether a walk records declarations or only the references and literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pass {
    /// The file itself: everything.
    Full,
    /// Text re-parsed out of context: references and literals only.
    ReferencesOnly,
}

/// The unit a walk reads from and how its offsets map back into it.
#[derive(Clone, Copy)]
struct Site<'u> {
    unit: &'u SourceUnit,
    language: &'u Language,
    /// Added to a node's offsets to find the text in the unit (zero for the file itself).
    delta: i64,
}

impl Site<'_> {
    /// The unit span a node covers.
    fn span_of(&self, node: Node<'_>) -> TextSpan {
        self.unit.text_span(shift(byte_span_of(node), self.delta))
    }

    fn text(&self, span: TextSpan) -> &str {
        self.unit.slice(span.bytes)
    }
}

/// One query match, sorted into the roles the host understands.
#[derive(Default)]
struct Captures<'t> {
    primary: Option<(usize, Node<'t>)>,
    name: Option<Node<'t>>,
    doc: Option<Node<'t>>,
    receiver: Option<Node<'t>>,
    attrs: Vec<(String, Node<'t>)>,
}

impl<'t> Captures<'t> {
    fn from_match(pack: &Pack, m: &tree_sitter::QueryMatch<'_, 't>) -> Self {
        m.captures().iter().fold(Self::default(), |mut acc, cap| {
            let idx = cap.index as usize;
            match &pack.roles[idx] {
                CaptureRole::Definition(..)
                | CaptureRole::Reference(_)
                | CaptureRole::Literal(_) => {
                    acc.primary = Some((idx, cap.node));
                }
                CaptureRole::Name => acc.name = Some(cap.node),
                CaptureRole::Doc => acc.doc = Some(cap.node),
                CaptureRole::Receiver => acc.receiver = Some(cap.node),
                CaptureRole::Attr(key) => acc.attrs.push((key.clone(), cap.node)),
                CaptureRole::Ignore => {}
            }
            acc
        })
    }
}

/// Runs the pack's query over `root` and records the matches.
fn collect(
    pack: &Pack,
    root: &Node<'_>,
    bytes: &[u8],
    site: Site<'_>,
    raw: &mut RawFacts,
    pass: Pass,
) {
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&pack.query, *root, bytes);
    while let Some(m) = matches.next() {
        let captures = Captures::from_match(pack, m);
        let Some((idx, node)) = captures.primary else {
            continue;
        };
        // Re-parsed text starts with a synthetic wrapper; nothing inside it is real.
        if pass == Pass::ReferencesOnly && node.start_byte() < MACRO_WRAPPER.len() {
            continue;
        }
        let rule = RuleId::new(&pack.capture_names[idx]);
        match (&pack.roles[idx], pass) {
            (CaptureRole::Definition(kind, exposed), Pass::Full) => {
                raw.push_definition(site, kind.clone(), exposed.as_ref(), node, &captures, rule);
            }
            (CaptureRole::Reference(role), _) => {
                raw.push_reference(site, role, node, captures.name, captures.receiver, &rule);
            }
            (CaptureRole::Literal(role), _) => raw.push_literal(site, role.clone(), node, rule),
            _ => {}
        }
    }
}

/// Re-parses Rust macro arguments, which the grammar keeps as opaque token trees.
///
/// `assert_eq!(relay.drain(10), 0)` holds a call the grammar never sees. Each token
/// tree is wrapped as `__m(<tokens>);`, parsed as Rust, and queried for references
/// and literals; spans map back to the original file.
fn macro_bodies(
    pack: &Pack,
    parser: &mut Parser,
    root: &Node<'_>,
    bytes: &[u8],
    site: Site<'_>,
    raw: &mut RawFacts,
    depth: u8,
) {
    if depth >= MAX_MACRO_DEPTH {
        return;
    }
    for (start, end) in token_trees(root) {
        let Some(inner) = bytes.get(start..end) else {
            continue;
        };
        let wrapped: Vec<u8> = [MACRO_WRAPPER.as_bytes(), inner, b");"].concat();
        let Some(sub) = parser.parse(&wrapped, None) else {
            continue;
        };
        let sub_site = Site {
            delta: site.delta + start as i64 - MACRO_WRAPPER.len() as i64,
            ..site
        };
        collect(
            pack,
            &sub.root_node(),
            &wrapped,
            sub_site,
            raw,
            Pass::ReferencesOnly,
        );
        macro_bodies(
            pack,
            parser,
            &sub.root_node(),
            &wrapped,
            sub_site,
            raw,
            depth + 1,
        );
    }
}

/// Byte ranges of the token trees of every macro invocation under `root`, delimiters
/// excluded, outermost only.
fn token_trees(root: &Node<'_>) -> Vec<(usize, usize)> {
    let mut stack = vec![*root];
    let mut out = Vec::new();
    while let Some(node) = stack.pop() {
        let mut cursor = node.walk();
        if node.kind() == "macro_invocation" {
            out.extend(
                node.children(&mut cursor)
                    .find(|n| n.kind() == "token_tree")
                    .filter(|tt| tt.end_byte() > tt.start_byte() + 2)
                    .map(|tt| (tt.start_byte() + 1, tt.end_byte() - 1)),
            );
        } else {
            stack.extend(node.children(&mut cursor));
        }
    }
    out
}

/// A reference or literal before its scope is known.
#[derive(Debug)]
enum RawSite {
    Reference(ReferenceFact),
    Literal(LiteralFact),
}

impl RawSite {
    fn start(&self) -> u64 {
        self.span().start()
    }

    fn span(&self) -> TextSpan {
        match self {
            Self::Reference(r) => r.span,
            Self::Literal(l) => l.span,
        }
    }

    fn with_scope(self, scope: Vec<String>) -> Self {
        match self {
            Self::Reference(r) => Self::Reference(ReferenceFact { scope, ..r }),
            Self::Literal(l) => Self::Literal(LiteralFact { scope, ..l }),
        }
    }

    fn into_fact(self) -> Fact {
        match self {
            Self::Reference(r) => Fact::Reference(r),
            Self::Literal(l) => Fact::Literal(l),
        }
    }
}

#[derive(Debug, Default)]
struct RawFacts {
    defs: Vec<DeclarationFact>,
    sites: Vec<RawSite>,
}

impl RawFacts {
    fn push_definition(
        &mut self,
        site: Site<'_>,
        kind: NodeKind,
        exposed: Option<&Language>,
        node: Node<'_>,
        captures: &Captures<'_>,
        rule: RuleId,
    ) {
        let span = site.span_of(node);
        let (name, name_span) = captures.name.map_or_else(
            || (first_word(site.text(span)), span.bytes),
            |n| {
                (
                    definition_name(site.unit.slice(byte_span_of(n))),
                    byte_span_of(n),
                )
            },
        );
        if name.is_empty() {
            return;
        }
        let doc = captures
            .doc
            .map(|n| clean_doc(site.unit.slice(byte_span_of(n))))
            .or_else(|| docstring(site.language, node, site.unit))
            .or_else(|| leading_comment_doc(node, site.unit))
            .filter(|d| !d.is_empty());
        let attributes = captures
            .attrs
            .iter()
            .map(|(key, n)| {
                (
                    key.clone(),
                    definition_name(site.unit.slice(byte_span_of(*n))),
                )
            })
            .chain(std::iter::once((
                "syntax".to_owned(),
                node.kind().to_owned(),
            )))
            .chain(exposed.map(|language| ("language".to_owned(), language.as_str().to_owned())))
            .collect::<BTreeMap<_, _>>();
        self.defs.push(DeclarationFact {
            kind,
            name,
            scope: Vec::new(),
            span,
            name_span,
            signature: Some(signature(node, site.unit)),
            doc,
            rule,
            attributes,
            hash: site.unit.hash_of(span.bytes),
        });
    }

    fn push_reference(
        &mut self,
        site: Site<'_>,
        role: &ReferenceRole,
        node: Node<'_>,
        name_node: Option<Node<'_>>,
        receiver_node: Option<Node<'_>>,
        rule: &RuleId,
    ) {
        let span = site.span_of(name_node.unwrap_or(node));
        let text = site.text(span);
        let receiver = receiver_node.and_then(|n| receiver_root(site.text(site.span_of(n))));
        let paths: Vec<(Vec<String>, Option<String>)> = match role {
            ReferenceRole::Import => expand_import(site.language, text),
            _ => vec![(reference_path(text), None)],
        };
        let hash = ContentHash::of(text);
        self.sites
            .extend(paths.into_iter().filter_map(|(path, binding)| {
                let name = path.iter().rev().find(|s| !is_path_noise(s)).cloned()?;
                Some(RawSite::Reference(ReferenceFact {
                    role: role.clone(),
                    name,
                    path,
                    span,
                    scope: Vec::new(),
                    rule: rule.clone(),
                    hash,
                    receiver: receiver.clone(),
                    binding,
                }))
            }));
    }

    fn push_literal(&mut self, site: Site<'_>, role: LiteralRole, node: Node<'_>, rule: RuleId) {
        let span = site.span_of(node);
        let value = unquote(site.text(span));
        if value.trim().is_empty() {
            return;
        }
        let hash = ContentHash::of(&value);
        self.sites.push(RawSite::Literal(LiteralFact {
            role,
            value,
            span,
            scope: Vec::new(),
            rule,
            hash,
        }));
    }

    /// Dedupes definitions by span, assigns scopes by span containment, and flattens.
    fn into_facts(self) -> Vec<Fact> {
        let mut defs = dedupe_definitions(self.defs);
        defs.sort_by(|a, b| {
            a.span
                .start()
                .cmp(&b.span.start())
                .then(b.span.end().cmp(&a.span.end()))
        });
        let mut sites = self.sites;
        // Outer paths before the inner paths they contain, so containment is one look back.
        sites.sort_by_key(|s| (s.start(), std::cmp::Reverse(s.span().end())));
        let sites = drop_shadowed_mentions(sites);
        let (defs, sites) = assign_scopes(defs, sites);
        defs.into_iter()
            .map(Fact::Declaration)
            .chain(sites.into_iter().map(RawSite::into_fact))
            .collect()
    }
}

/// Drops `Mention` sites that another site already covers: a call's path, an import's
/// path, or the inner segments of a longer path. `sites` must be sorted by start.
fn drop_shadowed_mentions(sites: Vec<RawSite>) -> Vec<RawSite> {
    let is_mention =
        |s: &RawSite| matches!(s, RawSite::Reference(r) if r.role == ReferenceRole::Mention);
    let covers = |outer: &RawSite, inner: &RawSite| {
        let (o, i) = (outer.span().bytes, inner.span().bytes);
        o.start <= i.start && i.end <= o.end
    };
    // Same-span duplicates: the non-mention wins regardless of order.
    let mut kept: Vec<RawSite> = Vec::with_capacity(sites.len());
    for site in sites {
        match kept.last() {
            Some(last) if is_mention(&site) && covers(last, &site) => {}
            Some(last) if is_mention(last) && last.span().bytes == site.span().bytes => {
                kept.pop();
                kept.push(site);
            }
            _ => kept.push(site),
        }
    }
    kept
}

/// Open declarations during the sweep, outermost first.
#[derive(Default)]
struct ScopeStack(Vec<(ByteSpan, String)>);

impl ScopeStack {
    /// Drops declarations that end at or before `at`.
    fn close_before(&mut self, at: u64) {
        while self.0.last().is_some_and(|(s, _)| s.end <= at) {
            self.0.pop();
        }
    }

    fn names(&self) -> Vec<String> {
        self.0.iter().map(|(_, n)| n.clone()).collect()
    }

    fn open(&mut self, span: ByteSpan, name: &str) {
        self.0.push((span, name.to_owned()));
    }
}

/// Sweeps declarations and sites together in start order, giving each its enclosing
/// declaration names.
///
/// `defs` must be sorted by start ascending then end descending, `sites` by start.
fn assign_scopes(
    mut defs: Vec<DeclarationFact>,
    sites: Vec<RawSite>,
) -> (Vec<DeclarationFact>, Vec<RawSite>) {
    let mut stack = ScopeStack::default();
    let mut pending = sites.into_iter().peekable();
    let mut scoped = Vec::with_capacity(pending.len());
    let scope_site = |stack: &mut ScopeStack, site: RawSite| {
        stack.close_before(site.start());
        let scope = stack.names();
        site.with_scope(scope)
    };
    for def in &mut defs {
        while let Some(site) = pending.next_if(|s| s.start() < def.span.start()) {
            scoped.push(scope_site(&mut stack, site));
        }
        stack.close_before(def.span.start());
        def.scope = stack.names();
        stack.open(def.span.bytes, &def.name);
    }
    scoped.extend(pending.map(|site| scope_site(&mut stack, site)));
    (defs, scoped)
}

/// Keeps one declaration per span, preferring the most specific kind.
fn dedupe_definitions(defs: Vec<DeclarationFact>) -> Vec<DeclarationFact> {
    defs.into_iter()
        .fold(
            BTreeMap::<(u64, u64), DeclarationFact>::new(),
            |mut by_span, def| {
                by_span
                    .entry((def.span.start(), def.span.end()))
                    .and_modify(|existing| merge_definition(existing, &def))
                    .or_insert(def);
                by_span
            },
        )
        .into_values()
        .collect()
}

/// Merges `incoming` into `existing`: the more specific kind wins, attributes union.
fn merge_definition(existing: &mut DeclarationFact, incoming: &DeclarationFact) {
    let attributes: BTreeMap<String, String> = existing
        .attributes
        .iter()
        .chain(incoming.attributes.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    if kind_rank(&incoming.kind) > kind_rank(&existing.kind) {
        *existing = incoming.clone();
    }
    existing.attributes = attributes;
}

fn kind_rank(kind: &NodeKind) -> u8 {
    match kind {
        NodeKind::Test => 3,
        NodeKind::Method => 2,
        NodeKind::Function => 1,
        _ => 0,
    }
}

fn byte_span_of(node: Node<'_>) -> ByteSpan {
    ByteSpan::new(node.start_byte() as u64, node.end_byte() as u64)
}

/// Shifts a span by `delta` bytes, clamping at zero.
fn shift(span: ByteSpan, delta: i64) -> ByteSpan {
    let start = (span.start as i64 + delta).max(0) as u64;
    let end = (span.end as i64 + delta).max(start as i64) as u64;
    ByteSpan::new(start, end)
}

/// Normalizes a declaration name: strips generics, quotes and leading paths.
fn definition_name(text: &str) -> String {
    let no_generics = strip_angle_brackets(&unquote(text.trim()));
    no_generics
        .rsplit("::")
        .next()
        .unwrap_or(&no_generics)
        .trim_start_matches(['&', '*', '('])
        .trim_end_matches(')')
        .trim()
        .to_owned()
}

/// Splits a referenced path into segments, dropping generic arguments.
fn reference_path(text: &str) -> Vec<String> {
    strip_angle_brackets(text.trim())
        .split([':', '.'])
        .map(|s| {
            s.trim()
                .trim_start_matches(['&', '*', '('])
                .trim_end_matches(')')
        })
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The leading identifier of a receiver expression, when the expression starts with
/// one that is not immediately called or indexed: `a` for `a.b.c`, `self` for
/// `self.inner`, `None` for `f().g`, `(a).b` or `"x".len`.
fn receiver_root(text: &str) -> Option<String> {
    let text = text.trim_start();
    let end = text
        .char_indices()
        .take_while(|(i, c)| c.is_alphanumeric() || *c == '_' || (*i > 0 && *c == '$'))
        .map(|(i, c)| i + c.len_utf8())
        .last()?;
    let root = &text[..end];
    let rest = text[end..].trim_start();
    let starts_identifier = root.chars().next().is_some_and(|c| !c.is_ascii_digit());
    let followed_by_access = rest.is_empty() || rest.starts_with(['.', ':', '?']);
    (starts_identifier && followed_by_access).then(|| root.to_owned())
}

fn is_path_noise(segment: &str) -> bool {
    matches!(segment, "*" | "self" | "super" | "crate" | "." | "..")
}

fn strip_angle_brackets(text: &str) -> String {
    text.chars()
        .scan(0usize, |depth, c| {
            Some(match c {
                '<' => {
                    *depth += 1;
                    None
                }
                '>' if *depth > 0 => {
                    *depth -= 1;
                    None
                }
                _ if *depth == 0 => Some(c),
                _ => None,
            })
        })
        .flatten()
        .collect()
}

fn first_word(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// Removes the quoting syntax of string literals across the supported languages.
///
/// Escape sequences are decoded unless the literal is raw (`r"…"`, `r#"…"#`, `rb"…"`),
/// so a query written across several lines with `\` continuations reads as the one
/// line the program sees.
fn unquote(text: &str) -> String {
    let t = text.trim();
    // A literal prefix (`r#`, `b`, `f`, `R`) is only a prefix when a quote follows it.
    let prefix_len = t
        .char_indices()
        .take_while(|(_, c)| matches!(c, 'b' | 'r' | 'f' | 'u' | 'R' | 'B' | 'F' | 'U' | '#'))
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    let body = &t[prefix_len..];
    let quoted = body.starts_with(['"', '\'', '`']);
    let raw = quoted && t[..prefix_len].contains(['r', 'R']);
    let t = if quoted {
        body.trim_end_matches('#')
    } else {
        t
    };
    let inner = ["\"\"\"", "'''", "\"", "'", "`"]
        .iter()
        .find(|q| t.len() >= 2 * q.len() && t.starts_with(*q) && t.ends_with(*q))
        .map_or(t, |q| &t[q.len()..t.len() - q.len()]);
    if raw || !quoted {
        inner.to_owned()
    } else {
        unescape(inner)
    }
}

/// Decodes the escape sequences string literals share across languages.
///
/// A backslash before a line break continues the string: the break is dropped and
/// the next line's indentation is kept, which reads as extra spaces in Rust (where
/// the compiler would drop it too) and as the author's spacing in Python and
/// JavaScript. Sequences this does not know (`\u{…}`, `\x..`) are kept verbatim.
fn unescape(body: &str) -> String {
    let (out, pending) = body.chars().fold(
        (String::with_capacity(body.len()), false),
        |(mut out, escaped), c| match (escaped, c) {
            (false, '\\') => (out, true),
            (false, c) => {
                out.push(c);
                (out, false)
            }
            (true, '\n' | '\r') => (out, false),
            (true, c) => {
                match c {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '0' => out.push('\0'),
                    '\\' | '"' | '\'' | '`' => out.push(c),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
                (out, false)
            }
        },
    );
    if pending { out + "\\" } else { out }
}

/// Cuts `text` at a character boundary at or before `max` bytes, marking the cut.
fn truncate_bytes(mut text: String, max: usize) -> String {
    if text.len() <= max {
        return text;
    }
    let cut = (0..=max)
        .rev()
        .find(|i| text.is_char_boundary(*i))
        .unwrap_or(0);
    text.truncate(cut);
    text.push('…');
    text
}

/// The declaration text up to its body, collapsed to one line.
fn signature(node: Node<'_>, unit: &SourceUnit) -> String {
    let start = node.start_byte();
    let end = node
        .child_by_field_name("body")
        .map_or(node.end_byte(), |b| b.start_byte())
        .max(start);
    let line = unit
        .slice(ByteSpan::new(start as u64, end as u64))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(['{', ':', '=', ' '])
        .to_owned();
    truncate_bytes(line, MAX_SIGNATURE_BYTES)
}

/// Python and similar: the first string statement of the body is the docstring.
fn docstring(language: &Language, node: Node<'_>, unit: &SourceUnit) -> Option<String> {
    (*language == Language::Python)
        .then(|| node.child_by_field_name("body"))
        .flatten()
        .and_then(|body| body.named_child(0))
        .filter(|first| first.kind() == "expression_statement")
        .and_then(|first| first.named_child(0))
        .filter(|string| string.kind() == "string")
        .map(|string| clean_doc(&unquote(unit.slice(byte_span_of(string)))))
}

/// Comments immediately preceding the node, skipping attributes and decorators.
fn leading_comment_doc(node: Node<'_>, unit: &SourceUnit) -> Option<String> {
    // Comments sit before the outermost wrapper (`export`, a decorator, `const x =`).
    let anchor = std::iter::successors(Some(node), |n| {
        n.prev_named_sibling()
            .is_none()
            .then(|| n.parent())
            .flatten()
            .filter(|p| {
                matches!(
                    p.kind(),
                    "export_statement"
                        | "decorated_definition"
                        | "lexical_declaration"
                        | "variable_declaration"
                        | "expression_statement"
                )
            })
    })
    .last()
    .unwrap_or(node);

    let mut pieces: Vec<&str> = Vec::new();
    let mut expected_row = anchor.start_position().row;
    let mut cursor = anchor.prev_named_sibling();
    while let Some(sib) = cursor {
        let kind = sib.kind();
        if !matches!(kind, "attribute_item" | "decorator") {
            if !kind.contains("comment") || sib.end_position().row + 1 < expected_row {
                break;
            }
            pieces.push(unit.slice(byte_span_of(sib)));
        }
        expected_row = sib.start_position().row;
        cursor = sib.prev_named_sibling();
    }
    (!pieces.is_empty()).then(|| {
        pieces.reverse();
        clean_doc(&pieces.join("\n"))
    })
}

fn clean_doc(raw: &str) -> String {
    let cleaned = raw
        .lines()
        .map(|line| {
            line.trim()
                .trim_start_matches("/**")
                .trim_start_matches("/*")
                .trim_end_matches("*/")
                .trim()
                .trim_start_matches("///")
                .trim_start_matches("//!")
                .trim_start_matches("//")
                .trim_start_matches("#:")
                .trim_start_matches('#')
                .trim_start_matches('*')
                .trim()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    truncate_bytes(cleaned, MAX_DOC_BYTES)
}

/// Expands an import clause into one or more referenced paths.
/// Expands an import into the paths it names, each with the local name it binds when
/// that differs from the path's last segment.
fn expand_import(language: &Language, text: &str) -> Vec<(Vec<String>, Option<String>)> {
    match language {
        Language::Rust => expand_rust_use(text)
            .into_iter()
            .map(|p| (p, None))
            .collect(),
        Language::Python => python_import_paths(text),
        Language::TypeScript | Language::Tsx | Language::JavaScript => js_import_paths(text),
        Language::Bash => vec![(specifier_path(&unquote(text)), None)],
        _ => vec![(reference_path(text), None)],
    }
}

/// Splits `name as alias` into the name and the alias, if any.
fn name_and_alias(part: &str) -> Option<(&str, Option<&str>)> {
    let words: Vec<&str> = part.split_whitespace().filter(|w| *w != "type").collect();
    match words.as_slice() {
        [name, "as", alias] => Some((name, Some(alias))),
        [name] => Some((name, None)),
        _ => None,
    }
}

/// Expands a Python import into one path per name it binds.
///
/// `from a.b import c, d as e` becomes `[a, b, c]` and `[a, b, d]`; `from . import x`
/// becomes `[., x]`; `from m import *` and a plain module (`a.b`, as the
/// `import a.b` patterns capture it) stay a module path.
fn python_import_paths(text: &str) -> Vec<(Vec<String>, Option<String>)> {
    let text = text.trim().trim_end_matches(';').trim();
    if let Some(list) = text.strip_prefix("import ") {
        // `import a.b as c, d`: one module path per item, aliased or not.
        return list
            .split(',')
            .filter_map(name_and_alias)
            .map(|(module, alias)| (python_module_path(module), alias.map(str::to_owned)))
            .collect();
    }
    let Some((module, names)) = text
        .strip_prefix("from ")
        .and_then(|rest| rest.split_once(" import "))
    else {
        return vec![(python_module_path(text), None)];
    };
    let module = python_module_path(module.trim());
    // Drop comments line by line before joining a parenthesized list.
    let cleaned = names
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ")
        .replace(['(', ')', '\\'], " ");
    let names: Vec<(&str, Option<&str>)> = cleaned
        .split(',')
        .filter_map(name_and_alias)
        .filter(|(name, _)| *name != "*")
        .collect();
    with_names(module, names)
}

/// Expands a JavaScript or TypeScript import into one path per name it binds.
///
/// `import X, { a, b as c } from "m"` becomes `[m, X]`, `[m, a]` and `[m, b]`;
/// `import * as ns from "@s/p"` becomes `[@s, p, ns]`; a bare specifier (`import
/// "./side"`, `require("m")`) stays a module path.
fn js_import_paths(text: &str) -> Vec<(Vec<String>, Option<String>)> {
    let text = text.trim().trim_end_matches(';').trim();
    if text.starts_with(['"', '\'', '`']) {
        return vec![(specifier_path(&unquote(text)), None)];
    }
    let Some(body) = text.strip_prefix("import") else {
        return vec![(specifier_path(&unquote(text)), None)];
    };
    let body = body.trim_start();
    let body = body.strip_prefix("type ").map_or(body, str::trim_start);
    let (clause, spec) = match body.rsplit_once(" from ") {
        Some((clause, spec)) => (clause.trim(), spec.trim()),
        None => ("", body.trim()),
    };
    let module = specifier_path(&unquote(spec));
    let names: Vec<(&str, Option<&str>)> = clause
        .split([',', '{', '}'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter_map(|part| match name_and_alias(part)? {
            // `* as ns` binds the whole module as `ns`.
            ("*", Some(ns)) => Some((ns, None)),
            other => Some(other),
        })
        .collect();
    with_names(module, names)
}

/// One path per bound name under `module`, each with its alias, or the module alone
/// when nothing is named.
fn with_names(
    module: Vec<String>,
    names: Vec<(&str, Option<&str>)>,
) -> Vec<(Vec<String>, Option<String>)> {
    if names.is_empty() {
        return vec![(module, None)];
    }
    names
        .into_iter()
        .map(|(name, alias)| {
            let path = module
                .iter()
                .cloned()
                .chain(std::iter::once(name.to_owned()))
                .collect();
            (path, alias.map(str::to_owned))
        })
        .collect()
}

fn specifier_path(spec: &str) -> Vec<String> {
    spec.split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn python_module_path(text: &str) -> Vec<String> {
    let dots = text.chars().take_while(|c| *c == '.').count();
    // `.x` is the same package; each further dot climbs one package.
    let relative: Vec<&str> = match dots {
        0 => Vec::new(),
        1 => vec!["."],
        n => vec![".."; n - 1],
    };
    relative
        .into_iter()
        .chain(text[dots..].split('.').filter(|s| !s.is_empty()))
        .map(str::to_owned)
        .collect()
}

/// Expands `a::{b, c::{d, e as f}}` into `[a,b]`, `[a,c,d]`, `[a,c,e]`.
fn expand_rust_use(text: &str) -> Vec<Vec<String>> {
    fn parse(chars: &[char], pos: &mut usize, prefix: &[String], out: &mut Vec<Vec<String>>) {
        let mut current: Vec<String> = prefix.to_vec();
        let mut token = String::new();
        let flush = |token: &mut String, current: &mut Vec<String>| {
            let t = token.trim();
            if !t.is_empty() {
                current.push(t.to_owned());
            }
            token.clear();
        };
        while *pos < chars.len() {
            let c = chars[*pos];
            *pos += 1;
            match c {
                '{' => {
                    flush(&mut token, &mut current);
                    loop {
                        parse(chars, pos, &current, out);
                        if *pos >= chars.len() || chars[*pos - 1] == '}' {
                            break;
                        }
                    }
                    return;
                }
                '}' | ',' => {
                    flush(&mut token, &mut current);
                    if current.len() > prefix.len() {
                        out.push(current);
                    }
                    return;
                }
                ':' if chars.get(*pos) == Some(&':') => {
                    *pos += 1;
                    flush(&mut token, &mut current);
                }
                ':' => {}
                ' ' | '\n' | '\t' if token.trim() == "as" => {
                    token.clear();
                    while *pos < chars.len() && !matches!(chars[*pos], ',' | '}') {
                        *pos += 1;
                    }
                }
                ' ' | '\n' | '\t' => token.push(' '),
                _ => token.push(c),
            }
        }
        flush(&mut token, &mut current);
        if current.len() > prefix.len() {
            out.push(current);
        }
    }

    let cleaned = strip_angle_brackets(text.trim().trim_end_matches(';'));
    let chars: Vec<char> = cleaned.chars().collect();
    let mut out = Vec::new();
    parse(&chars, &mut 0, &[], &mut out);
    out.into_iter()
        .map(|path| {
            path.into_iter()
                .filter_map(|seg| seg.split_whitespace().next().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .filter(|path| !path.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use surveyor_model::{CommitId, RepoId, RepoPath};

    use super::*;

    fn unit(path: &str, text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new(path),
            CommitId::new("c"),
            text.into(),
        )
    }

    #[test]
    fn packs_compile() {
        let host = TreeSitterExtractor::try_new().expect("packs compile");
        assert_eq!(host.grammars().count(), 6);
    }

    #[test]
    fn rust_definitions_references_and_scopes() {
        let src = r"
/// Relays events.
pub struct Relay { n: usize }

impl Relay {
    /// Drains the queue.
    pub fn drain(&mut self) -> usize { self.flush(); helper::count(1) }
}

impl std::fmt::Display for Relay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
}

fn helper() {}

#[cfg(test)]
mod tests {
    use super::{Relay, helper};
    #[test]
    fn drains() { Relay { n: 0 }.drain(); }
}
";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.rs", src))
            .expect("parses");
        let decls: Vec<&DeclarationFact> = facts.declarations().collect();
        let find = |name: &str| decls.iter().find(|d| d.name == name).copied();
        let relay = find("Relay").expect("struct");
        assert_eq!(relay.kind, NodeKind::Struct);
        assert_eq!(relay.doc.as_deref(), Some("Relays events."));
        let drain = find("drain").expect("method");
        assert_eq!(drain.kind, NodeKind::Method);
        assert_eq!(drain.scope, vec!["Relay".to_owned()]);
        assert_eq!(drain.doc.as_deref(), Some("Drains the queue."));
        assert_eq!(
            drain.signature.as_deref(),
            Some("pub fn drain(&mut self) -> usize")
        );
        let fmt = find("fmt").expect("trait method");
        assert_eq!(fmt.scope, vec!["Relay".to_owned()]);
        let impls: Vec<_> = decls.iter().filter(|d| d.kind == NodeKind::Impl).collect();
        assert_eq!(impls.len(), 2);
        assert!(
            impls
                .iter()
                .any(|i| i.attributes.get("trait").map(String::as_str) == Some("Display"))
        );
        let test = find("drains").expect("test");
        assert_eq!(test.kind, NodeKind::Test);
        assert_eq!(test.scope, vec!["tests".to_owned()]);

        let refs: Vec<&ReferenceFact> = facts.references().collect();
        let flush = refs.iter().find(|r| r.name == "flush").expect("call");
        assert_eq!(flush.role, ReferenceRole::Call(CallForm::Method));
        assert_eq!(flush.scope, vec!["Relay".to_owned(), "drain".to_owned()]);
        let count = refs
            .iter()
            .find(|r| r.name == "count")
            .expect("scoped call");
        assert_eq!(count.role, ReferenceRole::Call(CallForm::Path));
        assert_eq!(count.path, vec!["helper".to_owned(), "count".to_owned()]);
        let implements = refs
            .iter()
            .find(|r| r.role == ReferenceRole::Implements)
            .expect("impl");
        assert_eq!(implements.name, "Display");
        assert_eq!(implements.scope, vec!["Relay".to_owned()]);
        let imports: Vec<_> = refs
            .iter()
            .filter(|r| r.role == ReferenceRole::Import)
            .collect();
        assert_eq!(imports.len(), 2);
        assert!(imports.iter().any(|r| r.path == vec!["super", "Relay"]));
    }

    #[test]
    fn rust_calls_inside_macros_are_seen() {
        let src = "fn t() { assert_eq!(relay.drain(10), compute(1)); println!(\"{}\", format!(\"{}\", helper())); }";
        let u = unit("a.rs", src);
        let facts = TreeSitterExtractor::new().extract(&u).expect("parses");
        let refs: Vec<&ReferenceFact> = facts.references().collect();
        let names: Vec<&str> = refs
            .iter()
            .filter(|r| r.role.is_call())
            .map(|r| r.name.as_str())
            .collect();
        assert!(
            names.contains(&"drain") && names.contains(&"compute") && names.contains(&"helper"),
            "{names:?}"
        );
        assert!(
            names
                .iter()
                .all(|n| !n.contains('!') && !n.contains("__m") && !n.is_empty()),
            "{names:?}"
        );
        assert!(
            names.contains(&"assert_eq") && names.contains(&"format"),
            "{names:?}"
        );
        // The macros are calls of their own form; the calls inside them keep theirs.
        assert!(refs.iter().all(|r| {
            match r.name.as_str() {
                "assert_eq" | "println" | "format" => {
                    r.role == ReferenceRole::Call(CallForm::Macro)
                }
                "compute" | "helper" => r.role == ReferenceRole::Call(CallForm::Path),
                "drain" => r.role == ReferenceRole::Call(CallForm::Method),
                _ => true,
            }
        }));
        let drain = refs.iter().find(|r| r.name == "drain").expect("drain");
        assert_eq!(u.slice(drain.span.bytes), "drain");
        assert_eq!(drain.scope, vec!["t".to_owned()]);
    }

    #[test]
    fn rust_variants_and_path_values() {
        let src = "enum Fact { Decl(u8), Gap }\nfn pick(f: Fact) -> bool { matches!(f, Fact::Gap) }\nfn mk() -> Fact { Fact::Decl(1) }\nfn apply() { run(NodeKind::is_callable); }\nuse std::fmt::Display;\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.rs", src))
            .expect("parses");
        let decls: Vec<_> = facts.declarations().collect();
        let decl = decls.iter().find(|d| d.name == "Decl").expect("variant");
        assert_eq!(decl.kind, NodeKind::Variant);
        assert_eq!(decl.scope, vec!["Fact".to_owned()]);
        let refs: Vec<_> = facts.references().collect();
        let gap = refs
            .iter()
            .find(|r| r.name == "Gap")
            .expect("pattern mention");
        assert_eq!(gap.role, ReferenceRole::Mention);
        assert_eq!(gap.path, vec!["Fact", "Gap"]);
        assert!(
            refs.iter().any(|r| r.name == "Decl" && r.role.is_call()),
            "a tuple variant construction is a call"
        );
        assert_eq!(
            refs.iter().filter(|r| r.name == "Decl").count(),
            1,
            "the call's path is not also a mention"
        );
        let value = refs
            .iter()
            .find(|r| r.name == "is_callable")
            .expect("function value");
        assert_eq!(value.role, ReferenceRole::Mention);
        assert_eq!(value.scope, vec!["apply".to_owned()]);
        let display: Vec<_> = refs.iter().filter(|r| r.name == "Display").collect();
        assert_eq!(
            display.len(),
            1,
            "an import path is not also a mention: {display:?}"
        );
        assert_eq!(display[0].role, ReferenceRole::Import);
    }

    #[test]
    fn rust_sql_literal_is_tagged() {
        let src = r#"async fn load(pool: &PgPool) { sqlx::query!("SELECT id FROM users").fetch_all(pool).await; }"#;
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.rs", src))
            .expect("parses");
        let lit = facts.literals().next().expect("literal");
        assert_eq!(lit.role, LiteralRole::Sql);
        assert_eq!(lit.value, "SELECT id FROM users");
        assert_eq!(lit.scope, vec!["load".to_owned()]);
    }

    #[test]
    fn typescript_classes_and_imports() {
        let src = r#"
import { Client } from "./client";
/** Serves things. */
export class Server extends Base implements Handler {
  start(): void { this.client.connect(); }
}
export const run = async () => { new Server().start(); };
describe("server", () => { it("starts", () => {}); });
"#;
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.ts", src))
            .expect("parses");
        let decls: Vec<_> = facts.declarations().collect();
        let server = decls.iter().find(|d| d.name == "Server").expect("class");
        assert_eq!(server.kind, NodeKind::Class);
        assert_eq!(server.doc.as_deref(), Some("Serves things."));
        assert!(
            decls
                .iter()
                .any(|d| d.name == "start" && d.kind == NodeKind::Method)
        );
        assert!(
            decls
                .iter()
                .any(|d| d.name == "run" && d.kind == NodeKind::Function)
        );
        assert!(
            decls
                .iter()
                .any(|d| d.name == "server" && d.kind == NodeKind::Test)
        );
        let refs: Vec<_> = facts.references().collect();
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Import
                    && r.path == vec![".", "client", "Client"])
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Extends && r.name == "Base")
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Implements && r.name == "Handler")
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Call(CallForm::Method) && r.name == "connect")
        );
    }

    #[test]
    fn python_docstrings_and_imports() {
        let src = "import os.path\nfrom ..pkg import thing\n\nclass Repo(Base):\n    \"\"\"Stores rows.\"\"\"\n    def save(self):\n        cur.execute(\"INSERT INTO t VALUES (1)\")\n\ndef test_save():\n    Repo().save()\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.py", src))
            .expect("parses");
        let decls: Vec<_> = facts.declarations().collect();
        let repo = decls.iter().find(|d| d.name == "Repo").expect("class");
        assert_eq!(repo.doc.as_deref(), Some("Stores rows."));
        let save = decls.iter().find(|d| d.name == "save").expect("method");
        assert_eq!(save.kind, NodeKind::Method);
        assert_eq!(save.scope, vec!["Repo".to_owned()]);
        assert!(
            decls
                .iter()
                .any(|d| d.name == "test_save" && d.kind == NodeKind::Test)
        );
        let refs: Vec<_> = facts.references().collect();
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Import && r.path == vec!["os", "path"])
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Import && r.path == vec!["..", "pkg", "thing"])
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Extends && r.name == "Base")
        );
        assert_eq!(
            facts.literals().next().map(|l| l.value.as_str()),
            Some("INSERT INTO t VALUES (1)")
        );
    }

    #[test]
    fn bash_functions_and_sources() {
        let src = "source ./lib/common.sh\nbuild() {\n  cargo build\n}\nbuild\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("x.sh", src))
            .expect("parses");
        assert!(facts.declarations().any(|d| d.name == "build"));
        let refs: Vec<_> = facts.references().collect();
        assert!(refs.iter().any(|r| r.role == ReferenceRole::Import && r.path == vec![".", "lib", "common.sh"]));
        assert!(
            refs.iter()
                .any(|r| r.role.is_call() && r.name == "cargo" && r.scope == vec!["build"])
        );
    }

    #[test]
    fn use_tree_expansion_and_helpers() {
        let paths = expand_rust_use("crate::a::{b, c::{d, e as f}, self}");
        assert!(paths.contains(&vec!["crate".to_owned(), "a".to_owned(), "b".to_owned()]));
        assert!(paths.contains(&vec![
            "crate".to_owned(),
            "a".to_owned(),
            "c".to_owned(),
            "d".to_owned()
        ]));
        assert!(paths.contains(&vec![
            "crate".to_owned(),
            "a".to_owned(),
            "c".to_owned(),
            "e".to_owned()
        ]));
        assert_eq!(
            expand_rust_use("std::fmt::Display"),
            vec![vec!["std", "fmt", "Display"]]
        );
        assert_eq!(unquote("r#\"x\"#"), "x");
        assert_eq!(unquote("Relay"), "Relay");
        assert_eq!(unquote("b\"bytes\""), "bytes");
        assert_eq!(unquote("f'py {x}'"), "py {x}");
        assert_eq!(unquote("\"\"\"doc\"\"\""), "doc");
        assert_eq!(
            unquote(
                "\"INSERT INTO {schema}.t \\\n                (a, b) \\\n             VALUES ($1, $2)\""
            ),
            "INSERT INTO {schema}.t                 (a, b)              VALUES ($1, $2)",
            "a continued line drops the break and keeps the indentation"
        );
        assert_eq!(unquote("\"say \\\"hi\\\"\\n\""), "say \"hi\"\n");
        assert_eq!(
            unquote("r\"a\\nb\""),
            "a\\nb",
            "raw strings keep their backslashes"
        );
        assert_eq!(unquote("r#\"a\\\"b\"#"), "a\\\"b");
        assert_eq!(
            unquote("\"\\u{1F600}\""),
            "\\u{1F600}",
            "unknown escapes stay verbatim"
        );
        assert_eq!(definition_name("Vec<T>"), "Vec");
        assert_eq!(reference_path("Vec::<u8>::new"), vec!["Vec", "new"]);
        assert_eq!(python_module_path("..pkg.mod"), vec!["..", "pkg", "mod"]);
        assert_eq!(python_module_path(".sib"), vec![".", "sib"]);
        assert_eq!(python_module_path("os.path"), vec!["os", "path"]);
        assert_eq!(strip_angle_brackets("Map<K, Vec<V>>::new"), "Map::new");
    }

    #[test]
    fn imports_expand_to_the_names_they_bind() {
        let path = |segments: &[&str]| segments.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let plain = |segments: &[&str]| (path(segments), None::<String>);
        let p = |t: &str| python_import_paths(t);
        assert_eq!(p("import os.path"), vec![plain(&["os", "path"])]);
        assert_eq!(p("os.path"), vec![plain(&["os", "path"])]);
        assert_eq!(
            p("import pyarrow as pa, redis"),
            vec![(path(&["pyarrow"]), Some("pa".into())), plain(&["redis"])]
        );
        assert_eq!(
            p("from a.b import c, d as e"),
            vec![
                plain(&["a", "b", "c"]),
                (path(&["a", "b", "d"]), Some("e".into()))
            ]
        );
        assert_eq!(p("from . import x"), vec![plain(&[".", "x"])]);
        assert_eq!(
            p("from ..pkg import (\n    one,  # first\n    two,\n)"),
            vec![plain(&["..", "pkg", "one"]), plain(&["..", "pkg", "two"])]
        );
        assert_eq!(p("from m import *"), vec![plain(&["m"])]);
        let j = |t: &str| js_import_paths(t);
        assert_eq!(
            j("import X, { a, b as c } from \"m\";"),
            vec![
                plain(&["m", "X"]),
                plain(&["m", "a"]),
                (path(&["m", "b"]), Some("c".into()))
            ]
        );
        assert_eq!(
            j("import * as ns from '@s/p'"),
            vec![plain(&["@s", "p", "ns"])]
        );
        assert_eq!(
            j("import type { T } from \"./t\""),
            vec![plain(&[".", "t", "T"])]
        );
        assert_eq!(j("import \"./side\""), vec![plain(&[".", "side"])]);
        assert_eq!(j("\"left-pad\""), vec![plain(&["left-pad"])]);
    }

    #[test]
    fn receivers_keep_their_root() {
        assert_eq!(receiver_root("descriptor"), Some("descriptor".into()));
        assert_eq!(receiver_root("self.client"), Some("self".into()));
        assert_eq!(
            receiver_root("google.protobuf.message"),
            Some("google".into())
        );
        assert_eq!(receiver_root("f()"), None);
        assert_eq!(receiver_root("f().g"), None);
        assert_eq!(receiver_root("(a)"), None);
        assert_eq!(receiver_root("\"x\""), None);
        assert_eq!(receiver_root("xs[0]"), None);
        let src = "import descriptor\n\nclass A(google.protobuf.message.Message):\n    def run(self):\n        descriptor.Foo()\n        self.client.connect()\n        make().go()\n        plain()\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.py", src))
            .expect("parses");
        let refs: Vec<_> = facts.references().collect();
        let receiver_of = |name: &str| {
            refs.iter()
                .find(|r| r.name == name)
                .map(|r| r.receiver.clone())
                .expect(name)
        };
        assert_eq!(receiver_of("Foo"), Some("descriptor".into()));
        assert_eq!(receiver_of("connect"), Some("self".into()));
        assert_eq!(receiver_of("go"), None);
        assert_eq!(receiver_of("plain"), None);
        assert_eq!(receiver_of("Message"), Some("google".into()));
        let ts = "class V extends React.Component { f() { this.setState({}); ns.helper(); } }\nnew ns.Thing();\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("a.ts", ts))
            .expect("parses");
        let refs: Vec<_> = facts.references().collect();
        let receiver_of = |name: &str| {
            refs.iter()
                .find(|r| r.name == name)
                .map(|r| r.receiver.clone())
                .expect(name)
        };
        assert_eq!(receiver_of("Component"), Some("React".into()));
        assert_eq!(receiver_of("setState"), Some("this".into()));
        assert_eq!(receiver_of("helper"), Some("ns".into()));
        assert_eq!(receiver_of("Thing"), Some("ns".into()));
    }

    #[test]
    fn host_registrations_declare_functions_the_script_language_sees() {
        let src = "use rhai::{Engine, Module};\nfn sh(c: &str) {}\nfn install(engine: &mut Engine, m: &mut Module) {\n    engine.register_fn(\"exit\", || std::process::exit(0));\n    m.set_native_fn(\"sh\", |c: &str| Ok(sh(c)));\n    m.set_native_fn(\"sh\", |c: &str, _o: i64| Ok(sh(c)));\n    engine.register_fn(\"is_blank\", |s: &str| s.trim().is_empty());\n}\n";
        let facts = TreeSitterExtractor::new()
            .extract(&unit("host.rs", src))
            .expect("parses");
        let exposed: Vec<(String, Vec<String>)> = facts
            .declarations()
            .filter(|d| d.attributes.get("language").map(String::as_str) == Some("rhai"))
            .map(|d| (d.name.clone(), d.scope.clone()))
            .collect();
        assert_eq!(
            exposed,
            vec![
                ("exit".to_owned(), vec!["install".to_owned()]),
                ("sh".to_owned(), vec!["install".to_owned()]),
                ("sh".to_owned(), vec!["install".to_owned()]),
                ("is_blank".to_owned(), vec!["install".to_owned()]),
            ]
        );
        assert!(
            facts
                .declarations()
                .any(|d| d.name == "sh" && !d.attributes.contains_key("language")),
            "the plain Rust `sh` is still a Rust function"
        );
        assert!(
            role_for_capture("definition.function.Klingon").is_err(),
            "a language tag the model cannot parse is a pack error"
        );
    }
}
