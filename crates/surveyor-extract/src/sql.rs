//! SQL extraction with `sqlparser`: schema from migrations, tables from statements.
//!
//! Whole `.sql` files are parsed statement by statement. `CREATE TABLE`, `ALTER TABLE
//! ... ADD COLUMN`, `CREATE INDEX`, `CREATE SCHEMA` and `CREATE [MATERIALIZED] VIEW`
//! become declarations scoped by schema; every statement's table references become
//! `Read` or `Write` references, and a view's defining query reads its tables.
//! [`summarize_sql`] does the same for an SQL string literal found in code, which the
//! pipeline turns into a `Query` declaration.
//!
//! A `DO $$ ... $$` block does not parse as SQL, but the statements it hands to
//! `EXECUTE` as string literals do: `format($v$ CREATE VIEW %I.events_v1 AS ... $v$,
//! schema)` declares a view whose schema is a run-time argument. Such statements are
//! parsed with their `format()` specifiers replaced (see [`crate::template`]), cite
//! the literal's own span, carry the attribute `dynamic = true`, and resolve a
//! templated qualifier the way code does: by the bare name across schemas.
//!
//! Files that mix SQL with procedural blocks or client meta-commands parse partially:
//! statements that fail and yield nothing are skipped and counted in the
//! `sql.unparsed` attribute of the fact set's first declaration, never silently
//! invented.

use std::collections::{BTreeMap, HashSet};
use std::ops::ControlFlow;

use sqlparser::ast::{
    AlterTableOperation, FromTable, ObjectName, ObjectNamePart, ObjectType, Query, SchemaName,
    SetExpr, Statement, TableFactor, TableObject, Visit, Visitor,
};
use sqlparser::dialect::{
    AnsiDialect, BigQueryDialect, Dialect, DuckDbDialect, GenericDialect, MySqlDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlparser::parser::Parser;
use surveyor_model::{
    ByteSpan, DeclarationFact, ExtractorId, Fact, FactSet, Language, NodeKind, ReferenceFact,
    ReferenceRole, RuleId, SourceUnit, TextSpan,
};

use crate::profile::SqlDialect;
use crate::template::{PLACEHOLDER, leads_with_sql, substitute_format, table_name};
use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "sqlparser";

/// How deep string literals are searched for statements: a `DO` body, and the
/// literals inside it. A literal inside those is data, not a statement.
const QUOTE_DEPTH: u8 = 2;

fn dialect_for(dialect: SqlDialect) -> Box<dyn Dialect> {
    match dialect {
        SqlDialect::PostgreSql => Box::new(PostgreSqlDialect {}),
        SqlDialect::MySql => Box::new(MySqlDialect {}),
        SqlDialect::Sqlite => Box::new(SQLiteDialect {}),
        SqlDialect::DuckDb => Box::new(DuckDbDialect {}),
        SqlDialect::Snowflake => Box::new(SnowflakeDialect {}),
        SqlDialect::BigQuery => Box::new(BigQueryDialect {}),
        SqlDialect::Ansi => Box::new(AnsiDialect {}),
        SqlDialect::Generic => Box::new(GenericDialect {}),
    }
}

/// Tables a statement reads and writes, plus its kind.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatementSummary {
    /// Statement kind in upper case, such as `SELECT` or `INSERT`.
    pub kind: String,
    /// Qualified table names read, in first-seen order.
    pub reads: Vec<String>,
    /// Qualified table names written, in first-seen order.
    pub writes: Vec<String>,
}

/// Parses one SQL text and reports the tables it touches.
///
/// Returns `None` when the text does not parse in the given dialect or contains no
/// statement. Several statements are merged into one summary.
#[must_use]
pub fn summarize_sql(sql: &str, dialect: SqlDialect) -> Option<StatementSummary> {
    let d = dialect_for(dialect);
    let statements = Parser::parse_sql(d.as_ref(), sql).ok()?;
    if statements.is_empty() {
        return None;
    }
    let mut summary = StatementSummary::default();
    for stmt in &statements {
        let one = summarize_statement(stmt);
        if summary.kind.is_empty() {
            summary.kind.clone_from(&one.kind);
        }
        push_unique(&mut summary.reads, one.reads);
        push_unique(&mut summary.writes, one.writes);
    }
    Some(summary)
}

fn push_unique(into: &mut Vec<String>, from: Vec<String>) {
    for item in from {
        if !into.contains(&item) {
            into.push(item);
        }
    }
}

