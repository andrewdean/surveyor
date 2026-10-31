# Self-survey, 2026-10-31

Surveyor indexing Surveyor, and what the graph said about the tool. The point of the
exercise is signal for the agentic-engineering use case: every question below is one
an agent asks before it changes code, and every gap found became a change the same day.

## Setup

```bash
rust-analyzer scip . --output .surveyor/surveyor.scip      # 20 s, 3.2 MB
surveyor index                                             # profile in surveyor.toml
```

| | Tree-sitter only, no profile | with profile and SCIP | after the changes below |
|---|---:|---:|---:|
| files | 77 | 56 | 56 |
| nodes | 1,251 | 1,170 | 1,419 |
| edges | 7,283 | 5,163 | 6,369 |
| resolved edges | 24 | 3,163 | 4,154 |
| unresolved references | 992 | 102 | 111 |
| cold index | 0.2 s | 0.2 s | 0.2 s |

## What the first index got wrong

1. **The fixture repository was indexed as Surveyor's code.** Seven `Column` nodes,
   five `MIGRATES` edges and SQL queries belonged to `tests/fixtures/mini`. The default
   excludes cover build output and dependencies, not test data; the fix is a
   `surveyor.toml` with `tests/fixtures/**` excluded. Any repository with fixture trees
   has the same problem, so the profile is the right place, and the README now says so.
2. **`hubs` ranked error enums first.** `PageRank` rewards a two-node cycle (an
   `ErrorKind` and the error struct that wraps it) because rank has nowhere else to
   flow. The question an agent asks is "what does a change here ripple through", which
   is fan-in across files. `hubs` now ranks by distinct dependent files, then in-degree,
   with `PageRank` kept as a tiebreaker and reported for reference. The top of the list
   became `surveyor-model` (28 files), `RepoPath` (25), `NodeKind` (18), `ContentHash`
   (17), which is the dependency picture a maintainer would draw.
3. **A function used as a value had no dependents.** `affected NodeKind::is_callable`
   returned nothing, although the linker passes it to `link_named`. Only calls were
   references; a path used as a value or in a pattern was invisible, and so were enum
   variants, which produced most of the 102 unresolved names (`Call`, `Literal`,
   `Declaration` and friends are tuple-variant constructions). Enum variants are now
   `Variant` declarations and a scoped path used as a value or pattern is a `REFERENCES`
   edge. `affected NodeKind::is_callable` now reaches `Builder::references` and then
   `run`, which is correct.
4. **A trait method did not lead to its implementations.** `explain Extractor::extract`
   showed one caller and nothing else; `path main Lex::step` found no path because the
   call goes through `dyn Extractor`. The linker now emits `DISPATCHES_TO` from a
   contract method to every implementation that defines the same name, derived from the
   `IMPLEMENTS` edge and the method names (certain by the language's rules, recorded as
   `Resolved` with resolver `dispatch`). The trait method now lists its six
   implementations, and the path from `main` to `Lex::step` is twelve certain hops
   through the dispatch.
5. **No way to ask set questions.** "Which functions in the linker have no test" has no
   `explain` form. `surveyor nodes --kind Function --in crates/surveyor-link --without
   TESTED_BY` now answers it (the identifier builders in `ids.rs`, `declaration_identity`
   and `is_builtin_name`, which is a real list to act on).

## What held up

- `affected TextSpan --depth 1` listed the 34 declarations and files that touch the
  type, each with a citation, which is exactly the list that was needed when `TextSpan`
  replaced two fields during the FP pass.
- `affected CodeGraph::page_rank --depth 2` named the three callers (`god_nodes`, the
  CLI's `run`, one test) before the ranking changed; the change touched exactly those.
- `path main LanceStore::replace_repository --directed` is four certain hops.
- `surveyor update` with 32 of 56 files affected fell back to a full index on its own,
  which is the right call at that ratio; a one-file change relinks in under a second.

## Still open

- Changing Surveyor's own extractors or linker invalidated stored facts for files that
  did not change, and `update` relinked only the changed files. Fixed the same day: a
  snapshot records the engine that wrote it (crate version, a hash of the query packs
  and a linker revision, `0.1.0+<hash>`), and `update` re-indexes in full when the
  build differs. The first update after this change took that path on this repository.
- A refreshed SCIP index is an input the dataset depends on, and `update` did not
  notice it: after regenerating the index, `update` relinked only the one Markdown file
  that had changed and the files the index now covered kept their name-matched edges
  (unresolved stayed at 241 instead of returning to about 110). Fixed the same day by
  folding the SCIP index files' content hashes into the engine stamp, so a changed
  index is a changed engine and `update` re-indexes in full.
- Communities by label propagation collapse into one cluster of 821 members; the
  second cluster is the graph API. Louvain or Leiden is needed before `communities` is
  useful on a real codebase.
- `IMPORTS` edges stay syntactic at confidence 0.5 when a `use` names an item that
  exists under two kinds; the SCIP tier replaces call and type edges but not imports.
- Closures bound with `let decl = |...|` are not declarations, so their calls are gaps
  (`decl`, `reference` in `sql.rs`).
