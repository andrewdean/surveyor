//! The linking passes.
//!
//! Each pass reads the facts and the nodes built so far, decides through the pure
//! helpers in `resolve`, and appends nodes or edges. The builder is the one place
//! that mutates; everything it consults returns values.

use std::collections::{HashMap, HashSet};

use surveyor_model::{
    ByteSpan, Confidence, ContentHash, DeclarationFact, DeclaredRelation, DeclaredSource, Edge,
    EdgeId, EdgeKind, Evidence, FactSet, GrammarId, GraphDelta, Language, LiteralRole, Node,
    NodeId, NodeKind, Provenance, ReferenceFact, ReferenceRole, RepoPath, ResolverId, RuleId,
    SourceLocation, TextSpan, Unresolved, UnresolvedReason,
};

use crate::ids::{doc_id, file_id, package_id, repo_id, sql_id, symbol_id};
use crate::input::LinkInput;
use crate::resolve::{Resolution, Site, SymbolIndex, Tier, decide, is_method, rank};

/// Resolver recorded on edges derived from `sqlparser` facts.
const SQL_RESOLVER: &str = "sqlparser";

/// Grammar recorded when a fact set names none (native enrichers).
const NO_GRAMMAR: &str = "none";

/// Crate roots Rust imports never resolve to a package node.
const RUST_BUILTIN_ROOTS: &[&str] = &["std", "core", "alloc", "crate", "self", "super"];

/// Links one repository's facts into a graph delta.
#[must_use]
pub fn link(input: &LinkInput) -> GraphDelta {
    let mut b = Builder::new(input);
    b.root();
    b.packages();
    b.files();
    b.declarations();
    b.structure();
    b.declared_relations();
    b.imports();
    b.references();
    b.literals();
    b.tests();
    b.finish()
}

/// A declaration node inside one file, with its parent by span containment.
#[derive(Debug, Clone)]
struct DeclRef {
    set: usize,
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
}

