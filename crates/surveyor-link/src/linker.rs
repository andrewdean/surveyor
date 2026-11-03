//! The linking passes.
//!
//! Each pass reads the facts and the nodes built so far, decides through the pure
//! helpers in `resolve`, and appends nodes or edges. The builder is the one place
//! that mutates; everything it consults returns values.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use surveyor_model::{
    ByteSpan, CallForm, Confidence, ContentHash, DeclarationFact, DeclaredRelation, DeclaredSource,
    Edge, EdgeId, EdgeKind, Evidence, FactSet, GrammarId, GraphDelta, Language, LiteralRole, Node,
    NodeId, NodeKind, Provenance, ReferenceFact, ReferenceRole, RepoPath, ResolvedReference,
    ResolverId, RuleId, SourceLocation, SymbolTarget, TextSpan, Unresolved, UnresolvedReason,
};

use crate::ids::{doc_id, file_id, package_id, repo_id, sql_id, symbol_id};
use crate::input::LinkInput;
use crate::resolve::{Resolution, Site, SymbolIndex, Tier, decide, decide_table, is_method, rank};

/// Resolver recorded on edges derived from `sqlparser` facts.
const SQL_RESOLVER: &str = "sqlparser";

/// Resolver recorded on dispatch edges: an implementation's method with the name of a
/// method of the contract it implements, which the language guarantees is the override.
const DISPATCH_RESOLVER: &str = "dispatch";

/// Grammar recorded when a fact set names none (native enrichers).
const NO_GRAMMAR: &str = "none";

/// Crate roots Rust imports never resolve to a package node.
const RUST_BUILTIN_ROOTS: &[&str] = &["std", "core", "alloc", "crate", "self", "super"];

/// The part of an indexed graph that an incremental update keeps.
///
/// Nodes and edges of files that are not relinked, plus the file-less nodes (the
/// repository root, schemas, external packages). The relink passes resolve against
/// these as if they had been built in the same run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Seed {
    /// Surviving nodes.
    pub nodes: Vec<Node>,
    /// Surviving edges.
    pub edges: Vec<Edge>,
}

/// Links one repository's facts into a graph delta.
#[must_use]
pub fn link(input: &LinkInput) -> GraphDelta {
    run(Builder::new(input))
}

/// Links the facts of the relinked files against `seed`.
///
/// The delta holds the nodes and edges the relinked files produce plus every seed
/// node the passes changed (the root's commit, a table a migration merged into);
/// seed nodes and edges that stand are not repeated.
#[must_use]
pub fn relink(input: &LinkInput, seed: Seed) -> GraphDelta {
    let mut b = Builder::new(input);
    b.absorb(seed);
    run(b)
}

fn run(mut b: Builder<'_>) -> GraphDelta {
    b.root();
    b.packages();
    b.files();
    b.declarations();
    b.structure();
    b.declared_relations();
    b.imports();
    b.references();
    b.dispatch();
    b.literals();
    b.tests();
    b.finish()
}

/// A declaration node inside one file, with its parent by span containment.
#[derive(Debug, Clone)]
struct DeclRef {
    /// The fact set that declared it; `None` for a declaration the seed carries.
    set: Option<usize>,
    node: usize,
    span: TextSpan,
    parent: Option<usize>,
}

struct Builder<'a> {
    input: &'a LinkInput,
    nodes: Vec<Node>,
    node_pos: HashMap<NodeId, usize>,
    edges: Vec<Edge>,
    edge_ids: HashSet<EdgeId>,
    pair_seen: HashSet<(usize, EdgeKind, usize)>,
    unresolved: Vec<Unresolved>,
    index: SymbolIndex,
    root: usize,
    file_nodes: HashMap<RepoPath, usize>,
    /// Package directory to node, deepest first.
    package_dirs: Vec<(String, usize)>,
    package_by_name: HashMap<String, usize>,
    /// Declaration nodes per file, sorted by span start, with parent links.
    decls_by_file: HashMap<RepoPath, Vec<DeclRef>>,
    /// `DEFINES` and `IMPLEMENTS` edges the seed carries, for dispatch across the
    /// boundary between relinked and untouched files.
    seed_links: Vec<(usize, EdgeKind, usize)>,
    /// Nodes below this index came from the seed and are reported only when changed.
    seed_count: usize,
    /// Seed nodes a pass changed.
    dirty_seeds: BTreeSet<usize>,
    /// Precisely resolved reference sites per file, sorted by start offset.
    bindings: HashMap<RepoPath, Vec<&'a ResolvedReference>>,
    /// Classes, by file and name, that extend a type imported from outside the
    /// repository; filled once imports are linked.
    external_bases: HashMap<RepoPath, HashSet<String>>,
}

impl<'a> Builder<'a> {
    fn new(input: &'a LinkInput) -> Self {
        let mut bindings: HashMap<RepoPath, Vec<&'a ResolvedReference>> = HashMap::new();
        for set in &input.fact_sets {
            let found: Vec<&'a ResolvedReference> = set.resolved().collect();
            if !found.is_empty() {
                bindings.entry(set.path.clone()).or_default().extend(found);
            }
        }
        for sites in bindings.values_mut() {
            sites.sort_by_key(|b| (b.span.start(), b.span.end()));
        }
        Self {
            input,
            nodes: Vec::new(),
            node_pos: HashMap::new(),
            edges: Vec::new(),
            edge_ids: HashSet::new(),
            pair_seen: HashSet::new(),
            unresolved: Vec::new(),
            index: SymbolIndex::default(),
            root: 0,
            file_nodes: HashMap::new(),
            package_dirs: Vec::new(),
            package_by_name: HashMap::new(),
            decls_by_file: HashMap::new(),
            seed_links: Vec::new(),
            seed_count: 0,
            dirty_seeds: BTreeSet::new(),
            bindings,
            external_bases: HashMap::new(),
        }
    }

    /// Loads the surviving graph so the passes resolve against it.
    ///
    /// Nodes enter the symbol index; packages and files enter the lookups the passes
    /// use; edges seed the dedupe sets and the package, import and dependency
    /// relations that rank candidates.
    fn absorb(&mut self, seed: Seed) {
        for node in seed.nodes {
            let path = node.file().cloned();
            let span = node.span();
            let idx = self.push_node(node);
            let node = &self.nodes[idx];
            // A declaration the seed carries is looked up by file like a fresh one:
            // a relinked file's precise bindings, module imports, receivers and
            // dispatch all name declarations in files that did not change.
            let declares = !matches!(
                node.kind,
                NodeKind::File | NodeKind::Package | NodeKind::Workspace
            );
            if let (true, Some(file), Some(span)) = (declares, path.clone(), span) {
                self.decls_by_file.entry(file).or_default().push(DeclRef {
                    set: None,
                    node: idx,
                    span,
                    parent: None,
                });
            }
            match (&node.kind, path) {
                (NodeKind::File, Some(path)) => {
                    self.file_nodes.insert(path, idx);
                }
                (NodeKind::Package, _) => {
                    if let Some(dir) = node.attributes.get("dir") {
                        self.package_dirs.push((dir.clone(), idx));
                    }
                    let (coordinate, name) = (node.qualified_name.clone(), node.name.clone());
                    self.register_package(&coordinate, &name, idx);
                }
                _ => {}
            }
        }
        self.seed_count = self.nodes.len();
        for edge in &seed.edges {
            self.edge_ids.insert(edge.id.clone());
            let (Some(&src), Some(&dst)) =
                (self.node_pos.get(&edge.src), self.node_pos.get(&edge.dst))
            else {
                continue;
            };
            self.pair_seen.insert((src, edge.kind.clone(), dst));
            let (s, d) = (&self.nodes[src], &self.nodes[dst]);
            match (&edge.kind, &s.kind, &d.kind) {
                (EdgeKind::Contains, NodeKind::Package, NodeKind::File) => {
                    if let Some(file) = d.file() {
                        self.index
                            .package_of_file
                            .insert(file.clone(), s.id.to_string());
                    }
                }
                (EdgeKind::Imports, NodeKind::File, kind) => {
                    if let Some(file) = s.file() {
                        self.index
                            .imported_by_file
                            .entry(file.clone())
                            .or_default()
                            .insert(d.id.to_string());
                        if *kind == NodeKind::Package {
                            self.index
                                .imports_of_file
                                .entry(file.clone())
                                .or_default()
                                .insert(d.id.to_string());
                        }
                    }
                }
                (EdgeKind::DependsOn, NodeKind::Package, NodeKind::Package) => {
                    self.index
                        .deps_of_package
                        .entry(s.id.to_string())
                        .or_default()
                        .insert(d.id.to_string());
                }
                (EdgeKind::Defines | EdgeKind::Implements, _, _) => {
                    self.seed_links.push((src, edge.kind.clone(), dst));
                }
                _ => {}
            }
        }
    }

    /// Marks a seed node as changed so it is written back.
    fn touch(&mut self, idx: usize) {
        if idx < self.seed_count {
            self.dirty_seeds.insert(idx);
        }
    }

    // ---- node and edge plumbing ---------------------------------------------------

    fn push_node(&mut self, node: Node) -> usize {
        let idx = self.nodes.len();
        self.node_pos.insert(node.id.clone(), idx);
        self.index.insert(idx, &node);
        self.nodes.push(node);
        idx
    }

    fn node_id(&self, idx: usize) -> NodeId {
        self.nodes[idx].id.clone()
    }

    fn unique_id(&self, base: &NodeId) -> NodeId {
        // At most one collision per existing node, so the range is bounded.
        std::iter::once(base.clone())
            .chain((2..=self.nodes.len() + 2).map(|n| NodeId::new(format!("{base}#{n}"))))
            .find(|id| !self.node_pos.contains_key(id))
            .expect("more suffixes than nodes")
    }

    fn evidence(&self, file: &RepoPath, span: TextSpan, hash: ContentHash) -> Evidence {
        Evidence {
            repo: self.input.repo.clone(),
            location: SourceLocation::new(file.clone(), span),
            commit: self.input.commit.clone(),
            content_hash: hash,
        }
    }

    /// Evidence that cites a node's own declaration.
    fn evidence_of(&self, idx: usize) -> Option<Evidence> {
        let node = &self.nodes[idx];
        node.source()
            .map(|loc| self.evidence(&loc.file, loc.span, node.content_hash))
    }

    /// Evidence for a node that has a file but no span (a file node): an empty span at its start.
    fn evidence_of_file(&self, idx: usize) -> Option<Evidence> {
        let node = &self.nodes[idx];
        node.file().map(|file| {
            let span = TextSpan::new(ByteSpan::new(0, 0), surveyor_model::LineSpan::new(1, 1));
            self.evidence(file, span, ContentHash::of(b""))
        })
    }

    fn push_edge(
        &mut self,
        src: usize,
        kind: EdgeKind,
        dst: usize,
        provenance: Provenance,
        against: Vec<NodeId>,
    ) {
        self.push_edge_with(src, kind, dst, provenance, against, BTreeMap::new());
    }

    /// Like [`Self::push_edge`], with facts about the relationship itself.
    fn push_edge_with(
        &mut self,
        src: usize,
        kind: EdgeKind,
        dst: usize,
        provenance: Provenance,
        against: Vec<NodeId>,
        attributes: BTreeMap<String, String>,
    ) {
        let edge = Edge::new(self.node_id(src), kind, self.node_id(dst), provenance)
            .resolved_against(against)
            .with_attributes(attributes);
        if self.edge_ids.insert(edge.id.clone()) {
            self.edges.push(edge);
        }
    }

    /// Like [`Self::push_edge`] but keeps one edge per `(src, kind, dst)` triple.
    fn push_edge_once(
        &mut self,
        src: usize,
        kind: EdgeKind,
        dst: usize,
        provenance: Provenance,
        against: Vec<NodeId>,
    ) {
        if self.pair_seen.insert((src, kind.clone(), dst)) {
            self.push_edge(src, kind, dst, provenance, against);
        }
    }

