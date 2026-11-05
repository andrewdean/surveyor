//! Rhai extraction through the `rhai` crate's own parser.
//!
//! The language ships an AST, so no grammar is needed: script functions become
//! declarations, function calls become `Call` references, and `import` statements
//! become `Import` references. The `//` comment block right above a function is its
//! documentation. A call's string arguments that spell a command (`proc::shell(["mise",
//! "run", "x"])`) or hand a task or crate to a run helper (`mise_run(.., "x", ..)`,
//! `cargo_run(.., "jobs/Cargo.toml", "seed", ..)`) become `Command` literals, so the
//! linker can draw the edge from a script to the task or binary it runs. Rhai reports
//! line and column positions; byte spans are recovered from the source text.

use std::collections::BTreeMap;

use rhai::{ASTNode, Engine, Expr, Position, Stmt};
use surveyor_model::{
    ByteSpan, CallForm, ContentHash, DeclarationFact, ExtractorId, Fact, FactSet, Language,
    LiteralFact, LiteralRole, NodeKind, ReferenceFact, ReferenceRole, RuleId, SourceUnit,
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
        // The optimizer folds a constant array such as `["mise", "run", "x"]` into one
        // opaque value; the literal has to stay visible to be read as a command.
        engine.set_optimization_level(rhai::OptimizationLevel::None);
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
                doc: leading_comments(text, decl_start),
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
        let mut commands: Vec<LiteralFact> = Vec::new();
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
                    for (value, rule, at) in command_arguments(&name, &call.args) {
                        if let Some(start) = lines.offset(at) {
                            // The argument token: a quoted string, or an array up to
                            // its closing bracket; never past the end of its line.
                            let rest = text.get(start..).unwrap_or_default();
                            let line_end = rest.find('\n').unwrap_or(rest.len());
                            let token_end = if rest.starts_with('[') {
                                rest.find(']').map_or(line_end, |i| i + 1)
                            } else {
                                let quote = rest.chars().next().unwrap_or('"');
                                rest[quote.len_utf8()..]
                                    .find(quote)
                                    .map_or(line_end, |i| i + 2 * quote.len_utf8())
                            };
                            let end = start + token_end.min(line_end);
                            let bytes = ByteSpan::new(start as u64, end as u64);
                            commands.push(LiteralFact {
                                role: LiteralRole::Command,
                                hash: ContentHash::of(&value),
                                value,
                                span: unit.text_span(bytes),
                                scope: scope_at(start),
                                rule: RuleId::new(rule),
                            });
                        }
                    }
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
        // In source order, whatever order the walk met them in.
        commands.sort_by_key(|c| c.span.bytes.start);
        set.facts.extend(commands.into_iter().map(Fact::Literal));
        Ok(set)
    }
}

/// The commands a call's arguments spell, each with the rule that read it and the
/// position of the argument: `(command, rule, position)`.
///
/// Two readings. A string or array constant that starts with `mise run` or
/// `cargo run` is the command itself (`rhai.command`), whatever the call. An argument
/// of a run helper, a call whose name contains `run`, `task`, `shell` or `exec`, is
/// read by shape (`rhai.run-arg`): a `Cargo.toml` path followed by a string is a
/// package to `cargo run`, and a bare `a:b` task name is a task to `mise run`.
fn command_arguments(call: &str, args: &[Expr]) -> Vec<(String, &'static str, Position)> {
    let constant = |e: &Expr| -> Option<(String, Position)> {
        match e {
            Expr::StringConstant(s, pos) => Some((s.to_string(), *pos)),
            Expr::Array(items, pos) => {
                let words: Vec<String> = items
                    .iter()
                    .map_while(|i| match i {
                        Expr::StringConstant(s, _) => Some(s.to_string()),
                        _ => None,
                    })
                    .collect();
                (!words.is_empty()).then(|| (words.join(" "), *pos))
            }
            _ => None,
        }
    };
    let mut out = Vec::new();
    let constants: Vec<(String, Position)> = args.iter().filter_map(constant).collect();
    for (value, pos) in &constants {
        if value.starts_with("mise run ") || value.starts_with("cargo run ") {
            out.push((value.clone(), "rhai.command", *pos));
        }
    }
    let lower = call.to_lowercase();
    let helper = ["run", "task", "shell", "exec"]
        .iter()
        .any(|w| lower.contains(w));
    if !helper || !out.is_empty() {
        return out;
    }
    let string_args: Vec<(String, Position)> = args
        .iter()
        .filter_map(|e| match e {
            Expr::StringConstant(s, pos) => Some((s.to_string(), *pos)),
            _ => None,
        })
        .collect();
    if let Some(i) = string_args
        .iter()
        .position(|(v, _)| v.ends_with("Cargo.toml"))
        && let Some((package, _)) = string_args.get(i + 1)
        && is_package_name(package)
    {
        let (manifest, pos) = &string_args[i];
        out.push((
            format!("cargo run --manifest-path {manifest} -p {package}"),
            "rhai.run-arg",
            *pos,
        ));
        return out;
    }
    out.extend(
        string_args
            .iter()
            .filter(|(v, _)| is_task_name(v))
            .map(|(v, pos)| (format!("mise run {v}"), "rhai.run-arg", *pos)),
    );
    out
}

