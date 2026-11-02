---
title: "The precise tier on Causeway, TypeScript: what scip-typescript costs and closes"
date: 2026-11-01
status: measured
---

# The precise tier on Causeway, TypeScript: what scip-typescript costs and closes

The Rust measurement (`2026-11-01-scip-causeway.md`) settled section 17 of the RFD for
Rust adopters: the precise tier is the documented default, produced by a scheduled or
CI job, never required or bundled. It left the largest whole-graph gaps untouched,
because they were generated code outside Rust, and named `scip-typescript` as the next
measurement. This note measures what `scip-typescript` costs on Causeway's TypeScript
and what it changes, so the same decision can be made for TypeScript adopters on
numbers from the first adopter.

**Result.** The index costs 33 seconds of wall clock for all thirty-seven TypeScript
projects, one at a time, under 0.75 GB peak memory, and 10.1 MB of index files. With
it, Surveyor's TypeScript graph goes from 4 resolved edges to 5,202 (46% of the
TypeScript edges), TypeScript gaps fall from 4,155 to 995, two thirds of what remains
is in files no `tsconfig.json` covers, and both exit-test answers that touch
TypeScript now carry resolved edges. The index exposed one extractor defect, which is
fixed in the commit this note accompanies: `scip-typescript` leaves the position
encoding unspecified and counts columns in UTF-16 code units, which the extractor
read as UTF-8. Of the five generated-code names the Rust note pointed at, only
`messageDesc` is TypeScript; `MethodInfo`, `ConnectError` and `execute_unary` are
generated Python, and `Message` is Python stubs, so they are the case for measuring
`scip-python`, not for anything more in TypeScript.

## Setup

Causeway at commit `40f993d7`, indexed with the profile under
`examples/causeway-profile/`, as in the Rust run. The walker indexes 452 TypeScript
files (386 `.ts`, 66 `.tsx`). Causeway is one pnpm workspace whose `package.json`
globs name eleven package directories; thirty-seven of those packages carry a
`tsconfig.json` (the stale worktree under `.claude/` carries copies, which the profile
already keeps out). Five `bindings-ts` packages (`causeway-session`, `-request-ui`,
`-roadmap-ui`, `-scaffold-ui`, `-taxonomy-ui`) have `node_modules` but no
`tsconfig.json`, so no project covers them.

`scip-typescript` type-checks, so it needs the dependencies installed. Causeway's
`node_modules` were already in place from pnpm, so nothing was installed into the
checkout; `git status` listed the same eight pre-existing entries before and after the
run. The indexer was `@sourcegraph/scip-typescript` 0.6.0 under Node 25.2.1, installed
in a scratch directory, run once per project from that project's directory with
`--output` pointing outside the checkout. Node's heap limit was raised to 12 GB and
never approached. The machine is the Rust run's 128 GB machine.

| Project | Wall clock (s) | Peak RSS (MB) | Index (MB) | Documents |
|---|---|---|---|---|
| `services-node/relay-grpc` | 1.0 | 474 | 2.65 | 93 |
| `torrentfx/web/framework/runtime-web` | 1.6 | 713 | 1.34 | 23 |
| `apps-web/showcase` | 2.0 | 729 | 1.07 | 64 |
| `torrentfx/packages-node/causeway-platform` | 1.1 | 491 | 0.94 | 26 |
| `bindings-ts/causeway-catalog` | 0.4 | 346 | 0.52 | 20 |
| `torrentfx/packages-node/causeway-relay` | 1.0 | 489 | 0.38 | 11 |
| `web/ui-semantic-mapping` | 1.3 | 690 | 0.35 | 5 |
| `web/ui-query` | 1.5 | 752 | 0.29 | 10 |
| `packages/ui-components` | 0.8 | 402 | 0.28 | 10 |
| `web/ui-create-product` | 1.2 | 681 | 0.26 | 5 |
| `web/ui-semantic-graph` | 1.3 | 680 | 0.19 | 5 |
| `web/ui-request` | 1.2 | 669 | 0.17 | 8 |
| `torrentfx/web/framework/core` | 0.7 | 455 | 0.16 | 7 |
| `web/ui-roadmap` | 1.3 | 688 | 0.15 | 5 |
| `extensions/vscode-causeway-catalog` | 0.5 | 378 | 0.14 | 10 |
| `extensions/vscode-causeway-contracts` | 0.5 | 383 | 0.12 | 3 |
| `web/ui-catalog-filter` | 1.2 | 673 | 0.11 | 5 |
| `web/ui-data-grid` | 0.9 | 494 | 0.10 | 4 |
| `torrentfx/web/framework/app` | 1.2 | 662 | 0.09 | 2 |
| `extensions/vscode-causeway-storefront` | 0.3 | 210 | 0.09 | 4 |
| `web/ui-catalog-status` | 1.1 | 667 | 0.08 | 4 |
| `web/ui-ontology` | 1.1 | 668 | 0.08 | 3 |
| `bindings-ts/causeway-semantic` | 0.4 | 255 | 0.07 | 4 |
| `web/ui-dbt-demo` | 0.8 | 477 | 0.07 | 4 |
| `extensions/vscode-causeway-pure` | 0.7 | 435 | 0.07 | 5 |
| `web/ui-explorer-codecs` | 1.1 | 667 | 0.05 | 2 |
| `web/ui-presence` | 1.1 | 651 | 0.05 | 3 |
| `bindings-ts/causeway-catalog-ui` | 0.4 | 261 | 0.04 | 3 |
| `jobs-node/relay-worker` | 0.8 | 421 | 0.04 | 2 |
| `bindings-ts/causeway-explorer-ui` | 0.3 | 258 | 0.04 | 2 |
| `services-node/roadmap-relay` | 0.9 | 509 | 0.03 | 3 |
| `services-node/relay-restate` | 0.7 | 430 | 0.03 | 3 |
| `torrentfx/web/framework/runtime-vscode-webview` | 0.9 | 569 | 0.03 | 1 |
| `torrentfx/web/framework/runtime-vscode-host` | 0.3 | 257 | 0.02 | 1 |
| `web/ui-devtools` | 0.9 | 564 | 0.02 | 1 |
| `torrentfx/web/framework/adapters-wasm` | 0.3 | 256 | 0.01 | 1 |
| `torrentfx/web/framework/adapters-react` | 0.3 | 255 | 0.01 | 1 |

