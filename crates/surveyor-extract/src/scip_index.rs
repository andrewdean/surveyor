//! The precise tier: references an external SCIP indexer already resolved.
//!
//! A SCIP index lists, per document, every symbol occurrence with its range and
//! roles. [`ScipExtractor`] turns the non-definition occurrences of a file into
//! [`Fact::Resolved`] facts: each names the symbol at the site and where the index
//! says it is declared, as a file and the line of its name token when the declaration
//! is in this repository, as an external symbol when it is not, or as a local binding.
//! The linker pairs each fact with the Tree-sitter reference at the same site, so the
//! edge kind comes from syntax and the target from the indexer, and the syntactic
//! graph stands wherever the index is silent.

use std::collections::HashMap;
use std::path::Path;

use protobuf::{Enum, Message};
use scip::types::{Index, PositionEncoding, SymbolRole};
use surveyor_model::{
    ByteSpan, ContentHash, ExtractorId, Fact, FactSet, RepoPath, ResolvedReference, ResolverId,
    SourceUnit, SymbolTarget,
};

use crate::{ExtractError, Extractor};

/// Prefix of the extractor and resolver identifiers; the indexer's name follows.
const PREFIX: &str = "scip";

/// How a document counts characters within a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

impl Encoding {
    /// The encoding a document declares, or the one its indexer is known to use when
    /// it declares none.
    fn of(document: PositionEncoding, tool: &str) -> Self {
        match document {
            PositionEncoding::UTF8CodeUnitOffsetFromLineStart => Self::Utf8,
            PositionEncoding::UTF16CodeUnitOffsetFromLineStart => Self::Utf16,
            PositionEncoding::UTF32CodeUnitOffsetFromLineStart => Self::Utf32,
            PositionEncoding::UnspecifiedPositionEncoding => Self::assumed_for(tool),
        }
    }

    /// What an indexer that leaves the encoding unspecified counts in.
    ///
    /// scip-typescript takes its columns from the TypeScript compiler, whose positions
    /// index JavaScript strings, so they are UTF-16 code units. rust-analyzer and the
    /// other native indexers count UTF-8 units, and the protocol's own advice for them
    /// says so; that stays the default for every other tool.
    fn assumed_for(tool: &str) -> Self {
        match tool {
            "scip-typescript" => Self::Utf16,
            _ => Self::Utf8,
        }
    }

    /// The byte offset of `units` characters into `line`.
    fn byte_offset(self, line: &str, units: usize) -> usize {
        match self {
            Self::Utf8 => units.min(line.len()),
            Self::Utf16 => {
                let mut seen = 0;
                for (i, c) in line.char_indices() {
                    if seen >= units {
                        return i;
                    }
                    seen += c.len_utf16();
                }
                line.len()
            }
            Self::Utf32 => line
                .char_indices()
                .nth(units)
                .map_or(line.len(), |(i, _)| i),
        }
    }
}

/// One occurrence as the extractor needs it: zero-based positions and the symbol.
#[derive(Debug, Clone)]
struct Occurrence {
    symbol: String,
    start_line: usize,
    start_char: usize,
    end_line: usize,
    end_char: usize,
    definition: bool,
}

impl Occurrence {
    /// Reads a protocol range, which is `[line, start, end]` on one line or
    /// `[start_line, start, end_line, end]` across lines.
    fn parse(symbol: &str, range: &[i32], roles: i32) -> Option<Self> {
        let at = |i: usize| usize::try_from(*range.get(i)?).ok();
        let (start_line, start_char, end_line, end_char) = match range.len() {
            3 => (at(0)?, at(1)?, at(0)?, at(2)?),
            4 => (at(0)?, at(1)?, at(2)?, at(3)?),
            _ => return None,
        };
        Some(Self {
            symbol: symbol.to_owned(),
            start_line,
            start_char,
            end_line,
            end_char,
            definition: roles & SymbolRole::Definition.value() != 0,
        })
    }
}

#[derive(Debug)]
struct Document {
    /// The indexer that wrote this document, named on every fact drawn from it.
    resolver: ResolverId,
    encoding: Encoding,
    occurrences: Vec<Occurrence>,
}

