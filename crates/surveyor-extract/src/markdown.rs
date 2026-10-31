//! Design documents and runbooks from Markdown.
//!
//! A Markdown file under a design or runbook root becomes a `Design` or `Runbook`
//! declaration named by its first heading. Path-like tokens in the text (inline code
//! or bare `dir/file.ext` words) become `Path` literals, which the linker turns into
//! `DESIGNED_BY` and `DOCUMENTED_BY` edges from the files they name.

use std::collections::BTreeMap;

use globset::GlobSet;
use surveyor_model::{
    ByteSpan, ContentHash, DeclarationFact, ExtractorId, Fact, FactSet, Language, LiteralFact,
    LiteralRole, NodeKind, RuleId, SourceUnit,
};

use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "markdown";

/// Reads design documents and runbooks.
#[derive(Debug, Clone)]
pub struct MarkdownExtractor {
    designs: GlobSet,
    runbooks: GlobSet,
}

impl MarkdownExtractor {
    /// Creates an extractor that classifies files by the given glob sets.
    #[must_use]
    pub fn new(designs: GlobSet, runbooks: GlobSet) -> Self {
        Self { designs, runbooks }
    }

    fn kind_for(&self, unit: &SourceUnit) -> Option<NodeKind> {
        let path = unit.path.as_str();
        if self.runbooks.is_match(path) {
            Some(NodeKind::Runbook)
        } else if self.designs.is_match(path) {
            Some(NodeKind::Design)
        } else {
            None
        }
    }

    /// Extracts facts from a Markdown unit.
    ///
    /// # Errors
    /// Returns a parse error when the file is not UTF-8.
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("Markdown is not UTF-8"))?;
        let mut set = FactSet::empty(ExtractorId::new(EXTRACTOR_ID), unit);
        let Some(kind) = self.kind_for(unit) else {
            return Ok(set);
        };

        let (title, title_span) = first_heading(text).unwrap_or_else(|| {
            let stem = unit
                .path
                .file_name()
                .and_then(|n| n.rsplit_once('.'))
                .map_or("document", |(s, _)| s);
            (stem.to_owned(), ByteSpan::new(0, 0))
        });
        let mut attributes = BTreeMap::new();
        attributes.insert("path".to_owned(), unit.path.to_string());
        if let Some(uid) =
            front_matter_value(text, "uid").or_else(|| front_matter_value(text, "id"))
        {
            attributes.insert("uid".to_owned(), uid);
        }
        if let Some(state) =
            front_matter_value(text, "state").or_else(|| front_matter_value(text, "status"))
        {
            attributes.insert("state".to_owned(), state);
        }
        let whole = ByteSpan::new(0, unit.content.len() as u64);
        set.facts.push(Fact::Declaration(DeclarationFact {
            kind,
            name: title.clone(),
            scope: Vec::new(),
            span: unit.text_span(whole),
            name_span: title_span,
            signature: Some(title),
            doc: summary(text),
            rule: RuleId::new("markdown.document"),
            attributes,
            hash: unit.content_hash,
        }));

        set.facts
            .extend(path_tokens(text).into_iter().map(|(value, bytes)| {
                Fact::Literal(LiteralFact {
                    role: LiteralRole::Path,
                    hash: ContentHash::of(&value),
                    value,
                    span: unit.text_span(bytes),
                    scope: Vec::new(),
                    rule: RuleId::new("markdown.path"),
                })
            }));
        Ok(set)
    }
}

impl Extractor for MarkdownExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_ID)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        unit.language == Some(Language::Markdown) && self.kind_for(unit).is_some()
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

fn first_heading(text: &str) -> Option<(String, ByteSpan)> {
    let mut offset = 0usize;
    let mut in_front_matter = false;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_end();
        if i == 0 && trimmed == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            if trimmed == "---" {
                in_front_matter = false;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = rest.trim().trim_end_matches('#').trim().to_owned();
            return Some((
                title,
                ByteSpan::new(start as u64, (start + trimmed.len()) as u64),
            ));
        }
    }
    None
}

fn front_matter_value(text: &str, key: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some((k, v)) = line.split_once(':')
            && k.trim() == key
        {
            return Some(v.trim().trim_matches(['"', '\'']).to_owned());
        }
    }
    None
}

