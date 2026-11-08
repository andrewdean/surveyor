# Team mode: `surveyor serve`

One Surveyor on a host the team reaches, one dataset that CI keeps current, and every
agent and script reading from it over HTTP instead of each developer indexing and
running a binary of their own. `serve` puts the existing answers behind a port; it
owns no new state and adds no table. This document says what it serves, how a
dataset write reaches it, where its boundaries are, and what it costs. The RFD's
Table 7 names it as the Phase 3 stage that makes Surveyor a shared service.

## What it holds

`serve` opens the dataset `--store` names and, for every repository the dataset holds
(or the one `--repo-id` names), loads the graph as a query would: the repository's
partition plus the federated edges that touch it and their far endpoints, or every
partition with `--all-repos`. It builds the text index, loads the rules from the
repository's profile, runs them once for the cards, and keeps all of that in memory
behind two fronts.

A repository's checkout, for `source` and the source windows in context packs, comes
from `--root <repo>=<path>`, repeatable. A dataset of one repository takes
`--repo-root` as that checkout, which is the common `surveyor serve` from inside a
repository. A repository without a checkout serves every answer but `source`
(`409`) and `read_project_source` (an MCP error naming the `--root` to pass); its
context packs carry cards and citations without source windows, and it has no rules,
because rules come from the profile under its root.

## The JSON API

Every route under `/v1` returns the structure the CLI's `--json` prints for the same
command, built by the same functions, so a client reads one schema whether it
shelled out or called over HTTP. Table 1 lists them; a parameter's default is the
CLI's.

Table 1. Routes and the commands they answer as.

| Route | Answers as | Parameters |
|---|---|---|
| `GET /v1/repos` | (the server's own) | |
| `GET /v1/{repo}/stats` | `stats --repo-id {repo}` | |
| `GET /v1/{repo}/explain` | `explain <q>` | `q`, `kind` |
| `GET /v1/{repo}/card` | `card <q>...` | `q` (comma-separated), `kind` |
| `GET /v1/{repo}/affected` | `affected <q>` | `q`, `kind`, `depth` (3), `min_confidence` (0.25), `kinds` (comma-separated), `all_derivations` |
| `GET /v1/{repo}/path` | `path <from> <to>` | `from`, `to`, `mode` (`auto`, `directed`, `undirected`) |
| `GET /v1/{repo}/query` | `query <q>` | `q`, `kind`, `limit` (20) |
| `GET /v1/{repo}/search` | `search <q>` | `q`, `limit` (10) |
| `GET /v1/{repo}/context` | `context <task>` | `task`, `budget` (30000), `seeds` (comma-separated) |
| `GET /v1/{repo}/source` | `source <target>` | `target`, `context` (2) |
| `GET /v1/{repo}/check` | `check` | |

`/v1/repos` lists what the server holds: each repository's snapshot (commit, counts,
when it was indexed and by which Surveyor), its checkout when the server has one, how
many rules it loaded, the dataset generation it was loaded at and when, and where its
MCP service is mounted.

A refused request answers `{"error": "<one line>"}`: `400` for a name that resolves
to no node or to several (the message lists them, as the CLI does), an unknown kind
or mode; `404` for a repository the server does not hold or a `check` on one without
rules; `409` for `source` on a repository without a checkout; `401` without the
bearer token. `check` answers `200` whether the rules passed or not; the report says
which failed, and the exit code of `surveyor check` is the CLI's way of saying it.

Graph walks run on a blocking thread, so a long `affected` never stalls another
request or the reload.

## MCP over streamable HTTP

`/mcp/{repo}` mounts rmcp's streamable HTTP service, one per repository, over the
same server state the stdio server uses, so the tool names and schemas are the stdio
server's and a team's `.mcp.json` points at the URL instead of the binary
(`examples/serve/mcp.json`). Sessions are kept in memory by rmcp's local session
manager. The service validates the `Host` header against loopback by default, as the
MCP transport specification asks, to stop DNS rebinding against a local server;
behind a proxy, pass the proxy's name with `--host`.

A session's handler reads the repository's current state on every call, so a session
opened before a reload answers from the new graph after it.

## Reload

A task polls the dataset's generation, the sum of every table's Lance version, every
`--reload-every` seconds (30 by default; 0 never). Lance versions only rise, so any
write moves the sum, whichever table it touched: `update`, `index` and `federate`
alike. When it moved, the task reloads every repository (graph, index, rules, the rule
run) and swaps each one in whole; a request already answering finishes on the state it
started with. A repository that appears in the dataset after the server started is
logged and needs a restart, because its MCP mount is part of the router.