Table 1: scip-typescript per project, sorted by index size. Total 33.1 s, 10.1 MB, 363
documents and 13,976 definitions; the largest project, `relay-grpc`, is the generated
Connect client code. Every index parsed, every document matched a walked file, and no
project took more than two seconds or 0.75 GB.

Surveyor then indexed Causeway with the thirty-seven indexes passed as `--scip`
arguments: 4.7 s against 4.8 s for the name-only index of the same tree, at 1.5 GB
peak memory.

## Roots, paths and one encoding defect

`scip-typescript` writes `project_root` as the `file://` URI of its working directory
and lists every document relative to it, so an index built from
`web/ui-query` names `src/decider.ts` under
`file:///Users/andrew/dev/trusted/causeway/web/ui-query`. That is the shape the Rust
run taught `ScipExtractor::load_under` to expect, and the re-rooting matched without
change: no document path starts with `..`, because the indexer only emits the files
in a project's own `fileNames`, and all 363 re-rooted documents are files the walker
indexed. Declarations pool across the indexes as they did for Rust, because a
workspace package exports its `src/*.ts` directly and both the exporting and the
importing project name a symbol by the same package and path; 114 resolved edges
cross package boundaries, such as the showcase's `CatalogSearchDemo.tsx` using
`CatalogEntry` from `bindings-ts/causeway-catalog`'s generated `entry_pb.ts`, and
`main.tsx` using `BrowserBus` from `torrentfx/web/framework/runtime-web`.

The defect was in columns. Every document in these indexes leaves
`position_encoding` unspecified, and the extractor took unspecified to mean UTF-8,
which is right for rust-analyzer. `scip-typescript` takes its positions from the
TypeScript compiler, which indexes JavaScript strings, so its columns are UTF-16 code
units. The two agree on any line that is ASCII up to the occurrence, which is why the
first run looked right; on the 105 occurrences (of 97,596) that follow a non-ASCII
character, UTF-8 sliced the wrong text every time (`data` read as `: da`) and UTF-16
the right text every time. `Encoding::of` now takes the indexer's name and assumes
UTF-16 for `scip-typescript` when a document declares nothing; a unit test places an
identifier after an em dash. The fix moved seven edges from syntactic to resolved and
removed one gap, which is the size the count predicted.

## What the index changes

The counts below cover edges whose source is a `.ts` or `.tsx` file and gaps recorded
in such files; the rest of the graph is untouched by a TypeScript index.

| | name only | with SCIP |
|---|---|---|
| edges from `.ts`/`.tsx` files | 11,843 | 11,245 |
| of which `resolved` | 4 | 5,202 |
| `CALLS` at 1.0 | 3,271 | 3,432 (2,895 resolved) |
| `CALLS` at 0.75 | 65 | 22 |
| `CALLS` at 0.5 | 333 | 67 |
| `CALLS` below 0.5 (shared 1/n) | 545 | 251 |
| `USES_TYPE` at 1.0 | 1,683 | 1,694 (1,565 resolved) |
| `USES_TYPE` below 1.0 | 114 | 0 |
| `TESTED_BY` resolved | 0 | 724 of 863 |
| `EXTENDS` and `IMPLEMENTS` resolved | 0 | 14 of 14 |
| gaps in `.ts`/`.tsx` files | 4,155 | 995 |
| `no-declaration` | 4,011 | 941 |
| `ambiguous` | 97 | 40 |
| `ambiguous-receiver` | 46 | 13 |

Table 2: TypeScript edges and gaps on Causeway without and with the pooled
scip-typescript indexes. Whole-graph totals went from 263,506 edges and 40,539 gaps to
262,908 and 37,379.

