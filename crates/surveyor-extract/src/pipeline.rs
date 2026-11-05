//! Running the extractors over a repository and enriching their facts.
//!
//! [`extract_repository`] builds the default extractor set for a profile, reads every
//! unit, runs the extractors in parallel with `rayon`, and turns tagged SQL literals
//! into `Query` declarations with table references through `sqlparser`.

use std::collections::BTreeMap;
use std::path::Path;

use itertools::Itertools;
use rayon::prelude::*;
use surveyor_model::{
    CommitId, ContentHash, DeclarationFact, ExtractorId, Fact, FactSet, Language, LiteralFact,
    LiteralRole, NodeKind, ReferenceFact, ReferenceRole, RepoPath, RuleId, SourceUnit,
};

use crate::profile::SqlDialect;
use crate::sql::{StatementSummary, summarize_sql};
use crate::template::{SQL_LEADS, is_templated, substitute_placeholders, table_name};
use crate::{
    ExtractError, Extractors, ManifestExtractor, MarkdownExtractor, MiseExtractor, Profile,
    Repository, RhaiExtractor, ScipExtractor, SqlExtractor, TreeSitterExtractor,
};

/// One file the walk visited, whether or not it produced facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRecord {
    /// Repository-relative path.
    pub path: RepoPath,
    /// Language, if detected.
    pub language: Option<Language>,
    /// Hash of the file content.
    pub content_hash: ContentHash,
    /// Size in bytes.
    pub bytes: u64,
    /// Number of lines, as an editor counts them (see [`line_count`]).
    pub lines: u64,
    /// Extractors that produced facts for the file.
    pub extractors: Vec<ExtractorId>,
}

/// Everything extraction produced for a repository.
#[derive(Debug, Default)]
pub struct Extraction {
    /// Commit the files were read at.
    pub commit: Option<CommitId>,
    /// Files visited.
    pub units: Vec<UnitRecord>,
    /// Facts per extractor per file.
    pub fact_sets: Vec<FactSet>,
    /// Failures, which never abort the run.
    pub errors: Vec<ExtractError>,
}

impl Extraction {
    /// Number of facts across all fact sets.
    #[must_use]
    pub fn fact_count(&self) -> usize {
        self.fact_sets.iter().map(|s| s.facts.len()).sum()
    }

    fn merge(mut self, other: Self) -> Self {
        self.units.extend(other.units);
        self.fact_sets.extend(other.fact_sets);
        self.errors.extend(other.errors);
        self
    }
}

/// Builds the default extractor set for a profile.
pub fn default_extractors(profile: &Profile) -> Result<Extractors, ExtractError> {
    Ok(Extractors::new()
        .with(TreeSitterExtractor::try_new()?)
        .with(SqlExtractor::new(
            profile.sql.dialect,
            profile.sql.default_schema.clone(),
        ))
        .with(RhaiExtractor::new())
        .with(ManifestExtractor::new())
        .with(MiseExtractor::new())
        .with(MarkdownExtractor::new(
            profile.design_set()?,
            profile.runbook_set()?,
        )))
}

/// Walks `repo` and extracts facts from every unit.
///
/// # Errors
/// Returns an error when the profile or the walk itself fails; per-file failures are
/// collected in [`Extraction::errors`].
pub fn extract_repository(
    repo: &Repository,
    profile: &Profile,
) -> Result<Extraction, ExtractError> {
    extract_repository_with(repo, profile, &[] as &[&Path])
}

/// Like [`extract_repository`], with the precise tier from the SCIP indexes at `scip`.
///
/// # Errors
/// Returns an error when an index cannot be read, besides the failures of
/// [`extract_repository`].
pub fn extract_repository_with(
    repo: &Repository,
    profile: &Profile,
    scip: &[impl AsRef<Path>],
) -> Result<Extraction, ExtractError> {
    let extractors = extractors_with(profile, repo.root(), scip)?;
    let units = repo.units(profile)?;
    Ok(Extraction {
        commit: Some(repo.commit().clone()),
        ..extract_units(&extractors, &units, profile.sql.dialect)
    })
}