    fn syntactic(
        &self,
        set: &FactSet,
        rule: &RuleId,
        span: TextSpan,
        hash: ContentHash,
    ) -> Provenance {
        let grammar = set
            .grammar
            .clone()
            .unwrap_or_else(|| GrammarId::new(NO_GRAMMAR));
        Provenance::syntactic(grammar, rule.clone(), self.evidence(&set.path, span, hash))
    }

    fn resolved_sql(&self, file: &RepoPath, span: TextSpan, hash: ContentHash) -> Provenance {
        Provenance::resolved(
            ResolverId::new(SQL_RESOLVER),
            self.evidence(file, span, hash),
        )
    }

    fn declared(
        &self,
        source: DeclaredSource,
        file: &RepoPath,
        span: TextSpan,
        hash: ContentHash,
    ) -> Provenance {
        Provenance::declared(source, self.evidence(file, span, hash))
    }

    fn report(&mut self, r: &ReferenceFact, file: &RepoPath, reason: UnresolvedReason) {
        self.unresolved.push(Unresolved {
            role: r.role.clone(),
            name: r.name.clone(),
            site: SourceLocation::new(file.clone(), r.span),
            reason,
        });
    }

    // ---- pass 1: structure ----------------------------------------------------------

    fn root(&mut self) {
        let repo = &self.input.repo;
        let id = repo_id(repo);
        let commit = self.input.commit.as_str().to_owned();
        self.root = if let Some(existing) = self.node_pos.get(&id).copied() {
            self.nodes[existing]
                .attributes
                .insert("commit".to_owned(), commit);
            self.touch(existing);
            existing
        } else {
            let node = Node::synthetic(id, NodeKind::Repository, repo.as_str(), repo.clone())
                .with_attribute("commit", commit);
            self.push_node(node)
        };
    }

    fn packages(&mut self) {
        let found: Vec<(usize, DeclarationFact)> = self
            .input
            .fact_sets
            .iter()
            .enumerate()
            .flat_map(|(si, set)| {
                set.declarations()
                    .filter(|d| matches!(d.kind, NodeKind::Package | NodeKind::Workspace))
                    .map(move |d| (si, d.clone()))
            })
            .collect();
        for (si, d) in found {
            let set = &self.input.fact_sets[si];
            let coordinate = d.signature.clone().unwrap_or_else(|| d.name.clone());
            let id = package_id(&coordinate);
            if self.node_pos.contains_key(&id) {
                continue;
            }
            let dir = d.attributes.get("dir").cloned().unwrap_or_default();
            let node = Node {
                signature: Some(coordinate.clone()),
                attributes: d.attributes.clone(),
                content_hash: d.hash,
                ..Node::synthetic(id, d.kind.clone(), d.name.clone(), self.input.repo.clone())
                    .with_qualified_name(coordinate.clone())
                    .at(set.path.clone(), d.span)
            };
            let idx = self.push_node(node);
            let prov = self.declared(DeclaredSource::Manifest, &set.path, d.span, d.hash);
            self.push_edge(self.root, EdgeKind::Contains, idx, prov, Vec::new());
            if d.kind == NodeKind::Package {
                self.package_dirs.push((dir, idx));
                self.register_package(&coordinate, &d.name, idx);
            }
        }
        self.package_dirs
            .sort_by_key(|(dir, _)| std::cmp::Reverse(dir.len()));
    }

    /// Makes a package findable by its registry and name, and by its short name for
    /// scoped coordinates such as `npm/@scope/name`.
    fn register_package(&mut self, coordinate: &str, name: &str, idx: usize) {
        let registry = registry_of(coordinate);
        self.package_by_name
            .entry(package_key(registry, name))
            .or_insert(idx);
        if let Some((_, short)) = coordinate.rsplit_once('/') {
            self.package_by_name
                .entry(package_key(registry, short))
                .or_insert(idx);
        }
    }

    fn owner_package(&self, path: &RepoPath) -> Option<usize> {
        self.package_dirs
            .iter()
            .find(|(dir, _)| path.starts_with_dir(dir))
            .map(|(_, idx)| *idx)
    }

    fn files(&mut self) {
        for entry in &self.input.files {
            let name = entry
                .path
                .file_name()
                .unwrap_or(entry.path.as_str())
                .to_owned();
            let node = Node {
                language: entry.language.clone(),
                content_hash: entry.content_hash,
                ..Node::synthetic(
                    file_id(&entry.path),
                    NodeKind::File,
                    name,
                    self.input.repo.clone(),
                )
                .with_qualified_name(entry.path.as_str())
                .in_file(entry.path.clone())
                .with_attribute("bytes", entry.bytes.to_string())
                .with_attribute("lines", entry.lines.to_string())
            };
            let idx = self.push_node(node);
            self.file_nodes.insert(entry.path.clone(), idx);
            match self.owner_package(&entry.path) {
                Some(pkg) => {
                    self.index
                        .package_of_file
                        .insert(entry.path.clone(), self.nodes[pkg].id.to_string());
                    if let Some(ev) = self.evidence_of(pkg) {
                        let prov = Provenance::declared(DeclaredSource::Manifest, ev);
                        self.push_edge(pkg, EdgeKind::Contains, idx, prov, Vec::new());
                    }
                }
                None => {
                    if let Some(ev) = self.evidence_of_file(idx) {
                        let prov =
                            Provenance::declared(DeclaredSource::Other("directory".into()), ev);
                        self.push_edge(self.root, EdgeKind::Contains, idx, prov, Vec::new());
                    }
                }
            }
        }
    }

    // ---- pass 2: declarations --------------------------------------------------------

    fn declarations(&mut self) {
        for (si, set) in self.input.fact_sets.iter().enumerate() {
            let refs: Vec<DeclRef> = set
                .declarations()
                .filter(|d| !matches!(d.kind, NodeKind::Package | NodeKind::Workspace))
                .map(|d| DeclRef {
                    set: Some(si),
                    node: self.declaration_node(set, d),
                    span: d.span,
                    parent: None,
                })
                .collect();
            if !refs.is_empty() {
                self.decls_by_file
                    .entry(set.path.clone())
                    .or_default()
                    .extend(refs);
            }
        }
        for refs in self.decls_by_file.values_mut() {
            *refs = with_parents(std::mem::take(refs));
        }
    }

    /// Creates (or merges into) the node for a declaration; returns its index.
    fn declaration_node(&mut self, set: &FactSet, d: &DeclarationFact) -> usize {
        let (id, qualified) = declaration_identity(set, d);
        // Registrations of the same host function (overloads) are one declaration.
        let mergeable = d.attributes.contains_key("language")
            || matches!(
                d.kind,
                NodeKind::Schema
                    | NodeKind::Table
                    | NodeKind::View
                    | NodeKind::Column
                    | NodeKind::Index
                    | NodeKind::Constraint
            );
        if mergeable && let Some(&existing) = self.node_pos.get(&id) {
            let node = &mut self.nodes[existing];
            let before = node.attributes.len();
            for (k, v) in &d.attributes {
                node.attributes
                    .entry(k.clone())
                    .or_insert_with(|| v.clone());
            }
            if node.attributes.len() != before {
                self.touch(existing);
            }
            return existing;
        }
        let node = Node {
            id: self.unique_id(&id),
            kind: d.kind.clone(),
            name: d.name.clone(),
            qualified_name: qualified,
            repo: self.input.repo.clone(),
            language: declared_language(set, d),
            location: Some(surveyor_model::Location::Span(SourceLocation::new(
                set.path.clone(),
                d.span,
            ))),
            content_hash: d.hash,
            signature: d.signature.clone(),
            doc: d.doc.clone(),
            attributes: d.attributes.clone(),
        };
        self.push_node(node)
    }

    // ---- pass 3: structural edges ----------------------------------------------------

    fn structure(&mut self) {
        // Sorted so that the migration whose span cites a merged table is the same on
        // every run.
        let mut files: Vec<RepoPath> = self.decls_by_file.keys().cloned().collect();
        files.sort();
        for file in files {
            let refs = self.decls_by_file[&file].clone();
            let file_node = self.file_nodes.get(&file).copied();
            for r in &refs {
                // A seed declaration already has its structural edges.
                let Some(si) = r.set else { continue };
                let set = &self.input.fact_sets[si];
                let node = self.nodes[r.node].clone();
                let (span, hash) = (r.span, node.content_hash);
                match node.kind {
                    NodeKind::Schema => {
                        if let Some(f) = file_node {
                            let prov = self.resolved_sql(&file, span, hash);
                            self.push_edge_once(f, EdgeKind::Migrates, r.node, prov, Vec::new());
                        }
                    }
                    NodeKind::Table | NodeKind::View => {
                        let schema = node
                            .qualified_name
                            .rsplit_once('.')
                            .map_or("public", |(s, _)| s)
                            .to_owned();
                        let schema_idx = self.ensure_schema(&schema);
                        let prov = self.resolved_sql(&file, span, hash);
                        self.push_edge_once(
                            schema_idx,
                            EdgeKind::Contains,
                            r.node,
                            prov.clone(),
                            Vec::new(),
                        );
                        if let Some(f) = file_node {
                            self.push_edge_once(f, EdgeKind::Migrates, r.node, prov, Vec::new());
                        }
                    }
                    NodeKind::Column | NodeKind::Index | NodeKind::Constraint => {
                        let table_q = node
                            .qualified_name
                            .rsplit_once(['.', '#'])
                            .map_or("", |(t, _)| t)
                            .to_owned();
                        if let Some(&t) = self.node_pos.get(&NodeId::new(format!("sql:{table_q}")))
                        {
                            let prov = self.resolved_sql(&file, span, hash);
                            self.push_edge_once(
                                t,
                                EdgeKind::Contains,
                                r.node,
                                prov.clone(),
                                Vec::new(),
                            );
                            if node.kind == NodeKind::Column
                                && let Some(f) = file_node
                            {
                                self.push_edge_once(f, EdgeKind::Migrates, t, prov, Vec::new());
                            }
                        }
                    }
                    NodeKind::Design | NodeKind::Runbook => {
                        if let Some(f) = file_node {
                            let prov =
                                self.declared(DeclaredSource::FrontMatter, &file, span, hash);
                            self.push_edge_once(f, EdgeKind::Contains, r.node, prov, Vec::new());
                        }
                    }
                    _ => {
                        let prov = self.syntactic(set, &RuleId::new("definition"), span, hash);
                        match (r.parent, file_node) {
                            (Some(p), _) => self.push_edge_once(
                                refs[p].node,
                                EdgeKind::Defines,
                                r.node,
                                prov,
                                Vec::new(),
                            ),
                            (None, Some(f)) => {
                                self.push_edge_once(
                                    f,
                                    EdgeKind::Contains,
                                    r.node,
                                    prov,
                                    Vec::new(),
                                );
                            }
                            (None, None) => {}
                        }
                        if node.kind == NodeKind::Impl {
                            self.impl_type_edge(set, r.node, &node, &file);
                        }
                    }
                }
            }
        }
    }

    fn ensure_schema(&mut self, schema: &str) -> usize {
        let id = sql_id(&[schema], None);
        self.node_pos.get(&id).copied().unwrap_or_else(|| {
            let node = Node::synthetic(id, NodeKind::Schema, schema, self.input.repo.clone())
                .with_attribute("implicit", "true");
            self.push_node(node)
        })
    }

    /// `Impl -USES_TYPE-> Type` for the type an impl block is for.
    fn impl_type_edge(&mut self, set: &FactSet, impl_idx: usize, node: &Node, file: &RepoPath) {
        let candidates = self.candidates(&node.name, usize::MAX, NodeKind::is_type);
        let ranked = rank(&self.index, file, candidates, None);
        let site = Site {
            method_call: false,
            builtin: false,
            report: false,
            external_import: false,
            inherits_external: false,
            receiver_known: false,
        };
        let Resolution::Linked {
            targets,
            confidence,
        } = decide(&ranked, site, self.input.options.max_candidates)
        else {
            return;
        };
        let Some(span) = node.span() else { return };
        for dst in targets {
            let prov = self
                .syntactic(
                    set,
                    &RuleId::new("definition.impl"),
                    span,
                    node.content_hash,
                )
                .with_confidence(confidence);
            let against = vec![self.node_id(dst)];
            self.push_edge_once(impl_idx, EdgeKind::UsesType, dst, prov, against);
        }
    }