The edge count falls by 598 because the name tier drew 684 `CALLS`, `USES_TYPE` and
`TESTED_BY` targets the index does not confirm, most of them 1/n fan-out among
same-named declarations, and the index adds 153 the name tier never found. Of the 165
strong (1.0 or 0.75) name-matched edges that disappear, 89 now point at a different
declaration of the same name: `App.tsx` used the `Selection` it declares itself at
line 10, not `Sidebar.tsx`'s, and `mock-websocket.ts` its own `Direction` and
`MockFrame`, not the copies in `mock-websocket-js.ts`. The 153 edges the index adds
are mostly calls through parameters and imported values the name tier had silenced or
spread thin (`dispatch`, `bind`, `subscribe`) and types it could not place
(`Environment`, `ResolvedConfig`, `DependencyCriticality`).

The 995 gaps that remain split by whether a project covers the file. 663 are in the 89
walked TypeScript files that no `tsconfig.json` reaches: ten Playwright specs under
`apps/causeway-server`, a Rust application with no TypeScript project (394 gaps, all
Playwright's `locator`, `toBeVisible`, `goto`, `getByRole`); 31 `vite`, `vitest` and
`playwright` config files outside their package's `include: ["src"]` (39); four
`tests/` directories outside `include` (48); the five `bindings-ts` packages without
a `tsconfig.json` (34, among them the last 23 `messageDesc`); eleven test, script and
declaration files outside their project's file list (148); and 23 `ts-rs` bindings under
`torrentfx/crates`, which have no references and no gaps. The other 332 are in
covered files, and the index does resolve them: it names the target as a parameter or
a member of an inline type literal
(`Checkbox().(props)typeLiteral0:onChange.`, `runCommand().(dispatch)`), which
Tree-sitter never declares as a node, so the linker finds nothing at the file and line
the fact names and falls back to a syntactic gap. Treating a declared target with no
node at its site the way a local binding is treated would retire those; it is a
linker change, noted here and not made.

## The generated-code gaps

The Rust note read the whole-graph top gaps as generated TypeScript. One of the five
is:

- `messageDesc` (1,112, the `@bufbuild/protobuf` codegen helper): 1,089 become
  external symbols, which Surveyor records as neither edge nor gap; the index names
  them `npm . . codegenv2/message.d.ts/messageDesc()`, the package and version blank
  because the indexer could not pair the subpath export with a `package.json`. The
  remaining 23 are in the five `bindings-ts` packages without a `tsconfig.json`.
  `enumDesc`, `fileDesc`, `serviceDesc` and `extDesc` go the same way.
- `MethodInfo` (964), `ConnectError` (488) and `execute_unary` (472) are generated
  Python: the `*_connect.py` clients under `services-py/relay-grpc` and
  `services-py/unity-catalog-relay`, with a few in `torrentfx/packages-py`. Unchanged.
- `Message` (920) is 908 references in generated `.pyi` stubs in the same Python
  services and `torrentfx/packages-py`, and 12 in Rust. Unchanged.

After this index the whole-graph top gaps are `bind` (1,578, Rust), `MethodInfo`,
`pin` (Rust), `Message`, `query` and `expect_err` (Rust), then `ConnectError` and
`execute_unary`: four of the top nine are generated Python, and the Rust ones are
the macro and standard-library names the Rust note already listed.

## The exit-test questions

Two Phase 0 questions touch TypeScript, and both now carry resolved edges with the
same nodes as before:

- `affected markStoryReady` (question 4): the one dependent, `Viewer` at
  `Viewer.tsx:28`, now `[resolved]`.
- `explain relayGrpcConfigFromEnv` (question 16): the call to
  `serviceSettingsFromEnv`, the `RelayGrpcConfig` type and the caller at
  `server.ts:22` are resolved. The four test callers in `tests/config.test.ts` stay
  syntactic, because `relay-grpc`'s project includes `src/**` and `gen/**` and not
  `tests/`, which is the coverage rule above seen from one answer.

## What this means for section 17

A TypeScript adopter the size of Causeway pays half a minute and under a gigabyte for
an index, provided its dependencies are installed, which is the real precondition:
`scip-typescript` type-checks, so it runs where `pnpm install` has run, which is the
CI job, not the developer's `surveyor update`. Coverage follows each `tsconfig.json`'s
`include`: tests and config files outside it get nothing, and a package without one
gets nothing, so an adopter chooses coverage by the project files it already
maintains. The index is small enough and fast enough to run on every default-branch
build rather than nightly, and Surveyor's engine stamp already forces the 4.7 s full
re-index when it changes.

***Recommendation:*** the Rust decision holds for TypeScript: the precise tier is the
documented default, produced by a CI job that runs a pinned
`@sourcegraph/scip-typescript` once per `tsconfig.json` project after the dependency
install, never required or bundled; the Causeway profile should list those
thirty-seven indexes beside the Rust ones. `--pnpm-workspaces`, which would index the
workspace in one process and one file, was not measured. Measure `scip-python` next:
the largest gaps left in the whole graph are generated Python clients, and no
TypeScript or Rust index will touch them.
