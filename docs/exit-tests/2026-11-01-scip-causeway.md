---
title: "The precise tier on Causeway: what rust-analyzer's SCIP index costs and closes"
date: 2026-11-01
status: measured
---

# The precise tier on Causeway: what rust-analyzer's SCIP index costs and closes

The RFD's open question on SCIP (section 17) asks whether Surveyor should require an
external indexer, recommend one, or stay name-only. The Phase 0 exit test and the D3
work left the name tier at a known precision: a bare-name method call is a guess at
0.75 or 0.5, and 29,372 Rust references on Causeway found no declaration. This note
measures what `rust-analyzer scip` costs on Causeway and what it changes, so the
section 17 decision rests on numbers from the first adopter rather than on Cairn's
228 files.

**Result.** The index costs 26.5 minutes of wall clock, one workspace at a time, at
8.1 GB peak memory, and 94.5 MB of index files. With it, Surveyor's Rust graph goes
from 1.5% resolved edges to 61%, Rust gaps fall from 29,372 to 5,781, the 1/n
fan-out edges all but disappear, and every exit-test answer that carried a 0.75 lead
now carries a resolved edge. Running it also found two defects in Surveyor's precise
tier that only a monorepo shows; both are fixed in the commits this note accompanies.

## Setup

Causeway at commit `40f993d7`, indexed with the profile under
`examples/causeway-profile/`, as in the D4 and D5 runs. Causeway has no root
`Cargo.toml`; it holds fourteen Cargo workspaces (`crates/`, `component/`, `apps/`,
`services/`, `jobs/`, `tools/`, `tools/xtask/`, `bootstrap/`, `torrentfx/` and five
under `ffi/`). `rust-analyzer scip` loads one project, so I ran it once per
workspace, sequentially, with `CARGO_TARGET_DIR` pointed outside the checkout so
nothing was written into Causeway; that directory stayed empty, so rust-analyzer
reused existing check artifacts or ran no build scripts, which I did not verify.
The toolchain was rust-analyzer 1.95.0 on a 128 GB machine with a load average
between 7 and 10 during the run.

| Workspace | Wall clock (s) | Peak RSS (GB) | Index (MB) |
|---|---|---|---|
| `apps` | 294 | 7.3 | 17.6 |
| `tools` | 266 | 6.6 | 1.9 |
| `component` | 254 | 5.5 | 23.1 |
| `crates` | 185 | 7.4 | 33.7 |
| `torrentfx` | 151 | 1.9 | 3.3 |
| `bootstrap` | 145 | 2.0 | 3.0 |
| `jobs` | 107 | 7.6 | 3.1 |
| `services` | 45 | 5.2 | 3.7 |
| `ffi/py-drive` | 32 | 2.9 | 0.0 |
| `tools/xtask` | 28 | 0.6 | 0.2 |
| `ffi/wasm-ws-mock` | 22 | 1.0 | 0.1 |
| `ffi/wasm-prism` | 21 | 0.9 | 0.0 |
| `ffi/wasm-dbt-demo` | 21 | 2.1 | 0.2 |
| `ffi/node-drive` | 19 | 2.9 | 0.1 |

Table 1: rust-analyzer scip per workspace. Total 1,590 s; the four large workspaces
account for 63% of the time and 81% of the bytes. The logs hold hundreds of
`Encountered enclosing definition with no name` errors from rust-analyzer; every
index parsed and every document matched a walked file.

Surveyor then indexed Causeway with the thirteen non-empty indexes passed as
`--scip` arguments. That index took 3.9 s against 4.6 s for the name-only index of
the same tree, because the linker ranks fewer candidates when the index has already
named the target. Peak memory was 1.7 GB.

## Two defects the monorepo exposed

The first run resolved nothing across workspaces. A SCIP index lists its documents
relative to the project it was built from, so `src/main.rs` in the index of `apps/`
never matched the walker's `apps/src/main.rs`; `ScipExtractor::load_under` now
derives the prefix from the index's `project_root` and re-roots every document and
declaration. With that fixed, the second run still lost 14,655 of the 15,881 strong
cross-workspace edges the name tier had drawn, because an index names a declaration
in another workspace as an external symbol, and Surveyor treats an external symbol as
neither an edge nor a gap. The two indexes name that declaration by the same symbol
string (`rust-analyzer cargo catalog-component 0.1.0 placements/impl#[PgPlacementRepo]propose().`
appears identically in the `apps` and `component` indexes), so `load_all` now pools
every index's declarations into one extractor. With pooling, 16,673 cross-workspace
edges resolve and only 1,220 of the strong name-matched ones are gone.

