---
title: "The precise tier on Causeway for Python: scip-python over nine projects"
date: 2026-11-01
status: measured
---

# The precise tier on Causeway for Python: scip-python over nine projects

This is the third measurement of the precise tier on Causeway, after rust-analyzer
(`2026-11-01-scip-causeway.md`) and scip-typescript
(`2026-11-01-scip-typescript-causeway.md`). The TypeScript note ended by pointing at
Python: four of the five generated-code names that lead the whole-graph gaps
(`MethodInfo`, `ConnectError`, `execute_unary`, `Message`) are in generated Python,
so this note asks what `scip-python` does about them, and closes with the whole
graph indexed under all three tiers at once.

**Result.** `scip-python` over Causeway's nine Python projects costs 14 seconds, under
0.5 GB and 2.5 MB of index files. It resolves a quarter of the Python edges and cuts
Python gaps from 3,353 to 2,922. It does not touch the generated code, because
Causeway's own pyright configuration includes only `src/` and scip-python follows it;
2,264 of the remaining Python gaps are in `gen/` directories the adopter chose not to
type-check. Imports of sibling workspace packages did not resolve in this environment
either, so cross-project edges stay name-matched. With every index from all three
notes loaded together, Surveyor indexes Causeway in 3.2 s, 55% of all edges are
resolved and the gaps fall from 40,539 to 13,295.

## Setup

Causeway at commit `40f993d7` with the profile under `examples/causeway-profile/`,
indexed with the Surveyor release binary at commit `51a7f7d`. Causeway's Python is a
`uv` workspace of nine members (`services-py/*`, `jobs-py/*`,
`torrentfx/packages-py/*`, `ffi/py-drive`) whose root `pyproject.toml` asks for
per-project virtualenvs; the checkout has one root `.venv` (Python 3.12.13, 131
packages, the two `torrentfx` packages installed editable). The indexer was
`@sourcegraph/scip-python` 0.6.8 under Node 25.2.1, installed in a scratch directory,
run once per project with `--cwd` at the project, the root `.venv` first on `PATH`
so pyright found the environment, and `--output` outside the checkout. `ffi/py-drive`
was run twice with `--target-only`, for `python/` and `tests/`, to keep its
`.uv-cache` out. `git status` in Causeway listed the same eight pre-existing entries
before and after.

| Project | Project name | Wall clock (s) | Peak RSS (GB) | Index (KB) |
|---|---|---|---|---|
| `services-py/relay-grpc` | python-grpc | 1 | 0.36 | 174 |
| `services-py/unity-catalog-relay` | python-unity-catalog-relay | 2 | 0.39 | 412 |
| `services-py/snowflake-relay` | python-snowflake-relay | 2 | 0.38 | 143 |
| `services-py/relay-restate` | python-restate | 1 | 0.35 | 38 |
| `jobs-py/relay-worker` | python-worker | 1 | 0.34 | 32 |
| `torrentfx/packages-py/causeway-platform` | causeway-platform | 3 | 0.49 | 1,155 |
| `torrentfx/packages-py/causeway-relay` | causeway-relay | 2 | 0.45 | 486 |
| `ffi/py-drive` (`python/`) | causeway-drive | 1 | 0.20 | 39 |
| `ffi/py-drive` (`tests/`) | causeway-drive-tests | 1 | 0.29 | 58 |

Table 1: scip-python per project. Total 14 s and 2.5 MB. The indexes name the
project's own symbols, the standard library and installed packages (`protobuf`,
`pydantic`, `restate-sdk`), and each index's `project_root` is the project
directory, so Surveyor's re-rooting places `src/config.py` at
`services-py/snowflake-relay/src/config.py`.

## What the indexes cover, and what they do not

scip-python indexes the files pyright considers project files. `services-py/relay-grpc`
has 97 Python files, 89 of them generated connect stubs under `gen/`, and its
`pyproject.toml` says `[tool.pyright] include = ["src"]` with `gen` on `extraPaths`;
the index holds eight documents. Running scip-python with `--cwd` at `gen/` itself
still finds the parent `pyproject.toml` and writes an index with no documents, and
`--target-only gen` does the same. The adopter decided not to type-check its
generated code, and the indexer respects that. The generated files are still walked
and extracted by Tree-sitter, so their nodes exist and their references are name
matched; what the index cannot do is tell the linker that `MethodInfo` there is
`connectrpc`'s. That is a name-tier question, taken up at the end.

