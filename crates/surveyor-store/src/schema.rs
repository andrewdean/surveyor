//! Arrow schemas and row conversions for the five tables.
//!
//! Writing folds model values into Arrow builders; reading maps each row index through
//! typed column accessors and collects a `Result`, so one malformed row fails the read
//! instead of producing a half-converted vector.

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::builder::{
    BooleanBuilder, Float32Builder, Int64Builder, StringBuilder, UInt32Builder, UInt64Builder,
};
use arrow_array::cast::AsArray;
use arrow_array::types::{Float32Type, Int64Type, UInt32Type, UInt64Type};
use arrow_array::{Array, ArrayRef, RecordBatch};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use surveyor_model::{
    ByteSpan, ContentHash, Edge, EdgeId, EdgeKind, Language, LineSpan, Location, Node, NodeId,
    NodeKind, Provenance, RepoId, RepoPath, SourceLocation, TextSpan, Unresolved, UnresolvedReason,
};

use crate::StoreError;

pub(crate) const NODES: &str = "nodes";
pub(crate) const EDGES: &str = "edges";
pub(crate) const FILES: &str = "files";
pub(crate) const UNRESOLVED: &str = "unresolved";
pub(crate) const META: &str = "meta";

fn utf8(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

pub(crate) fn nodes_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        utf8("id", false),
        utf8("repo", false),
        utf8("kind", false),
        utf8("name", false),
        utf8("qualified_name", false),
        utf8("language", true),
        utf8("file", true),
        Field::new("span_start", DataType::UInt64, true),
        Field::new("span_end", DataType::UInt64, true),
        Field::new("line_start", DataType::UInt32, true),
        Field::new("line_end", DataType::UInt32, true),
        utf8("content_hash", false),
        utf8("signature", true),
        utf8("doc", true),
        utf8("attributes", false),
    ]))
}

pub(crate) fn edges_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        utf8("id", false),
        utf8("repo", false),
        utf8("src", false),
        utf8("dst", false),
        utf8("kind", false),
        utf8("derivation", false),
        Field::new("confidence", DataType::Float32, false),
        utf8("file", true),
        Field::new("observed_at", DataType::Int64, false),
        utf8("provenance", false),
        utf8("resolved_against", false),
    ]))
}

pub(crate) fn files_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        utf8("repo", false),
        utf8("path", false),
        utf8("language", true),
        utf8("content_hash", false),
        Field::new("bytes", DataType::UInt64, false),
        Field::new("lines", DataType::UInt64, false),
        utf8("commit", false),
        utf8("names", false),
    ]))
}

pub(crate) fn unresolved_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        utf8("repo", false),
        utf8("role", false),
        utf8("name", false),
        utf8("file", false),
        Field::new("span_start", DataType::UInt64, false),
        Field::new("span_end", DataType::UInt64, false),
        Field::new("line_start", DataType::UInt32, false),
        Field::new("line_end", DataType::UInt32, false),
        utf8("reason", false),
    ]))
}

pub(crate) fn meta_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        utf8("repo", false),
        utf8("commit", false),
        Field::new("indexed_at", DataType::Int64, false),
        utf8("surveyor_version", false),
        Field::new("nodes", DataType::UInt64, false),
        Field::new("edges", DataType::UInt64, false),
        Field::new("files", DataType::UInt64, false),
        Field::new("unresolved", DataType::UInt64, false),
        Field::new("dirty", DataType::Boolean, false),
    ]))
}

/// True when `actual` has the fields of `wanted`, by name and type, in order.
///
/// Field metadata and nullability are ignored: Lance may rewrite both, and neither
/// changes what the row conversions read.
pub(crate) fn schema_matches(actual: &Schema, wanted: &Schema) -> bool {
    actual.fields().len() == wanted.fields().len()
        && actual
            .fields()
            .iter()
            .zip(wanted.fields())
            .all(|(a, w)| a.name() == w.name() && a.data_type() == w.data_type())
}