/// The extractors for `profile`, with the precise tier from the SCIP indexes at
/// `scip` re-rooted under `repo_root`. Built once per run: an index loads in
/// milliseconds, but an update extracts twice.
///
/// # Errors
/// Returns an error when the profile selects an unknown language pack or an index
/// cannot be read.
pub fn extractors_with(
    profile: &Profile,
    repo_root: &Path,
    scip: &[impl AsRef<Path>],
) -> Result<Extractors, ExtractError> {
    let mut extractors = default_extractors(profile)?;
    if !scip.is_empty() {
        extractors.push(Box::new(ScipExtractor::load_all(scip, Some(repo_root))?));
    }
    Ok(extractors)
}

/// Reads `paths` from `repo` and extracts facts from them, for incremental update.
///
/// Paths that are missing or binary are skipped without error.
///
/// # Errors
/// Returns an error when the profile selects an unknown language pack or a path
/// cannot be read.
pub fn extract_paths<'a>(
    repo: &Repository,
    profile: &Profile,
    paths: impl IntoIterator<Item = &'a RepoPath>,
) -> Result<Extraction, ExtractError> {
    extract_paths_using(&default_extractors(profile)?, repo, profile, paths)
}

/// Like [`extract_paths`], with `extractors` from [`extractors_with`], so that an
/// incremental update keeps the precise tier for the files it re-extracts.
///
/// # Errors
/// Returns an error when a path cannot be read.
pub fn extract_paths_using<'a>(
    extractors: &Extractors,
    repo: &Repository,
    profile: &Profile,
    paths: impl IntoIterator<Item = &'a RepoPath>,
) -> Result<Extraction, ExtractError> {
    let units = paths
        .into_iter()
        .filter(|p| repo.root().join(p.as_str()).is_file())
        .map(|p| repo.unit(p))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Extraction {
        commit: Some(repo.commit().clone()),
        ..extract_units(extractors, &units, profile.sql.dialect)
    })
}

/// Runs `extractors` over `units` in parallel and enriches SQL literals.
#[must_use]
pub fn extract_units(
    extractors: &Extractors,
    units: &[SourceUnit],
    dialect: SqlDialect,
) -> Extraction {
    units
        .par_iter()
        .map(|unit| extract_unit(extractors, unit, dialect))
        .reduce(Extraction::default, Extraction::merge)
}

fn extract_unit(extractors: &Extractors, unit: &SourceUnit, dialect: SqlDialect) -> Extraction {
    let extracted = extractors.run(unit);
    let fact_sets: Vec<FactSet> = extracted
        .facts
        .into_iter()
        .map(|set| FactSet {
            facts: enrich_sql_literals(set.facts, dialect),
            ..set
        })
        .collect();
    let record = UnitRecord {
        path: unit.path.clone(),
        language: unit.language.clone(),
        content_hash: unit.content_hash,
        bytes: unit.content.len() as u64,
        lines: line_count(&unit.content),
        extractors: fact_sets.iter().map(|s| s.extractor.clone()).collect(),
    };
    Extraction {
        commit: None,
        units: vec![record],
        fact_sets,
        errors: extracted.errors,
    }
}

/// Number of lines in `content`, as an editor counts them.
///
/// Every newline ends a line and a final unterminated line counts too, so
/// `"a\nb\n"` and `"a\nb"` both have two lines and an empty file has none. The
/// count is over bytes, so a binary file gets the number of newline bytes it
/// happens to contain, which is as meaningful as its line count can be.
#[must_use]
pub fn line_count(content: &[u8]) -> u64 {
    if content.is_empty() {
        return 0;
    }
    // Splitting on newlines yields one segment per line plus an empty tail when the
    // last line is terminated; the tail is not a line.
    let segments = content.split(|&b| b == b'\n').count();
    (segments - usize::from(content.ends_with(b"\n"))) as u64
}

