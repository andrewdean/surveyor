//! What a command line names: a mise task, a cargo package's binary, a script.
//!
//! A `Command` literal is the text a task's `run =` entry or a script's `proc::shell`
//! call would hand to a shell. Reading it is syntax, not execution: `mise run x`
//! names the task `x`, `cargo run -p y` names the binary of package `y`, and any
//! token that is a path in the repository names that file. The linker turns each into
//! a `RUNS` edge when the named node exists and records nothing when it does not.

use surveyor_model::NodeId;

/// Something a command line names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTarget {
    /// A task by the name its runner uses, such as `devdata:validate`.
    Task(String),
    /// A cargo package's binary: the package named by `-p`, and the `--bin` when given.
    Binary {
        /// The package name.
        package: String,
        /// The binary name, when the command picks one.
        bin: Option<String>,
    },
    /// A repository path a token spells out, such as `.mise/scripts/tasks/db/migrate.rhai`.
    Path(String),
    /// A bare program name, such as `cw-devdata`, which may be a package's binary.
    Program(String),
}

/// The cargo subcommands that run a package's binary.
const CARGO_RUNNERS: &[&str] = &["run", "r"];

/// Files a command names as arguments, never as something it runs.
const MANIFESTS: &[&str] = &["Cargo.toml", "package.json", "pyproject.toml"];

/// What `command` names, in the order the tokens appear.
///
/// Tokens that look like paths (a slash, or a known script extension) are kept as
/// [`CommandTarget::Path`] whether or not the file exists; the linker checks.
#[must_use]
pub fn command_targets(command: &str) -> Vec<CommandTarget> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let mut out = Vec::new();
    match tokens.as_slice() {
        ["mise", "run" | "r" | "x", task, ..] => out.push(CommandTarget::Task(unquote(task))),
        ["cargo", sub, rest @ ..] if CARGO_RUNNERS.contains(sub) => {
            let value = |flag: &str| {
                rest.iter()
                    .position(|t| *t == flag)
                    .and_then(|i| rest.get(i + 1))
                    .map(|t| unquote(t))
                    .or_else(|| {
                        rest.iter()
                            .find_map(|t| t.strip_prefix(&format!("{flag}=")).map(unquote))
                    })
            };
            if let Some(package) = value("-p").or_else(|| value("--package")) {
                out.push(CommandTarget::Binary {
                    package,
                    bin: value("--bin"),
                });
            } else if let Some(bin) = value("--bin") {
                out.push(CommandTarget::Binary {
                    package: bin.clone(),
                    bin: Some(bin),
                });
            }
        }
        [program, ..] if !program.starts_with('-') && !looks_like_path(program) => {
            out.push(CommandTarget::Program(unquote(program)));
        }
        _ => {}
    }
    out.extend(
        tokens
            .iter()
            .map(|t| unquote(t))
            .filter(|t| looks_like_path(t) && !is_manifest(t))
            .map(|t| CommandTarget::Path(t.trim_start_matches("./").to_owned())),
    );
    out
}

/// A package manifest: `--manifest-path jobs/Cargo.toml` names an argument, not a
/// script the command runs.
fn is_manifest(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| MANIFESTS.contains(&name))
}

/// The names a command references, for incremental update: the task, package and
/// program names and the paths, so a file that names them relinks when they change.
#[must_use]
pub fn command_names(command: &str) -> Vec<String> {
    command_targets(command)
        .into_iter()
        .map(|t| match t {
            CommandTarget::Task(name) | CommandTarget::Program(name) => name,
            CommandTarget::Binary { package, .. } => package,
            CommandTarget::Path(path) => path,
        })
        .collect()
}

/// `task:mise/<name>`, the identifier of a mise task, for callers that hold a name.
#[must_use]
pub fn mise_task_id(name: &str) -> NodeId {
    crate::ids::task_id(&format!("mise/{name}"))
}

fn unquote(token: &str) -> String {
    token.trim_matches(['"', '\'', '`']).to_owned()
}

/// A token with a directory separator, or a file name with a script's extension.
fn looks_like_path(token: &str) -> bool {
    let t = token.trim_matches(['"', '\'', '`']);
    if t.starts_with('-') || t.starts_with('$') || t.contains("://") || t.len() < 3 {
        return false;
    }
    let has_separator = t.contains('/') && !t.starts_with('/');
    let script = t.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && matches!(ext, "rhai" | "sh" | "bash" | "py" | "js" | "ts" | "sql")
    });
    has_separator || script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mise_cargo_programs_and_paths_are_read_from_command_text() {
        assert_eq!(
            command_targets("mise run devdata:validate"),
            vec![CommandTarget::Task("devdata:validate".into())]
        );
        assert_eq!(
            command_targets("mise run db:psql -- -X -f .mise/scripts/bench.sql"),
            vec![
                CommandTarget::Task("db:psql".into()),
                CommandTarget::Path(".mise/scripts/bench.sql".into()),
            ]
        );
        assert_eq!(
            command_targets(
                "cargo run --manifest-path jobs/Cargo.toml -p causeway-seed -- --profile people"
            ),
            vec![CommandTarget::Binary {
                package: "causeway-seed".into(),
                bin: None
            }],
            "the manifest is an argument, not something run"
        );
        assert_eq!(
            command_targets("cargo run --bin xtask --package=tools"),
            vec![CommandTarget::Binary {
                package: "tools".into(),
                bin: Some("xtask".into())
            }]
        );
        assert_eq!(
            command_targets(".mise/scripts/tasks/devdata/loaders.rhai run"),
            vec![CommandTarget::Path(
                ".mise/scripts/tasks/devdata/loaders.rhai".into()
            )]
        );
        assert_eq!(
            command_targets("cw-devdata validate"),
            vec![CommandTarget::Program("cw-devdata".into())]
        );
        assert_eq!(
            command_targets("cwsh test ./loaders_test.rhai"),
            vec![
                CommandTarget::Program("cwsh".into()),
                CommandTarget::Path("loaders_test.rhai".into()),
            ]
        );
        assert!(command_targets("").is_empty());
        assert_eq!(
            command_names("cargo run -p api -- --flag"),
            vec!["api".to_owned()]
        );
        assert_eq!(mise_task_id("db:migrate").as_str(), "task:mise/db:migrate");
    }
}
