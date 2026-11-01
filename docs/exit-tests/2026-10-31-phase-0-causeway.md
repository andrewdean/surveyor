---
title: "Phase 0 exit test: Surveyor against graphicle on Causeway"
date: 2026-10-31
status: passed
---

# Phase 0 exit test: Surveyor against graphicle on Causeway

The RFD's Phase 0 exit test reads: on twenty questions drawn from recent Causeway pull
requests, Surveyor matches or beats graphicle's `explain` and `affected`, and every
answer cites spans that exist. This document records the run on 2026-10-31.

**Result: passed.** Surveyor beat graphicle on nine questions, matched it on eight, and
both tools failed the same three. Graphicle beat Surveyor on none. All 285 spans
Surveyor cited exist and contain the text the edge claims. The run also surfaced five
defects in Surveyor, listed at the end. The two that mislead an agent most (undirected
paths by default, silent choice among same-named declarations) were fixed the same day;
the fix is noted under each.

## Setup

| Item | Value |
|---|---|
| Causeway tree | local checkout at `40f993d7` (2026-10-13), the same tree both tools indexed |
| Surveyor | `40b2e6d`, release build; cold index 4.3 s wall for 4,088 files into 58,550 nodes and 327,286 edges; 77 MB dataset; a query answers in 0.55 s including graph load |
| graphicle | 0.1.1, the app-managed Causeway project scanned 2026-10-31 23:17; 64,427 nodes and 149,983 edges, 98 percent extracted, 2 percent inferred |
| graphicle access | the MCP server Causeway's agents use (`graphicle mcp --tools essentials`): `get_node`, `get_neighbors`, `shortest_path`. The CLI's `explain` and `affected` need a `.graphicle/graph.json` the app-managed project does not write, so the MCP tools stand in: `get_node` plus `get_neighbors` for `explain`, incoming `get_neighbors` for `affected` at depth 1. The essentials set has no transitive impact tool |
| Surveyor access | the CLI: `explain`, `affected` (depth 3), `path` |

The local Causeway checkout is eighteen days behind `origin/main`, so "recent pull
requests" means the twenty most recent code-changing pull requests whose merge commit
is in the tree: 1009, 1008, 1004, 999, 996, 994, 992, 989, 986, 985, 984, 983, 982, 981
and 980, with two questions drawn from 992, 986, 981 and 980 and one from each of the
others; 1007 and 998 contributed the two Rhai questions. Docs-only pull requests (1006,
1001, 995) and the two whose code change was a single test file (993, 988) were skipped.
Each question names a symbol the pull request added or changed. Ground truth is the
source at that commit: `rg` for definitions and references, read by hand for paths.

Scoring: *beats* when one tool returns facts the other lacks and nothing false at depth 1;
*match* when both return the same correct facts; *both fail* when neither answers.
A tool that reports an ambiguous name honestly is not penalized for asking.

## The twenty questions

