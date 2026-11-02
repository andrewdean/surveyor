//! Rhai extraction through the `rhai` crate's own parser.
//!
//! The language ships an AST, so no grammar is needed: script functions become
//! declarations, function calls become `Call` references, and `import` statements
//! become `Import` references. Rhai reports line and column positions; byte spans are
//! recovered from the source text.

use std::collections::BTreeMap;

use rhai::{ASTNode, Engine, Expr, Position, Stmt};
use surveyor_model::{
    ByteSpan, CallForm, DeclarationFact, ExtractorId, Fact, FactSet, Language, NodeKind,
    ReferenceFact, ReferenceRole, RuleId, SourceUnit,
};

use crate::{ExtractError, Extractor};

const EXTRACTOR_ID: &str = "rhai";

/// Extracts functions, calls and imports from `.rhai` scripts.
#[derive(Debug, Clone, Default)]
pub struct RhaiExtractor;

impl RhaiExtractor {
    /// Creates the extractor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extracts facts from a Rhai unit.
    ///
    /// # Errors
    /// Returns a parse error when the script does not compile.
    #[expect(
        clippy::too_many_lines,
        reason = "declarations, then one AST walk; splitting would share five locals"
    )]
    pub fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        let text = unit
            .text()
            .ok_or_else(|| ExtractError::parse("Rhai file is not UTF-8"))?;
        // A shebang is not Rhai; blank it out with a same-length comment so offsets hold.
        let owned;
        let text = if let Some(rest) = text.strip_prefix("#!") {
            owned = format!("//{rest}");
            owned.as_str()
        } else {
            text
        };
        let mut engine = Engine::new_raw();
        // Task scripts nest closures and maps deeper than the default limit of 64.
        engine.set_max_expr_depths(256, 256);
        let ast = engine
            .compile(text)
            .map_err(|e| ExtractError::parse(e.to_string()))?;
        let lines = LineIndex::new(text);
        let mut set = FactSet::empty(ExtractorId::new(EXTRACTOR_ID), unit);

        // Function definitions: rhai keeps no declaration span, so find `fn <name>(`
        // with the same arity in the text and take the brace block that follows. The
        // AST lists functions in no particular order, so each declaration found is
        // claimed and never handed to an overload of the same arity twice.
        let mut fn_spans: Vec<(ByteSpan, String)> = Vec::new();
        for fn_def in ast.iter_fn_def() {
            let name = fn_def.name.to_string();
            let claimed = |at: usize| fn_spans.iter().any(|(s, _)| s.start as usize == at);
            let Some(decl_start) = find_fn_decl(text, &name, fn_def.params.len(), &claimed) else {
                continue;
            };
            let end = block_end(text, decl_start);
            let bytes = ByteSpan::new(decl_start as u64, end as u64);
            let span = unit.text_span(bytes);
            let signature = format!(
                "fn {}({})",
                name,
                fn_def
                    .params
                    .iter()
                    .map(rhai::ImmutableString::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let name_start = text[decl_start..end]
                .find(&name)
                .map_or(decl_start, |i| decl_start + i);
            let mut attributes = BTreeMap::new();
            attributes.insert("params".to_owned(), fn_def.params.len().to_string());
            set.facts.push(Fact::Declaration(DeclarationFact {
                kind: NodeKind::Function,
                name: name.clone(),
                scope: Vec::new(),
                span,
                name_span: ByteSpan::new(name_start as u64, (name_start + name.len()) as u64),
                signature: Some(signature),
                doc: None,
                rule: RuleId::new("rhai.fn"),
                attributes,
                hash: unit.hash_of(bytes),
            }));
            fn_spans.push((bytes, name));
        }

        let scope_at = |offset: usize| -> Vec<String> {
            fn_spans
                .iter()
                .filter(|(s, _)| s.start as usize <= offset && offset < s.end as usize)
                .map(|(_, n)| n.clone())
                .collect()
        };

        let mut sites: Vec<ReferenceFact> = Vec::new();
        let mut visit = |path: &[ASTNode<'_>]| -> bool {
            let Some(node) = path.last() else { return true };
            match node {
                ASTNode::Expr(Expr::FnCall(call, pos) | Expr::MethodCall(call, pos))
                | ASTNode::Stmt(Stmt::FnCall(call, pos)) => {
                    if call.op_token.is_some() {
                        return true;
                    }
                    let name = call.name.to_string();
                    let mut path: Vec<String> = call
                        .namespace
                        .path
                        .iter()
                        .map(|m| m.name.to_string())
                        .collect();
                    path.push(name.clone());
                    push_site(
                        &mut sites,
                        unit,
                        &lines,
                        *pos,
                        &name,
                        path,
                        ReferenceRole::Call(CallForm::Path),
                        "rhai.call",
                        &scope_at,
                    );
                }
                ASTNode::Stmt(Stmt::Import(parts, pos)) => {
                    if let Expr::StringConstant(s, _) = &parts.0 {
                        let spec = s.to_string();
                        let path: Vec<String> = spec
                            .split(['/', ':'])
                            .filter(|p| !p.is_empty())
                            .map(str::to_owned)
                            .collect();
                        let name = path.last().cloned().unwrap_or_else(|| spec.clone());
                        push_site(
                            &mut sites,
                            unit,
                            &lines,
                            *pos,
                            &spec,
                            path,
                            ReferenceRole::Import,
                            "rhai.import",
                            &scope_at,
                        );
                        let _ = name;
                    }
                }
                _ => {}
            }
            true
        };
        // `walk` covers top-level statements and every function body.
        ast.walk(&mut visit);
        set.facts.extend(sites.into_iter().map(Fact::Reference));
        Ok(set)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one call site builder shared by calls and imports"
)]
fn push_site(
    sites: &mut Vec<ReferenceFact>,
    unit: &SourceUnit,
    lines: &LineIndex,
    pos: Position,
    token: &str,
    path: Vec<String>,
    role: ReferenceRole,
    rule: &str,
    scope_at: &dyn Fn(usize) -> Vec<String>,
) {
    let Some(start) = lines.offset(pos) else {
        return;
    };
    let end = (start + token.len()).min(unit.content.len()).max(start);
    let bytes = ByteSpan::new(start as u64, end as u64);
    let name = path.last().cloned().unwrap_or_else(|| token.to_owned());
    sites.push(ReferenceFact {
        role,
        name,
        path,
        span: unit.text_span(bytes),
        scope: scope_at(start),
        rule: RuleId::new(rule),
        hash: unit.hash_of(bytes),
        receiver: None,
        binding: None,
    });
}

