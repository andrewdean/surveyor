//! Design documents, runbooks and every other document from Markdown.
//!
//! A Markdown file under a design or runbook root becomes a `Design` or `Runbook`
//! declaration named by its first heading; any other Markdown file (a README, a
//! guide) becomes a `Document`. Each carries its first paragraph as documentation and
//! its section headings as a `headings` attribute, so a plain-English question can
//! land on the document that answers it. Path-like tokens in the text (inline code or
//! bare `dir/file.ext` words) become `Path` literals, which the linker turns into
//! `DESIGNED_BY` and `DOCUMENTED_BY` edges from the files they name.

use std::collections::BTreeMap;

use globset::GlobSet;
use surveyor_model::{
    ByteSpan, ContentHash, DeclarationFact, ExtractorId, Fact, FactSet, Language, LiteralFact,
    LiteralRole, NodeKind, RuleId, SourceUnit,
};

use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "markdown";

/// Longest `headings` attribute kept, in bytes; a generated reference with hundreds of
/// sections keeps its first ones.
const MAX_HEADINGS_BYTES: usize = 4000;

/// Reads design documents, runbooks and other Markdown documents.
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

    fn kind_for(&self, unit: &SourceUnit) -> NodeKind {
        let path = unit.path.as_str();
        if self.runbooks.is_match(path) {
            NodeKind::Runbook
        } else if self.designs.is_match(path) {
            NodeKind::Design
        } else {
            NodeKind::Document
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
        let kind = self.kind_for(unit);

        let (title, title_span) = document_title(text, &unit.path);
        let mut attributes = BTreeMap::new();
        attributes.insert("path".to_owned(), unit.path.to_string());
        let headings = headings(text);
        if !headings.is_empty() {
            attributes.insert("headings".to_owned(), headings);
        }
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
        unit.language == Some(Language::Markdown)
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

/// The document's name: the front matter `title`, else the first heading, else the
/// file stem. A heading that only names a section (`Summary`, `Overview`) is skipped
/// when a better name exists.
fn document_title(text: &str, path: &surveyor_model::RepoPath) -> (String, ByteSpan) {
    const SECTION_HEADINGS: &[&str] =
        &["summary", "overview", "abstract", "introduction", "context"];
    let headings: Vec<(String, ByteSpan)> = level_one_headings(text);
    let named = headings
        .iter()
        .find(|(h, _)| !SECTION_HEADINGS.contains(&h.to_lowercase().as_str()));
    front_matter_entry(text, "title")
        .or_else(|| named.or(headings.first()).cloned())
        .unwrap_or_else(|| {
            let stem = path
                .file_name()
                .and_then(|n| n.rsplit_once('.'))
                .map_or("document", |(s, _)| s);
            (stem.to_owned(), ByteSpan::new(0, 0))
        })
}

/// A front matter `key: value` line as `(value, span of the line)`.
fn front_matter_entry(text: &str, key: &str) -> Option<(String, ByteSpan)> {
    let mut offset = 0usize;
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim() != "---" {
        return None;
    }
    offset += first.len();
    for line in lines {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_end();
        if trimmed == "---" {
            return None;
        }
        if let Some((k, v)) = trimmed.split_once(':')
            && k.trim() == key
        {
            let value = v.trim().trim_matches(['"', '\'']).to_owned();
            return (!value.is_empty()).then(|| {
                (
                    value,
                    ByteSpan::new(start as u64, (start + trimmed.len()) as u64),
                )
            });
        }
    }
    None
}

/// Every `# heading` outside the front matter, with the span of its line.
fn level_one_headings(text: &str) -> Vec<(String, ByteSpan)> {
    let mut offset = 0usize;
    let mut in_front_matter = false;
    let mut out = Vec::new();
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_end();
        if i == 0 && trimmed == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            in_front_matter = trimmed != "---";
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = rest.trim().trim_end_matches('#').trim().to_owned();
            out.push((
                title,
                ByteSpan::new(start as u64, (start + trimmed.len()) as u64),
            ));
        }
    }
    out
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

/// The section headings (`##` and deeper) outside the front matter and code fences,
/// one per line, cut at [`MAX_HEADINGS_BYTES`].
fn headings(text: &str) -> String {
    let mut in_front_matter = false;
    let mut in_fence = false;
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if i == 0 && t == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            in_front_matter = t != "---";
            continue;
        }
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || !t.starts_with("##") {
            continue;
        }
        let heading = t
            .trim_start_matches('#')
            .trim()
            .trim_end_matches('#')
            .trim();
        if heading.is_empty() {
            continue;
        }
        if out.len() + heading.len() + 1 > MAX_HEADINGS_BYTES {
            break;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(heading);
    }
    out
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
            // Inside backticks a slash is enough: `crates/outbox` names a directory the
            // linker may know as a package.
            if looks_like_path(inner) || looks_like_dir(inner) {
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

/// A backticked `dir/sub` with no extension: a directory a package may live in.
fn looks_like_dir(token: &str) -> bool {
    token.len() >= 4
        && token.len() <= 200
        && token.contains('/')
        && !token.contains(' ')
        && !token.contains("://")
        && !token.starts_with('/')
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '-' | '_'))
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
    fn title_comes_from_front_matter_then_heading_then_stem() {
        let ex = MarkdownExtractor::new(set("docs/rfd/**/*.md"), set("docs/runbooks/**/*.md"));
        let doc = |path: &str, text: &str| {
            let u = SourceUnit::new(
                RepoId::new("r"),
                RepoPath::new(path),
                CommitId::new("c"),
                text.into(),
            );
            let facts = ex.extract(&u).expect("extracts");
            let d = facts.declarations().next().expect("declaration").clone();
            (d.name.clone(), u.slice(d.name_span).to_owned())
        };
        let (name, cited) = doc(
            "docs/rfd/RFD-1.md",
            "---\nuid: x\ntitle: Operational resilience: stuck work\n---\n\n# Summary\n\nBody.\n",
        );
        assert_eq!(name, "Operational resilience: stuck work");
        assert_eq!(cited, "title: Operational resilience: stuck work");
        let (name, _) = doc("docs/rfd/RFD-2.md", "# Summary\n\n# Real title\n");
        assert_eq!(name, "Real title");
        let (name, _) = doc("docs/rfd/RFD-3.md", "# Summary\n\nonly a section heading\n");
        assert_eq!(name, "Summary");
        let (name, _) = doc("docs/rfd/RFD-4-no-heading.md", "plain text\n");
        assert_eq!(name, "RFD-4-no-heading");
    }

    #[test]
    fn any_other_markdown_is_a_document_with_its_headings() {
        let text = "# causeway-devdata\n\nThe development data package: the inputs of the corpus generator.\n\n## Loading\n\n```sh\n## not a heading\nmise run devdata:load\n```\n\n### The people loader\n\nRuns `causeway-seed --profile people`.\n\n## Gate ##\n";
        let u = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("devdata/README.md"),
            CommitId::new("c"),
            text.into(),
        );
        let ex = MarkdownExtractor::new(set("docs/rfd/**/*.md"), set("docs/runbooks/**/*.md"));
        assert!(ex.accepts(&u));
        let facts = ex.extract(&u).expect("extracts");
        let d = facts.declarations().next().expect("document");
        assert_eq!(d.kind, NodeKind::Document);
        assert_eq!(d.name, "causeway-devdata");
        assert_eq!(
            d.doc.as_deref(),
            Some("The development data package: the inputs of the corpus generator.")
        );
        assert_eq!(
            d.attributes.get("headings").map(String::as_str),
            Some("Loading\nThe people loader\nGate")
        );
        assert!(headings("# only a title\n").is_empty());
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