fn object_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .map(|part| match part {
            ObjectNamePart::Identifier(ident) => ident.value.clone(),
            ObjectNamePart::Function(f) => f.to_string(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Names a statement introduces itself, which read like tables but are not.
///
/// A common table expression (`WITH recent AS (...) SELECT ... FROM recent`) and a
/// table function (`FROM jsonb_array_elements(tags)`) both appear as relations to
/// `sqlparser`; neither is a table the schema declares.
#[derive(Debug, Default)]
struct LocalNames(HashSet<String>);

impl Visitor for LocalNames {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.0.extend(
            query
                .with
                .iter()
                .flat_map(|with| with.cte_tables.iter())
                .map(|cte| cte.alias.name.value.clone()),
        );
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if let TableFactor::Table {
            name,
            args: Some(_),
            ..
        }
        | TableFactor::Function { name, .. } = factor
        {
            self.0.insert(object_name(name));
        }
        ControlFlow::Continue(())
    }
}

fn local_names(stmt: &Statement) -> HashSet<String> {
    let mut names = LocalNames::default();
    let _ = stmt.visit(&mut names);
    names.0
}

fn summarize_statement(stmt: &Statement) -> StatementSummary {
    let local = local_names(stmt);
    let mut all: Vec<String> = Vec::new();
    let _ = sqlparser::ast::visit_relations(stmt, |rel: &ObjectName| {
        let name = object_name(rel);
        if !all.contains(&name) && !local.contains(&name) {
            all.push(name);
        }
        ControlFlow::<()>::Continue(())
    });
    let mut writes: Vec<String> = Vec::new();
    let kind = match stmt {
        // `WITH ... INSERT/UPDATE/DELETE/MERGE` parses as a query wrapping the statement.
        Statement::Query(query) => match query.body.as_ref() {
            SetExpr::Insert(inner)
            | SetExpr::Update(inner)
            | SetExpr::Delete(inner)
            | SetExpr::Merge(inner) => {
                let StatementSummary {
                    kind,
                    reads,
                    writes: inner_writes,
                } = summarize_statement(inner);
                push_unique(
                    &mut all,
                    reads.into_iter().filter(|t| !local.contains(t)).collect(),
                );
                writes = inner_writes;
                return StatementSummary {
                    kind,
                    reads: all.into_iter().filter(|t| !writes.contains(t)).collect(),
                    writes,
                };
            }
            _ => "SELECT",
        },
        Statement::Insert(insert) => {
            if let TableObject::TableName(name) = &insert.table {
                writes.push(object_name(name));
            }
            "INSERT"
        }
        Statement::Update(update) => {
            if let TableFactor::Table { name, .. } = &update.table.relation {
                writes.push(object_name(name));
            }
            "UPDATE"
        }
        Statement::Delete(delete) => {
            let from = match &delete.from {
                FromTable::WithFromKeyword(t) | FromTable::WithoutKeyword(t) => t,
            };
            for t in from {
                if let TableFactor::Table { name, .. } = &t.relation {
                    writes.push(object_name(name));
                }
            }
            for name in &delete.tables {
                writes.push(object_name(name));
            }
            "DELETE"
        }
        Statement::Merge(merge) => {
            if let TableFactor::Table { name, .. } = &merge.table {
                writes.push(object_name(name));
            }
            "MERGE"
        }
        Statement::CreateTable(_) => "CREATE TABLE",
        Statement::AlterTable(_) => "ALTER TABLE",
        Statement::CreateIndex(_) => "CREATE INDEX",
        Statement::CreateSchema { .. } => "CREATE SCHEMA",
        Statement::CreateView(view) if view.materialized => "CREATE MATERIALIZED VIEW",
        Statement::CreateView(_) => "CREATE VIEW",
        // Dropping a relation changes the schema, so it is a write; dropping anything
        // else (an index, a sequence) names no relation.
        Statement::Drop {
            object_type: ObjectType::Table | ObjectType::View | ObjectType::MaterializedView,
            names,
            ..
        } => {
            writes.extend(names.iter().map(object_name));
            "DROP"
        }
        Statement::Drop { .. } => {
            return StatementSummary {
                kind: "DROP".to_owned(),
                ..StatementSummary::default()
            };
        }
        Statement::Truncate { .. } => "TRUNCATE",
        Statement::Copy { .. } => "COPY",
        _ => "OTHER",
    };
    let reads = all.into_iter().filter(|t| !writes.contains(t)).collect();
    StatementSummary {
        kind: kind.to_owned(),
        reads,
        writes,
    }
}

/// How a statement names a relation once format placeholders are accounted for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Named {
    /// `schema.table`, or `table` in the default schema.
    Qualified { schema: String, table: String },
    /// `%I.table`: the schema is a run-time argument of `format()`.
    Templated { table: String },
    /// `%I`: the relation itself is an argument; nothing to declare or resolve.
    Unknown,
}

/// Where a statement came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementSource {
    /// Written in the file as a statement of its own.
    File,
    /// Handed to `EXECUTE` as a string literal inside a procedural block.
    Executed,
}

/// Extracts schema declarations and table references from `.sql` files.
#[derive(Debug, Clone)]
pub struct SqlExtractor {
    dialect: SqlDialect,
    default_schema: String,
}