## What the index changes

The counts below cover edges whose source is a `.rs` file and gaps recorded in
`.rs` files; the rest of the graph (TypeScript, Python, SQL, Rhai, manifests) is
untouched by a Rust index.

| | name only | with SCIP |
|---|---|---|
| edges from `.rs` files | 222,027 | 197,198 |
| of which `resolved` | 3,244 | 119,439 |
| `CALLS` at 1.0 | 43,901 | 57,495 |
| `CALLS` at 0.75 | 7,593 | 455 |
| `CALLS` at 0.5 | 5,505 | 703 |
| `CALLS` below 0.5 (shared 1/n) | 19,731 | 1,739 |
| `USES_TYPE` at 1.0 | 36,243 | 35,437 |
| `USES_TYPE` below 1.0 | 5,375 | 1,366 |
| `IMPLEMENTS` resolved | 0 | 439 of 464 |
| gaps in `.rs` files | 29,372 | 5,781 |
| `no-declaration` | 19,306 | 4,026 |
| `qualifier-mismatch` | 4,748 | 820 |
| `ambiguous` | 4,153 | 649 |
| `ambiguous-receiver` | 1,039 | 160 |
| `no-table` | 126 | 126 |

Table 2: Rust edges and gaps on Causeway without and with the pooled SCIP indexes.
Whole-graph totals went from 263,517 edges and 40,646 gaps to 238,688 and 17,055.

The edge count falls because the name tier drew 19,345 `CALLS` and `USES_TYPE`
targets the index does not confirm, most of them 1/n fan-out among same-named
declarations, and the index adds 3,650 targets the name tier never found. Of the
3,547 strong (1.0 or 0.75) name-matched edges that disappear, 1,176 now point at a
different declaration of the same name, which is the name tier being wrong with a
straight face: `Metrics::new` in `torrentfx` linked to `Registry::new` in
`causeway-code` at 1.0, `extract_zip` linked to a sibling `extract`, a test's call
to `SeededGovernanceRead::list_policy_subjects` resolves to the trait method the
test actually invokes. The 4,683 edges the index adds are mostly method calls the
name tier silenced as standard-library names (`as_str`, `span`, `key`, `content`)
and that the index attributes to Causeway's own types.

The gaps that remain in Rust are concentrated in generated and macro-produced code:
`fetch_one` (sqlx), `pp`, `Publisher`, `Integer`/`Boolean`/`Bool` (Pure model
builders), `parse_oql`. The whole-graph top gaps (`messageDesc`, `MethodInfo`,
`Message`, `ConnectError`, `execute_unary`) are in generated TypeScript, which a
Rust index cannot touch and `scip-typescript` would.

## The exit-test questions

Every answer that carried a lead now carries a resolved edge, and one answer gained
a caller the name tier missed:

- `affected taxonomy_placement_proposals --depth 2` (question 12): depth 2 held
  `load_placements` and `propose` at 0.75; both are now `[resolved]`, and `decide` in
  the admin host (`placement.rs:90`) joins them. The name tier had not linked that
  call at all.
- `affected CspNonce --depth 2` (D3's case): the same nine nodes, the `USES_TYPE`
  edges now resolved, and the depth-2 test reached at 1.0 instead of 0.38.
- `affected workflow_reports --depth 2`: the same three nodes, all resolved.
- `explain PgPlacementRepo::propose`: five callers, the admin host's among them,
  where the name tier showed only the four tests in the same file.

## What this means for section 17

An adopter the size of Causeway pays about half an hour of machine time and 8 GB of
memory for a Rust index, once per workspace, and gets a graph in which three fifths
of the Rust edges are resolved and four fifths of the Rust gaps are gone. The cost
does not fit inside `surveyor update`: rust-analyzer indexes a whole workspace or
nothing, so the index refreshes on its own cadence (a nightly job, or a CI step on
the default branch) and the syntactic tier keeps the graph current between
refreshes. Surveyor already handles that split: the engine stamp hashes the index
files, so a new index forces a full re-index, which is 4 s here, and `update` runs
without the index in between.

***Recommendation:*** make the precise tier the documented default for Rust
adopters without requiring it: `surveyor index` accepts the indexes when present and
the profile's `[scip] indexes` names where they live; the Causeway profile should
list the fourteen workspace indexes under a directory the adopter's scheduled job
writes. Do not bundle rust-analyzer; pin the version in the profile's documentation
and record it in the index metadata Surveyor already logs. Measure `scip-typescript`
next, since the largest remaining gaps are in generated TypeScript.