| # | PR | Question | Surveyor | graphicle | Verdict |
|---|---|---|---|---|---|
| 1 | 1008 | explain `LoginOrigin` | enum at `browser_auth/handlers.rs:51-56`, doc line, variants `Here`/`Move`, method `of`, impl | same members, no doc | match |
| 2 | 1004 | affected `PgPolicyRepo::complete_scan` | depth 1: `PolicyScanner::process_lease` (`main.rs:908`), five repo tests, the trait method; depth 2-3: `process_claimed_lease`, `run_queue`, `run_once` | asks to disambiguate three `complete_scan`; then lists what the method uses and no caller at all | Surveyor |
| 3 | 999 | explain `TmpfsSize` | struct `scratch.rs:36`, four methods, used by `DeclaredVolume`, `size`, `tmpfs`, `mib`, re-exported from `lib.rs:91` | asks to disambiguate against a phantom `tmpfssize` node with no file; then the same users minus the re-export | match |
| 4 | 996 | affected `markStoryReady` | `Viewer` at `Viewer.tsx:28` | the same, plus the import | match |
| 5 | 992 | explain `AdvisoryLockKey` | struct `advisory_lock.rs:118`, three methods, used by `sweep_lock`, `advisory_lock_key`, `try_session_lock`, `try_transaction_lock`, `unlock`, four imports | the same set | match |
| 6 | 992 | affected `advisory_lock_key` | depth 1: `PgPolicyRepo::claim_scans`, `refresh_effective_governance`, `sweep_lock`, `PgTermsRepo::sync_in_transaction`, one test: the five callers in source | one caller, the test in the same file; the four path-qualified callers in other crates are missing | Surveyor |
| 7 | 986 | affected `SearchSyncPass` | `search_sync_pass`, `run_search_sync_pass`, the worker's test import; then `run_search_sync_worker`, the worker's `run` | asks to disambiguate against a phantom node; then the two functions and the variants | Surveyor (transitive) |
| 8 | 986 | path worker `main` to `SearchSyncPass` | default search: a five-hop *lead* through another binary's `main` via `install_crypto_provider`; `--directed`: the correct seven-hop chain `main` → `run_job_to_exit` → `Job::run` → `SearchSyncWorker::run` (dispatch) → `run` → `run_search_sync_worker` → `search_sync_pass` → `SearchSyncPass`, every hop certain | "path exceeds max_hops=8 (10 hops found)" | Surveyor, with a defect (see D1) |
| 9 | 985 | affected `PeopleService::batch_get_parties` | one test caller | the same; graphicle's ambiguity list omits the `PeopleRepo` trait method and `PgPeopleRepo` impl that exist in `repo.rs`, while Surveyor's `query` lists all eleven | match (see D2) |
| 10 | 985 | explain `reconcile_in_transaction` | the `ResourceIdentityWrite` trait method and its twelve implementations by `DISPATCHES_TO`, each cited | asks to disambiguate eight; the trait method then shows its parameter types and no implementation | Surveyor |
| 11 | 984 | affected `workflow_reports` | depth 1: `RouteAccess::from_config` (`access.rs:113`) and the test at `tests.rs:240`, exactly the two callers | `from_config` only | Surveyor (depth 1); depth 2 carries name-matched noise (D3) |
| 12 | 984 | affected table `taxonomy_placement_proposals` | table node with 24 columns, constraint and index from migrations 0052 and 0058; zero dependents | table from migration 0052, its index, its foreign key to `containers`; no code | both fail: `placements.rs` reaches the table only through `format!("... {schema}.taxonomy_placement_proposals")` (D5) |
| 13 | 983 | explain `ImageSource` | enum `images.rs:20-28`, variants, used by `classify`, imported by `content.rs` | the same | match |
| 14 | 982 | explain `S3Credentials` | struct `garage.rs:132`, three methods, twelve users incl. `ObjectStoreWorkspace::from_env`, `TopologyFragment::ObjectStore` and two tests | seven users, two imports; `from_env` and the tests missing | Surveyor |
| 15 | 981 | affected `PrefixedSettings` (Python) | depth 1: the five subclasses `RelayWorkerSettings`, `DatabricksSettings`, `SnowflakeConnection`, `ExampleSettings` and a test class; then their configs and the relay functions that use them | an import and a document reference; no subclass | Surveyor |
| 16 | 981 | explain `relayGrpcConfigFromEnv` (TypeScript) | calls `serviceSettingsFromEnv`; `TESTED_BY`; four callers (`server.ts:22`, three test calls) | calls `serviceSettingsFromEnv`; one test, two imports | Surveyor |
| 17 | 980 | affected `CspNonce` | depth 1: `Source::write`, `ContentSecurityPolicy::header_value`, `SecurityHeaders::add`, the import, three impls | methods, `Render`, the same three users and the import | match at depth 1; Surveyor's depth 2 is wrong (D3) |
| 18 | 980 | path `security_headers` to `CspNonce::generate` | four certain hops ending at the call in `send` (`security_headers.rs:313`) | two hops: file imports `CspNonce`, which has method `generate` | match |
| 19 | 1007 | explain Rhai `resolve_otel` | no node | no node | both fail: `.mise/` is a hidden directory (D4) |
| 20 | 998 | affected Rhai `register_restate_handlers` | no node | no node | both fail (D4) |

Tally: Surveyor 9, match 8, both fail 3, graphicle 0.

## Citations