impl SqlExtractor {
    /// Creates an extractor for `dialect`; unqualified tables live in `default_schema`.
    pub fn new(dialect: SqlDialect, default_schema: impl Into<String>) -> Self {
        Self {
            dialect,
            default_schema: default_schema.into(),
        }
    }

    /// The dialect this extractor parses.
    #[must_use]
    pub fn dialect(&self) -> SqlDialect {
        self.dialect
    }

    /// Splits `schema.table` or `table` into `(schema, table)` using the default schema.
    fn split_table<'a>(&'a self, name: &'a str) -> (String, String) {
        match name.rsplit_once('.') {
            Some((schema, table)) => (schema.to_owned(), table.to_owned()),
            None => (self.default_schema.clone(), name.to_owned()),
        }
    }

    /// Reads a relation name as the statement wrote it, placeholders included.
    fn name_relation(&self, name: &str) -> Named {
        if !name.contains(PLACEHOLDER) {
            let (schema, table) = self.split_table(name);
            return Named::Qualified { schema, table };
        }
        match table_name(name) {
            Some(bare) if !bare.contains('.') => Named::Templated { table: bare },
            Some(qualified) => {
                let (schema, table) = self.split_table(&qualified);
                Named::Qualified { schema, table }
            }
            None => Named::Unknown,
        }
    }

    /// Where a declared relation lives; a templated schema falls back to the default,
    /// which is where bare names in the same file live too.
    fn declared_in(&self, name: &str) -> Option<(String, String)> {
        match self.name_relation(name) {
            Named::Qualified { schema, table } => Some((schema, table)),
            Named::Templated { table } => Some((self.default_schema.clone(), table)),
            Named::Unknown => None,
        }
    }

    /// The reference path for a relation a statement touches: `[schema, table]`, or
    /// `[table]` when the schema is decided at run time.
    fn relation_path(&self, name: &str) -> Option<Vec<String>> {
        match self.name_relation(name) {
            Named::Qualified { schema, table } => Some(vec![schema, table]),
            Named::Templated { table } => Some(vec![table]),
            Named::Unknown => None,
        }
    }

    /// Extracts facts from an SQL unit.
    ///
    /// # Errors
    /// Returns an error only when the file is not UTF-8; unparsable statements are
    /// skipped and counted.
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("SQL file is not UTF-8"))?;
        let dialect = dialect_for(self.dialect);
        let mut set = FactSet::empty(ExtractorId::new(EXTRACTOR_ID), unit);
        let unparsed = split_statements(text)
            .into_iter()
            .map(|chunk| (chunk, Parser::parse_sql(dialect.as_ref(), chunk.text)))
            .fold(0usize, |unparsed, (chunk, parsed)| match parsed {
                Ok(statements) => {
                    let span = unit.text_span(statement_bytes(chunk));
                    for stmt in &statements {
                        self.push_statement_facts(
                            stmt,
                            span,
                            unit,
                            &mut set,
                            StatementSource::File,
                        );
                    }
                    unparsed
                }
                Err(_) if self.push_executed_facts(dialect.as_ref(), chunk, unit, &mut set) > 0 => {
                    unparsed
                }
                Err(_) => unparsed + 1,
            });
        if unparsed > 0
            && let Some(Fact::Declaration(first)) = set.facts.iter_mut().next()
        {
            first
                .attributes
                .insert("sql.unparsed".to_owned(), unparsed.to_string());
        }
        Ok(set)
    }

    /// Statements a procedural chunk hands to `EXECUTE` as string literals.
    ///
    /// Returns how many statements were recovered. Each literal that leads with a
    /// statement keyword is parsed with its `format()` specifiers replaced; one that
    /// still does not parse (`'CREATE VIEW v AS ' || body`) yields nothing.
    fn push_executed_facts(
        &self,
        dialect: &dyn Dialect,
        chunk: Chunk<'_>,
        unit: &SourceUnit,
        set: &mut FactSet,
    ) -> usize {
        quoted_statements(chunk.text)
            .into_iter()
            .filter_map(|quoted| {
                let parsed = Parser::parse_sql(dialect, &substitute_format(&quoted.sql())).ok()?;
                Some((quoted, parsed))
            })
            .map(|(quoted, statements)| {
                let span = unit.text_span(trimmed_bytes(quoted.raw, chunk.offset + quoted.start));
                for stmt in &statements {
                    self.push_statement_facts(stmt, span, unit, set, StatementSource::Executed);
                }
                statements.len()
            })
            .sum()
    }

    #[expect(clippy::too_many_lines, reason = "one match over the statement kinds")]
    fn push_statement_facts(
        &self,
        stmt: &Statement,
        span: TextSpan,
        unit: &SourceUnit,
        set: &mut FactSet,
        source: StatementSource,
    ) {
        let decl =
            |kind: NodeKind, name: String, scope: Vec<String>, attrs: BTreeMap<String, String>| {
                let attributes = attrs
                    .into_iter()
                    .chain(
                        (source == StatementSource::Executed)
                            .then(|| ("dynamic".to_owned(), "true".to_owned())),
                    )
                    .collect();
                Fact::Declaration(DeclarationFact {
                    kind,
                    name,
                    scope,
                    span,
                    name_span: span.bytes,
                    signature: Some(first_line(unit.slice(span.bytes))),
                    doc: None,
                    rule: RuleId::new("sql.statement"),
                    attributes,
                    hash: unit.hash_of(span.bytes),
                })
            };
        let reference = |role: ReferenceRole, path: Vec<String>, rule: &str| {
            Fact::Reference(ReferenceFact {
                role,
                name: path.last().cloned().unwrap_or_default(),
                path,
                span,
                scope: Vec::new(),
                rule: RuleId::new(rule),
                hash: unit.hash_of(span.bytes),
                receiver: None,
                binding: None,
            })
        };
        match stmt {
            Statement::CreateSchema { schema_name, .. } => {
                let name = match schema_name {
                    SchemaName::Simple(n) | SchemaName::NamedAuthorization(n, _) => object_name(n),
                    SchemaName::UnnamedAuthorization(i) => i.value.clone(),
                };
                set.facts
                    .push(decl(NodeKind::Schema, name, Vec::new(), BTreeMap::new()));
            }
            Statement::CreateTable(create) => {
                let Some((schema, table)) = self.declared_in(&object_name(&create.name)) else {
                    return;
                };
                set.facts.push(decl(
                    NodeKind::Table,
                    table.clone(),
                    vec![schema.clone()],
                    BTreeMap::new(),
                ));
                let in_table = vec![schema.clone(), table.clone()];
                set.facts.extend(create.columns.iter().map(|col| {
                    let attrs = BTreeMap::from([("type".to_owned(), col.data_type.to_string())]);
                    decl(
                        NodeKind::Column,
                        col.name.value.clone(),
                        in_table.clone(),
                        attrs,
                    )
                }));
                set.facts.extend(
                    create
                        .constraints
                        .iter()
                        .enumerate()
                        .map(|(i, constraint)| {
                            let text = constraint.to_string();
                            let name =
                                constraint_name(&text).unwrap_or_else(|| format!("constraint_{i}"));
                            let attrs = BTreeMap::from([("definition".to_owned(), text)]);
                            decl(NodeKind::Constraint, name, in_table.clone(), attrs)
                        }),
                );
            }
            Statement::AlterTable(alter) => {
                let name = object_name(&alter.name);
                let (Some((schema, table)), Some(path)) =
                    (self.declared_in(&name), self.relation_path(&name))
                else {
                    return;
                };
                set.facts
                    .extend(alter.operations.iter().filter_map(|op| match op {
                        AlterTableOperation::AddColumn { column_def, .. } => {
                            let attrs = BTreeMap::from([(
                                "type".to_owned(),
                                column_def.data_type.to_string(),
                            )]);
                            Some(decl(
                                NodeKind::Column,
                                column_def.name.value.clone(),
                                vec![schema.clone(), table.clone()],
                                attrs,
                            ))
                        }
                        _ => None,
                    }));
                set.facts
                    .push(reference(ReferenceRole::Write, path, "sql.alter-table"));
            }
            Statement::CreateView(view) => {
                let own_name = object_name(&view.name);
                let Some((schema, name)) = self.declared_in(&own_name) else {
                    return;
                };
                let attrs =
                    BTreeMap::from([("materialized".to_owned(), view.materialized.to_string())]);
                set.facts
                    .push(decl(NodeKind::View, name, vec![schema], attrs));
                // The statement's relations include the view itself; the rest is
                // what its query selects from.
                set.facts.extend(
                    summarize_statement(stmt)
                        .reads
                        .into_iter()
                        .filter(|table| *table != own_name)
                        .filter_map(|table| self.relation_path(&table))
                        .map(|path| reference(ReferenceRole::Read, path, "sql.view")),
                );
            }
            Statement::CreateIndex(index) => {
                let Some((schema, table)) = self.declared_in(&object_name(&index.table_name))
                else {
                    return;
                };
                let name = index
                    .name
                    .as_ref()
                    .map_or_else(|| format!("idx_{table}"), object_name);
                let mut attrs = BTreeMap::new();
                attrs.insert(
                    "columns".to_owned(),
                    index
                        .columns
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                set.facts
                    .push(decl(NodeKind::Index, name, vec![schema, table], attrs));
            }
            other => {
                let summary = summarize_statement(other);
                let touched = [
                    (ReferenceRole::Read, summary.reads),
                    (ReferenceRole::Write, summary.writes),
                ];
                set.facts
                    .extend(touched.into_iter().flat_map(|(role, names)| {
                        names
                            .into_iter()
                            .filter_map(|table| self.relation_path(&table))
                            .map(move |path| reference(role.clone(), path, "sql.statement"))
                            .collect::<Vec<_>>()
                    }));
            }
        }
    }
}