/// A row of the `files` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    pub path: RepoPath,
    pub language: Option<Language>,
    pub content_hash: ContentHash,
    pub bytes: u64,
    pub lines: u64,
    /// Names the file references, for incremental update.
    pub names: Vec<String>,
}

fn finish(columns: Vec<ArrayRef>, schema: SchemaRef) -> Result<RecordBatch, StoreError> {
    Ok(RecordBatch::try_new(schema, columns)?)
}

pub(crate) fn nodes_batch(repo: &RepoId, nodes: &[Node]) -> Result<RecordBatch, StoreError> {
    let mut id = StringBuilder::new();
    let mut repo_b = StringBuilder::new();
    let mut kind = StringBuilder::new();
    let mut name = StringBuilder::new();
    let mut qualified = StringBuilder::new();
    let mut language = StringBuilder::new();
    let mut file = StringBuilder::new();
    let mut span_start = UInt64Builder::new();
    let mut span_end = UInt64Builder::new();
    let mut line_start = UInt32Builder::new();
    let mut line_end = UInt32Builder::new();
    let mut hash = StringBuilder::new();
    let mut signature = StringBuilder::new();
    let mut doc = StringBuilder::new();
    let mut attributes = StringBuilder::new();
    for n in nodes {
        let span = n.span();
        id.append_value(n.id.as_str());
        repo_b.append_value(repo.as_str());
        kind.append_value(n.kind.as_str());
        name.append_value(&n.name);
        qualified.append_value(&n.qualified_name);
        language.append_option(n.language.as_ref().map(Language::as_str));
        file.append_option(n.file().map(RepoPath::as_str));
        span_start.append_option(span.map(|s| s.start()));
        span_end.append_option(span.map(|s| s.end()));
        line_start.append_option(span.map(|s| s.lines.start));
        line_end.append_option(span.map(|s| s.lines.end));
        hash.append_value(n.content_hash.to_hex());
        signature.append_option(n.signature.as_deref());
        doc.append_option(n.doc.as_deref());
        attributes.append_value(serde_json::to_string(&n.attributes)?);
    }
    finish(
        vec![
            Arc::new(id.finish()),
            Arc::new(repo_b.finish()),
            Arc::new(kind.finish()),
            Arc::new(name.finish()),
            Arc::new(qualified.finish()),
            Arc::new(language.finish()),
            Arc::new(file.finish()),
            Arc::new(span_start.finish()),
            Arc::new(span_end.finish()),
            Arc::new(line_start.finish()),
            Arc::new(line_end.finish()),
            Arc::new(hash.finish()),
            Arc::new(signature.finish()),
            Arc::new(doc.finish()),
            Arc::new(attributes.finish()),
        ],
        nodes_schema(),
    )
}

pub(crate) fn edges_batch(repo: &RepoId, edges: &[Edge]) -> Result<RecordBatch, StoreError> {
    let mut id = StringBuilder::new();
    let mut repo_b = StringBuilder::new();
    let mut src = StringBuilder::new();
    let mut dst = StringBuilder::new();
    let mut kind = StringBuilder::new();
    let mut derivation = StringBuilder::new();
    let mut confidence = Float32Builder::new();
    let mut file = StringBuilder::new();
    let mut observed = Int64Builder::new();
    let mut provenance = StringBuilder::new();
    let mut against = StringBuilder::new();
    for e in edges {
        id.append_value(e.id.as_str());
        repo_b.append_value(repo.as_str());
        src.append_value(e.src.as_str());
        dst.append_value(e.dst.as_str());
        kind.append_value(e.kind.as_str());
        derivation.append_value(e.provenance.derivation.kind().as_str());
        confidence.append_value(e.provenance.confidence.get());
        file.append_option(
            e.provenance
                .evidence
                .first()
                .map(|ev| ev.location.file.as_str()),
        );
        observed.append_value(e.provenance.observed_at.as_unix_millis());
        provenance.append_value(serde_json::to_string(&e.provenance)?);
        against.append_value(serde_json::to_string(&e.resolved_against)?);
    }
    finish(
        vec![
            Arc::new(id.finish()),
            Arc::new(repo_b.finish()),
            Arc::new(src.finish()),
            Arc::new(dst.finish()),
            Arc::new(kind.finish()),
            Arc::new(derivation.finish()),
            Arc::new(confidence.finish()),
            Arc::new(file.finish()),
            Arc::new(observed.finish()),
            Arc::new(provenance.finish()),
            Arc::new(against.finish()),
        ],
        edges_schema(),
    )
}