/// `a:b` or `a:b:c`: a namespaced task name, never a path or a sentence.
fn is_task_name(value: &str) -> bool {
    value.contains(':')
        && value.len() >= 3
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.'))
        && !value.starts_with(':')
        && !value.ends_with(':')
}

/// A cargo package name: letters, digits, `-` and `_`.
fn is_package_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
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

/// The `//` or `///` comment lines directly above the line a declaration starts on,
/// joined into one block; `None` when the line above is not a comment.
fn leading_comments(text: &str, at: usize) -> Option<String> {
    let before = text.get(..at)?;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    if !before[line_start..].trim().is_empty() {
        return None;
    }
    let mut lines: Vec<&str> = before[..line_start]
        .lines()
        .rev()
        .map(str::trim)
        .take_while(|line| line.starts_with("//"))
        .map(|line| {
            line.trim_start_matches('/')
                .strip_prefix(' ')
                .unwrap_or_else(|| line.trim_start_matches('/'))
        })
        .collect();
    lines.reverse();
    (!lines.is_empty()).then(|| lines.join("\n").trim().to_owned())
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
    fn the_comment_block_above_a_function_is_its_documentation() {
        let src = "// The loader registry.\n\n// Run a mise task with `vars` merged in;\n// DRY_RUN=1 prints it instead.\nfn mise_run(repo, vars, task, args) {\n    task\n}\n\nlet x = 1;\nfn bare() { 1 }\n";
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("loaders.rhai"),
            CommitId::new("c"),
            src.into(),
        );
        let facts = RhaiExtractor::new().extract(&unit).expect("compiles");
        let docs: BTreeMap<String, Option<String>> = facts
            .declarations()
            .map(|d| (d.name.clone(), d.doc.clone()))
            .collect();
        assert_eq!(
            docs["mise_run"].as_deref(),
            Some("Run a mise task with `vars` merged in;\nDRY_RUN=1 prints it instead."),
            "the blank line keeps the file header out"
        );
        assert_eq!(docs["bare"], None);
    }

    #[test]
    fn commands_in_call_arguments_become_command_literals() {
        let src = "fn run_loader(key, ctx) {\n    mise_run(ctx.repo, ctx.vars, \"devdata:validate\", []);\n    dd::cargo_run(ctx.repo, ctx.vars, \"jobs/Cargo.toml\", \"causeway-seed\", [\"--profile\", \"people\"], #{});\n    dd::task_run(ctx.repo, ctx.vars, \"ops:metadata:process\");\n    print(\"not: a task\");\n}\n\nproc::shell([\"mise\", \"run\", \"devdata:load\"], #{ dir: \"x\" });\n";
        let unit = SourceUnit::new(
            RepoId::new("r"),
            RepoPath::new("loaders.rhai"),
            CommitId::new("c"),
            src.into(),
        );
        let facts = RhaiExtractor::new().extract(&unit).expect("compiles");
        let commands: Vec<(String, String, Vec<String>)> = facts
            .literals()
            .map(|l| (l.value.clone(), l.rule.to_string(), l.scope.clone()))
            .collect();
        assert_eq!(
            commands,
            vec![
                (
                    "mise run devdata:validate".to_owned(),
                    "rhai.run-arg".to_owned(),
                    vec!["run_loader".to_owned()]
                ),
                (
                    "cargo run --manifest-path jobs/Cargo.toml -p causeway-seed".to_owned(),
                    "rhai.run-arg".to_owned(),
                    vec!["run_loader".to_owned()]
                ),
                (
                    "mise run ops:metadata:process".to_owned(),
                    "rhai.run-arg".to_owned(),
                    vec!["run_loader".to_owned()]
                ),
                (
                    "mise run devdata:load".to_owned(),
                    "rhai.command".to_owned(),
                    Vec::new()
                ),
            ]
        );
        let first = facts.literals().next().expect("literal");
        assert_eq!(first.span.lines.start, 2);
        assert_eq!(unit.slice(first.span.bytes), "\"devdata:validate\"");
        let cargo = facts.literals().nth(1).expect("literal");
        assert_eq!(unit.slice(cargo.span.bytes), "\"jobs/Cargo.toml\"");
        let last = facts.literals().last().expect("literal");
        assert_eq!(last.span.lines.start, 8);
        assert_eq!(
            unit.slice(last.span.bytes),
            "[\"mise\", \"run\", \"devdata:load\"]"
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