/// Where a symbol is declared inside the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Definition {
    file: RepoPath,
    /// One-based line of the name token.
    line: u32,
}

/// Emits resolved references from one or more SCIP indexes.
///
/// Several indexes pool their declarations: a monorepo indexed one workspace at a
/// time still resolves a call from one workspace to a declaration in another,
/// because both indexes name that declaration by the same symbol.
#[derive(Default)]
pub struct ScipExtractor {
    documents: HashMap<RepoPath, Document>,
    definitions: HashMap<String, Definition>,
}

impl std::fmt::Debug for ScipExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScipExtractor")
            .field("documents", &self.documents.len())
            .field("definitions", &self.definitions.len())
            .finish()
    }
}

impl ScipExtractor {
    /// Reads every index in `paths` for the repository rooted at `repo_root`, pooling
    /// their declarations so references resolve across the indexes.
    ///
    /// # Errors
    /// Returns the first index that cannot be read or is not a SCIP index.
    pub fn load_all(
        paths: &[impl AsRef<Path>],
        repo_root: Option<&Path>,
    ) -> Result<Self, ExtractError> {
        paths.iter().try_fold(Self::default(), |pool, path| {
            Self::load_under(path, repo_root).map(|index| pool.merge(index))
        })
    }

    /// Pools another index's documents and declarations into this one; the first
    /// declaration of a symbol wins.
    #[must_use]
    fn merge(mut self, other: Self) -> Self {
        self.documents.extend(other.documents);
        for (symbol, definition) in other.definitions {
            self.definitions.entry(symbol).or_insert(definition);
        }
        self
    }

    /// Reads the index at `path`.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be read and a parse error when it is
    /// not a SCIP index.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ExtractError> {
        Self::load_under(path, None)
    }

    /// Reads the index at `path` for the repository rooted at `repo_root`.
    ///
    /// An index lists its documents relative to the project it was built from. When
    /// that project is a workspace inside the repository (a monorepo with several
    /// Cargo workspaces, indexed one at a time), the documents are re-rooted so they
    /// name the same files the walker does: `src/main.rs` in an index of `apps/`
    /// becomes `apps/src/main.rs`.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be read and a parse error when it is
    /// not a SCIP index.
    pub fn load_under(
        path: impl AsRef<Path>,
        repo_root: Option<&Path>,
    ) -> Result<Self, ExtractError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| ExtractError::io(e).at(path))?;
        let index = Index::parse_from_bytes(&bytes)
            .map_err(|e| ExtractError::parse(format!("not a SCIP index: {e}")).at(path))?;
        let extractor = Self::from_index_under(&index, repo_root);
        tracing::info!(
            index = %path.display(),
            tool = %index.metadata.tool_info.name,
            root = %index.metadata.project_root,
            documents = extractor.documents.len(),
            definitions = extractor.definitions.len(),
            "loaded SCIP index"
        );
        Ok(extractor)
    }

    /// Builds the extractor from a parsed index whose project is the repository root.
    #[must_use]
    pub fn from_index(index: &Index) -> Self {
        Self::from_index_under(index, None)
    }

    /// Builds the extractor from a parsed index, re-rooting its documents under the
    /// directory the index's project occupies in the repository at `repo_root`.
    #[must_use]
    pub fn from_index_under(index: &Index, repo_root: Option<&Path>) -> Self {
        let tool = index.metadata.tool_info.name.as_str();
        let resolver = ResolverId::new(format!(
            "{PREFIX}/{}",
            if tool.is_empty() { "unknown" } else { tool }
        ));
        let sample = index.documents.first().map(|d| d.relative_path.as_str());
        let prefix =
            repo_root.and_then(|root| project_prefix(&index.metadata.project_root, root, sample));
        let mut definitions: HashMap<String, Definition> = HashMap::new();
        let documents: HashMap<RepoPath, Document> = index
            .documents
            .iter()
            .map(|doc| {
                let path = match &prefix {
                    Some(prefix) => RepoPath::new(format!("{prefix}/{}", doc.relative_path)),
                    None => RepoPath::new(&doc.relative_path),
                };
                let occurrences: Vec<Occurrence> = doc
                    .occurrences
                    .iter()
                    .filter(|o| !scip::symbol::is_local_symbol(&o.symbol) || o.symbol_roles == 0)
                    .filter_map(|o| Occurrence::parse(&o.symbol, &o.range, o.symbol_roles))
                    .collect();
                for o in occurrences.iter().filter(|o| o.definition) {
                    definitions
                        .entry(o.symbol.clone())
                        .or_insert_with(|| Definition {
                            file: path.clone(),
                            line: o.start_line as u32 + 1,
                        });
                }
                let encoding = Encoding::of(
                    doc.position_encoding
                        .enum_value_or(PositionEncoding::UnspecifiedPositionEncoding),
                    tool,
                );
                (
                    path,
                    Document {
                        resolver: resolver.clone(),
                        encoding,
                        occurrences,
                    },
                )
            })
            .collect();
        Self {
            documents,
            definitions,
        }
    }

    /// The resolver recorded on the facts, such as `scip/rust-analyzer`.
    pub fn resolvers(&self) -> impl Iterator<Item = &ResolverId> {
        self.documents
            .values()
            .map(|d| &d.resolver)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
    }

    /// Number of documents the index covers.
    #[must_use]
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    fn target(&self, symbol: &str) -> SymbolTarget {
        if scip::symbol::is_local_symbol(symbol) {
            return SymbolTarget::Local;
        }
        match self.definitions.get(symbol) {
            Some(def) => SymbolTarget::Declared {
                file: def.file.clone(),
                line: def.line,
                name: symbol_name(symbol),
            },
            None => SymbolTarget::External {
                symbol: symbol.to_owned(),
            },
        }
    }
}