    /// Declarations called `name` that pass `accept`, excluding `exclude`.
    fn candidates(
        &self,
        name: &str,
        exclude: usize,
        accept: impl Fn(&NodeKind) -> bool,
    ) -> Vec<(usize, &Node)> {
        self.index
            .by_name(name)
            .iter()
            .map(|&i| (i, &self.nodes[i]))
            .filter(|(i, n)| *i != exclude && accept(&n.kind))
            .collect()
    }

    // ---- pass 4: declared relations --------------------------------------------------

    fn declared_relations(&mut self) {
        let rels: Vec<(usize, DeclaredRelation)> = self
            .input
            .fact_sets
            .iter()
            .enumerate()
            .flat_map(|(si, set)| set.relations().map(move |r| (si, r.clone())))
            .collect();
        for (si, r) in rels {
            let set = &self.input.fact_sets[si];
            let src_id = match r.src_kind {
                NodeKind::Package => package_id(&r.src_name),
                _ => NodeId::new(r.src_name.clone()),
            };
            let Some(&src) = self.node_pos.get(&src_id) else {
                continue;
            };
            let dst = match r.dst_kind {
                NodeKind::Package => Some(self.ensure_external_package(&r.dst_name)),
                _ => self.node_pos.get(&NodeId::new(r.dst_name.clone())).copied(),
            };
            let Some(dst) = dst else { continue };
            if r.kind == EdgeKind::DependsOn {
                let (from, to) = (
                    self.node_id(src).into_string(),
                    self.node_id(dst).into_string(),
                );
                self.index
                    .deps_of_package
                    .entry(from)
                    .or_default()
                    .insert(to);
            }
            let prov = self.declared(r.source.clone(), &set.path, r.span, r.hash);
            self.push_edge_with(src, r.kind.clone(), dst, prov, Vec::new(), r.attributes);
        }
    }

    fn ensure_external_package(&mut self, coordinate: &str) -> usize {
        let id = package_id(coordinate);
        if let Some(&idx) = self.node_pos.get(&id) {
            return idx;
        }
        let name = coordinate
            .split_once('/')
            .map_or(coordinate, |(_, n)| n)
            .to_owned();
        let node = Node::synthetic(id, NodeKind::Package, name.clone(), self.input.repo.clone())
            .with_qualified_name(coordinate)
            .with_attribute("external", "true");
        let idx = self.push_node(node);
        self.register_package(coordinate, &name, idx);
        idx
    }

    // ---- pass 5: imports, then references ----------------------------------------------

    /// Innermost declaration node containing `span` in `file`, else the file node.
    fn enclosing(&self, file: &RepoPath, span: TextSpan) -> Option<usize> {
        let in_decls = self.decls_by_file.get(file).and_then(|refs| {
            let upper = refs.partition_point(|r| r.span.start() <= span.start());
            std::iter::successors(upper.checked_sub(1), |i| refs[*i].parent)
                .map(|i| &refs[i])
                .find(|r| r.span.start() <= span.start() && span.end() <= r.span.end())
                .map(|r| r.node)
        });
        in_decls.or_else(|| self.file_nodes.get(file).copied())
    }

    fn parent_decl(&self, file: &RepoPath, node: usize) -> Option<usize> {
        let refs = self.decls_by_file.get(file)?;
        let me = refs.iter().position(|r| r.node == node)?;
        refs[me].parent.map(|p| refs[p].node)
    }

    /// Every reference of `role`-matching facts, with its set index and source node.
    fn sites(&self, keep: impl Fn(&ReferenceRole) -> bool) -> Vec<(usize, usize, ReferenceFact)> {
        self.input
            .fact_sets
            .iter()
            .enumerate()
            .flat_map(|(si, set)| {
                set.references()
                    .filter(|r| keep(&r.role))
                    .filter_map(move |r| {
                        self.enclosing(&set.path, r.span)
                            .map(|src| (si, src, r.clone()))
                    })
            })
            .collect()
    }

    /// Imports first, so a file's imported packages are known when its calls resolve.
    fn imports(&mut self) {
        for (si, src, r) in self.sites(|role| *role == ReferenceRole::Import) {
            self.link_import(si, src, &r);
        }
    }

    /// Records the classes whose base type comes from outside the repository, so a
    /// `self` call inside them can be the base class's method.
    fn external_bases(&mut self) {
        let bases: Vec<(RepoPath, String)> = self
            .sites(|role| *role == ReferenceRole::Extends)
            .into_iter()
            .filter_map(|(si, _, r)| {
                let file = &self.input.fact_sets[si].path;
                let external = self.index.imports_external(file, &r.name)
                    || r.receiver
                        .as_deref()
                        .is_some_and(|root| self.index.imports_external(file, root));
                let class = r.scope.last()?.clone();
                external.then(|| (file.clone(), class))
            })
            .collect();
        for (file, class) in bases {
            self.external_bases.entry(file).or_default().insert(class);
        }
    }

    fn references(&mut self) {
        self.external_bases();
        for (si, src, r) in self.sites(|role| *role != ReferenceRole::Import) {
            match r.role {
                // A macro is never a function: `write!` does not call a `write`. Rust
                // macro declarations are not extracted yet, so a miss is not a gap.
                ReferenceRole::Call(CallForm::Macro) => {
                    self.link_named(si, src, &r, false, false, |k| *k == NodeKind::Macro);
                }
                ReferenceRole::Call(form) => {
                    self.link_named(si, src, &r, is_method(form), true, NodeKind::is_callable);
                }
                ReferenceRole::TypeUse if self.input.options.link_type_uses => {
                    self.link_named(si, src, &r, false, false, NodeKind::is_type);
                }
                ReferenceRole::Implements => self.link_named(si, src, &r, false, true, |k| {
                    matches!(k, NodeKind::Trait | NodeKind::Interface | NodeKind::Class)
                }),
                ReferenceRole::Extends => self.link_named(si, src, &r, false, true, |k| {
                    matches!(
                        k,
                        NodeKind::Class | NodeKind::Interface | NodeKind::Struct | NodeKind::Trait
                    )
                }),
                ReferenceRole::Read | ReferenceRole::Write => self.link_table(si, src, &r),
                // A path used as a value or in a pattern: a function handed to a caller, a
                // variant matched. Any symbol may be the target; a miss is not a gap.
                ReferenceRole::Mention => self.link_named(si, src, &r, false, false, |k| {
                    k.is_symbol() || *k == NodeKind::Module
                }),
                _ => {}
            }
        }
    }