The second limit is cross-project. `SnowflakeConnection` in `snowflake-relay`
extends `PrefixedSettings` from `causeway-platform`. The `causeway-platform` index
declares that class as
`scip-python python causeway-platform 0.1.0 \`causeway_platform.settings\`/PrefixedSettings#`,
but the `snowflake-relay` index carries no `causeway-platform` symbol at all: pyright
did not resolve the import in that project's environment, so there is no occurrence
for Surveyor to pool. The `EXTENDS` edge stays as the name tier drew it, at 1.0.
Whether a per-project `.venv` with the workspace packages installed would change
that is untested.

## What the index changes

| | name only | with scip-python |
|---|---|---|
| edges from `.py` files | 7,477 | 7,162 |
| of which `resolved` | 0 | 1,811 |
| `CALLS` at 1.0 | 1,452 | 1,549 |
| `CALLS` at 0.75 or 0.5 | 206 | 72 |
| `CALLS` shared 1/n below 0.5 | 1,193 | 1,013 |
| `USES_TYPE` at 0.5 | 974 | 920 |
| `EXTENDS` resolved | 0 | 11 of 14 |
| gaps in `.py` files | 3,353 | 2,922 |
| of those, in `gen/` directories | 2,575 | 2,264 |
| `no-declaration` | 3,296 | 2,880 |
| `ambiguous-receiver` | 42 | 35 |

Table 2: Python edges and gaps without and with the nine indexes. Whole-graph totals
went from 263,506 edges and 40,539 gaps to 263,191 and 40,046.

Outside the generated directories, 658 Python gaps remain, and their names say what
they are: `setenv`, `delenv`, `raises`, `parametrize`, `fixture` (pytest, 290 sites),
`assertEqual`, `deepcopy`, `urlparse`, `read_text` (the standard library), and 24
sites in `docs/research` and `policy/rights` scripts no project covers. Those are
calls into dependencies the index did not mark external because pyright did not
resolve them, or files no index covers.

## The exit-test question

Question 15, `affected PrefixedSettings` (by its Python identifier, since a Rust
struct and a variant in `cw-vsa` share the name), returns the same ten nodes with
and without the index: five subclasses at depth 1, then three tests and two config
classes. With the index, the two subclasses in `causeway-platform`'s own tests, the
three tests and both `USES_TYPE` edges are `[resolved]`; the three subclasses in other
projects (`RelayWorkerSettings`, `DatabricksSettings`, `SnowflakeConnection`) keep
their name-matched edges for the cross-project reason above. The answer was already
right; the index changed its provenance, not its content.

## All three tiers together

Loading the thirteen rust-analyzer indexes, the thirty-seven scip-typescript indexes
and these nine in one `surveyor index`:

| | name only | all 59 indexes |
|---|---|---|
| index time | 4.8 s | 3.2 s |
| peak memory | | 1.7 GB |
| edges | 263,506 | 237,764 |
| `resolved` edges | 6,960 | 130,347 |
| gaps | 40,539 | 13,295 |
| `no-declaration` | | 11,292 |
| `qualifier-mismatch` | | 990 |
| `ambiguous` | | 696 |
| `ambiguous-receiver` | | 212 |

Table 3: the whole Causeway graph under the three precise tiers. The twelve most
frequent remaining gap names are `MethodInfo` 912, `Message` 858, `ConnectError` 456,
`execute_unary` 446, `curry` 189, `from` 176, `is_blank` 158, `Unavailable` 121,
`setenv` 117, `sh` 107, `trimmed` 102 and `locator` 93: generated Python and
TypeScript, Rhai and shell, and pytest.

## What this means

For Python the precise tier is nearly free (14 s, half a gigabyte) and worth having,
and the section 17 recommendation holds unchanged: produce the indexes on a schedule
or in CI, name them in the profile, never require them. The two limits found here
are the adopter's, not Surveyor's: generated code excluded from type checking, and
imports between workspace projects that pyright did not resolve in a shared
virtualenv.

The generated-code gaps point at the name tier instead. The 2,264 sites in `gen/` are
uses of names the file imports from `connectrpc` and `google.protobuf`, packages
Surveyor already knows from `pyproject.toml`; the linker records what each file
imports (`imported_by_file`) and could treat a name imported from a package outside
the repository as external, neither an edge nor a gap, the way the index does. That
change would remove the four generated-code names from the top of the gap list
without any index, for Python and TypeScript alike, and is the next thing to do
before measuring anything further.

***Recommendation:*** keep scip-python in the documented default set alongside
rust-analyzer and scip-typescript; tell adopters that the index covers what their
pyright configuration includes and that workspace packages resolve only when the
project's environment can import them; and make names imported from external
packages silent in the name tier, which is the fix the remaining gaps call for.