pub(crate) fn files_batch(
    repo: &RepoId,
    commit: &str,
    files: &[FileRow],
) -> Result<RecordBatch, StoreError> {
    let mut repo_b = StringBuilder::new();
    let mut path = StringBuilder::new();
    let mut language = StringBuilder::new();
    let mut hash = StringBuilder::new();
    let mut bytes = UInt64Builder::new();
    let mut lines = UInt64Builder::new();
    let mut commit_b = StringBuilder::new();
    let mut names = StringBuilder::new();
    for f in files {
        repo_b.append_value(repo.as_str());
        path.append_value(f.path.as_str());
        language.append_option(f.language.as_ref().map(Language::as_str));
        hash.append_value(f.content_hash.to_hex());
        bytes.append_value(f.bytes);
        lines.append_value(f.lines);
        commit_b.append_value(commit);
        names.append_value(serde_json::to_string(&f.names)?);
    }
    finish(
        vec![
            Arc::new(repo_b.finish()),
            Arc::new(path.finish()),
            Arc::new(language.finish()),
            Arc::new(hash.finish()),
            Arc::new(bytes.finish()),
            Arc::new(lines.finish()),
            Arc::new(commit_b.finish()),
            Arc::new(names.finish()),
        ],
        files_schema(),
    )
}