/// The short name of a symbol: its last descriptor, or the text after the last
/// separator when the symbol does not parse.
fn symbol_name(symbol: &str) -> String {
    scip::symbol::parse_symbol(symbol)
        .ok()
        .and_then(|s| s.descriptors.last().map(|d| d.name.clone()))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            symbol
                .trim_end_matches(['.', '#', '/', '!', ')', '('])
                .rsplit(['/', '#', '.', ' '])
                .next()
                .unwrap_or(symbol)
                .to_owned()
        })
}

/// Byte offsets at which each line starts.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// The repository-relative directory an index's project root names, if it is a
/// proper subdirectory of the repository.
///
/// `project_root` is the `file://` URI the indexer recorded. The repository root is
/// compared both as written and canonicalized, so a symlinked checkout still matches.
/// `None` when the project is the repository itself or lies outside it; the latter is
/// logged, since every document of such an index will go unmatched.
fn project_prefix(project_root: &str, repo_root: &Path, sample: Option<&str>) -> Option<String> {
    let project =
        Path::new(percent_decode(project_root.strip_prefix("file://")?).as_ref()).to_path_buf();
    let roots = std::iter::once(repo_root.to_path_buf())
        .chain(std::fs::canonicalize(repo_root).ok())
        .collect::<Vec<_>>();
    let relative = roots
        .iter()
        .find_map(|root| project.strip_prefix(root).ok())
        .map(|rel| rel.to_string_lossy().trim_matches('/').to_owned())
        .or_else(|| {
            // Built from another checkout (a CI job, a clone at another path): the
            // documents still fit where one of them exists under this repository.
            let found = suffix_under(&project, repo_root, sample?)?;
            tracing::info!(
                project = %project.display(),
                repo = %repo_root.display(),
                under = %found,
                "SCIP index was built at another path; re-rooted where its documents exist"
            );
            Some(found)
        });
    match relative {
        Some(rel) if rel.is_empty() => None,
        Some(rel) => Some(rel),
        None => {
            tracing::warn!(
                project = %project.display(),
                repo = %repo_root.display(),
                "SCIP index project is outside the repository and none of its documents exist under it; the index will not match"
            );
            None
        }
    }
}

/// The shortest suffix of `project`'s path, the empty suffix being the repository
/// root itself, at which `sample` exists under `repo_root`.
fn suffix_under(project: &Path, repo_root: &Path, sample: &str) -> Option<String> {
    let parts: Vec<&str> = project
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    (0..=parts.len())
        .map(|k| parts[parts.len() - k..].join("/"))
        .find(|prefix| repo_root.join(prefix).join(sample).is_file())
}