    /// The resolved reference a precise index recorded inside `r`'s span, by name.
    ///
    /// A method call's span is the name token itself; a path call's span covers the
    /// whole path, in which case the rightmost occurrence of the name is the target.
    fn binding_at(&self, file: &RepoPath, r: &ReferenceFact) -> Option<&'a ResolvedReference> {
        let sites = self.bindings.get(file)?;
        let from = sites.partition_point(|b| b.span.start() < r.span.start());
        sites[from..]
            .iter()
            .take_while(|b| b.span.start() < r.span.end())
            .filter(|b| b.span.end() <= r.span.end() && b.name == r.name)
            .last()
            .copied()
    }

    /// The innermost declaration called `name` whose span covers `line` in `file`.
    fn declared_at(&self, file: &RepoPath, line: u32, name: &str) -> Option<usize> {
        // Declarations are sorted by start, so the last match is the innermost.
        self.decls_by_file
            .get(file)?
            .iter()
            .rev()
            .filter(|d| d.span.lines.start <= line && line <= d.span.lines.end)
            .map(|d| d.node)
            .find(|&n| self.nodes[n].name == name)
    }

    /// Links `r` through a precise binding when one covers the site.
    ///
    /// Returns `true` when the binding settled the reference: an edge was drawn to the
    /// declaration it names, or the target is outside the repository or a local
    /// binding and so is neither an edge nor a gap. Returns `false` when no index
    /// covers the site, or the named declaration is not in the graph, so name
    /// matching proceeds as usual.
    fn link_resolved(
        &mut self,
        src: usize,
        r: &ReferenceFact,
        file: &RepoPath,
        accept: &impl Fn(&NodeKind) -> bool,
    ) -> bool {
        let Some(binding) = self.binding_at(file, r) else {
            return false;
        };
        let SymbolTarget::Declared {
            file: declared_in,
            line,
            name,
        } = &binding.target
        else {
            return true;
        };
        let Some(dst) = self.declared_at(declared_in, *line, name).filter(|&d| {
            d != src
                && (accept(&self.nodes[d].kind)
                    || (r.role.is_call() && self.nodes[d].kind.is_type()))
        }) else {
            return false;
        };
        let prov = Provenance::resolved(
            binding.resolver.clone(),
            self.evidence(file, r.span, r.hash),
        );
        let against = vec![self.node_id(dst)];
        let kind = r.role.edge_kind();
        if r.role == ReferenceRole::TypeUse {
            self.push_edge_once(src, kind, dst, prov, against);
        } else {
            self.push_edge(src, kind, dst, prov, against);
        }
        true
    }

    /// The declarations the receiver of `r` narrows it to, when the receiver names
    /// something the file imported from the repository or the enclosing class.
    ///
    /// `db.connect()` after `from . import db` yields `connect` in `db.py`;
    /// `Repo.create()` after `from .db import Repo` yields `create` inside `Repo`;
    /// `self.save()` yields `save` inside an enclosing class or impl. `None` when the
    /// receiver is not such a name; an empty list when it is but declares no such
    /// member, in which case the caller falls back to matching by name.
    fn receiver_candidates(
        &self,
        file: &RepoPath,
        r: &ReferenceFact,
        accept: &impl Fn(&NodeKind) -> bool,
    ) -> Option<Vec<(usize, &Node)>> {
        let root = r.receiver.as_deref()?;
        let named = |d: &DeclRef| {
            let node = &self.nodes[d.node];
            node.name == r.name && accept(&node.kind)
        };
        if matches!(root, "self" | "this" | "cls" | "Self") {
            let decls = self.decls_by_file.get(file)?;
            return Some(
                decls
                    .iter()
                    .filter(|d| named(d))
                    .filter(|d| {
                        // `parent` indexes the file's declaration list, not the nodes.
                        d.parent.is_some_and(|p| {
                            r.scope.iter().any(|s| self.nodes[decls[p].node].name == *s)
                        })
                    })
                    .map(|d| (d.node, &self.nodes[d.node]))
                    .collect(),
            );
        }
        let targets = self.index.imported_targets_by_file.get(file)?.get(root)?;
        let found: Vec<(usize, &Node)> = targets
            .iter()
            .flat_map(|&target| {
                let decls = self.nodes[target]
                    .file()
                    .and_then(|f| self.decls_by_file.get(f))
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let members: Vec<&DeclRef> = if self.nodes[target].kind == NodeKind::File {
                    // Top-level declarations of the module first; anything in it otherwise.
                    let top: Vec<&DeclRef> = decls
                        .iter()
                        .filter(|d| named(d) && d.parent.is_none())
                        .collect();
                    if top.is_empty() {
                        decls.iter().filter(|d| named(d)).collect()
                    } else {
                        top
                    }
                } else {
                    decls
                        .iter()
                        .filter(|d| named(d) && d.parent.is_some_and(|p| decls[p].node == target))
                        .collect()
                };
                members
                    .into_iter()
                    .map(|d| (d.node, &self.nodes[d.node]))
                    .collect::<Vec<_>>()
            })
            .collect();
        // The same declaration may be reached through the module and the symbol.
        let mut seen = HashSet::new();
        Some(found.into_iter().filter(|(i, _)| seen.insert(*i)).collect())
    }

    fn link_named(
        &mut self,
        si: usize,
        src: usize,
        r: &ReferenceFact,
        method_call: bool,
        report: bool,
        accept: impl Fn(&NodeKind) -> bool,
    ) {
        let set = &self.input.fact_sets[si];
        let file = set.path.clone();
        if self.link_resolved(src, r, &file, &accept) {
            return;
        }
        let language = set.language.clone();
        let same_language = |n: &Node| match (&language, &n.language) {
            (Some(a), Some(b)) => same_family(a, b),
            _ => true,
        };
        let qualifier = r
            .path
            .iter()
            .rev()
            .nth(1)
            .filter(|q| !matches!(q.as_str(), "self" | "crate" | "super" | "this" | "Self"))
            .cloned();
        let typed: Vec<(usize, &Node)> = self
            .candidates(&r.name, src, &accept)
            .into_iter()
            .filter(|(_, n)| same_language(n))
            .collect();
        // `Foo(...)` in Python, TypeScript or JavaScript constructs a class.
        let by_name = if typed.is_empty() && r.role.is_call() {
            self.candidates(&r.name, src, NodeKind::is_type)
                .into_iter()
                .filter(|(_, n)| same_language(n))
                .collect()
        } else {
            typed
        };
        let (candidates, receiver_known) = match self.receiver_candidates(&file, r, &accept) {
            Some(narrowed) if !narrowed.is_empty() => (narrowed, true),
            _ => (by_name, false),
        };
        let ranked = rank(&self.index, &file, candidates, qualifier.as_deref());
        // A name the file imports from outside the repository is that dependency's,
        // whatever declarations here share it; so is a name reached through such an
        // import (`descriptor.Foo()`, `np.array()`).
        let external_import = self.index.imports_external(&file, &r.name)
            || r.path
                .first()
                .filter(|root| **root != r.name)
                .is_some_and(|root| self.index.imports_external(&file, root))
            || r.receiver
                .as_deref()
                .is_some_and(|root| self.index.imports_external(&file, root));
        let on_self = r
            .receiver
            .as_deref()
            .is_some_and(|root| matches!(root, "self" | "this" | "cls" | "Self"));
        let inherits_external = method_call
            && on_self
            && self
                .external_bases
                .get(&file)
                .is_some_and(|classes| r.scope.iter().rev().any(|name| classes.contains(name)));
        let site = Site {
            method_call,
            builtin: is_builtin_name(language.as_ref(), &r.name),
            report,
            external_import,
            inherits_external,
            receiver_known,
        };
        match decide(&ranked, site, self.input.options.max_candidates) {
            Resolution::Silent => {}
            Resolution::Unresolved(reason) => self.report(r, &file, reason),
            Resolution::Linked {
                targets,
                confidence,
            } => {
                let kind = r.role.edge_kind();
                for dst in targets {
                    let set = &self.input.fact_sets[si];
                    let prov = self
                        .syntactic(set, &r.rule, r.span, r.hash)
                        .with_confidence(confidence);
                    let against = vec![self.node_id(dst)];
                    if r.role == ReferenceRole::TypeUse {
                        self.push_edge_once(src, kind.clone(), dst, prov, against);
                    } else {
                        self.push_edge(src, kind.clone(), dst, prov, against);
                    }
                }
            }
        }
    }

    fn link_table(&mut self, si: usize, src: usize, r: &ReferenceFact) {
        let set = &self.input.fact_sets[si];
        let file = set.path.clone();
        let (schema, table) = match r.path.as_slice() {
            [s, t] => (Some(s.as_str()), t.as_str()),
            [t] => (None, t.as_str()),
            _ => (None, r.name.as_str()),
        };
        let candidates = self.index.tables(schema, table);
        let (targets, confidence) =
            match decide_table(candidates, self.input.options.max_candidates) {
                Resolution::Linked {
                    targets,
                    confidence,
                } => (targets, confidence),
                Resolution::Unresolved(reason) => {
                    self.report(r, &file, reason);
                    return;
                }
                Resolution::Silent => return,
            };
        let kind = match (&set.language, &r.role) {
            (Some(Language::Sql), ReferenceRole::Write) => EdgeKind::Migrates,
            _ => r.role.edge_kind(),
        };
        // The symbol that holds a query literal touches the table too.
        let holder = (self.nodes[src].kind == NodeKind::Query)
            .then(|| self.parent_decl(&file, src))
            .flatten();
        for dst in targets {
            let prov = self
                .resolved_sql(&file, r.span, r.hash)
                .with_confidence(confidence);
            let against = vec![self.node_id(dst)];
            self.push_edge_once(src, kind.clone(), dst, prov.clone(), against.clone());
            if let Some(parent) = holder {
                self.push_edge_once(parent, kind.clone(), dst, prov, against);
            }
        }
    }

    fn link_import(&mut self, si: usize, src: usize, r: &ReferenceFact) {
        let set = &self.input.fact_sets[si];
        let file = set.path.clone();
        let language = set.language.clone();
        let relative = r.path.first().is_some_and(|s| s == "." || s == "..");

        let targets: Vec<usize> = match language {
            Some(Language::TypeScript | Language::Tsx | Language::JavaScript) => {
                if relative {
                    self.module_targets(&file, &r.path, language.as_ref())
                } else {
                    self.npm_package(&r.path).into_iter().collect()
                }
            }
            Some(Language::Bash | Language::Rhai) => {
                // `source ./x.sh` resolves against the working directory, which is
                // usually the repository root; try the script's directory first.
                self.resolve_relative_file(&file, &r.path, language.as_ref())
                    .or_else(|| {
                        self.resolve_relative_file(
                            &RepoPath::new("root"),
                            &r.path,
                            language.as_ref(),
                        )
                    })
                    .into_iter()
                    .collect()
            }
            Some(Language::Python) => {
                let found = self.module_targets(&file, &r.path, language.as_ref());
                if found.is_empty() {
                    r.path
                        .first()
                        .and_then(|p| self.package_by_name.get(&package_key("pypi", p)).copied())
                        .into_iter()
                        .collect()
                } else {
                    found
                }
            }
            Some(Language::Rust) => self.rust_import_targets(src, &file, r),
            _ => Vec::new(),
        };
        let root = r
            .path
            .first()
            .filter(|s| !matches!(s.as_str(), "." | ".."))
            .cloned();
        if self.is_external_import(language.as_ref(), r, relative, &targets) {
            let names = self
                .index
                .external_names_by_file
                .entry(file.clone())
                .or_default();
            names.insert(r.name.clone());
            names.extend(r.binding.clone());
            // `import a.b` binds `a`; recording the root too covers that form.
            names.extend(root);
        } else if !targets.is_empty() {
            // The bound name stands for what the import resolved to: a receiver that
            // is this name narrows a reference to those declarations.
            let bound = self
                .index
                .imported_targets_by_file
                .entry(file.clone())
                .or_default();
            // `from m import X` resolved to the module and `X` inside it: the name
            // stands for `X`, not the module. A module-only import stands for the file.
            let symbols: Vec<usize> = targets
                .iter()
                .copied()
                .filter(|&t| self.nodes[t].kind != NodeKind::File)
                .collect();
            let stands_for = if symbols.is_empty() {
                targets.clone()
            } else {
                symbols
            };
            let binding = r.binding.clone().unwrap_or_else(|| r.name.clone());
            bound
                .entry(binding)
                .or_default()
                .extend(stands_for.iter().copied());
            if let Some(root) = root.filter(|root| *root != r.name && r.binding.is_none()) {
                bound.entry(root).or_default().extend(stands_for);
            }
        }
        if targets.is_empty() {
            return;
        }
        let imported: Vec<String> = targets
            .iter()
            .filter(|&&t| self.nodes[t].kind == NodeKind::Package)
            .map(|&t| self.nodes[t].id.to_string())
            .collect();
        self.index
            .imports_of_file
            .entry(file.clone())
            .or_default()
            .extend(imported);
        self.index
            .imported_by_file
            .entry(file.clone())
            .or_default()
            .extend(targets.iter().map(|&t| self.nodes[t].id.to_string()));
        // A module, the package it belongs to and the symbol imported from it are one
        // binding, not alternatives; only same-kind targets share the confidence.
        let alternatives = targets
            .iter()
            .fold(HashMap::<&NodeKind, usize>::new(), |mut by_kind, &t| {
                *by_kind.entry(&self.nodes[t].kind).or_default() += 1;
                by_kind
            })
            .into_values()
            .max()
            .unwrap_or(1);
        let confidence = Confidence::new(1.0 / alternatives as f32);
        for dst in targets {
            let set = &self.input.fact_sets[si];
            let prov = self
                .syntactic(set, &r.rule, r.span, r.hash)
                .with_confidence(confidence);
            let against = vec![self.node_id(dst)];
            self.push_edge_once(src, EdgeKind::Imports, dst, prov, against);
        }
    }

    /// Whether an import binds names from outside the repository.
    ///
    /// True when every target is a package the repository does not contain, when a
    /// Rust `use` names the standard library, or when a non-relative Python or
    /// JavaScript import resolves to nothing in the repository, which is a dependency
    /// no manifest lists (`google.protobuf` from the `protobuf` distribution) far more
    /// often than a module the walker missed.
    fn is_external_import(
        &self,
        language: Option<&Language>,
        r: &ReferenceFact,
        relative: bool,
        targets: &[usize],
    ) -> bool {
        let root = r.path.first().map_or("", String::as_str);
        if targets.is_empty() {
            return match language {
                Some(Language::Rust) => matches!(root, "std" | "core" | "alloc"),
                Some(
                    Language::Python | Language::TypeScript | Language::Tsx | Language::JavaScript,
                ) => !relative,
                _ => false,
            };
        }
        targets.iter().all(|&t| {
            self.nodes[t].kind == NodeKind::Package
                && !self.package_dirs.iter().any(|(_, p)| *p == t)
        })
    }

    /// The file an import path names, or the file its prefix names together with the
    /// symbol the last segment names inside it (`from app.db import connect`,
    /// `import { Client } from "./client"`).
    fn module_targets(
        &self,
        file: &RepoPath,
        path: &[String],
        language: Option<&Language>,
    ) -> Vec<usize> {
        let resolve = |segments: &[String]| match language {
            Some(Language::Python) => self.resolve_python_module(file, segments),
            _ => self.resolve_relative_file(file, segments, language),
        };
        if let Some(module) = resolve(path) {
            return vec![module];
        }
        let Some((symbol, module_path)) = path.split_last() else {
            return Vec::new();
        };
        let Some(module) = resolve(module_path) else {
            return Vec::new();
        };
        let declared = self.nodes[module].file().and_then(|module_file| {
            self.decls_by_file
                .get(module_file)?
                .iter()
                .map(|d| d.node)
                .find(|&n| self.nodes[n].name == *symbol && self.nodes[n].kind.is_symbol())
        });
        std::iter::once(module).chain(declared).collect()
    }

    /// Targets of a Rust `use`: the crate's package node when the root names a
    /// dependency, plus the imported symbols when the path is specific enough.
    fn rust_import_targets(&self, src: usize, file: &RepoPath, r: &ReferenceFact) -> Vec<usize> {
        let root = r.path.first().map_or("", String::as_str);
        if matches!(root, "std" | "core" | "alloc") {
            return Vec::new();
        }
        let same_package = RUST_BUILTIN_ROOTS.contains(&root);
        let package_idx = if same_package {
            None
        } else {
            self.package_by_name
                .get(&package_key("cargo", root))
                .copied()
        };
        let symbols: Vec<usize> = if r.path.len() >= 2 || same_package {
            let candidates = self.candidates(&r.name, src, |_| true);
            let qualifier = r
                .path
                .iter()
                .rev()
                .nth(1)
                .filter(|q| !RUST_BUILTIN_ROOTS.contains(&q.as_str()))
                .cloned();
            let ranked: Vec<usize> = rank(&self.index, file, candidates, qualifier.as_deref())
                .candidates
                .into_iter()
                .filter(|(tier, i)| match package_idx {
                    Some(p) => self.nodes[*i]
                        .file()
                        .and_then(|f| self.index.package_of_file.get(f))
                        .is_some_and(|owner| *owner == self.nodes[p].id.to_string()),
                    None => *tier != Tier::SameRepo || !same_package,
                })
                .map(|(_, i)| i)
                .collect();
            if !ranked.is_empty() && ranked.len() <= self.input.options.max_candidates {
                ranked
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        package_idx.into_iter().chain(symbols).collect()
    }

    fn npm_package(&self, path: &[String]) -> Option<usize> {
        let first = path.first()?;
        let name = if first.starts_with('@') {
            format!("{first}/{}", path.get(1)?)
        } else {
            first.clone()
        };
        self.package_by_name
            .get(&package_key("npm", &name))
            .copied()
    }

    fn resolve_relative_file(
        &self,
        from: &RepoPath,
        segments: &[String],
        language: Option<&Language>,
    ) -> Option<usize> {
        let base = segments
            .iter()
            .fold(
                from.parent()
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<&str>>(),
                |mut parts, seg| {
                    match seg.as_str() {
                        "." => {}
                        ".." => {
                            parts.pop();
                        }
                        other => parts.push(other),
                    }
                    parts
                },
            )
            .join("/");
        let exts: &[&str] = match language {
            Some(Language::TypeScript | Language::Tsx | Language::JavaScript) => &[
                "",
                ".ts",
                ".tsx",
                ".js",
                ".mjs",
                ".cjs",
                ".jsx",
                ".d.ts",
                "/index.ts",
                "/index.tsx",
                "/index.js",
            ],
            Some(Language::Rhai) => &["", ".rhai"],
            _ => &[""],
        };
        // `./x.js` in TypeScript sources refers to `./x.ts`.
        let js_as_ts = base
            .rsplit_once('.')
            .filter(|(_, e)| e.eq_ignore_ascii_case("js"))
            .map(|(stem, _)| format!("{stem}.ts"));
        exts.iter()
            .map(|ext| format!("{base}{ext}"))
            .chain(js_as_ts)
            .find_map(|candidate| self.file_nodes.get(&RepoPath::new(candidate)).copied())
    }

    fn resolve_python_module(&self, from: &RepoPath, segments: &[String]) -> Option<usize> {
        let relative = segments.first().is_some_and(|s| s == "." || s == "..");
        let (roots, parts): (Vec<String>, Vec<&str>) = if relative {
            let mut dir: Vec<&str> = from.parent().split('/').filter(|s| !s.is_empty()).collect();
            let skip_same = usize::from(segments.first().is_some_and(|s| s == "."));
            let ups = segments
                .iter()
                .skip(skip_same)
                .take_while(|s| *s == "..")
                .count();
            dir.truncate(dir.len().saturating_sub(ups));
            (
                vec![dir.join("/")],
                segments
                    .iter()
                    .skip(skip_same + ups)
                    .map(String::as_str)
                    .collect(),
            )
        } else {
            let roots = std::iter::once(String::new())
                .chain(std::iter::once("src".to_owned()))
                .chain(
                    self.package_dirs
                        .iter()
                        .flat_map(|(dir, _)| [dir.clone(), format!("{dir}/src")]),
                )
                .collect();
            (roots, segments.iter().map(String::as_str).collect())
        };
        let module = parts.join("/");
        roots
            .iter()
            .map(|root| {
                if root.is_empty() {
                    String::new()
                } else {
                    format!("{root}/")
                }
            })
            .flat_map(|prefix| {
                [
                    format!("{prefix}{module}.py"),
                    format!("{prefix}{module}/__init__.py"),
                ]
            })
            .find_map(|candidate| self.file_nodes.get(&RepoPath::new(candidate)).copied())
    }

    // ---- pass 6: literals ------------------------------------------------------------

    fn literals(&mut self) {
        for si in 0..self.input.fact_sets.len() {
            let set = &self.input.fact_sets[si];
            let Some(doc) = self.node_pos.get(&doc_id(&set.path)).copied() else {
                continue;
            };
            let kind = if self.nodes[doc].kind == NodeKind::Design {
                EdgeKind::DesignedBy
            } else {
                EdgeKind::DocumentedBy
            };
            let cited: Vec<(usize, TextSpan, ContentHash)> = set
                .literals()
                .filter(|l| l.role == LiteralRole::Path)
                .filter_map(|l| self.path_target(&l.value).map(|t| (t, l.span, l.hash)))
                .collect();
            for (target, span, hash) in cited {
                let set = &self.input.fact_sets[si];
                let prov = self.declared(DeclaredSource::Citation, &set.path, span, hash);
                self.push_edge_once(target, kind.clone(), doc, prov, Vec::new());
            }
        }
    }

    /// The file or package node a documentation path names.
    fn path_target(&self, value: &str) -> Option<usize> {
        let value = value.trim_start_matches("./").trim_end_matches('/');
        self.file_nodes
            .get(&RepoPath::new(value))
            .copied()
            .or_else(|| {
                self.package_dirs
                    .iter()
                    .find(|(d, _)| d == value)
                    .map(|(_, i)| *i)
            })
    }

    // ---- pass 7: dispatch ------------------------------------------------------------

    /// `Trait::m -DISPATCHES_TO-> Impl::m` for every method an implementation defines
    /// under the same name as a method of the trait or interface it implements.
    ///
    /// A call to the trait method reaches these bodies at runtime; without the edge a
    /// path stops at the trait and `affected` never climbs from an implementation to the
    /// callers of its contract.
    fn dispatch(&mut self) {
        let position = |id: &NodeId| self.node_pos.get(id).copied();
        let seeded = |kind: EdgeKind| {
            self.seed_links
                .iter()
                .filter(move |(_, k, _)| *k == kind)
                .map(|(s, _, d)| (*s, *d))
        };
        let implements: Vec<(usize, usize)> = self
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Implements)
            .filter_map(|e| Some((position(&e.src)?, position(&e.dst)?)))
            .chain(seeded(EdgeKind::Implements))
            .collect();
        let children: HashMap<usize, Vec<usize>> = self
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Defines)
            .filter_map(|e| Some((position(&e.src)?, position(&e.dst)?)))
            .chain(seeded(EdgeKind::Defines))
            .fold(HashMap::new(), |mut acc, (parent, child)| {
                acc.entry(parent).or_default().push(child);
                acc
            });
        let methods = |owner: usize| -> Vec<usize> {
            children
                .get(&owner)
                .into_iter()
                .flatten()
                .copied()
                .filter(|&m| self.nodes[m].kind == NodeKind::Method)
                .collect()
        };
        let pairs: Vec<(usize, usize)> = implements
            .iter()
            .flat_map(|&(implementor, contract)| {
                let by_name: HashMap<&str, usize> = methods(contract)
                    .into_iter()
                    .map(|m| (self.nodes[m].name.as_str(), m))
                    .collect();
                methods(implementor)
                    .into_iter()
                    .filter_map(|m| by_name.get(self.nodes[m].name.as_str()).map(|&c| (c, m)))
                    .collect::<Vec<_>>()
            })
            .collect();
        for (contract_method, implementation) in pairs {
            let Some(evidence) = self.evidence_of(implementation) else {
                continue;
            };
            let prov = Provenance::resolved(ResolverId::new(DISPATCH_RESOLVER), evidence);
            let against = vec![self.node_id(contract_method), self.node_id(implementation)];
            self.push_edge_once(
                contract_method,
                EdgeKind::DispatchesTo,
                implementation,
                prov,
                against,
            );
        }
    }

    // ---- pass 8: tests ---------------------------------------------------------------

    fn tests(&mut self) {
        let tested: Vec<(usize, usize, Provenance)> = self
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .filter_map(|e| Some((*self.node_pos.get(&e.src)?, *self.node_pos.get(&e.dst)?, e)))
            .filter(|(s, _, _)| self.nodes[*s].kind == NodeKind::Test)
            .map(|(s, d, e)| (d, s, e.provenance.clone()))
            .collect();
        for (symbol, test, prov) in tested {
            self.push_edge_once(symbol, EdgeKind::TestedBy, test, prov, Vec::new());
        }
    }

    fn finish(self) -> GraphDelta {
        let Self {
            mut nodes,
            seed_count,
            dirty_seeds,
            edges,
            unresolved,
            ..
        } = self;
        let fresh = nodes.split_off(seed_count);
        let changed_seeds = nodes
            .into_iter()
            .enumerate()
            .filter(|(i, _)| dirty_seeds.contains(i))
            .map(|(_, n)| n);
        GraphDelta {
            nodes: changed_seeds.chain(fresh).collect(),
            edges,
            unresolved,
        }
    }
}