impl Extractor for SqlExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_ID)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        unit.language == Some(Language::Sql)
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

fn constraint_name(definition: &str) -> Option<String> {
    let rest = definition.strip_prefix("CONSTRAINT ")?;
    rest.split_whitespace()
        .next()
        .map(|s| s.trim_matches('"').to_owned())
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_owned()
}

#[derive(Debug, Clone, Copy)]
struct Chunk<'a> {
    text: &'a str,
    offset: usize,
}

/// Where the statement splitter is inside the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lex<'a> {
    Code,
    SingleQuoted,
    DoubleQuoted,
    /// Inside `$tag$ ... $tag$`; the tag keeps both dollar signs.
    DollarQuoted(&'a [u8]),
    LineComment,
    BlockComment,
}

impl<'a> Lex<'a> {
    /// The state after reading the start of `rest`, and how many bytes that consumed.
    fn step(self, rest: &'a [u8]) -> (Self, usize) {
        let (c, next) = (rest[0], rest.get(1).copied());
        match (self, c, next) {
            (Self::LineComment, b'\n', _)
            | (Self::SingleQuoted, b'\'', _)
            | (Self::DoubleQuoted, b'"', _) => (Self::Code, 1),
            (Self::BlockComment, b'*', Some(b'/')) => (Self::Code, 2),
            (Self::DollarQuoted(tag), b'$', _) if rest.starts_with(tag) => (Self::Code, tag.len()),
            (Self::Code, b'-', Some(b'-')) => (Self::LineComment, 2),
            (Self::Code, b'/', Some(b'*')) => (Self::BlockComment, 2),
            (Self::Code, b'\'', _) => (Self::SingleQuoted, 1),
            (Self::Code, b'"', _) => (Self::DoubleQuoted, 1),
            (Self::Code, b'$', _) => match dollar_tag(rest) {
                Some(tag) => (Self::DollarQuoted(tag), tag.len()),
                None => (Self::Code, 1),
            },
            (state, _, _) => (state, 1),
        }
    }
}