Indexing on push through a repository store synced with `gix`, which the RFD's
section 8.2 describes for the team profile, stays out: the adopter runs `update` into
the shared dataset from CI, as Causeway's job already does into its cache.

## Boundaries

The listener binds `127.0.0.1:7700` unless `--listen` says otherwise. `--token`
(or `SURVEYOR_TOKEN`) requires `Authorization: Bearer <token>` on every request, the
MCP endpoint included, compared in constant time. TLS is the reverse proxy's job in
this release, which keeps `openssl` out of the tree: nothing in the lock file names
it, and the HTTP stack is axum over hyper with no TLS feature. A session ledger
(`--session`) records every answer the server gives, from every client, in one
ledger; per-client ledgers are a question for the harness work, not the server.

```bash
surveyor --store /srv/surveyor/causeway serve \
  --listen 0.0.0.0:7700 --root causeway=/srv/checkouts/causeway \
  --token "$SURVEYOR_TOKEN" --host surveyor.internal
curl -s -H "Authorization: Bearer $SURVEYOR_TOKEN" \
  "http://surveyor.internal:7700/v1/causeway/affected?q=DbError&depth=3"
```

## Measured

The test (`apps/surveyor/tests/serve.rs`) starts `serve` on an ephemeral port over
the `mini` fixture and compares every JSON route with the CLI's `--json` for the same
command and every MCP tool over streamable HTTP with the stdio server's answer, byte
for byte after parsing; it then gates the port with a token, writes an `update` into
the dataset and reads the new symbol through a route and through an MCP session
opened before the reload, and serves a second repository without a checkout.

On the Causeway scratch store (release build, name tier, `docs/exit-tests/raw/2026-11-08-phase-3-serve.txt`):

Table 2. Causeway (67,771 nodes, 292,106 edges, 5 rules) served from a warm process: one warm-up request, then twenty timed with curl; the answer's size is the warm-up's.

| Route | min / median / max (ms) | Answer |
|---|---|---|
| `/v1/repos` | 0.5 / 0.6 / 4.3 | 538 B |
| `/v1/causeway/stats` | 345 / 366 / 392 | 1.2 KB |
| `/v1/causeway/explain?q=DbError` | 7.9 / 9.1 / 24.5 | 731 KB |
| `/v1/causeway/card?q=DbError` | 4.9 / 6.0 / 6.7 | 2.0 KB |
| `/v1/causeway/affected?q=DbError&depth=3` | 11.2 / 12.8 / 17.7 | 1.48 MB |
| `/v1/causeway/affected` on the top hub, a document | 0.6 / 0.7 / 3.2 | 355 B (nothing depends on a document) |
| `/v1/causeway/path` between the two top hubs | 0.4 / 0.6 / 0.9 | 1.5 KB |
| `/v1/causeway/query?q=outbox` | 4.7 / 5.4 / 6.1 | 7.2 KB |
| `/v1/causeway/search` (a sentence) | 0.5 / 0.6 / 0.7 | 5.7 KB |
| `/v1/causeway/context` at 8,000 tokens | 46.6 / 56.2 / 60.9 | 76 KB |
| `/v1/causeway/source` on the top hub | 1.3 / 2.0 / 3.1 | 338 KB |
| `/v1/causeway/check` | 0.3 / 0.3 / 0.5 | 937 B (the kept rule run) |
| MCP `get_node` over streamable HTTP | 4.7 / 5.6 / 8.5 | |

`stats` is the slow route because it counts from the store as the CLI does, so its
numbers are the CLI's; every other route answers from memory. The explain and
affected answers are large because `DbError` is the enum every database call in
Causeway returns, 1,866 nodes within three hops; the time is the serialization.
Eight clients sending 25 `explain` requests each finish in 581 ms, 343 requests a
second, through the blocking pool.

Load and reload: the server loads Causeway's graph, text index and rules in 1.06 s
at start. A line added to a document and written by `update` (1.3 s) reached the
server 1.0 s after its next poll saw the new generation, 2.0 s after `update`
returned with a 2 s poll; a session opened before the reload answers from the new
graph. The record is `docs/exit-tests/raw/2026-11-08-phase-3-serve.txt`, made by
the script beside it.