impl<'a> Builder<'a> {
    fn new(input: &'a LinkInput) -> Self {
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
        let edge = Edge::new(self.node_id(src), kind, self.node_id(dst), provenance)
            .resolved_against(against);
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
        let node = Node::synthetic(
            repo_id(repo),
            NodeKind::Repository,
            repo.as_str(),
            repo.clone(),
        )
        .with_attribute("commit", self.input.commit.as_str());
        self.root = self.push_node(node);
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
                self.package_by_name.insert(normalize_pkg(&d.name), idx);
                if let Some((_, short)) = coordinate.rsplit_once('/') {
                    self.package_by_name
                        .entry(normalize_pkg(short))
                        .or_insert(idx);
                }
            }
        }
        self.package_dirs
            .sort_by_key(|(dir, _)| std::cmp::Reverse(dir.len()));
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
                    set: si,
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
        let mergeable = matches!(
            d.kind,
            NodeKind::Schema
                | NodeKind::Table
                | NodeKind::Column
                | NodeKind::Index
                | NodeKind::Constraint
        );
        if mergeable && let Some(&existing) = self.node_pos.get(&id) {
            let node = &mut self.nodes[existing];
            for (k, v) in &d.attributes {
                node.attributes
                    .entry(k.clone())
                    .or_insert_with(|| v.clone());
            }
            return existing;
        }
        let node = Node {
            id: self.unique_id(&id),
            kind: d.kind.clone(),
            name: d.name.clone(),
            qualified_name: qualified,
            repo: self.input.repo.clone(),
            language: set.language.clone(),
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
        let files: Vec<RepoPath> = self.decls_by_file.keys().cloned().collect();
        for file in files {
            let refs = self.decls_by_file[&file].clone();
            let file_node = self.file_nodes.get(&file).copied();
            for r in &refs {
                let set = &self.input.fact_sets[r.set];
                let node = self.nodes[r.node].clone();
                let (span, hash) = (r.span, node.content_hash);
                match node.kind {
                    NodeKind::Schema => {
                        if let Some(f) = file_node {
                            let prov = self.resolved_sql(&file, span, hash);
                            self.push_edge_once(f, EdgeKind::Migrates, r.node, prov, Vec::new());
                        }
                    }
                    NodeKind::Table => {
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
            self.push_edge(src, r.kind.clone(), dst, prov, Vec::new());
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
        self.package_by_name
            .entry(normalize_pkg(&name))
            .or_insert(idx);
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

    fn references(&mut self) {
        for (si, src, r) in self.sites(|role| *role != ReferenceRole::Import) {
            match r.role {
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
                _ => {}
            }
        }
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
        let candidates = if typed.is_empty() && r.role.is_call() {
            self.candidates(&r.name, src, NodeKind::is_type)
                .into_iter()
                .filter(|(_, n)| same_language(n))
                .collect()
        } else {
            typed
        };
        let ranked = rank(&self.index, &file, candidates, qualifier.as_deref());
        let site = Site {
            method_call,
            builtin: is_builtin_name(language.as_ref(), &r.name),
            report,
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
        let Some(dst) = self.index.table(schema, table) else {
            self.report(r, &file, UnresolvedReason::NoTable);
            return;
        };
        let kind = match (&set.language, &r.role) {
            (Some(Language::Sql), ReferenceRole::Write) => EdgeKind::Migrates,
            _ => r.role.edge_kind(),
        };
        let prov = self.resolved_sql(&file, r.span, r.hash);
        let against = vec![self.node_id(dst)];
        self.push_edge_once(src, kind.clone(), dst, prov.clone(), against.clone());
        // The symbol that holds a query literal touches the table too.
        if self.nodes[src].kind == NodeKind::Query
            && let Some(parent) = self.parent_decl(&file, src)
        {
            self.push_edge_once(parent, kind, dst, prov, against);
        }
    }

    fn link_import(&mut self, si: usize, src: usize, r: &ReferenceFact) {
        let set = &self.input.fact_sets[si];
        let file = set.path.clone();
        let language = set.language.clone();
        let relative = r.path.first().is_some_and(|s| s == "." || s == "..");

        let targets: Vec<usize> = match language {
            Some(Language::TypeScript | Language::Tsx | Language::JavaScript) => {
                let found = if relative {
                    self.resolve_relative_file(&file, &r.path, language.as_ref())
                } else {
                    self.npm_package(&r.path)
                };
                found.into_iter().collect()
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
            Some(Language::Python) => self
                .resolve_python_module(&file, &r.path)
                .or_else(|| {
                    r.path
                        .first()
                        .and_then(|p| self.package_by_name.get(&normalize_pkg(p)).copied())
                })
                .into_iter()
                .collect(),
            Some(Language::Rust) => {
                let root = r.path.first().map_or("", String::as_str);
                if matches!(root, "std" | "core" | "alloc") {
                    return;
                }
                let same_package = RUST_BUILTIN_ROOTS.contains(&root);
                let package_idx = if same_package {
                    None
                } else {
                    self.package_by_name.get(&normalize_pkg(root)).copied()
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
                    let ranked: Vec<usize> =
                        rank(&self.index, &file, candidates, qualifier.as_deref())
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
            _ => Vec::new(),
        };
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
        let confidence = Confidence::new(1.0 / targets.len() as f32);
        for dst in targets {
            let set = &self.input.fact_sets[si];
            let prov = self
                .syntactic(set, &r.rule, r.span, r.hash)
                .with_confidence(confidence);
            let against = vec![self.node_id(dst)];
            self.push_edge_once(src, EdgeKind::Imports, dst, prov, against);
        }
    }

    fn npm_package(&self, path: &[String]) -> Option<usize> {
        let first = path.first()?;
        let name = if first.starts_with('@') {
            format!("{first}/{}", path.get(1)?)
        } else {
            first.clone()
        };
        self.package_by_name.get(&normalize_pkg(&name)).copied()
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

    // ---- pass 7: tests ---------------------------------------------------------------

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
        GraphDelta {
            nodes: self.nodes,
            edges: self.edges,
            unresolved: self.unresolved,
        }
    }
}

/// Stable identifier and qualified name for a declaration.
fn declaration_identity(set: &FactSet, d: &DeclarationFact) -> (NodeId, String) {
    let schema = || d.scope.first().map_or("public", String::as_str);
    let table = || d.scope.get(1).map_or("", String::as_str);
    match &d.kind {
        NodeKind::Schema => (sql_id(&[&d.name], None), d.name.clone()),
        NodeKind::Table => (
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
                symbol_id(set.language.as_ref(), &set.path, &q),
                d.qualified_name(),
            )
        }
        NodeKind::Impl => {
            let q = match d.attributes.get("trait") {
                Some(t) => format!("{}#impl({t})", d.qualified_name()),
                None => format!("{}#impl", d.qualified_name()),
            };
            (symbol_id(set.language.as_ref(), &set.path, &q), q)
        }
        _ => {
            let q = d.qualified_name();
            (symbol_id(set.language.as_ref(), &set.path, &q), q)
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

fn normalize_pkg(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
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
fn is_builtin_name(language: Option<&Language>, name: &str) -> bool {
    match language {
        Some(Language::Python) => BUILTIN_PYTHON.contains(&name) || BUILTIN_COMMON.contains(&name),
        Some(Language::TypeScript | Language::Tsx | Language::JavaScript) => {
            BUILTIN_JS.contains(&name)
        }
        Some(Language::Bash) => BUILTIN_BASH.contains(&name),
        _ => BUILTIN_COMMON.contains(&name),
    }
}

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
    use std::collections::BTreeMap;

    use surveyor_extract::{
        Extractors, ManifestExtractor, SqlDialect, SqlExtractor, TreeSitterExtractor, extract_units,
    };
    use surveyor_model::{CommitId, DerivationKind, LineSpan, RepoId, SourceUnit};

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
            .with(SqlExtractor::new(SqlDialect::PostgreSql, "public"))
            .with(ManifestExtractor::new());
        let extraction = extract_units(&extractors, units, SqlDialect::PostgreSql);
        assert!(extraction.errors.is_empty(), "{:?}", extraction.errors);
        let files = extraction.units.iter().map(|u| FileEntry {
            path: u.path.clone(),
            language: u.language.clone(),
            content_hash: u.content_hash,
            bytes: u.bytes,
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
}