/// The `$tag$` at the start of `rest`: `$$` or `$name$` where the name starts with a
/// letter or underscore, so a `$1` parameter is never a tag.
fn dollar_tag(rest: &[u8]) -> Option<&[u8]> {
    let name_len = rest
        .get(1..)?
        .iter()
        .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
        .count();
    let tag = rest.get(..name_len + 2)?;
    let named_well = name_len == 0 || !rest[1].is_ascii_digit();
    (tag[name_len + 1] == b'$' && named_well).then_some(tag)
}

/// Splits a file into statements on `;` outside quotes, comments and `$tag$` bodies.
fn split_statements(text: &str) -> Vec<Chunk<'_>> {
    let bytes = text.as_bytes();
    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut state = Lex::Code;
    let mut i = 0usize;
    while i < bytes.len() {
        if state == Lex::Code && bytes[i] == b';' {
            let piece = &text[start..=i];
            if !piece.trim().is_empty() {
                chunks.push(Chunk {
                    text: piece,
                    offset: start,
                });
            }
            start = i + 1;
            i += 1;
            continue;
        }
        let (next_state, consumed) = state.step(&bytes[i..]);
        state = next_state;
        i += consumed;
    }
    let tail = &text[start..];
    if !tail.trim().is_empty() {
        chunks.push(Chunk {
            text: tail,
            offset: start,
        });
    }
    chunks
}

/// A string literal inside procedural SQL: its raw text and where it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Quoted<'a> {
    raw: &'a str,
    /// Byte offset of `raw` in the text that was scanned.
    start: usize,
    /// Single-quoted, so `''` in `raw` stands for one quote.
    single: bool,
}

impl Quoted<'_> {
    fn sql(&self) -> String {
        if self.single {
            self.raw.replace("''", "'")
        } else {
            self.raw.to_owned()
        }
    }
}

/// String literals in `text` that lead with a statement keyword, in text order.
///
/// Dollar-quoted bodies are searched again up to [`QUOTE_DEPTH`], so a
/// `format($view$ CREATE VIEW ... $view$, schema)` inside a `DO $$ ... $$` block is
/// found at its own position.
fn quoted_statements(text: &str) -> Vec<Quoted<'_>> {
    let mut found = Vec::new();
    collect_quoted(text, 0, 0, &mut found);
    found.sort_by_key(|q| q.start);
    found
        .into_iter()
        .filter(|q| leads_with_sql(q.raw))
        .collect()
}