pub(crate) fn unresolved_batch(
    repo: &RepoId,
    items: &[Unresolved],
) -> Result<RecordBatch, StoreError> {
    let mut repo_b = StringBuilder::new();
    let mut role = StringBuilder::new();
    let mut name = StringBuilder::new();
    let mut file = StringBuilder::new();
    let mut span_start = UInt64Builder::new();
    let mut span_end = UInt64Builder::new();
    let mut line_start = UInt32Builder::new();
    let mut line_end = UInt32Builder::new();
    let mut reason = StringBuilder::new();
    for u in items {
        repo_b.append_value(repo.as_str());
        role.append_value(serde_json::to_string(&u.role)?);
        name.append_value(&u.name);
        file.append_value(u.site.file.as_str());
        span_start.append_value(u.site.span.start());
        span_end.append_value(u.site.span.end());
        line_start.append_value(u.site.span.lines.start);
        line_end.append_value(u.site.span.lines.end);
        reason.append_value(u.reason.as_str());
    }
    finish(
        vec![
            Arc::new(repo_b.finish()),
            Arc::new(role.finish()),
            Arc::new(name.finish()),
            Arc::new(file.finish()),
            Arc::new(span_start.finish()),
            Arc::new(span_end.finish()),
            Arc::new(line_start.finish()),
            Arc::new(line_end.finish()),
            Arc::new(reason.finish()),
        ],
        unresolved_schema(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MetaRow {
    pub repo: RepoId,
    pub commit: String,
    pub indexed_at: i64,
    pub surveyor_version: String,
    pub nodes: u64,
    pub edges: u64,
    pub files: u64,
    pub unresolved: u64,
    /// Whether the working tree differed from `commit` when indexed.
    pub dirty: bool,
}

pub(crate) fn meta_batch(row: &MetaRow) -> Result<RecordBatch, StoreError> {
    let mut dirty = BooleanBuilder::new();
    dirty.append_value(row.dirty);
    finish(
        vec![
            Arc::new(arrow_array::StringArray::from(vec![row.repo.as_str()])),
            Arc::new(arrow_array::StringArray::from(vec![row.commit.as_str()])),
            Arc::new(arrow_array::Int64Array::from(vec![row.indexed_at])),
            Arc::new(arrow_array::StringArray::from(vec![
                row.surveyor_version.as_str(),
            ])),
            Arc::new(arrow_array::UInt64Array::from(vec![row.nodes])),
            Arc::new(arrow_array::UInt64Array::from(vec![row.edges])),
            Arc::new(arrow_array::UInt64Array::from(vec![row.files])),
            Arc::new(arrow_array::UInt64Array::from(vec![row.unresolved])),
            Arc::new(dirty.finish()),
        ],
        meta_schema(),
    )
}

// ---- reading -----------------------------------------------------------------------

/// Typed access to one batch's columns.
struct Cols<'a> {
    batch: &'a RecordBatch,
}

impl<'a> Cols<'a> {
    fn col(&self, name: &str) -> Result<&'a ArrayRef, StoreError> {
        self.batch
            .column_by_name(name)
            .ok_or_else(|| StoreError::schema(format!("missing column {name}")))
    }

    fn str(&self, name: &str, row: usize) -> Result<Option<&'a str>, StoreError> {
        let arr = self
            .col(name)?
            .as_string_opt::<i32>()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not utf8")))?;
        Ok((!arr.is_null(row)).then(|| arr.value(row)))
    }

    fn req(&self, name: &str, row: usize) -> Result<&'a str, StoreError> {
        self.str(name, row)?
            .ok_or_else(|| StoreError::schema(format!("null in column {name}")))
    }

    fn parsed<T: std::str::FromStr>(&self, name: &str, row: usize) -> Result<T, StoreError>
    where
        T::Err: std::fmt::Display,
    {
        self.req(name, row)?
            .parse::<T>()
            .map_err(|e| StoreError::schema(format!("column {name}: {e}")))
    }

    fn u64(&self, name: &str, row: usize) -> Result<Option<u64>, StoreError> {
        let arr = self
            .col(name)?
            .as_primitive_opt::<UInt64Type>()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not u64")))?;
        Ok((!arr.is_null(row)).then(|| arr.value(row)))
    }

    fn u32(&self, name: &str, row: usize) -> Result<Option<u32>, StoreError> {
        let arr = self
            .col(name)?
            .as_primitive_opt::<UInt32Type>()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not u32")))?;
        Ok((!arr.is_null(row)).then(|| arr.value(row)))
    }

    fn i64(&self, name: &str, row: usize) -> Result<i64, StoreError> {
        let arr = self
            .col(name)?
            .as_primitive_opt::<Int64Type>()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not i64")))?;
        Ok(arr.value(row))
    }

    fn bool(&self, name: &str, row: usize) -> Result<bool, StoreError> {
        let arr = self
            .col(name)?
            .as_boolean_opt()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not boolean")))?;
        Ok(arr.value(row))
    }

    fn f32(&self, name: &str, row: usize) -> Result<f32, StoreError> {
        let arr = self
            .col(name)?
            .as_primitive_opt::<Float32Type>()
            .ok_or_else(|| StoreError::schema(format!("column {name} is not f32")))?;
        Ok(arr.value(row))
    }

    /// The span stored in the four span columns, when all four are present.
    fn text_span(&self, row: usize) -> Result<Option<TextSpan>, StoreError> {
        let bytes = self.u64("span_start", row)?.zip(self.u64("span_end", row)?);
        let lines = self.u32("line_start", row)?.zip(self.u32("line_end", row)?);
        Ok(bytes.zip(lines).map(|((s, e), (ls, le))| {
            TextSpan::new(ByteSpan::new(s, e.max(s)), LineSpan::new(ls, le.max(ls)))
        }))
    }

    /// The location stored in the file and span columns.
    fn location(&self, row: usize) -> Result<Option<Location>, StoreError> {
        let file = self.str("file", row)?.map(RepoPath::new);
        let span = self.text_span(row)?;
        Ok(file.map(|file| match span {
            Some(span) => Location::Span(SourceLocation::new(file, span)),
            None => Location::File { file },
        }))
    }
}