/// Stable identifier and qualified name for a declaration.
/// The language a declaration belongs to: the file's, unless the pack exposed it to
/// another (`definition.function.rhai` on a Rust `register_fn("name", ..)`).
fn declared_language(set: &FactSet, d: &DeclarationFact) -> Option<Language> {
    d.attributes
        .get("language")
        .and_then(|l| l.parse().ok())
        .or_else(|| set.language.clone())
}

fn declaration_identity(set: &FactSet, d: &DeclarationFact) -> (NodeId, String) {
    let language = declared_language(set, d);
    // A function a host exposes to another language is known there by its bare name,
    // not by the Rust function that registered it: `rhai:modules/proc.rs::sh`, so that
    // `proc::sh(..)` in a script matches it through the file stem.
    if d.attributes.contains_key("language") {
        return (
            symbol_id(language.as_ref(), &set.path, &d.name),
            d.name.clone(),
        );
    }
    let schema = || d.scope.first().map_or("public", String::as_str);
    let table = || d.scope.get(1).map_or("", String::as_str);
    match &d.kind {
        NodeKind::Schema => (sql_id(&[&d.name], None), d.name.clone()),
        NodeKind::Table | NodeKind::View => (
            sql_id(&[schema(), &d.name], None),
            format!("{}.{}", schema(), d.name),
        ),

        NodeKind::Column => (
            sql_id(&[schema(), table(), &d.name], None),
            format!("{}.{}.{}", schema(), table(), d.name),
        ),
        NodeKind::Index | NodeKind::Constraint => (
            sql_id(&[schema(), table()], Some(&d.name)),
            format!("{}.{}#{}", schema(), table(), d.name),
        ),
        NodeKind::Design | NodeKind::Runbook => (doc_id(&set.path), d.name.clone()),
        NodeKind::Query => {
            let fp8: String = d
                .attributes
                .get("fingerprint")
                .map_or("", String::as_str)
                .chars()
                .take(8)
                .collect();
            let q = match d.scope.as_slice() {
                [] => format!("query#{fp8}"),
                scope => format!("{}::query#{fp8}", scope.join("::")),
            };
            (
                symbol_id(language.as_ref(), &set.path, &q),
                d.qualified_name(),
            )
        }
        NodeKind::Impl => {
            let q = match d.attributes.get("trait") {
                Some(t) => format!("{}#impl({t})", d.qualified_name()),
                None => format!("{}#impl", d.qualified_name()),
            };
            (symbol_id(language.as_ref(), &set.path, &q), q)
        }
        _ => {
            let q = d.qualified_name();
            (symbol_id(language.as_ref(), &set.path, &q), q)
        }
    }
}

/// Sorts declarations by span and links each to its innermost container.
fn with_parents(mut refs: Vec<DeclRef>) -> Vec<DeclRef> {
    refs.sort_by(|a, b| {
        a.span
            .start()
            .cmp(&b.span.start())
            .then(b.span.end().cmp(&a.span.end()))
    });
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..refs.len() {
        while stack.last().is_some_and(|&top| {
            refs[top].span.end() <= refs[i].span.start() || refs[top].span == refs[i].span
        }) {
            stack.pop();
        }
        refs[i].parent = stack.last().copied();
        stack.push(i);
    }
    refs
}

pub(crate) fn normalize_pkg(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}