fn collect_quoted<'a>(text: &'a str, base: usize, depth: u8, out: &mut Vec<Quoted<'a>>) {
    let bytes = text.as_bytes();
    let mut state = Lex::Code;
    let mut open = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let (next, consumed) = state.step(&bytes[i..]);
        match (state, next) {
            (Lex::Code, Lex::SingleQuoted | Lex::DollarQuoted(_)) => open = i + consumed,
            // `''` is an escaped quote inside the literal, not its end.
            (Lex::SingleQuoted, Lex::Code) if bytes.get(i + 1) == Some(&b'\'') => {
                i += 2;
                continue;
            }
            (Lex::SingleQuoted, Lex::Code) => out.push(Quoted {
                raw: &text[open..i],
                start: base + open,
                single: true,
            }),
            (Lex::DollarQuoted(_), Lex::Code) => {
                let body = &text[open..i];
                out.push(Quoted {
                    raw: body,
                    start: base + open,
                    single: false,
                });
                if depth + 1 < QUOTE_DEPTH {
                    collect_quoted(body, base + open, depth + 1, out);
                }
            }
            _ => {}
        }
        state = next;
        i += consumed;
    }
}

/// The byte span of a statement's chunk, trimmed of surrounding whitespace.
fn statement_bytes(chunk: Chunk<'_>) -> ByteSpan {
    trimmed_bytes(chunk.text, chunk.offset)
}

/// The byte span of `text`, which starts at `offset`, trimmed of surrounding whitespace.
fn trimmed_bytes(text: &str, offset: usize) -> ByteSpan {
    let leading = text.len() - text.trim_start().len();
    let start = offset + leading;
    let end = (offset + text.trim_end().len()).max(start);
    ByteSpan::new(start as u64, end as u64)
}

#[cfg(test)]
mod tests {
    use surveyor_model::{CommitId, RepoId, RepoPath};

    use super::*;