/// Decodes `%XX` escapes in a URI path; anything else passes through unchanged.
fn percent_decode(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('%') {
        return std::borrow::Cow::Borrowed(text);
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes.get(i..i + 3) {
            Some([b'%', hi, lo]) if hi.is_ascii_hexdigit() && lo.is_ascii_hexdigit() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("00");
                out.push(u8::from_str_radix(hex, 16).unwrap_or(b'%'));
                i += 3;
            }
            _ => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    std::borrow::Cow::Owned(String::from_utf8_lossy(&out).into_owned())
}

impl Extractor for ScipExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(PREFIX)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        self.documents.contains_key(&unit.path)
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let document = self
            .documents
            .get(&unit.path)
            .ok_or_else(|| ExtractError::parse("file is not in the SCIP index"))?;
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("file is not UTF-8"))?;
        let starts = line_starts(text);
        let line_text = |line: usize| -> Option<&str> {
            let start = *starts.get(line)?;
            let end = starts.get(line + 1).map_or(text.len(), |e| e - 1);
            text.get(start..end.max(start))
        };
        let facts = document
            .occurrences
            .iter()
            .filter(|o| !o.definition)
            .filter_map(|o| {
                let first = line_text(o.start_line)?;
                let last = line_text(o.end_line)?;
                let start =
                    starts[o.start_line] + document.encoding.byte_offset(first, o.start_char);
                let end = starts[o.end_line] + document.encoding.byte_offset(last, o.end_char);
                (start < end).then(|| {
                    let bytes = ByteSpan::new(start as u64, end as u64);
                    let target = self.target(&o.symbol);
                    let name = match &target {
                        SymbolTarget::Declared { name, .. } => name.clone(),
                        _ => symbol_name(&o.symbol),
                    };
                    Fact::Resolved(ResolvedReference {
                        name,
                        span: unit.text_span(bytes),
                        target,
                        resolver: document.resolver.clone(),
                        hash: ContentHash::of(unit.slice(bytes)),
                    })
                })
            })
            .collect();
        Ok(FactSet {
            facts,
            ..FactSet::empty(self.id(), unit)
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use scip::types::{Document as ScipDocument, Metadata, Occurrence as ScipOccurrence, ToolInfo};
    use surveyor_model::{CommitId, RepoId};

    use super::*;

    fn occurrence(symbol: &str, range: &[i32], roles: i32) -> ScipOccurrence {
        ScipOccurrence {
            range: range.to_vec(),
            symbol: symbol.to_owned(),
            symbol_roles: roles,
            ..Default::default()
        }
    }

    /// Two files: `store.rs` declares `pending`; `relay.rs` calls it, calls a local
    /// closure, and names a type from the standard library.
    pub(crate) fn index() -> Index {
        let pending = "rust-analyzer cargo outbox 0.1.0 store/impl#[Store]pending().";
        let string = "rust-analyzer cargo alloc https://github.com/rust-lang/rust/library/alloc string/String#";
        Index {
            metadata: Some(Metadata {
                tool_info: Some(ToolInfo {
                    name: "rust-analyzer".into(),
                    ..Default::default()
                })
                .into(),
                project_root: "file:///repo".into(),
                ..Default::default()
            })
            .into(),
            documents: vec![
                ScipDocument {
                    relative_path: "src/store.rs".into(),
                    occurrences: vec![occurrence(pending, &[2, 11, 18], 1)],
                    ..Default::default()
                },
                ScipDocument {
                    relative_path: "src/relay.rs".into(),
                    occurrences: vec![
                        occurrence(pending, &[1, 21, 28], 0),
                        occurrence("local 3", &[1, 8, 9], 1),
                        occurrence("local 3", &[2, 4, 5], 0),
                        occurrence(string, &[0, 23, 29], 0),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn an_index_of_a_workspace_is_rerooted_under_the_repository() {
        let mut index = index();
        index.metadata.as_mut().unwrap().project_root = "file:///repo/apps/my%20svc".into();
        let under = ScipExtractor::from_index_under(&index, Some(Path::new("/repo")));
        assert!(
            under
                .documents
                .contains_key(&RepoPath::new("apps/my svc/src/relay.rs"))
        );
        assert!(
            under
                .documents
                .contains_key(&RepoPath::new("apps/my svc/src/store.rs"))
        );
        assert_eq!(
            under.definitions.values().next().map(|d| d.file.as_str()),
            Some("apps/my svc/src/store.rs"),
            "declarations are re-rooted too"
        );
        let same = ScipExtractor::from_index_under(&index, Some(Path::new("/repo/apps/my svc")));
        assert!(same.documents.contains_key(&RepoPath::new("src/relay.rs")));
        let outside = ScipExtractor::from_index_under(&index, Some(Path::new("/elsewhere")));
        assert!(
            outside
                .documents
                .contains_key(&RepoPath::new("src/relay.rs"))
        );
        assert_eq!(
            project_prefix("file:///repo", Path::new("/repo"), None),
            None
        );
        assert_eq!(
            project_prefix("file:///repo/crates/", Path::new("/repo"), None),
            Some("crates".to_owned())
        );
    }

    #[test]
    fn an_index_built_at_another_checkout_is_rerooted_where_its_documents_exist() {
        let dir = tempfile::tempdir().expect("tempdir");
        for file in ["apps/my svc/src/relay.rs", "apps/my svc/src/store.rs"] {
            let path = dir.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).expect("dirs");
            std::fs::write(&path, "").expect("file");
        }
        let mut index = index();
        index.metadata.as_mut().unwrap().project_root = "file:///ci/work/repo/apps/my%20svc".into();
        let under = ScipExtractor::from_index_under(&index, Some(dir.path()));
        assert!(
            under
                .documents
                .contains_key(&RepoPath::new("apps/my svc/src/relay.rs")),
            "the documents sit at the suffix of the project path that exists here"
        );
        // The project was the repository itself: its documents sit at the root.
        for file in ["src/relay.rs", "src/store.rs"] {
            let path = dir.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).expect("dirs");
            std::fs::write(&path, "").expect("file");
        }
        index.metadata.as_mut().unwrap().project_root = "file:///ci/work/repo".into();
        let at_root = ScipExtractor::from_index_under(&index, Some(dir.path()));
        assert!(
            at_root
                .documents
                .contains_key(&RepoPath::new("src/relay.rs"))
        );
    }

    #[test]
    fn indexes_of_two_workspaces_pool_their_declarations() {
        // `store.rs` is declared in the `core` workspace's index; the `app` workspace's
        // index only references it, as an external symbol from its own point of view.
        let pending = "rust-analyzer cargo outbox 0.1.0 store/impl#[Store]pending().";
        let mut core = index();
        core.metadata.as_mut().unwrap().project_root = "file:///repo/core".into();
        let app = Index {
            metadata: core.metadata.clone(),
            documents: vec![ScipDocument {
                relative_path: "src/main.rs".into(),
                occurrences: vec![occurrence(pending, &[2, 10, 17], 0)],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut app = app;
        app.metadata.as_mut().unwrap().project_root = "file:///repo/app".into();
        let pooled = ScipExtractor::from_index_under(&core, Some(Path::new("/repo"))).merge(
            ScipExtractor::from_index_under(&app, Some(Path::new("/repo"))),
        );
        assert_eq!(pooled.document_count(), 3);
        assert_eq!(
            pooled.target(pending),
            SymbolTarget::Declared {
                file: RepoPath::new("core/src/store.rs"),
                line: 3,
                name: "pending".into(),
            },
            "a symbol declared in another workspace's index is in the repository"
        );
        let alone = ScipExtractor::from_index_under(&app, Some(Path::new("/repo")));
        assert!(matches!(
            alone.target(pending),
            SymbolTarget::External { .. }
        ));
    }

    #[test]
    fn occurrences_become_resolved_facts_with_declaration_lines() {
        let extractor = ScipExtractor::from_index(&index());
        assert_eq!(
            extractor
                .resolvers()
                .map(ResolverId::as_str)
                .collect::<Vec<_>>(),
            vec!["scip/rust-analyzer"]
        );
        assert_eq!(extractor.document_count(), 2);
        let relay = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("src/relay.rs"),
            CommitId::new("c"),
            b"fn drain(s: &Store) -> String {\n    let f = |x| x; s.pending(); f(1);\n    f(2)\n}\n".to_vec(),
        );
        assert!(extractor.accepts(&relay));
        let set = extractor.extract(&relay).expect("extracts");
        let facts: Vec<&ResolvedReference> = set.resolved().collect();
        assert_eq!(facts.len(), 3, "{facts:?}");
        let call = facts.iter().find(|f| f.name == "pending").expect("call");
        assert_eq!(relay.slice(call.span.bytes), "pending");
        assert_eq!(call.span.lines.start, 2);
        assert_eq!(
            call.target,
            SymbolTarget::Declared {
                file: RepoPath::new("src/store.rs"),
                line: 3,
                name: "pending".into()
            }
        );
        let local = facts
            .iter()
            .find(|f| f.target == SymbolTarget::Local)
            .expect("local");
        assert_eq!(relay.slice(local.span.bytes), "f");
        let string = facts.iter().find(|f| f.name == "String").expect("external");
        assert!(
            matches!(&string.target, SymbolTarget::External { symbol } if symbol.contains("alloc"))
        );
        assert_eq!(relay.slice(string.span.bytes), "String");
        assert!(!extractor.accepts(&SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("src/other.rs"),
            CommitId::new("c"),
            Vec::new(),
        )));
    }

    #[test]
    fn an_unspecified_encoding_is_utf16_for_scip_typescript() {
        let unspecified = PositionEncoding::UnspecifiedPositionEncoding;
        assert_eq!(
            Encoding::of(unspecified, "scip-typescript"),
            Encoding::Utf16
        );
        assert_eq!(Encoding::of(unspecified, "rust-analyzer"), Encoding::Utf8);
        assert_eq!(Encoding::of(unspecified, ""), Encoding::Utf8);
        assert_eq!(
            Encoding::of(
                PositionEncoding::UTF8CodeUnitOffsetFromLineStart,
                "scip-typescript"
            ),
            Encoding::Utf8
        );
        // `label` sits after an em dash, which is one UTF-16 unit but three UTF-8
        // bytes; the TypeScript compiler reports its column in UTF-16 units.
        let text = "const s = \"\u{2014}\"; render(label);\n";
        let column = text
            .encode_utf16()
            .position(|u| u == u16::from(b'l'))
            .unwrap();
        let label = "scip-typescript npm ui 1.0.0 src/`view.ts`/label.";
        let index = Index {
            metadata: Some(Metadata {
                tool_info: Some(ToolInfo {
                    name: "scip-typescript".into(),
                    ..Default::default()
                })
                .into(),
                project_root: "file:///repo".into(),
                ..Default::default()
            })
            .into(),
            documents: vec![ScipDocument {
                relative_path: "src/view.ts".into(),
                occurrences: vec![occurrence(label, &[0, column as i32, column as i32 + 5], 0)],
                ..Default::default()
            }],
            ..Default::default()
        };
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("src/view.ts"),
            CommitId::new("c"),
            text.as_bytes().to_vec(),
        );
        let set = ScipExtractor::from_index(&index)
            .extract(&unit)
            .expect("extracts");
        let fact = set.resolved().next().expect("one fact");
        assert_eq!(unit.slice(fact.span.bytes), "label");
    }

    #[test]
    fn utf16_columns_map_to_bytes() {
        assert_eq!(Encoding::Utf8.byte_offset("héllo", 2), 2);
        assert_eq!(Encoding::Utf16.byte_offset("héllo", 2), 3);
        assert_eq!(Encoding::Utf32.byte_offset("héllo", 2), 3);
        assert_eq!(Encoding::Utf16.byte_offset("😀x", 2), 4);
        assert_eq!(
            symbol_name("rust-analyzer cargo core 1.0 fmt/macros/Debug:"),
            "Debug"
        );
        assert_eq!(
            symbol_name("rust-analyzer cargo a 0.1.0 traits/impl#[T][Display]fmt()."),
            "fmt"
        );
        assert_eq!(symbol_name("local 12"), "12");
    }
}
