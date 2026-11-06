//! The identifiers a task text names: what an agent would look up before reading.

/// File extensions that mark a token as a path to source.
const SOURCE_EXTENSIONS: &[&str] = &[
    ".rs", ".ts", ".tsx", ".js", ".py", ".sql", ".toml", ".rhai", ".yml", ".yaml", ".md", ".proto",
    ".sh", ".json",
];

/// Characters that may wrap or trail a token in prose without being part of it.
const TRIM: &[char] = &[
    '"', '\'', '(', ')', '[', ']', '{', '}', ',', ';', ':', '.', '!', '?', '*', '<', '>',
];

/// The identifier-like terms of `text`, in order of first appearance, without repeats.
///
/// A term counts when it is written in backticks, carries a scope (`a::b`), an
/// underscore, a path separator or a source extension, reads as CamelCase, or joins
/// two names with a dot (`dataset.load`). Plain words, all-caps abbreviations and
/// numbers are left to the text ranking.
#[must_use]
pub fn identifiers_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |term: &str| {
        let term = term.trim_matches(TRIM);
        let term = term.strip_suffix("'s").unwrap_or(term);
        if term.len() >= 2 && !out.iter().any(|t| t == term) {
            out.push(term.to_owned());
        }
    };
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('`') else {
            break;
        };
        for term in after[..end].split_whitespace() {
            push(term);
        }
        rest = &after[end + 1..];
    }
    for raw in text.split(|c: char| c.is_whitespace() || c == '=' || c == '`') {
        let term = raw.trim_matches(TRIM);
        let term = term.strip_suffix("'s").unwrap_or(term);
        if looks_like_identifier(term) {
            push(term);
        }
    }
    out
}

fn looks_like_identifier(term: &str) -> bool {
    if term.len() < 3 || term.chars().all(|c| !c.is_alphabetic()) {
        return false;
    }
    if term.contains("::") || term.contains('/') || term.contains('_') {
        return true;
    }
    // A hyphenated name of three parts or more, or two parts one of which is not a
    // word a sentence would hyphenate (`causeway-db-core`, `rpc-access`, `search-relay`
    // resolve to packages; `slow-consumer` resolves to nothing and costs a lookup).
    if term.matches('-').count() >= 1
        && term
            .split('-')
            .all(|part| part.len() >= 2 && part.chars().all(char::is_alphanumeric))
    {
        return true;
    }
    if SOURCE_EXTENSIONS.iter().any(|ext| term.ends_with(ext)) {
        return true;
    }
    if camel_case(term) {
        return true;
    }
    term.split_once('.').is_some_and(|(a, b)| {
        a.len() >= 2
            && b.len() >= 2
            && a.chars().all(char::is_alphanumeric)
            && b.chars().all(char::is_alphanumeric)
            && a.chars().any(char::is_alphabetic)
            && b.chars().any(char::is_alphabetic)
    })
}

/// `BrowserBus`, `dataset`, `PortListStuckWork`: a lowercase letter and an uppercase
/// one after the first character, and no character that is not a letter or digit.
fn camel_case(term: &str) -> bool {
    term.chars().all(char::is_alphanumeric)
        && term.chars().any(char::is_lowercase)
        && term.chars().skip(1).any(char::is_uppercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_terms_an_agent_would_look_up() {
        let text = "Make the browser bus (`BrowserBus`) hold `dataset.load` until the \
                    codec registers; see torrentfx/web/framework/app/src/index.ts and \
                    causeway_db_core's init, SameSite=Lax, HTTP 403, PortListStuckWork, \
                    /app/tasks, and the migrate step.";
        assert_eq!(
            identifiers_in(text),
            vec![
                "BrowserBus",
                "dataset.load",
                "torrentfx/web/framework/app/src/index.ts",
                "causeway_db_core",
                "SameSite",
                "PortListStuckWork",
                "/app/tasks",
            ]
        );
        assert_eq!(
            identifiers_in("the causeway-db-core's crate and @causeway/rpc-access"),
            vec!["causeway-db-core", "@causeway/rpc-access"]
        );
    }

    #[test]
    fn plain_words_abbreviations_and_numbers_are_not_identifiers() {
        assert!(identifiers_in("fix the TLS and DSN handling in 2026, e.g. now").is_empty());
        assert_eq!(identifiers_in("`fn` `x`"), vec!["fn"]);
    }
}