    fn unit(text: &str) -> SourceUnit {
        SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("m/001.sql"),
            CommitId::new("c"),
            text.into(),
        )
    }

    #[test]
    fn migration_yields_schema_tables_and_columns() {
        let sql = "CREATE SCHEMA cw_catalog;\n\nCREATE TABLE cw_catalog.asset (\n  id uuid PRIMARY KEY,\n  name text NOT NULL,\n  CONSTRAINT asset_name_uq UNIQUE (name)\n);\n\nCREATE INDEX asset_name_idx ON cw_catalog.asset (name);\nALTER TABLE cw_catalog.asset ADD COLUMN owner text;\n";
        let facts = SqlExtractor::new(SqlDialect::PostgreSql, "public")
            .extract(&unit(sql))
            .expect("parses");
        let decls: Vec<_> = facts.declarations().collect();
        assert!(
            decls
                .iter()
                .any(|d| d.kind == NodeKind::Schema && d.name == "cw_catalog")
        );
        let table = decls
            .iter()
            .find(|d| d.kind == NodeKind::Table)
            .expect("table");
        assert_eq!(table.name, "asset");
        assert_eq!(table.scope, vec!["cw_catalog".to_owned()]);
        assert_eq!(
            table.signature.as_deref(),
            Some("CREATE TABLE cw_catalog.asset (")
        );
        let cols: Vec<_> = decls
            .iter()
            .filter(|d| d.kind == NodeKind::Column)
            .collect();
        assert_eq!(cols.len(), 3);
        assert!(cols.iter().any(|c| c.name == "owner"));
        assert!(
            decls
                .iter()
                .any(|d| d.kind == NodeKind::Constraint && d.name == "asset_name_uq")
        );
        assert!(
            decls
                .iter()
                .any(|d| d.kind == NodeKind::Index && d.name == "asset_name_idx")
        );
        let u = unit(sql);
        assert!(u.slice(table.span.bytes).starts_with("CREATE TABLE"));
    }

    #[test]
    fn statement_summary_distinguishes_reads_and_writes() {
        let s = summarize_sql(
            "INSERT INTO cw.outbox (id) SELECT id FROM cw.events e JOIN cw.users u ON u.id = e.user_id",
            SqlDialect::PostgreSql,
        )
        .expect("parses");
        assert_eq!(s.kind, "INSERT");
        assert_eq!(s.writes, vec!["cw.outbox"]);
        assert_eq!(s.reads, vec!["cw.events", "cw.users"]);
        assert!(summarize_sql("not sql at all", SqlDialect::PostgreSql).is_none());
        let u = summarize_sql(
            "UPDATE t SET a = 1 WHERE id IN (SELECT id FROM s)",
            SqlDialect::Generic,
        )
        .expect("parses");
        assert_eq!(u.writes, vec!["t"]);
        assert_eq!(u.reads, vec!["s"]);
    }

    #[test]
    fn ctes_and_table_functions_are_not_tables() {
        let s = summarize_sql(
            "WITH recent AS (SELECT * FROM cw.events WHERE at > now() - interval '1 day'), \
             ranked AS (SELECT r.*, row_number() OVER () AS n FROM recent r) \
             SELECT u.id, t.value FROM ranked k \
             JOIN cw.users u ON u.id = k.user_id \
             CROSS JOIN jsonb_array_elements(k.tags) AS t \
             JOIN LATERAL unnest(u.roles) AS role ON true",
            SqlDialect::PostgreSql,
        )
        .expect("parses");
        assert_eq!(s.kind, "SELECT");
        assert_eq!(s.reads, vec!["cw.events", "cw.users"]);
        let w = summarize_sql(
            "WITH expired AS (SELECT id FROM cw.sessions WHERE expired) \
             INSERT INTO cw.audit (id) SELECT id FROM expired",
            SqlDialect::PostgreSql,
        )
        .expect("parses");
        assert_eq!(
            w.kind, "INSERT",
            "a WITH-prefixed insert is still an insert"
        );
        assert_eq!(w.writes, vec!["cw.audit"]);
        assert_eq!(w.reads, vec!["cw.sessions"]);
    }

    #[test]
    fn a_view_is_declared_and_reads_the_tables_its_query_names() {
        let sql = "CREATE OR REPLACE VIEW cw.pending AS\n  SELECT o.id, u.email FROM cw.outbox o JOIN cw.users u ON u.id = o.user_id\n  WHERE o.sent_at IS NULL;\n\
                   CREATE MATERIALIZED VIEW daily_counts AS WITH recent AS (SELECT * FROM events) SELECT count(*) FROM recent;";
        let facts = SqlExtractor::new(SqlDialect::PostgreSql, "public")
            .extract(&unit(sql))
            .expect("parses");
        let views: Vec<_> = facts
            .declarations()
            .filter(|d| d.kind == NodeKind::View)
            .collect();
        assert_eq!(views.len(), 2, "{views:?}");
        let pending = &views[0];
        assert_eq!(pending.name, "pending");
        assert_eq!(pending.scope, vec!["cw".to_owned()]);
        assert_eq!(
            pending.signature.as_deref(),
            Some("CREATE OR REPLACE VIEW cw.pending AS")
        );
        assert_eq!(
            pending.attributes.get("materialized").map(String::as_str),
            Some("false")
        );
        let daily = &views[1];
        assert_eq!(daily.scope, vec!["public".to_owned()]);
        assert_eq!(
            daily.attributes.get("materialized").map(String::as_str),
            Some("true")
        );
        let reads: Vec<(Vec<String>, String)> = facts
            .references()
            .filter(|r| r.role == ReferenceRole::Read)
            .map(|r| (r.path.clone(), r.rule.to_string()))
            .collect();
        let path = |s: &str, t: &str| vec![s.to_owned(), t.to_owned()];
        assert_eq!(
            reads,
            vec![
                (path("cw", "outbox"), "sql.view".to_owned()),
                (path("cw", "users"), "sql.view".to_owned()),
                (path("public", "events"), "sql.view".to_owned()),
            ],
            "a view reads its tables, never itself or its CTEs"
        );
        assert!(facts.references().all(|r| r.role != ReferenceRole::Write));
        let kind = summarize_sql(
            "CREATE MATERIALIZED VIEW m AS SELECT 1 FROM t",
            SqlDialect::PostgreSql,
        )
        .map(|s| s.kind);
        assert_eq!(kind.as_deref(), Some("CREATE MATERIALIZED VIEW"));
    }

    #[test]
    fn statements_executed_from_a_do_block_are_declared_at_their_literal() {
        let sql = "CREATE TABLE registry (id uuid);\n\
                   DO $$\n\
                   DECLARE\n    pure_schema TEXT := regexp_replace(current_schema(), '_pure$', '_catalog');\n\
                   BEGIN\n\
                   \x20   EXECUTE format('DROP VIEW IF EXISTS %I.pure_class_concepts_v1', pure_schema);\n\
                   \x20   EXECUTE format($view$\n\
                   \x20       CREATE OR REPLACE VIEW %I.pure_class_concepts_v1 AS\n\
                   \x20       SELECT cm.class_element_id, 'it''s' AS note\n\
                   \x20       FROM %I.pure_class_mappings AS cm\n\
                   \x20       JOIN %I.concept_dataset_map AS m ON m.entity_id = cm.dataset_entity_id\n\
                   \x20       JOIN cw.registry r ON r.id = m.entity_id\n\
                   \x20   $view$, pure_schema, pure_schema, catalog_schema);\n\
                   \x20   EXECUTE format($table$ CREATE TABLE %I.audit (id uuid, note text) $table$, pure_schema);\n\
                   \x20   EXECUTE format($q$ INSERT INTO %I SELECT 1 $q$, some_table);\n\
                   \x20   EXECUTE 'CREATE OR REPLACE VIEW export_v1 AS ' || view_sql;\n\
                   END $$;\n";
        let u = unit(sql);
        let facts = SqlExtractor::new(SqlDialect::PostgreSql, "public")
            .extract(&u)
            .expect("extracts");
        let decls: Vec<_> = facts.declarations().collect();
        let view = decls
            .iter()
            .find(|d| d.kind == NodeKind::View)
            .expect("the view the block creates");
        assert_eq!(view.name, "pure_class_concepts_v1");
        assert_eq!(
            view.scope,
            vec!["public".to_owned()],
            "a templated schema falls back to the default"
        );
        assert_eq!(
            view.attributes.get("dynamic").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            view.signature.as_deref(),
            Some("CREATE OR REPLACE VIEW %I.pure_class_concepts_v1 AS")
        );
        assert!(
            u.slice(view.span.bytes)
                .starts_with("CREATE OR REPLACE VIEW %I.pure")
        );
        assert!(u.slice(view.span.bytes).ends_with("r.id = m.entity_id"));
        let table = decls
            .iter()
            .find(|d| d.kind == NodeKind::Table && d.name == "audit")
            .expect("the table the block creates");
        assert_eq!(
            table.attributes.get("dynamic").map(String::as_str),
            Some("true")
        );
        assert!(
            decls
                .iter()
                .any(|d| d.kind == NodeKind::Column && d.name == "note")
        );
        assert!(
            decls.iter().all(|d| d.name != "export_v1"),
            "a literal that is only the head of a statement declares nothing"
        );
        assert_eq!(
            decls[0].attributes.get("sql.unparsed"),
            None,
            "a block that yielded statements is not counted as unparsed"
        );
        let refs: Vec<(ReferenceRole, Vec<String>)> = facts
            .references()
            .map(|r| (r.role.clone(), r.path.clone()))
            .collect();
        let bare = |t: &str| vec![t.to_owned()];
        assert_eq!(
            refs,
            vec![
                (ReferenceRole::Write, bare("pure_class_concepts_v1")),
                (ReferenceRole::Read, bare("pure_class_mappings")),
                (ReferenceRole::Read, bare("concept_dataset_map")),
                (
                    ReferenceRole::Read,
                    vec!["cw".to_owned(), "registry".to_owned()]
                ),
            ],
            "a templated qualifier leaves the bare name to the linker; a templated table names nothing"
        );
    }

    #[test]
    fn tagged_dollar_quotes_hold_a_block_together() {
        let sql = "DO $do$ BEGIN EXECUTE format($v$ CREATE VIEW %1$I.e_v1 AS SELECT a FROM %2$I.e; $v$, g, c); END $do$;\nSELECT 1 FROM t2;";
        let chunks = split_statements(sql);
        assert_eq!(chunks.len(), 2, "{chunks:?}");
        assert!(chunks[0].text.ends_with("END $do$;"));
        let facts = SqlExtractor::new(SqlDialect::PostgreSql, "public")
            .extract(&unit(sql))
            .expect("extracts");
        let view = facts
            .declarations()
            .find(|d| d.kind == NodeKind::View)
            .expect("view");
        assert_eq!(view.name, "e_v1");
        let reads: Vec<Vec<String>> = facts
            .references()
            .filter(|r| r.role == ReferenceRole::Read)
            .map(|r| r.path.clone())
            .collect();
        assert_eq!(
            reads,
            vec![
                vec!["e".to_owned()],
                vec!["public".to_owned(), "t2".to_owned()]
            ]
        );
        assert_eq!(dollar_tag(b"$1, $2"), None);
        assert_eq!(dollar_tag(b"$$x"), Some(&b"$$"[..]));
        assert_eq!(dollar_tag(b"$fn_1$ body"), Some(&b"$fn_1$"[..]));
        assert_eq!(dollar_tag(b"$I.name"), None);
    }

    #[test]
    fn dropping_a_relation_is_a_write() {
        let dropped =
            summarize_sql("DROP TABLE IF EXISTS a, cw.b", SqlDialect::PostgreSql).expect("parses");
        assert_eq!(dropped.kind, "DROP");
        assert_eq!(dropped.writes, vec!["a", "cw.b"]);
        assert!(dropped.reads.is_empty());
        let view = summarize_sql("DROP VIEW v", SqlDialect::PostgreSql).expect("parses");
        assert_eq!(view.writes, vec!["v"]);
        let index = summarize_sql("DROP INDEX i", SqlDialect::PostgreSql).expect("parses");
        assert!(
            index.reads.is_empty() && index.writes.is_empty(),
            "{index:?}"
        );
    }

    #[test]
    fn unparsable_statements_are_skipped_not_fatal() {
        let sql = "\\connect db;\nCREATE TABLE a (id int);\nDO $$ BEGIN RAISE NOTICE 'x;y'; END $$;\nSELECT * FROM a;";
        let facts = SqlExtractor::new(SqlDialect::PostgreSql, "public")
            .extract(&unit(sql))
            .expect("extracts");
        let table = facts
            .declarations()
            .find(|d| d.kind == NodeKind::Table)
            .expect("table");
        assert_eq!(
            table.attributes.get("sql.unparsed").map(String::as_str),
            Some("2")
        );
        assert!(
            facts
                .references()
                .any(|r| r.role == ReferenceRole::Read && r.name == "a")
        );
    }
}