Every span Surveyor cited was opened and checked for the text the edge claims: the
subject's name at a call site, the declaration's name at a definition, the member's
name at a member line. 285 citations, 285 verified (274 by script, the remaining 11 by
eye where the script could not parse the line: variant lines such as `Here,` and path
hops whose span holds the callee's name). The raw answers are in
`raw/2026-10-31-phase-0-surveyor-answers.txt`.

Graphicle's citations were not re-verified; its `checked: true` flag says it compared
the span against the current file itself, and spot checks of ten lines agreed.

## Where Surveyor won, and why

Three capabilities account for all nine wins:

- **Calls through a trait object.** `repo.complete_scan(...)` and
  `causeway_db_core::advisory_lock_key(...)` are the common shape of a call in Causeway.
  Surveyor's `DISPATCHES_TO` edges and package-qualified path resolution follow them;
  graphicle's graph links a method to its struct and stops.
- **Inheritance and implementation as dependence.** `EXTENDS` in Python and
  `DISPATCHES_TO` in Rust make "what depends on this trait or base class" answerable.
- **Transitive impact with provenance.** `affected` walks three levels and labels each
  hop's derivation and confidence; the MCP essentials have one level.

## Defects in Surveyor found by this run

Ranked by how much they would mislead an agent:

- **D1. `path` searches undirected by default and reports leads as answers.** Question 8's
  default answer walked from the worker's `main` to `install_crypto_provider`, then
  *backwards* into another binary's `main`, and called every hop certain. `--directed`
  gave the right chain. **Fixed:** `best_path` takes a `PathMode` and returns a
  `FoundPath` with its `direction`; the default tries the directed search through every
  tier first and falls back to undirected only when none exists, and the CLI and MCP
  label such a result a lead with each reversed hop marked `<-`. Question 8's default
  answer is now the seven-hop directed chain.
- **D2. A name that matches several declarations is picked silently.** `affected
  batch_get_parties` chose the `PeopleRepo` trait method out of eleven candidates without
  saying so, and `explain nonexistent/file.rs::reconcile_in_transaction` ignored the file
  qualifier and answered for a different file. Graphicle refuses and lists the candidates,
  which is the right behavior for an agent caller. **Fixed:** `CodeGraph::lookup`
  returns `Exact`, `Unique`, `Ambiguous` (the tied candidates) or `NotFound` (with the
  same-named nodes a wrong qualifier excluded), and the CLI and MCP refuse the last two
  with the identifiers to repeat with. `affected batch_get_parties` now refuses and lists
  the candidates, and the misqualified `reconcile_in_transaction` is an error.
- **D3. Transitive `affected` fans out through common method names.** Question 17's depth 2
  listed every `fmt` call in the crate because `CspNonce` has a `Display` impl, and
  question 11's depth 2 reached `ValkeyCache::connect` through an unrelated `from_config`.
  Depth 1 is clean because those edges carry their own site; the damage is at depth 2 and
  3, where a `[syntactic 0.33]` hop is treated like a certain one. Two fixes compound:
  stop walking through trait-standard names (`fmt`, `from`, `new`, `default`, `clone`,
  `eq`, `hash`, `drop`) unless the edge is `Resolved`, and multiply confidence along the
  walk so a 0.33 hop followed by a 0.5 hop ranks below the threshold. **Fixed**, at the
  linker rather than the walk, once the offending edges were read: every `fmt` reached
  `Source::write` through `write!`, a macro extracted as a call named `write` and linked
  at confidence 1 to the only `write` in the package; `ValkeyCache::connect` reached
  `RouteAccess::from_config` through `Builder::from_config`, whose qualifier matched
  nothing and fell back to every candidate; and `writer.write(&batch)` linked to the
  same `Source::write` at confidence 1 because the by-name method rule only discounted
  calls into other packages. Macros are now a call form of their own that never
  resolves to a function; a qualifier that names no declaration is a
  `qualifier-mismatch` gap; and a reference is certain only when anchored (same file,
  matching qualifier, or an import of the file), so a standard-library name resolves
  only when anchored and a method call by bare name is 0.75 nearby and 0.5 remote.
  `affected` multiplies confidence along the path, shows it, and drops paths below
  `--min-confidence` (default 0.25). Re-indexed on 2026-11-01: question 17's depth 2 is
  down from 23 nodes to the two callers in `security_headers.rs`, question 11's depth 2
  is exactly `Server::configure`, and question 8's directed chain is unchanged. Across
  Causeway the change removed 72,168 edges (`CALLS` 141,570 to 84,007) and 6,470 gaps
  (44,594 to 38,124); 4,748 references are now `qualifier-mismatch`, led by
  `sqlx::query`, `Utc::now`, `from_str` and `Uuid::new_v4`, each of which had been an
  edge at confidence 1 to the one local function of that name.
- **D4. Hidden directories are skipped.** `walk.rs` sets `.hidden(true)`, so Causeway's
  161 Rhai files under `.mise/` are not indexed and the two Rhai questions could not be
  asked. The profile needs a `[walk] include = [".mise/**"]` list, and Causeway's profile
  needs the entry.
- **D5. Runtime SQL with a schema placeholder is not resolved.** `placements.rs` names
  its table as `{schema}.taxonomy_placement_proposals` inside `format!`; the placeholder
  becomes `__p.taxonomy_placement_proposals` and the resolver, finding no schema `__p`,
  draws nothing. When the schema is a placeholder the table should resolve by bare name,
  with `confidence 1/n` across schemas that declare it. This is the light dataflow
  the RFD's Phase 1 plan already names, and it is the one question class where a
  Causeway reader would have been better served by grep.

## What this does not show

The test compares graph answers, not agent outcomes. Graphicle also carries project
memory, documents and session capture that Surveyor does not have and this test did not
touch. Graphicle's CLI `affected` (transitive) was unavailable because the app-managed
project writes no `graph.json`; a `graphicle graph extract` into a scratch directory
would allow that comparison and was not run.