/// The registry part of a package coordinate: `cargo` in `cargo/serde`.
fn registry_of(coordinate: &str) -> &str {
    coordinate.split_once('/').map_or("", |(r, _)| r)
}

/// The lookup key for a package: registry plus normalized name, so a Rust `use` never
/// resolves to the `PyPI` or npm package that shares the crate's name.
fn package_key(registry: &str, name: &str) -> String {
    format!("{registry}/{}", normalize_pkg(name))
}

/// Rust prelude items and the methods of `Option`, `Result`, strings and collections.
const BUILTIN_COMMON: &[&str] = &[
    "clone",
    "to_string",
    "to_owned",
    "into",
    "from",
    "as_ref",
    "as_mut",
    "as_str",
    "len",
    "is_empty",
    "iter",
    "into_iter",
    "iter_mut",
    "map",
    "filter",
    "collect",
    "unwrap",
    "expect",
    "unwrap_or",
    "unwrap_or_else",
    "unwrap_or_default",
    "ok",
    "ok_or",
    "ok_or_else",
    "and_then",
    "or_else",
    "map_err",
    "push",
    "pop",
    "get",
    "insert",
    "remove",
    "contains",
    "contains_key",
    "entry",
    "or_default",
    "or_insert",
    "or_insert_with",
    "first",
    "last",
    "next",
    "enumerate",
    "zip",
    "rev",
    "take",
    "skip",
    "any",
    "all",
    "find",
    "position",
    "sum",
    "count",
    "min",
    "max",
    "sort",
    "sort_by",
    "sort_by_key",
    "dedup",
    "extend",
    "join",
    "split",
    "trim",
    "starts_with",
    "ends_with",
    "parse",
    "format",
    "write",
    "writeln",
    "println",
    "eprintln",
    "print",
    "eprint",
    "panic",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "vec",
    "dbg",
    "todo",
    "unimplemented",
    "unreachable",
    "matches",
    "cfg",
    "include_str",
    "include_bytes",
    "env",
    "concat",
    "stringify",
    "default",
    "new",
    "with_capacity",
    "await",
    "send",
    "recv",
    "lock",
    "read",
    "spawn",
    "sleep",
    "chars",
    "bytes",
    "lines",
    "to_vec",
    "to_lowercase",
    "to_uppercase",
    "replace",
    "cmp",
    "partial_cmp",
    "add",
    "sub",
    "mul",
    "div",
    "neg",
    "not",
    "index",
    "index_mut",
    "deref",
    "deref_mut",
    "write_str",
    "write_fmt",
    "poll",
    "call",
    "eq",
    "ne",
    "hash",
    "fmt",
    "display",
    "drop",
    "then",
    "then_some",
    "is_some",
    "is_none",
    "is_ok",
    "is_err",
    "copied",
    "cloned",
    "as_bytes",
    "as_slice",
    "as_deref",
    "borrow",
    "borrow_mut",
    "truncate",
    "clear",
    "keys",
    "values",
    "flat_map",
    "filter_map",
    "fold",
    "for_each",
    "chain",
    "context",
    "with_context",
    "bail",
    "anyhow",
    "ensure",
    "Ok",
    "Err",
    "Some",
    "None",
    "Box",
    "Vec",
    "String",
    "Option",
    "Result",
    "Arc",
    "Rc",
    "HashMap",
    "HashSet",
    "BTreeMap",
    "BTreeSet",
    "Default",
    "Into",
    "From",
    "Iterator",
    "Self",
    "self",
];

/// Python builtins and the common methods of its standard types.
const BUILTIN_PYTHON: &[&str] = &[
    "print",
    "len",
    "range",
    "str",
    "int",
    "float",
    "list",
    "dict",
    "set",
    "tuple",
    "bool",
    "isinstance",
    "getattr",
    "setattr",
    "hasattr",
    "open",
    "enumerate",
    "zip",
    "sorted",
    "min",
    "max",
    "sum",
    "any",
    "all",
    "abs",
    "round",
    "type",
    "super",
    "iter",
    "next",
    "format",
    "join",
    "split",
    "strip",
    "append",
    "extend",
    "items",
    "keys",
    "values",
    "get",
    "pop",
    "update",
    "startswith",
    "endswith",
    "replace",
    "lower",
    "upper",
    "encode",
    "decode",
    "read",
    "write",
    "close",
    "execute",
    "fetchone",
    "fetchall",
    "commit",
    "rollback",
    "cursor",
    "connect",
    "dumps",
    "loads",
    "Exception",
    "ValueError",
    "TypeError",
    "KeyError",
    "RuntimeError",
    "object",
    "property",
    "staticmethod",
    "classmethod",
    "dataclass",
    "field",
    "Path",
    "datetime",
    "timedelta",
];

/// JavaScript and TypeScript globals, array and promise methods, test framework names.
const BUILTIN_JS: &[&str] = &[
    "console",
    "log",
    "warn",
    "error",
    "info",
    "debug",
    "require",
    "fetch",
    "then",
    "catch",
    "finally",
    "map",
    "filter",
    "reduce",
    "forEach",
    "find",
    "some",
    "every",
    "push",
    "pop",
    "shift",
    "unshift",
    "slice",
    "splice",
    "concat",
    "join",
    "split",
    "trim",
    "toString",
    "toLowerCase",
    "toUpperCase",
    "includes",
    "indexOf",
    "startsWith",
    "endsWith",
    "replace",
    "match",
    "test",
    "keys",
    "values",
    "entries",
    "assign",
    "freeze",
    "stringify",
    "parse",
    "JSON",
    "Object",
    "Array",
    "Promise",
    "resolve",
    "reject",
    "all",
    "setTimeout",
    "clearTimeout",
    "setInterval",
    "clearInterval",
    "Error",
    "Map",
    "Set",
    "Date",
    "now",
    "Math",
    "max",
    "min",
    "floor",
    "ceil",
    "round",
    "Number",
    "String",
    "Boolean",
    "Symbol",
    "useState",
    "useEffect",
    "useMemo",
    "useCallback",
    "useRef",
    "describe",
    "it",
    "expect",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
    "jest",
    "vi",
    "mock",
    "toBe",
    "toEqual",
    "toHaveBeenCalled",
    "get",
    "set",
    "has",
    "delete",
    "add",
    "clear",
    "size",
    "length",
    "bind",
    "call",
    "apply",
    "addEventListener",
    "removeEventListener",
    "querySelector",
    "createElement",
    "appendChild",
    "preventDefault",
    "stopPropagation",
];

/// Shell builtins and the binaries scripts call most.
const BUILTIN_BASH: &[&str] = &[
    "echo", "printf", "cd", "ls", "cat", "grep", "sed", "awk", "cut", "sort", "uniq", "head",
    "tail", "tr", "wc", "find", "xargs", "mkdir", "rm", "cp", "mv", "ln", "chmod", "chown",
    "touch", "test", "true", "false", "exit", "return", "export", "set", "unset", "local",
    "declare", "readonly", "shift", "read", "eval", "exec", "trap", "wait", "sleep", "date", "env",
    "dirname", "basename", "realpath", "pwd", "which", "command", "type", "source", "if", "then",
    "else", "fi", "for", "while", "do", "done", "case", "esac", "git", "cargo", "rustup", "npm",
    "npx", "pnpm", "yarn", "node", "python", "python3", "pip", "uv", "docker", "kubectl", "helm",
    "mise", "just", "make", "curl", "wget", "jq", "yq", "tar", "gzip", "unzip", "ssh", "scp",
    "rsync", "sudo", "brew", "apt", "apt-get", "psql", "pg_dump", "sqlx", "tee", "diff", "patch",
    "install",
];

/// Names the standard library or runtime provides; a miss on these is not a gap.
pub(crate) fn is_builtin_name(language: Option<&Language>, name: &str) -> bool {
    match language {
        Some(Language::Python) => BUILTIN_PYTHON.contains(&name) || BUILTIN_COMMON.contains(&name),
        Some(Language::TypeScript | Language::Tsx | Language::JavaScript) => {
            BUILTIN_JS.contains(&name)
        }
        Some(Language::Bash) => BUILTIN_BASH.contains(&name),
        Some(Language::Rhai) => BUILTIN_RHAI.contains(&name) || BUILTIN_COMMON.contains(&name),
        _ => BUILTIN_COMMON.contains(&name),
    }
}

/// Functions and methods the Rhai standard packages provide, which no script or host
/// declares: a call to one with no declaration in reach is not a gap.
const BUILTIN_RHAI: &[&str] = &[
    "print",
    "debug",
    "type_of",
    "is_def_var",
    "is_def_fn",
    "is_shared",
    "eval",
    "call",
    "curry",
    "is_anonymous",
    "Fn",
    "sleep",
    "timestamp",
    "elapsed",
    "pop",
    "shift",
    "truncate",
    "pad",
    "extract",
    "splice",
    "reverse",
    "retain",
    "drain",
    "dedup",
    "reduce",
    "reduce_rev",
    "some",
    "all",
    "none",
    "find_map",
    "index_of",
    "fill_with",
    "mixin",
    "chars",
    "split_rev",
    "to_upper",
    "to_lower",
    "make_upper",
    "make_lower",
    "sub_string",
    "crop",
    "to_debug",
    "to_int",
    "to_float",
    "to_decimal",
    "to_binary",
    "to_octal",
    "to_hex",
    "to_radians",
    "to_degrees",
    "parse_int",
    "parse_float",
    "parse_json",
    "sign",
    "fraction",
    "sqrt",
    "exp",
    "ln",
    "log",
    "sin",
    "cos",
    "tan",
    "is_zero",
    "is_odd",
    "is_even",
    "bits",
    "bit",
    "set_bit",
    "range",
    "zip",
    "shuffle",
    "to_chars",
    "tag",
    "set_tag",
    "for_each",
    "contains",
    "starts_with",
    "ends_with",
    "trim",
    "split",
    "replace",
    "keys",
    "values",
    "is_empty",
    "abs",
    "min",
    "max",
    "floor",
    "ceil",
    "round",
    "int",
    "to_string",
    "len",
    "push",
    "insert",
    "remove",
    "clear",
    "append",
    "filter",
    "map",
    "sort",
    "get",
    "set",
    "take",
];

/// TypeScript, TSX and JavaScript share declarations; every other language stands alone.
fn same_family(a: &Language, b: &Language) -> bool {
    let js = |l: &Language| {
        matches!(
            l,
            Language::TypeScript | Language::Tsx | Language::JavaScript
        )
    };
    a == b || (js(a) && js(b))
}

#[cfg(test)]
mod tests {

    use surveyor_extract::{
        Extractors, ManifestExtractor, RhaiExtractor, SqlDialect, SqlExtractor,
        TreeSitterExtractor, extract_units,
    };
    use surveyor_model::{CommitId, DerivationKind, Fact, LineSpan, RepoId, SourceUnit};

    use super::*;
    use crate::input::FileEntry;