/// Converts every row of `batch` with `row`, failing on the first malformed row.
fn rows<T>(
    batch: &RecordBatch,
    row: impl Fn(&Cols<'_>, usize) -> Result<T, StoreError>,
) -> Result<Vec<T>, StoreError> {
    let cols = Cols { batch };
    (0..batch.num_rows()).map(|i| row(&cols, i)).collect()
}

pub(crate) fn nodes_from_batch(batch: &RecordBatch) -> Result<Vec<Node>, StoreError> {
    rows(batch, |c, row| {
        Ok(Node {
            id: NodeId::new(c.req("id", row)?),
            kind: c.parsed::<NodeKind>("kind", row)?,
            name: c.req("name", row)?.to_owned(),
            qualified_name: c.req("qualified_name", row)?.to_owned(),
            repo: RepoId::new(c.req("repo", row)?),
            language: c
                .str("language", row)?
                .and_then(|l| l.parse::<Language>().ok()),
            location: c.location(row)?,
            content_hash: c.parsed::<ContentHash>("content_hash", row)?,
            signature: c.str("signature", row)?.map(str::to_owned),
            doc: c.str("doc", row)?.map(str::to_owned),
            attributes: serde_json::from_str::<BTreeMap<String, String>>(
                c.req("attributes", row)?,
            )?,
        })
    })
}

pub(crate) fn edges_from_batch(batch: &RecordBatch) -> Result<Vec<Edge>, StoreError> {
    rows(batch, |c, row| {
        // The scalar columns exist for filtering; the JSON provenance is the record.
        let _ = (c.f32("confidence", row)?, c.i64("observed_at", row)?);
        Ok(Edge {
            id: EdgeId::from(c.req("id", row)?.to_owned()),
            src: NodeId::new(c.req("src", row)?),
            dst: NodeId::new(c.req("dst", row)?),
            kind: c.parsed::<EdgeKind>("kind", row)?,
            provenance: serde_json::from_str::<Provenance>(c.req("provenance", row)?)?,
            resolved_against: serde_json::from_str::<Vec<NodeId>>(c.req("resolved_against", row)?)?,
        })
    })
}

pub(crate) fn files_from_batch(batch: &RecordBatch) -> Result<Vec<FileRow>, StoreError> {
    rows(batch, |c, row| {
        Ok(FileRow {
            path: RepoPath::new(c.req("path", row)?),
            language: c
                .str("language", row)?
                .and_then(|l| l.parse::<Language>().ok()),
            content_hash: c.parsed::<ContentHash>("content_hash", row)?,
            bytes: c.u64("bytes", row)?.unwrap_or(0),
            lines: c.u64("lines", row)?.unwrap_or(0),
            names: serde_json::from_str::<Vec<String>>(c.req("names", row)?)?,
        })
    })
}

pub(crate) fn unresolved_from_batch(batch: &RecordBatch) -> Result<Vec<Unresolved>, StoreError> {
    rows(batch, |c, row| {
        let span = c
            .text_span(row)?
            .ok_or_else(|| StoreError::schema("unresolved row without a span"))?;
        Ok(Unresolved {
            role: serde_json::from_str(c.req("role", row)?)?,
            name: c.req("name", row)?.to_owned(),
            site: SourceLocation::new(RepoPath::new(c.req("file", row)?), span),
            reason: c.parsed::<UnresolvedReason>("reason", row)?,
        })
    })
}

pub(crate) fn meta_from_batch(batch: &RecordBatch) -> Result<Vec<MetaRow>, StoreError> {
    rows(batch, |c, row| {
        Ok(MetaRow {
            repo: RepoId::new(c.req("repo", row)?),
            commit: c.req("commit", row)?.to_owned(),
            indexed_at: c.i64("indexed_at", row)?,
            surveyor_version: c.req("surveyor_version", row)?.to_owned(),
            nodes: c.u64("nodes", row)?.unwrap_or(0),
            edges: c.u64("edges", row)?.unwrap_or(0),
            files: c.u64("files", row)?.unwrap_or(0),
            unresolved: c.u64("unresolved", row)?.unwrap_or(0),
            dirty: c.bool("dirty", row)?,
        })
    })
}
