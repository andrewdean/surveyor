//! Debug helper: print the facts every extractor finds in one file.
//!
//! ```text
//! cargo run -p surveyor-extract --example dump -- <repo-root> <file>
//! cargo run -p surveyor-extract --example dump -- <file>
//! ```
//!
//! With a repository root, the file's path is made repository-relative and the root's
//! `surveyor.toml` selects the SQL dialect and documentation directories, so
//! manifests, migrations and design documents are classified as they would be during
//! `surveyor index`. Extractor failures are printed, not panicked on; only an unreadable
//! file is fatal.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use surveyor_extract::{Profile, default_extractors, extract_units};
use surveyor_model::{CommitId, Fact, RepoId, RepoPath, SourceUnit};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (root, file) = match args.as_slice() {
        [file] => (None, PathBuf::from(file)),
        [root, file] => (Some(PathBuf::from(root)), PathBuf::from(file)),
        _ => {
            eprintln!("usage: dump [<repo-root>] <file>");
            return ExitCode::from(2);
        }
    };
    match run(root.as_deref(), &file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("dump: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(root: Option<&Path>, file: &Path) -> Result<(), String> {
    let content = std::fs::read(file).map_err(|e| format!("read {}: {e}", file.display()))?;
    let (profile, relative) = match root {
        Some(root) => {
            let profile = Profile::load(root).map_err(|e| e.to_string())?;
            let absolute = file.canonicalize().map_err(|e| e.to_string())?;
            let root = root.canonicalize().map_err(|e| e.to_string())?;
            let relative = absolute.strip_prefix(&root).map_err(|_outside| {
                format!("{} is not under {}", file.display(), root.display())
            })?;
            (profile, RepoPath::new(relative.to_string_lossy()))
        }
        None => (Profile::default(), RepoPath::new(file.to_string_lossy())),
    };
    let extractors = default_extractors(&profile).map_err(|e| e.to_string())?;
    let unit = SourceUnit::new(
        RepoId::new("dump"),
        relative,
        CommitId::new("worktree"),
        content,
    );
    let extraction = extract_units(
        &extractors,
        std::slice::from_ref(&unit),
        profile.sql.dialect,
    );

    for set in &extraction.fact_sets {
        println!("== {} ({} facts)", set.extractor, set.facts.len());
        set.facts.iter().for_each(print_fact);
    }
    if extraction.fact_sets.is_empty() {
        println!(
            "no extractor accepted {} (language {:?})",
            unit.path, unit.language
        );
    }
    for err in &extraction.errors {
        eprintln!("error: {err}");
    }
    Ok(())
}

fn print_fact(fact: &Fact) {
    match fact {
        Fact::Declaration(d) => println!(
            "DECL {} {} scope={:?} lines={} rule={}",
            d.kind, d.name, d.scope, d.span.lines, d.rule
        ),
        Fact::Reference(r) => println!(
            "REF  {:?} {} path={:?} scope={:?} lines={}",
            r.role, r.name, r.path, r.scope, r.span.lines
        ),
        Fact::Literal(l) => println!("LIT  {:?} {:?} lines={}", l.role, l.value, l.span.lines),
        Fact::Resolved(r) => println!("RES  {} lines={} -> {:?}", r.name, r.span.lines, r.target),
        Fact::Relation(r) => println!(
            "REL  {} {} -> {} lines={}",
            r.kind, r.src_name, r.dst_name, r.span.lines
        ),
    }
}