    fn unit(path: &str, text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("demo"),
            RepoPath::new(path),
            CommitId::new("c0"),
            text.into(),
        )
    }

    fn linked(units: &[SourceUnit]) -> (GraphDelta, BTreeMap<String, usize>) {
        let extractors = Extractors::new()
            .with(TreeSitterExtractor::new())
            .with(RhaiExtractor::new())
            .with(SqlExtractor::new(SqlDialect::PostgreSql, "public"))
            .with(ManifestExtractor::new());
        let extraction = extract_units(&extractors, units, SqlDialect::PostgreSql);
        assert!(extraction.errors.is_empty(), "{:?}", extraction.errors);
        let files = extraction.units.iter().map(|u| FileEntry {
            path: u.path.clone(),
            language: u.language.clone(),
            content_hash: u.content_hash,
            bytes: u.bytes,
            lines: u.lines,
        });
        let input = LinkInput::new(RepoId::new("demo"), CommitId::new("c0"))
            .with_files(files)
            .with_fact_sets(extraction.fact_sets);
        let delta = link(&input);
        let pos = delta
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.to_string(), i))
            .collect();
        (delta, pos)
    }

    fn edge<'a>(delta: &'a GraphDelta, src: &str, kind: &EdgeKind, dst: &str) -> Option<&'a Edge> {
        delta
            .edges
            .iter()
            .find(|e| e.src.as_str() == src && &e.kind == kind && e.dst.as_str() == dst)
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "one fixture, many assertions")]
    fn rust_calls_impls_tests_and_sql() {
        let units = vec![
            unit(
                "Cargo.toml",
                "[package]\nname = \"outbox\"\n\n[dependencies]\nsqlx = \"0.8\"\n",
            ),
            unit("src/lib.rs", "pub mod relay;\npub use relay::Relay;\n"),
            unit(
                "src/relay.rs",
                r#"pub trait Drain { fn drain(&mut self) -> usize; }
pub struct Relay;
impl Drain for Relay {
    fn drain(&mut self) -> usize { flush_all(); 1 }
}
pub fn flush_all() {}
pub async fn load(pool: &sqlx::PgPool) {
    sqlx::query!("SELECT id FROM cw.outbox WHERE sent = false").fetch_all(pool).await;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drains() { Relay.drain(); flush_all(); }
}
"#,
            ),
            unit(
                "migrations/001_outbox.sql",
                "CREATE SCHEMA cw;\nCREATE TABLE cw.outbox (id uuid PRIMARY KEY, sent boolean NOT NULL);\n",
            ),
        ];
        let (delta, _) = linked(&units);
        let ids: Vec<&str> = delta.nodes.iter().map(|n| n.id.as_str()).collect();
        for expected in [
            "pkg:cargo/outbox",
            "pkg:cargo/sqlx",
            "rust:src/relay.rs::Relay",
            "rust:src/relay.rs::Relay#impl(Drain)",
            "rust:src/relay.rs::Relay::drain",
            "sql:cw.outbox",
            "sql:cw.outbox.sent",
        ] {
            assert!(ids.contains(&expected), "{expected} missing from {ids:?}");
        }
        assert!(
            edge(
                &delta,
                "pkg:cargo/outbox",
                &EdgeKind::DependsOn,
                "pkg:cargo/sqlx"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "pkg:cargo/outbox",
                &EdgeKind::Contains,
                "file:src/relay.rs"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "file:src/relay.rs",
                &EdgeKind::Contains,
                "rust:src/relay.rs::Relay"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "rust:src/relay.rs::Relay#impl(Drain)",
                &EdgeKind::Defines,
                "rust:src/relay.rs::Relay::drain"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "rust:src/relay.rs::Relay#impl(Drain)",
                &EdgeKind::UsesType,
                "rust:src/relay.rs::Relay"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "rust:src/relay.rs::Relay#impl(Drain)",
                &EdgeKind::Implements,
                "rust:src/relay.rs::Drain"
            )
            .is_some()
        );
        let call = edge(
            &delta,
            "rust:src/relay.rs::Relay::drain",
            &EdgeKind::Calls,
            "rust:src/relay.rs::flush_all",
        )
        .expect("call edge");
        assert!((call.provenance.confidence.get() - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            call.provenance.evidence[0].location.span.lines,
            LineSpan::new(4, 4)
        );
        assert!(
            edge(
                &delta,
                "rust:src/relay.rs::flush_all",
                &EdgeKind::TestedBy,
                "rust:src/relay.rs::tests::drains"
            )
            .is_some()
        );
        // `Relay.drain()` in the test resolves to the trait's method and the impl's method.
        let drain_calls: Vec<_> = delta
            .edges
            .iter()
            .filter(|e| {
                e.src.as_str() == "rust:src/relay.rs::tests::drains"
                    && e.kind == EdgeKind::Calls
                    && e.dst.as_str().ends_with("::drain")
            })
            .collect();
        assert_eq!(drain_calls.len(), 2);
        assert!(
            drain_calls
                .iter()
                .all(|e| (e.provenance.confidence.get() - 0.5).abs() < 1e-6)
        );

        let query = delta
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Query)
            .expect("query node");
        let reads = edge(&delta, query.id.as_str(), &EdgeKind::Reads, "sql:cw.outbox")
            .expect("query reads table");
        assert_eq!(reads.provenance.derivation.kind(), DerivationKind::Resolved);
        assert!(
            edge(
                &delta,
                "rust:src/relay.rs::load",
                &EdgeKind::Reads,
                "sql:cw.outbox"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "file:migrations/001_outbox.sql",
                &EdgeKind::Migrates,
                "sql:cw.outbox"
            )
            .is_some()
        );
        assert!(edge(&delta, "sql:cw", &EdgeKind::Contains, "sql:cw.outbox").is_some());
        assert!(
            edge(
                &delta,
                "file:src/lib.rs",
                &EdgeKind::Imports,
                "rust:src/relay.rs::Relay"
            )
            .is_some()
        );
        assert!(delta.unresolved.iter().all(|u| u.name != "flush_all"));
        assert!(
            delta
                .edges
                .iter()
                .all(|e| !e.provenance.evidence.is_empty()),
            "every edge cites evidence"
        );
    }

    /// Builds a resolved fact for the occurrence of `token` on `line` (one-based) of `unit`.
    fn resolved(unit: &SourceUnit, line: u32, token: &str, target: SymbolTarget) -> Fact {
        let text = unit.text().expect("utf8");
        let start: usize = text
            .lines()
            .take(line as usize - 1)
            .map(|l| l.len() + 1)
            .sum();
        let col = text
            .lines()
            .nth(line as usize - 1)
            .expect("line")
            .find(token)
            .expect("token");
        let bytes = ByteSpan::new((start + col) as u64, (start + col + token.len()) as u64);
        Fact::Resolved(ResolvedReference {
            name: token.to_owned(),
            span: unit.text_span(bytes),
            target,
            resolver: ResolverId::new("scip/test"),
            hash: ContentHash::of(token),
        })
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one fixture linked twice, with and without the index"
    )]
    fn precise_bindings_replace_name_matching_and_silence_externals() {
        let units = vec![
            unit(
                "src/a.rs",
                "pub struct Relay;\nimpl Relay {\n    pub fn drain(&self) -> usize { 1 }\n}\npub fn other() {}\n",
            ),
            unit(
                "src/b.rs",
                "pub struct Sink;\nimpl Sink {\n    pub fn drain(&self) -> usize { 2 }\n}\n",
            ),
            unit(
                "src/c.rs",
                "fn run(r: &Relay, v: Vec<u8>) -> usize { r.drain() + sqlx::query(\"x\") + other() }\n",
            ),
        ];
        // Without an index: `r.drain()` matches two declarations at 0.5 each and
        // `sqlx::query` is a gap.
        let (plain, _) = linked(&units);
        let drains = |d: &GraphDelta| {
            d.edges
                .iter()
                .filter(|e| {
                    e.src.as_str() == "rust:src/c.rs::run" && e.dst.as_str().ends_with("::drain")
                })
                .map(|e| {
                    (
                        e.dst.to_string(),
                        e.provenance.derivation.kind(),
                        e.provenance.confidence.get(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(drains(&plain).len(), 2);
        assert!(plain.unresolved.iter().any(|u| u.name == "query"));

        // With an index covering c.rs: one certain resolved edge, no gap for the
        // external call, and the uncovered call to `other` stays syntactic.
        let extractors = Extractors::new().with(TreeSitterExtractor::new());
        let extraction = extract_units(&extractors, &units, SqlDialect::PostgreSql);
        let c = &units[2];
        let scip = FactSet {
            facts: vec![
                resolved(
                    c,
                    1,
                    "drain",
                    SymbolTarget::Declared {
                        file: RepoPath::new("src/a.rs"),
                        line: 3,
                        name: "drain".into(),
                    },
                ),
                resolved(
                    c,
                    1,
                    "query",
                    SymbolTarget::External {
                        symbol: "rust-analyzer cargo sqlx 0.8.0 query/query().".into(),
                    },
                ),
                resolved(
                    c,
                    1,
                    "Vec",
                    SymbolTarget::External {
                        symbol: "rust-analyzer cargo alloc 1.0 vec/Vec#".into(),
                    },
                ),
            ],
            ..FactSet::empty(surveyor_model::ExtractorId::new("scip/test"), c)
        };
        let files = extraction.units.iter().map(|u| FileEntry {
            path: u.path.clone(),
            language: u.language.clone(),
            content_hash: u.content_hash,
            bytes: u.bytes,
            lines: u.lines,
        });
        let input = LinkInput::new(RepoId::new("demo"), CommitId::new("c0"))
            .with_files(files)
            .with_fact_sets(extraction.fact_sets)
            .with_fact_sets([scip]);
        let precise = link(&input);
        let found = drains(&precise);
        assert_eq!(
            found,
            vec![(
                "rust:src/a.rs::Relay::drain".to_owned(),
                DerivationKind::Resolved,
                1.0
            )]
        );
        assert!(precise.unresolved.iter().all(|u| u.name != "query"));
        let other = edge(
            &precise,
            "rust:src/c.rs::run",
            &EdgeKind::Calls,
            "rust:src/a.rs::other",
        )
        .expect("uncovered call stays");
        assert_eq!(
            other.provenance.derivation.kind(),
            DerivationKind::Syntactic
        );
        let drain_edge = precise
            .edges
            .iter()
            .find(|e| e.dst.as_str() == "rust:src/a.rs::Relay::drain" && e.kind == EdgeKind::Calls)
            .expect("resolved edge");
        assert_eq!(drain_edge.provenance.evidence[0].citation(), "src/c.rs:1");
        assert!(matches!(
            &drain_edge.provenance.derivation,
            surveyor_model::Derivation::Resolved { resolver } if resolver.as_str() == "scip/test"
        ));
        assert_eq!(
            drain_edge.resolved_against,
            vec![NodeId::new("rust:src/a.rs::Relay::drain")]
        );
    }

    #[test]
    fn typescript_relative_imports_and_heritage() {
        let units = vec![
            unit(
                "package.json",
                "{\n \"name\": \"web\",\n \"dependencies\": { \"react\": \"18\" }\n}\n",
            ),
            unit(
                "src/base.ts",
                "export class Base {}\nexport interface Handler { handle(): void; }\n",
            ),
            unit(
                "src/server.ts",
                "import { Base, Handler } from \"./base\";\nimport React from \"react\";\nimport fs from \"fs\";\nexport class Server extends Base implements Handler {\n  handle(): void { this.start(); }\n  start(): void {}\n}\n",
            ),
        ];
        let (delta, _) = linked(&units);
        assert!(
            edge(
                &delta,
                "file:src/server.ts",
                &EdgeKind::Imports,
                "file:src/base.ts"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "file:src/server.ts",
                &EdgeKind::Imports,
                "pkg:npm/react"
            )
            .is_some()
        );
        assert!(
            !delta.nodes.iter().any(|n| n.id.as_str() == "pkg:npm/fs"),
            "undeclared modules get no node"
        );
        assert!(
            edge(
                &delta,
                "typescript:src/server.ts::Server",
                &EdgeKind::Extends,
                "typescript:src/base.ts::Base"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "typescript:src/server.ts::Server",
                &EdgeKind::Implements,
                "typescript:src/base.ts::Handler"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "typescript:src/server.ts::Server::handle",
                &EdgeKind::Calls,
                "typescript:src/server.ts::Server::start"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "typescript:src/server.ts::Server",
                &EdgeKind::Defines,
                "typescript:src/server.ts::Server::handle"
            )
            .is_some()
        );
    }

    #[test]
    fn python_modules_resolve_to_files() {
        let units = vec![
            unit("app/__init__.py", ""),
            unit("app/db.py", "def connect():\n    pass\n"),
            unit(
                "app/main.py",
                "from app.db import connect\nfrom . import db\nimport json\n\ndef run():\n    connect()\n",
            ),
        ];
        let (delta, _) = linked(&units);
        assert!(
            edge(
                &delta,
                "file:app/main.py",
                &EdgeKind::Imports,
                "file:app/db.py"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "python:app/main.py::run",
                &EdgeKind::Calls,
                "python:app/db.py::connect"
            )
            .is_some()
        );
        assert!(
            delta
                .unresolved
                .iter()
                .all(|u| u.reason != UnresolvedReason::NoTable)
        );
    }

    #[test]
    fn a_view_reads_its_tables_and_code_that_selects_from_it_reaches_them() {
        let units = [
            unit(
                "migrations/001.sql",
                "CREATE SCHEMA cw;\nCREATE TABLE cw.outbox (id uuid, sent_at timestamptz);\n",
            ),
            unit(
                "migrations/002.sql",
                "CREATE OR REPLACE VIEW cw.pending_outbox AS\n  SELECT id FROM cw.outbox WHERE sent_at IS NULL;\n",
            ),
            unit(
                "src/report.rs",
                r#"fn backlog(schema: &str) {
    let _ = sqlx::query("SELECT count(*) FROM cw.pending_outbox");
    let _ = sqlx::query(&format!("SELECT count(*) FROM {schema}.pending_outbox"));
}
"#,
            ),
        ];
        let (delta, _) = linked(&units);
        let views: Vec<&Node> = delta
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::View)
            .collect();
        let [view] = views.as_slice() else {
            panic!("one view node, found {views:?}");
        };
        assert_eq!(view.id.as_str(), "sql:cw.pending_outbox");
        assert_eq!(view.qualified_name, "cw.pending_outbox");
        let reads = edge(
            &delta,
            "sql:cw.pending_outbox",
            &EdgeKind::Reads,
            "sql:cw.outbox",
        )
        .expect("the view reads the table its query names");
        assert_eq!(reads.provenance.derivation.kind(), DerivationKind::Resolved);
        assert!((reads.provenance.confidence.get() - 1.0).abs() < f32::EPSILON);
        assert!(
            edge(
                &delta,
                "rust:src/report.rs::backlog",
                &EdgeKind::Reads,
                "sql:cw.pending_outbox"
            )
            .is_some(),
            "code that selects from the view links to it, qualified or behind a placeholder"
        );
        assert!(
            edge(
                &delta,
                "file:migrations/002.sql",
                &EdgeKind::Migrates,
                "sql:cw.pending_outbox"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "sql:cw",
                &EdgeKind::Contains,
                "sql:cw.pending_outbox"
            )
            .is_some()
        );
        assert!(
            delta
                .unresolved
                .iter()
                .all(|u| u.reason != UnresolvedReason::NoTable),
            "{:?}",
            delta.unresolved
        );
    }

    #[test]
    fn a_table_behind_a_schema_placeholder_links_to_every_schema_that_declares_it() {
        let units = [
            unit(
                "migrations/001.sql",
                "CREATE SCHEMA a;\nCREATE TABLE a.events (id uuid);\nCREATE TABLE a.only_here (id uuid);\n",
            ),
            unit(
                "migrations/002.sql",
                "CREATE SCHEMA b;\nCREATE TABLE b.events (id uuid);\n",
            ),
            unit(
                "src/repo.rs",
                r#"fn record(schema: &str) {
    let _ = sqlx::query(&format!("INSERT INTO {schema}.events (id) VALUES ($1)"));
}
fn count(schema: &str) {
    let _ = sqlx::query(&format!("SELECT count(*) FROM {schema}.only_here"));
}
fn anything(schema: &str, table: &str) {
    let _ = sqlx::query(&format!("SELECT 1 FROM {schema}.{table}"));
}
"#,
            ),
        ];
        let (delta, _) = linked(&units);
        let writes: Vec<_> = delta
            .edges
            .iter()
            .filter(|e| e.src.as_str() == "rust:src/repo.rs::record" && e.kind == EdgeKind::Writes)
            .collect();
        let mut written: Vec<&str> = writes.iter().map(|e| e.dst.as_str()).collect();
        written.sort_unstable();
        assert_eq!(written, vec!["sql:a.events", "sql:b.events"]);
        assert!(
            writes
                .iter()
                .all(|e| (e.provenance.confidence.get() - 0.5).abs() < 1e-6),
            "two schemas declare `events`, so each edge carries half the confidence"
        );
        let only = edge(
            &delta,
            "rust:src/repo.rs::count",
            &EdgeKind::Reads,
            "sql:a.only_here",
        )
        .expect("a bare name declared once resolves");
        assert!((only.provenance.confidence.get() - 1.0).abs() < f32::EPSILON);
        assert!(
            delta
                .unresolved
                .iter()
                .all(|u| u.reason != UnresolvedReason::NoTable),
            "a query whose table is itself a placeholder names nothing to miss: {:?}",
            delta.unresolved
        );
    }

    #[test]
    fn names_imported_from_outside_the_repository_are_silent() {
        let units = [
            unit(
                "svc/pyproject.toml",
                "[project]\nname = \"svc\"\ndependencies = [\"connectrpc>=0.10\"]\n",
            ),
            unit("svc/app/__init__.py", ""),
            unit(
                "svc/app/db.py",
                "def connect():\n    pass\n\nclass MethodInfo:\n    pass\n",
            ),
            unit(
                "svc/app/gen/stub.py",
                "from connectrpc.client import MethodInfo, ConnectClient\nfrom google.protobuf import descriptor\nfrom google.protobuf import message as _message\nfrom app.db import connect\n\ndef make():\n    MethodInfo(1)\n    descriptor.Foo()\n    connect()\n\nclass RoadmapClient(ConnectClient):\n    def fetch(self):\n        return self.execute_unary(1)\n\nclass Envelope(_message.Message):\n    pass\n\nclass Plain:\n    def run(self):\n        self.missing()\n",
            ),
            unit(
                "web/package.json",
                "{\n  \"name\": \"web\",\n  \"dependencies\": {\n    \"left-pad\": \"^1\"\n  }\n}\n",
            ),
            unit(
                "web/src/a.ts",
                "import { pad } from \"left-pad\";\nimport { Thing } from \"./thing\";\nexport function go() { pad(1); new Thing(); }\n",
            ),
            unit("web/src/thing.ts", "export class Thing {}\n"),
        ];
        let (delta, _) = linked(&units);
        let gaps: Vec<&str> = delta.unresolved.iter().map(|u| u.name.as_str()).collect();
        for external in [
            "MethodInfo",
            "descriptor",
            "Foo",
            "pad",
            "execute_unary",
            "Message",
        ] {
            assert!(
                !gaps.contains(&external),
                "{external} comes from a dependency: {gaps:?}"
            );
        }
        assert_eq!(
            gaps,
            vec!["missing"],
            "a self call in a class with no external base is still a gap"
        );
        assert!(
            edge(
                &delta,
                "python:svc/app/gen/stub.py::make",
                &EdgeKind::Calls,
                "python:svc/app/db.py::MethodInfo"
            )
            .is_none(),
            "the import of `MethodInfo` from connectrpc shadows the local class of that name"
        );
        assert!(
            edge(
                &delta,
                "python:svc/app/gen/stub.py::make",
                &EdgeKind::Calls,
                "python:svc/app/db.py::connect"
            )
            .is_some(),
            "a name imported from a repository module still links"
        );
        let symbol_import = edge(
            &delta,
            "file:svc/app/gen/stub.py",
            &EdgeKind::Imports,
            "python:svc/app/db.py::connect",
        )
        .expect("a from-import names the symbol as well as the module");
        assert!(
            (symbol_import.provenance.confidence.get() - 1.0).abs() < f32::EPSILON,
            "the module and its symbol are one binding, not alternatives"
        );
        assert!(
            edge(
                &delta,
                "file:svc/app/gen/stub.py",
                &EdgeKind::Imports,
                "file:svc/app/db.py"
            )
            .is_some()
        );
        assert!(
            edge(
                &delta,
                "typescript:web/src/a.ts::go",
                &EdgeKind::UsesType,
                "typescript:web/src/thing.ts::Thing"
            )
            .is_some()
        );
    }

    #[test]
    fn a_receiver_that_names_an_import_or_the_class_narrows_the_candidates() {
        let units = [
            unit("app/__init__.py", ""),
            unit(
                "app/db.py",
                "def connect():\n    pass\n\nclass Repo:\n    def connect(self):\n        pass\n\n    @staticmethod\n    def create():\n        pass\n",
            ),
            unit(
                "app/util.py",
                "def connect():\n    pass\n\ndef create():\n    pass\n",
            ),
            unit(
                "app/main.py",
                "from app import db\nfrom app.db import Repo\n\nclass Svc:\n    def run(self):\n        db.connect()\n        Repo.create()\n        self.helper()\n\n    def helper(self):\n        pass\n\ndef helper():\n    pass\n",
            ),
        ];
        let (delta, _) = linked(&units);
        let calls: Vec<(&str, f32)> = delta
            .edges
            .iter()
            .filter(|e| {
                e.src.as_str() == "python:app/main.py::Svc::run" && e.kind == EdgeKind::Calls
            })
            .map(|e| (e.dst.as_str(), e.provenance.confidence.get()))
            .collect();
        let dst = |name: &str| {
            calls
                .iter()
                .filter(|(d, _)| d.ends_with(name))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            dst("::connect"),
            vec![&("python:app/db.py::connect", 1.0)],
            "`db.connect()` is the module's top-level function, not `Repo.connect` or `util.connect`: {calls:?}"
        );
        assert_eq!(
            dst("::create"),
            vec![&("python:app/db.py::Repo::create", 1.0)],
            "`Repo.create()` is the class's member, not `util.create`: {calls:?}"
        );
        assert_eq!(
            dst("::helper"),
            vec![&("python:app/main.py::Svc::helper", 1.0)],
            "`self.helper()` is the enclosing class's method, not the module function: {calls:?}"
        );
    }

    #[test]
    fn rhai_calls_resolve_to_the_functions_a_rust_host_registers() {
        let units = [
            unit(
                "host/src/lib.rs",
                "fn sh(c: &str) {}\nfn install(m: &mut rhai::Module) {\n    m.set_native_fn(\"sh\", |c: &str| Ok(sh(c)));\n    m.set_native_fn(\"sh\", |c: &str, _o: i64| Ok(sh(c)));\n    m.set_native_fn(\"repo_root\", || Ok(String::new()));\n}\n",
            ),
            unit(
                "host/src/modules/proc.rs",
                "pub fn module() -> rhai::Module {\n    let mut m = rhai::Module::new();\n    m.set_native_fn(\"shell\", |c: &str| Ok(()));\n    m\n}\n",
            ),
            unit(
                "tasks/build.rhai",
                "fn build() {\n    let root = repo_root();\n    sh(\"cargo build\");\n    proc::shell(\"cargo test\");\n    type_of(root);\n    nowhere();\n}\n",
            ),
        ];
        let (delta, _) = linked(&units);
        let from_build: Vec<(&str, f32)> = delta
            .edges
            .iter()
            .filter(|e| {
                e.src.as_str() == "rhai:tasks/build.rhai::build" && e.kind == EdgeKind::Calls
            })
            .map(|e| (e.dst.as_str(), e.provenance.confidence.get()))
            .collect();
        assert_eq!(
            from_build,
            vec![
                ("rhai:host/src/lib.rs::repo_root", 1.0),
                ("rhai:host/src/lib.rs::sh", 1.0),
                ("rhai:host/src/modules/proc.rs::shell", 1.0),
            ],
            "the two `sh` registrations are one declaration, and the Rust `sh` is not a target"
        );
        let gaps: Vec<&str> = delta
            .unresolved
            .iter()
            .filter(|u| u.site.file.as_str() == "tasks/build.rhai")
            .map(|u| u.name.as_str())
            .collect();
        assert_eq!(gaps, vec!["nowhere"], "`type_of` is the standard library's");
        let host = delta
            .nodes
            .iter()
            .find(|n| n.id.as_str() == "rhai:host/src/lib.rs::sh")
            .expect("host function node");
        assert_eq!(host.language, Some(Language::Rhai));
        assert_eq!(host.kind, NodeKind::Function);
    }
}
