# The agent harness: session ledgers and hooks

A session ledger records what Surveyor told an agent, so that what the agent later
claims to have read can be checked against what it was shown. This document says
what the ledger holds, how a command or a hook feeds it, and how to wire it into
Claude Code, and what `surveyor pr`, the pull-request comment and gate that reads
the ledger, puts in front of a reviewer. The RFD's section 12.2 is the design this
implements.

## The ledger

A session is a file of JSON lines, `<store>/sessions/<id>.jsonl`, one record per
answer. Nothing is written unless a session is named: `--session <id>` on any
command, or `SURVEYOR_SESSION` in the environment. The identifier has to be a plain
file name (letters, digits, `.`, `_`, `-`, not starting with a dot) so the path
cannot leave the directory; Claude Code's session identifiers qualify as they are.

Table 1. The four kinds of record. Every record carries `at_ms`, milliseconds
since the epoch, and `kind`.

| Kind | Written by | Fields |
|---|---|---|
| `started` | `session start`, the `SessionStart` hook | `task` (optional), `repo`, `commit` of the dataset |
| `pack` | `context`, MCP `context_pack`, `session start --task`, the first `UserPromptSubmit` | `task`, `budget`, `tokens`, `seeds`, `consulted`, `gaps` |
| `answer` | every CLI query command, every MCP tool call, the `PostToolUse` hook | `source` (`cli`, `mcp`, `hook`), `op`, `args`, `consulted` |
| `note` | `session note` | `consulted` |

`consulted` is the list of nodes the answer named, each as `{id, citation}` with
the `file:lines` the answer cited for it: a node's own location for `explain`,
`card`, `query`, `search` and `nodes`; the evidence of the edge that reached it for
`affected` and `path`; the card's location for a pack item; the violation's
citation for `check`. The record is derived from the same typed answer the command
prints, so the ledger never cites what the agent did not see.

The CLI commands that record are `explain`, `card`, `context`, `affected`, `path`,
`query`, `search`, `nodes`, `source` and `check`. The MCP server records every tool
call when it was started with `SURVEYOR_SESSION` set, reading the node references
out of the JSON it returned (any object with an `id` beside a `kind`, `name`,
`qualified_name` or `file`), and records `context_pack` as a `pack`.

## `surveyor session`

```bash
eval "$(surveyor session start --task "make Relay::drain send in batches" | head -1)"
surveyor session start --task "..." --json     # the same with the pack as JSON
surveyor session note Relay::drain crates/api/src/main.rs:1-40
surveyor session summary [--write]
```