impl Extractor for RhaiExtractor {
    fn id(&self) -> ExtractorId {
        ExtractorId::new(EXTRACTOR_ID)
    }

    fn accepts(&self, unit: &SourceUnit) -> bool {
        unit.language == Some(Language::Rhai)
    }

    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError> {
        Self::extract(self, unit)
    }
}

struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    /// Byte offset of a Rhai position (one-based line and character column).
    fn offset(&self, pos: Position) -> Option<usize> {
        let line = pos.line()?.checked_sub(1)?;
        let col = pos.position().unwrap_or(1).saturating_sub(1);
        let start = *self.starts.get(line)?;
        Some(start + col)
    }
}

/// Offset of the first `fn <name>(<arity params>)` not yet `claimed`, or `None`.
fn find_fn_decl(
    text: &str,
    name: &str,
    arity: usize,
    claimed: &dyn Fn(usize) -> bool,
) -> Option<usize> {
    let mut search = 0usize;
    while let Some(rel) = text.get(search..)?.find("fn ") {
        let at = search + rel;
        let after = &text[at + 3..];
        let candidate = after.trim_start();
        let params = candidate
            .strip_prefix(name)
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix('('))
            .and_then(|rest| rest.split_once(')'))
            .map(|(params, _)| params);
        let preceded_ok = at == 0
            || !text.as_bytes()[at - 1].is_ascii_alphanumeric() && text.as_bytes()[at - 1] != b'_';
        if preceded_ok && params.is_some_and(|p| param_count(p) == arity) && !claimed(at) {
            return Some(at);
        }
        search = at + 3;
    }
    None
}

/// Number of parameters in the text between a declaration's parentheses.
fn param_count(params: &str) -> usize {
    params.split(',').filter(|p| !p.trim().is_empty()).count()
}

/// End of the brace block that starts at or after `from`.
fn block_end(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let Some(open) = bytes[from..]
        .iter()
        .position(|b| *b == b'{')
        .map(|i| from + i)
    else {
        return text.len();
    };
    let mut depth = 0usize;
    for (i, b) in bytes[open..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i + 1;
                }
            }
            _ => {}
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use surveyor_model::{CommitId, RepoId, RepoPath};

    use super::*;

    #[test]
    fn functions_calls_and_imports() {
        let src = "import \"host/fs\" as fs;\n\nfn deploy(env) {\n    let x = fs::read(env);\n    notify(x);\n}\n\ndeploy(\"prod\");\n";
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("t.rhai"),
            CommitId::new("c"),
            src.into(),
        );
        let facts = RhaiExtractor::new().extract(&unit).expect("compiles");
        let deploy = facts
            .declarations()
            .find(|d| d.name == "deploy")
            .expect("fn");
        assert_eq!(deploy.kind, NodeKind::Function);
        assert!(
            unit.slice(deploy.span.bytes)
                .starts_with("fn deploy(env) {")
        );
        assert!(unit.slice(deploy.span.bytes).ends_with('}'));
        let refs: Vec<_> = facts.references().collect();
        let read = refs
            .iter()
            .find(|r| r.name == "read")
            .expect("namespaced call");
        assert_eq!(read.path, vec!["fs", "read"]);
        assert_eq!(read.scope, vec!["deploy".to_owned()]);
        assert!(refs.iter().any(|r| r.name == "notify" && r.role.is_call()));
        assert!(
            refs.iter()
                .any(|r| r.name == "deploy" && r.scope.is_empty())
        );
        assert!(
            refs.iter()
                .any(|r| r.role == ReferenceRole::Import && r.path == vec!["host", "fs"])
        );
    }

    #[test]
    fn overloads_keep_their_own_declarations() {
        let text =
            "fn resolve(c) {\n    resolve(c, #{})\n}\n\nfn resolve(c, opts) {\n    opts\n}\n";
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("cluster.rhai"),
            CommitId::new("c"),
            text.as_bytes().to_vec(),
        );
        let facts = RhaiExtractor::new().extract(&unit).expect("parses");
        let mut decls: Vec<(String, u64, u32)> = facts
            .declarations()
            .map(|d| {
                (
                    d.signature.clone().unwrap_or_default(),
                    d.span.bytes.start,
                    d.span.lines.start,
                )
            })
            .collect();
        decls.sort_by_key(|(_, start, _)| *start);
        assert_eq!(
            decls,
            vec![
                ("fn resolve(c)".to_owned(), 0, 1),
                ("fn resolve(c, opts)".to_owned(), 39, 5),
            ]
        );
    }
}
