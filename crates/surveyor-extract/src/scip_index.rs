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
    fn of(document: PositionEncoding) -> Self {
        match document {
            PositionEncoding::UTF16CodeUnitOffsetFromLineStart => Self::Utf16,
            PositionEncoding::UTF32CodeUnitOffsetFromLineStart => Self::Utf32,
            // rust-analyzer and the other native indexers count UTF-8 units, and the
            // protocol's own advice for them says so; take that as the default too.
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

/// Emits resolved references from a SCIP index.
pub struct ScipExtractor {
    resolver: ResolverId,
    documents: HashMap<RepoPath, Document>,
    definitions: HashMap<String, Definition>,
}

impl std::fmt::Debug for ScipExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScipExtractor")
            .field("resolver", &self.resolver)
            .field("documents", &self.documents.len())
            .field("definitions", &self.definitions.len())
            .finish()
    }
}

impl ScipExtractor {
    /// Reads the index at `path`.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be read and a parse error when it is
    /// not a SCIP index.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ExtractError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| ExtractError::io(e).at(path))?;
        let index = Index::parse_from_bytes(&bytes)
            .map_err(|e| ExtractError::parse(format!("not a SCIP index: {e}")).at(path))?;
        let extractor = Self::from_index(&index);
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

    /// Builds the extractor from a parsed index.
    #[must_use]
    pub fn from_index(index: &Index) -> Self {
        let tool = index.metadata.tool_info.name.as_str();
        let resolver = ResolverId::new(format!(
            "{PREFIX}/{}",
            if tool.is_empty() { "unknown" } else { tool }
        ));
        let mut definitions: HashMap<String, Definition> = HashMap::new();
        let documents: HashMap<RepoPath, Document> = index
            .documents
            .iter()
            .map(|doc| {
                let path = RepoPath::new(&doc.relative_path);
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
                );
                (
                    path,
                    Document {
                        encoding,
                        occurrences,
                    },
                )
            })
            .collect();
        Self {
            resolver,
            documents,
            definitions,
        }
    }

    /// The resolver recorded on the facts, such as `scip/rust-analyzer`.
    #[must_use]
    pub fn resolver(&self) -> &ResolverId {
        &self.resolver
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

impl Extractor for ScipExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(self.resolver.as_str())
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
                        resolver: self.resolver.clone(),
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
mod tests {
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
    fn index() -> Index {
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
    fn occurrences_become_resolved_facts_with_declaration_lines() {
        let extractor = ScipExtractor::from_index(&index());
        assert_eq!(extractor.resolver().as_str(), "scip/rust-analyzer");
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