`start` runs `update` (unless `--no-update`), records `started`, plans the pack for
the task inside `--budget` (the default is `surveyor context`'s 30,000 tokens),
records it and prints it. In text mode the first line is `SURVEYOR_SESSION=<id>`,
so a harness script can export it and every later `surveyor` command in that shell
records into the same ledger. `--id` names the session; otherwise `--session` or a
generated `s-<unix seconds>-<pid>` is used.

`note` records what the agent read outside Surveyor: a node by name or identifier,
a `path:line-line` citation, or any text. `summary` folds the ledger into the
consulted set, every node with its first citation and the commands that named it,
the pack's seeds and gaps, and prints it as Markdown or, with `--json`, as the
structure `surveyor pr --session` embeds. `--write` also leaves
`<id>.summary.md` and `<id>.summary.json` beside the ledger.

## The Claude Code adapter

`surveyor hook claude-code` reads the JSON payload Claude Code writes to a hook's
stdin and dispatches on `hook_event_name`. It takes the repository root from the
payload's `cwd` when `--repo-root` is not given, and the dataset from `--store`,
`SURVEYOR_STORE` or `<root>/.surveyor`. Claude Code's `session_id` is the session
identifier, which is the join key between the hooks and whatever the agent ran by
hand with `--session`.

Table 2. What each event does. A hook always exits 0: an error is printed to stderr,
which Claude Code shows in its transcript view and never to the model, because a
ledger failure must not block an agent.

| Event | Does | Prints (added to the model's context) |
|---|---|---|
| `SessionStart` | `update`, then records `started` | one line: dataset commit, node, edge and rule counts, the session id, and how to use the tools |
| `UserPromptSubmit` | plans a pack for the prompt inside `[harness] prompt_budget` (default 12,000 tokens), once per session, when the prompt seeds anything | the pack, headed by its token count and a reminder to verify citations |
| `PostToolUse` with matcher `mcp__surveyor__.*` | records an `answer` from `tool_name`, `tool_input` and the node references in `tool_response` | nothing |
| `Stop`, `SessionEnd` | writes `<id>.summary.md` and `.json` beside the ledger | nothing |

The pack is planned once, for the first prompt that seeds anything, because a pack
per prompt would drown the conversation; a later task in the same session is served
by the `context_pack` tool, which the orientation line names. The budget is smaller
than a planned task's because it lands on every session whether or not the prompt is
a task; `[harness] prompt_budget` in `surveyor.toml` changes it and, unlike the other
profile sections, does not change the engine version, so editing it never forces a
full reindex.

`examples/claude-code/settings.json` holds the four hooks. In a repository that
keeps its dataset under `.surveyor` and has `surveyor` on `PATH`, its `hooks` block
goes into `.claude/settings.json` as it is. Causeway ignores `.claude/` in git and
keeps its dataset under `~/Library/Application Support/Surveyor/`, so its hooks live
in `.claude/settings.local.json` with the variables inline, replacing the
`SessionStart` hook that runs `surveyor update` today:

```json
"command": "SURVEYOR_STORE='/Users/<you>/Library/Application Support/Surveyor/causeway' SURVEYOR_PROFILE='/Users/<you>/Library/Application Support/Surveyor/causeway.toml' surveyor hook claude-code"
```

The `PostToolUse` matcher assumes the server is registered as `surveyor` in
`.mcp.json`; a server registered under another name changes the prefix Claude Code
puts on its tool names, and the matcher with it.

## Another agent host

The adapter is the one place that knows Claude Code's field names. Another host
either gets its own `surveyor hook <host>` subcommand, which is a payload struct and
a match over its event names, or calls `surveyor session start`, `note` and
`summary` from its own hook scripts, which is what `examples/harness/open-pr.sh`
does around `surveyor pr`.

## `surveyor pr`

```bash
surveyor pr                              # against the merge base with the default branch
surveyor pr --base origin/main --out dir # CI: dir/pr.md and dir/pr.json, exit code kept
surveyor pr --diff change.diff --json    # a harness that already has the diff
surveyor pr --session "$SURVEYOR_SESSION" --no-gate
```

One command with two consumers: CI posts its Markdown as the pull request's comment
and reads its exit code; a harness script runs it before `gh pr create` and refuses
to continue on exit 1. Surveyor never calls GitHub. `examples/ci/github-pr-comment.yml`
shows the job (one comment per pull request, found again by its first line
`<!-- surveyor-pr -->` and edited in place) and `examples/harness/open-pr.sh` the
refusal; `docs/causeway-pr-comment.md` is the proposed Causeway diff.

**Input.** `--base <ref>` measures against the merge base of `HEAD` and the ref; the
default ref is the default branch (`origin/HEAD`, then `origin/main`, `origin/master`,
`main`, `master`). The working tree counts: an uncommitted edit is part of the
change, as `update` already treats it, so run `update` first (the report says when
the dataset is behind the tree). `--diff <file>` (or `-` for stdin) takes a unified
diff instead and reads only its headers.

**Mapping.** Each changed line maps to the innermost declaration whose span holds
it, the containment rule the linker's `enclosing` applies to references. A range over
an `impl` header and its first method names both; a range inside one method names
the method alone; a changed line with no declaration around it (a manifest, a
workflow, a design, a blank line between functions) names the file node, and a file
the dataset never indexed is listed as such. Deleted files are listed by path, and
symbols the change removed are named by extracting the base version of each
modified or deleted file and comparing declarations by kind and scoped name, which
a diff file cannot do, so it says so. A query literal is a declaration of its own,
so a changed `sqlx::query("...")` names the query beside the function around it.

**Affected.** From every changed node, `affected` at `--depth` 3 under the impact
filter and the floor `--min-confidence` 0.25, merged across the changed nodes (a node
keeps its strongest path, by confidence then hops; changed nodes are never their own
impact), then grouped: code symbols; tables, views and columns; contracts and topics;
workloads; tasks and workflows; documents and designs; files and packages. Each row
carries the node's own location, the depth, the confidence product, the edge kind
and the citation of the edge that brought it. The Markdown caps each group at `--rows` 40 and says how many
more the JSON holds. Groups no node of the dataset falls in are named as kinds the
graph cannot see rather than printed as empty headings.

**Invariants.** The rule run `check` performs, from the profile's `[rules] dirs` and
`--rules`, as a table with every finding's citation.

**Exit.** 0 when every `fail` rule passes, 1 when one fails, 2 when the report could
not be made (no dataset, a rule that does not load, an unknown base, an unreadable
diff). `--no-gate` forces 0 for a job that only comments; the text still says the
rules failed. `--out <dir>` writes `pr.md` and `pr.json`, prints one line on stderr
and nothing on stdout. `--session <id>` (or `SURVEYOR_SESSION`) embeds the ledger's
summary, so the comment carries what the agent consulted beside what it changed.