/// Replaces each `Sql` literal that parses with a `Query` declaration plus table
/// references; every other fact passes through unchanged.
fn enrich_sql_literals(facts: Vec<Fact>, dialect: SqlDialect) -> Vec<Fact> {
    facts
        .into_iter()
        .flat_map(|fact| match fact {
            Fact::Literal(lit) if lit.role == LiteralRole::Sql => {
                match parse_literal(&lit, dialect) {
                    Some(summary) => query_facts(&lit, &summary),
                    None => vec![Fact::Literal(lit)],
                }
            }
            other => vec![other],
        })
        .collect()
}

/// Parses an SQL literal, standing in for `{placeholders}`; `None` when it is not SQL.
fn parse_literal(lit: &LiteralFact, dialect: SqlDialect) -> Option<StatementSummary> {
    let lead = lit
        .value
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_uppercase();
    if !SQL_LEADS.contains(&lead.as_str()) {
        return None;
    }
    let parsable = if is_templated(&lit.value) {
        substitute_placeholders(&lit.value)
    } else {
        lit.value.clone()
    };
    summarize_sql(&parsable, dialect).map(|summary| StatementSummary {
        kind: summary.kind,
        reads: summary
            .reads
            .iter()
            .filter_map(|t| table_name(t))
            .unique()
            .collect(),
        writes: summary
            .writes
            .iter()
            .filter_map(|t| table_name(t))
            .unique()
            .collect(),
    })
}

/// The `Query` declaration and its table references for one parsed literal.
fn query_facts(lit: &LiteralFact, summary: &StatementSummary) -> Vec<Fact> {
    let fingerprint = ContentHash::of(normalize_sql(&lit.value)).short();
    let name = format!("{} #{}", summary.kind.to_lowercase(), &fingerprint[..8]);
    let attributes: BTreeMap<String, String> = [
        ("statement".to_owned(), summary.kind.clone()),
        ("fingerprint".to_owned(), fingerprint),
        ("sql".to_owned(), lit.value.clone()),
    ]
    .into_iter()
    .chain(is_templated(&lit.value).then(|| ("templated".to_owned(), "true".to_owned())))
    .collect();
    let query_scope: Vec<String> = lit
        .scope
        .iter()
        .cloned()
        .chain(std::iter::once(name.clone()))
        .collect();
    let declaration = Fact::Declaration(DeclarationFact {
        kind: NodeKind::Query,
        name,
        scope: lit.scope.clone(),
        span: lit.span,
        name_span: lit.span.bytes,
        signature: Some(first_line(&lit.value)),
        doc: None,
        rule: RuleId::new("sql.literal"),
        attributes,
        hash: ContentHash::of(&lit.value),
    });
    let references = [
        (ReferenceRole::Read, &summary.reads),
        (ReferenceRole::Write, &summary.writes),
    ]
    .into_iter()
    .flat_map(|(role, tables)| tables.iter().map(move |table| (role.clone(), table)))
    .map(|(role, table)| {
        let path: Vec<String> = table.split('.').map(str::to_owned).collect();
        Fact::Reference(ReferenceFact {
            role,
            name: path.last().cloned().unwrap_or_else(|| table.clone()),
            path,
            span: lit.span,
            scope: query_scope.clone(),
            rule: RuleId::new("sql.literal"),
            hash: ContentHash::of(&lit.value),
            receiver: None,
            binding: None,
        })
    });
    std::iter::once(declaration).chain(references).collect()
}

fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn first_line(text: &str) -> String {
    let line = text.trim().lines().next().unwrap_or_default().trim();
    match line.char_indices().nth(160) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use surveyor_model::RepoId;

    use super::*;

    #[test]
    fn line_count_counts_as_an_editor_does() {
        assert_eq!(line_count(b""), 0);
        assert_eq!(line_count(b"\n"), 1);
        assert_eq!(line_count(b"a\nb\n"), 2);
        assert_eq!(line_count(b"a\nb"), 2);
        assert_eq!(line_count(b"a\r\nb\r\n"), 2);
        assert_eq!(line_count(b"a\n\n\n"), 3);
    }

    #[test]
    fn unit_records_carry_bytes_and_lines() {
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("a.rs"),
            CommitId::new("c"),
            "fn a() {}\nfn b() {}\n".into(),
        );
        let extractors = Extractors::new().with(TreeSitterExtractor::new());
        let extraction = extract_units(&extractors, &[unit], SqlDialect::PostgreSql);
        let record = &extraction.units[0];
        assert_eq!(record.bytes, 20);
        assert_eq!(record.lines, 2);
    }

    #[test]
    fn extractors_with_an_index_keep_the_precise_tier_for_re_extracted_paths() {
        use protobuf::Message;
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("dirs");
        let wide = "// 0123456789012345678901234567890\n".repeat(3);
        std::fs::write(dir.path().join("src/relay.rs"), &wide).expect("file");
        std::fs::write(dir.path().join("src/store.rs"), &wide).expect("file");
        let index_path = dir.path().join("index.scip");
        let bytes = crate::scip_index::tests::index()
            .write_to_bytes()
            .expect("bytes");
        std::fs::write(&index_path, bytes).expect("index");
        let profile = Profile::default();
        let repo = Repository::open_with(dir.path(), None, &profile).expect("repo");
        let paths = [RepoPath::new("src/relay.rs")];
        let name_only = extract_paths(&repo, &profile, &paths).expect("extract");
        assert!(
            name_only
                .fact_sets
                .iter()
                .all(|s| s.extractor.as_str() != "scip"),
            "without indexes only the syntactic tier runs"
        );
        // The index was built at `/repo`, another checkout: it re-roots where its
        // documents exist, and the re-extracted path keeps its precise facts.
        let extractors = extractors_with(&profile, dir.path(), &[&index_path]).expect("extractors");
        let precise = extract_paths_using(&extractors, &repo, &profile, &paths).expect("extract");
        assert!(
            precise
                .fact_sets
                .iter()
                .any(|s| s.extractor.as_str() == "scip" && s.path.as_str() == "src/relay.rs"),
            "{:?}",
            precise
                .fact_sets
                .iter()
                .map(|s| (s.extractor.to_string(), s.path.to_string()))
                .collect::<Vec<_>>()
        );
    }

    fn extract(src: &str) -> FactSet {
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("a.rs"),
            CommitId::new("c"),
            src.into(),
        );
        let extractors = Extractors::new().with(TreeSitterExtractor::new());
        let mut extraction = extract_units(
            &extractors,
            std::slice::from_ref(&unit),
            SqlDialect::PostgreSql,
        );
        assert!(extraction.errors.is_empty());
        assert_eq!(extraction.units[0].extractors.len(), 1);
        extraction.fact_sets.remove(0)
    }

    #[test]
    fn sql_literals_become_queries_with_table_references() {
        let set = extract(
            r#"async fn load(pool: &PgPool) {
    sqlx::query!("SELECT id FROM cw_catalog.asset WHERE owner = $1").fetch_all(pool).await;
    sqlx::query("not sql").execute(pool).await;
}"#,
        );
        let query = set
            .declarations()
            .find(|d| d.kind == NodeKind::Query)
            .expect("query");
        assert_eq!(query.scope, vec!["load".to_owned()]);
        assert_eq!(
            query.attributes.get("statement").map(String::as_str),
            Some("SELECT")
        );
        let read = set
            .references()
            .find(|r| r.role == ReferenceRole::Read)
            .expect("read");
        assert_eq!(read.path, vec!["cw_catalog", "asset"]);
        assert_eq!(read.scope.len(), 2);
        assert_eq!(
            set.literals().count(),
            1,
            "the non-SQL literal is kept as a literal"
        );
    }

    #[test]
    fn bound_and_templated_sql_is_recognized() {
        let set = extract(
            r#"fn f(table: &str) {
    let sql = "DELETE FROM cw.events WHERE id = $1";
    let greeting = "hello";
    let q = format!("UPDATE {table} SET seen = true WHERE id IN (SELECT id FROM cw.users)");
    sqlx::query(&sql);
}"#,
        );
        let queries: Vec<_> = set
            .declarations()
            .filter(|d| d.kind == NodeKind::Query)
            .collect();
        assert_eq!(queries.len(), 2, "{queries:?}");
        assert!(
            queries
                .iter()
                .any(|q| q.attributes.get("statement").map(String::as_str) == Some("DELETE"))
        );
        let update = queries
            .iter()
            .find(|q| q.attributes.get("statement").map(String::as_str) == Some("UPDATE"))
            .expect("update");
        assert_eq!(
            update.attributes.get("templated").map(String::as_str),
            Some("true")
        );
        let reads: Vec<_> = set
            .references()
            .filter(|r| r.role == ReferenceRole::Read)
            .map(|r| r.path.join("."))
            .collect();
        assert!(reads.contains(&"cw.users".to_owned()), "{reads:?}");
        assert!(set.references().all(|r| !r.name.contains("__p")));
        assert_eq!(
            set.literals().filter(|l| l.value == "hello").count(),
            1,
            "non-SQL strings stay literals"
        );
        assert_eq!(
            substitute_placeholders("UPDATE {t} SET a = {{x}}"),
            "UPDATE __p SET a = __p"
        );
    }

    #[test]
    fn a_placeholder_schema_leaves_the_table_name_to_the_linker() {
        assert_eq!(table_name("cw.users"), Some("cw.users".to_owned()));
        assert_eq!(table_name("__p.users"), Some("users".to_owned()));
        assert_eq!(table_name("__p.__p.users"), Some("users".to_owned()));
        assert_eq!(table_name("__p_suffix.users"), Some("users".to_owned()));
        assert_eq!(table_name("__p"), None);
        assert_eq!(table_name("__p.__p"), None);
        assert_eq!(table_name("cw.__p"), None);
        assert_eq!(table_name("__p_events"), None);

        let set = extract(
            r#"fn f(schema: &str, table: &str) {
    let a = format!("INSERT INTO {schema}.proposals (id) VALUES ($1)");
    let b = format!("SELECT id FROM {schema}.{table} WHERE id = $1");
    let c = format!("DELETE FROM {schema}.proposals p USING {schema}.decisions d WHERE d.id = p.id");
}"#,
        );
        let mut refs: Vec<(ReferenceRole, String)> = set
            .references()
            .filter(|r| matches!(r.role, ReferenceRole::Read | ReferenceRole::Write))
            .map(|r| (r.role.clone(), r.path.join(".")))
            .collect();
        refs.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then(format!("{:?}", a.0).cmp(&format!("{:?}", b.0)))
        });
        assert_eq!(
            refs,
            vec![
                (ReferenceRole::Read, "decisions".to_owned()),
                (ReferenceRole::Write, "proposals".to_owned()),
                (ReferenceRole::Write, "proposals".to_owned()),
            ],
            "the fully templated query keeps no reference; the rest name bare tables"
        );
        assert_eq!(
            set.declarations()
                .filter(|d| d.kind == NodeKind::Query)
                .count(),
            3
        );
    }

    #[test]
    fn a_query_continued_across_lines_still_parses() {
        let set = extract(
            "fn propose(schema: &str) {\n    let sql = format!(\n        \"INSERT INTO {schema}.taxonomy_placement_proposals \\\n            (taxonomy_path, container_id) \\\n         VALUES ($1, $2) RETURNING proposal_id\"\n    );\n    sqlx::query_as(&sql);\n}\n",
        );
        let query = set
            .declarations()
            .find(|d| d.kind == NodeKind::Query)
            .expect("the continued literal parses as one statement");
        assert_eq!(
            query.attributes.get("statement").map(String::as_str),
            Some("INSERT")
        );
        let write = set
            .references()
            .find(|r| r.role == ReferenceRole::Write)
            .expect("write");
        assert_eq!(write.path, vec!["taxonomy_placement_proposals"]);
    }
}
