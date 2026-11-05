//! Templated SQL: text that becomes a statement only once a program fills it in.
//!
//! Two template dialects reach the extractor: Rust `format!` strings in code
//! (`"DELETE FROM {schema}.events"`) and Postgres `format()` strings inside
//! procedural blocks (`format($v$ CREATE VIEW %I.events_v1 ... $v$, schema)`). Both
//! are parsed with a neutral identifier in place of each placeholder, and a name that
//! still contains it afterwards is read by [`table_name`]: a templated qualifier leaves
//! the table known, a templated table leaves nothing to resolve.

/// Identifier that stands in for a placeholder while parsing templated SQL.
pub(crate) const PLACEHOLDER: &str = "__p";

/// Statement kinds a piece of text must start with to count as SQL.
pub(crate) const SQL_LEADS: &[&str] = &[
    "SELECT", "INSERT", "UPDATE", "DELETE", "WITH", "CREATE", "ALTER", "DROP", "TRUNCATE", "MERGE",
    "COPY", "LOCK", "SET", "BEGIN", "COMMIT", "EXPLAIN", "REFRESH", "VALUES",
];

/// True when the text's first word is a statement keyword.
pub(crate) fn leads_with_sql(text: &str) -> bool {
    let lead = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_uppercase();
    SQL_LEADS.contains(&lead.as_str())
}

/// True when the text carries Rust-style `{placeholders}`.
pub(crate) fn is_templated(sql: &str) -> bool {
    sql.contains('{') && sql.contains('}')
}

/// The table a parsed name refers to once its placeholders are accounted for.
///
/// A placeholder in a qualifier (`{schema}.events`) leaves the table name itself
/// known, so the qualifier is dropped and the linker resolves the bare name across
/// schemas. A placeholder in the table position (`{table}`, `{prefix}_events`) leaves
/// nothing to resolve, so the name is `None`.
pub(crate) fn table_name(parsed: &str) -> Option<String> {
    let parts: Vec<&str> = parsed
        .split('.')
        .filter(|part| !part.contains(PLACEHOLDER))
        .collect();
    let known_table = parsed
        .rsplit('.')
        .next()
        .is_some_and(|table| !table.contains(PLACEHOLDER));
    (known_table && !parts.is_empty()).then(|| parts.join("."))
}

/// Replaces `{...}` interpolation sites with the neutral identifier.
pub(crate) fn substitute_placeholders(sql: &str) -> String {
    let (out, _) = sql.chars().fold(
        (String::with_capacity(sql.len()), 0usize),
        |(mut out, depth), c| match (c, depth) {
            ('{', 0) => {
                out.push_str(PLACEHOLDER);
                (out, 1)
            }
            ('{', d) => (out, d + 1),
            ('}', d) if d > 0 => (out, d - 1),
            (c, 0) => {
                out.push(c);
                (out, 0)
            }
            (_, d) => (out, d),
        },
    );
    out
}

/// Replaces Postgres `format()` specifiers (`%I`, `%L`, `%s`, `%2$I`) with the
/// neutral identifier and `%%` with a percent sign.
pub(crate) fn substitute_format(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut rest = sql;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let position = after.trim_start_matches(|c: char| c.is_ascii_digit());
        let position = position
            .strip_prefix('$')
            .filter(|_| position.len() < after.len());
        let spec = position.unwrap_or(after);
        rest = match spec.chars().next() {
            Some('I' | 'L' | 's') => {
                out.push_str(PLACEHOLDER);
                &spec[1..]
            }
            Some('%') if position.is_none() => {
                out.push('%');
                &after[1..]
            }
            _ => {
                out.push('%');
                after
            }
        };
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_specifiers_become_the_placeholder() {
        assert_eq!(
            substitute_format("CREATE VIEW %I.x AS SELECT %L, %2$s FROM %1$I.t WHERE p = 100%%"),
            "CREATE VIEW __p.x AS SELECT __p, __p FROM __p.t WHERE p = 100%"
        );
        assert_eq!(substitute_format("50% off"), "50% off");
        assert_eq!(substitute_format("%"), "%");
        assert_eq!(substitute_format("%3$"), "%3$");
    }

    #[test]
    fn a_lead_keyword_marks_sql() {
        assert!(leads_with_sql("\n  CREATE OR REPLACE VIEW v AS SELECT 1"));
        assert!(leads_with_sql("select 1"));
        assert!(!leads_with_sql("DECLARE s TEXT;"));
        assert!(!leads_with_sql(""));
    }
}