/// The first paragraph after the title, as a short summary.
fn summary(text: &str) -> Option<String> {
    let mut seen_title = false;
    let mut para = String::new();
    let mut in_front_matter = false;
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if i == 0 && t == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            if t == "---" {
                in_front_matter = false;
            }
            continue;
        }
        if t.starts_with('#') {
            if seen_title && !para.is_empty() {
                break;
            }
            seen_title = true;
            continue;
        }
        if t.is_empty() {
            if !para.is_empty() {
                break;
            }
            continue;
        }
        if t.starts_with(['|', '>', '-', '*', '!', '[']) {
            continue;
        }
        if !para.is_empty() {
            para.push(' ');
        }
        para.push_str(t);
    }
    (!para.is_empty()).then_some(para)
}

/// Path-like tokens: inline code or bare words containing a slash and an extension.
fn path_tokens(text: &str) -> Vec<(String, ByteSpan)> {
    let mut out: Vec<(String, ByteSpan)> = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'`'
            && let Some(len) = bytes[i + 1..].iter().position(|b| *b == b'`')
        {
            let inner = &text[i + 1..i + 1 + len];
            if looks_like_path(inner) {
                out.push((
                    inner.to_owned(),
                    ByteSpan::new((i + 1) as u64, (i + 1 + len) as u64),
                ));
            }
            i += len + 2;
            continue;
        }
        if c.is_ascii_whitespace() || c == b'(' || c == b'[' {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && !matches!(bytes[i], b'(' | b')' | b'[' | b']' | b'`' | b',' | b';')
        {
            i += 1;
        }
        if i == start {
            // A delimiter with no word before it: step over it.
            i += 1;
            continue;
        }
        let word = text[start..i].trim_end_matches(['.', ':', '"', '\'', '*']);
        if looks_like_path(word) && !word.contains("://") {
            out.push((
                word.to_owned(),
                ByteSpan::new(start as u64, (start + word.len()) as u64),
            ));
        }
    }
    out.sort_by_key(|(_, s)| s.start);
    out.dedup_by(|a, b| a.1 == b.1);
    out
}

fn looks_like_path(token: &str) -> bool {
    if token.len() < 4 || token.len() > 200 || token.contains(' ') || token.contains("://") {
        return false;
    }
    let has_slash = token.contains('/');
    let last = token.rsplit('/').next().unwrap_or(token);
    let has_ext = last.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && ext.len() <= 6 && ext.chars().all(|c| c.is_ascii_alphanumeric())
    });
    let endswith_slash = token.ends_with('/');
    if has_slash {
        has_ext || endswith_slash
    } else {
        has_ext
            && matches!(
                last,
                "Cargo.toml" | "package.json" | "pyproject.toml" | "Dockerfile"
            )
    }
}

#[cfg(test)]
mod tests {
    use globset::{Glob, GlobSetBuilder};
    use surveyor_model::{CommitId, RepoId, RepoPath};

    use super::*;

    fn set(p: &str) -> GlobSet {
        let mut b = GlobSetBuilder::new();
        b.add(Glob::new(p).expect("glob"));
        b.build().expect("set")
    }

    #[test]
    fn design_document_with_paths() {
        let text = "---\nuid: RFD-abc123\nstate: published\n---\n# Outbox relay design\n\nThe relay in `crates/outbox/src/relay.rs` drains rows. See docs/runbooks/stuck.md and https://x.y/a.md.\n\n## Details\n";
        let u = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("docs/rfd/0001-outbox.md"),
            CommitId::new("c"),
            text.into(),
        );
        let ex = MarkdownExtractor::new(set("docs/rfd/**/*.md"), set("docs/runbooks/**/*.md"));
        assert!(ex.accepts(&u));
        let facts = ex.extract(&u).expect("extracts");
        let d = facts.declarations().next().expect("design");
        assert_eq!(d.kind, NodeKind::Design);
        assert_eq!(d.name, "Outbox relay design");
        assert_eq!(
            d.attributes.get("uid").map(String::as_str),
            Some("RFD-abc123")
        );
        assert_eq!(u.slice(d.name_span), "# Outbox relay design");
        assert!(d.doc.as_deref().unwrap_or("").starts_with("The relay in"));
        let paths: Vec<_> = facts.literals().map(|l| l.value.clone()).collect();
        assert_eq!(
            paths,
            vec!["crates/outbox/src/relay.rs", "docs/runbooks/stuck.md"]
        );
        let lit = facts.literals().next().expect("literal");
        assert_eq!(u.slice(lit.span.bytes), "crates/outbox/src/relay.rs");
    }
}
