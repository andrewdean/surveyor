# The agent harness: session ledgers and hooks

A session ledger records what Surveyor told an agent, so that what the agent later
claims to have read can be checked against what it was shown. This document says
what the ledger holds, how a command or a hook feeds it, and how to wire it into
Claude Code. The pull-request comment that reads the ledger is `surveyor pr`
(stage 2 of Phase 3); the RFD's section 12.2 is the design this implements.

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
structure `surveyor pr --session` will embed. `--write` also leaves
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
`summary` from its own hook scripts, which is what the `examples/harness/`
scripts of stage 2 do.
