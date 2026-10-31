# Surveyor

Surveyor builds a code intelligence graph with provenance. It parses a repository with
Tree-sitter and native parsers, links declarations and references into typed nodes and
edges, records how every edge was derived and which source span supports it, stores the
graph in a Lance dataset, and answers questions from the command line or over MCP.

The design is in `RFD-surveyor-agentic-code-intelligence.md` (kept beside this repository
for now). This checkout implements Phase 0 of that plan.

## What it does today

- **Extraction.** Rust, TypeScript, TSX, JavaScript, Python and Bash through Tree-sitter
  query packs in `languages/`; SQL files and SQL string literals through `sqlparser`;
  Rhai through the `rhai` crate's AST; `Cargo.toml`, `package.json` and `pyproject.toml`;
  Markdown design documents and runbooks. Rust macro arguments are re-parsed so calls
  inside `assert_eq!` and friends are seen.
- **Linking.** Declarations become nodes with stable identifiers such as
  `rust:crates/outbox/src/relay.rs::Relay::drain` or `sql:cw_outbox.outbox`. References
  resolve by name within the same file, then package, then repository; a reference with
  one candidate becomes a certain edge, one with a few becomes one edge per candidate
  with confidence `1/n`, and one with none or too many is recorded as unresolved, never
  guessed. SQL references resolve against the schema the migrations declare.
- **Provenance.** Every edge carries a derivation (`syntactic`, `resolved`, `declared`,
  later `observed` and `inferred`), a confidence, and evidence: repository, file, byte
  span, lines, commit and a hash of the cited text.
- **Storage.** Five Lance tables (`nodes`, `edges`, `files`, `unresolved`, `meta`)
  partitioned by repository; each index run is a new dataset version.
- **Compute.** Neighbors by edge kind, reverse impact to a depth, shortest paths,
  PageRank hubs, label-propagation communities and dependency cycles over `petgraph`.
- **Interfaces.** A CLI and an MCP server whose first tools keep the names graphicle and
  Graphify publish (`query_graph`, `get_node`, `get_neighbors`, `shortest_path`,
  `god_nodes`, `get_community`, `read_project_source`) plus `affected`, `find_nodes`
  and `graph_stats`.

## Numbers

Measured on 2026-10-31 on a laptop, debug build with optimized dependencies:

| Repository | Files | Nodes | Edges | Cold index |
|---|---:|---:|---:|---:|
| Causeway (Rust, TypeScript, Python, SQL, Rhai, shell) | 4,088 | 53,838 | 296,737 | 50 s |
| Cairn (Rust) | 228 | 1,730 | 11,646 | 1 s |
| `tests/fixtures/mini` | 22 | 85 | 139 | 0.1 s |

Of the Causeway time, extraction takes 19 s, linking 26 s and the Lance write 5 s; the
fresh dataset is 68 MB. Loading the graph for a query takes about 2.5 seconds. The
Surveyor crates themselves are compiled without optimization in this measurement; a
release build will be faster.

## Use

```bash
cargo build --release
```

Index a checkout (the dataset lands in `<repo>/.surveyor`, or wherever `--store` points):

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway index
```

Ask it questions:

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway explain Relay::drain
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway affected cw_catalog.asset --depth 3
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway path main cw_outbox.outbox --directed
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway hubs
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway source rust:crates/a/src/lib.rs::Relay::drain
```

Every command takes `--json`. Serve the same answers to an agent over MCP on stdio:

```json
{
  "mcpServers": {
    "surveyor": {
      "command": "surveyor",
      "args": ["--repo-root", "/path/to/repo", "--store", "/path/to/store", "mcp"]
    }
  }
}
```

A repository can carry a `surveyor.toml` to name itself, choose the SQL dialect and
default schema, and point at migration and documentation directories; every field has a
default. See `tests/fixtures/mini/surveyor.toml`.

## Layout

```
crates/surveyor-model     nodes, edges, provenance, evidence, facts (no I/O)
crates/surveyor-extract   Tree-sitter host, query packs, sqlparser, rhai, manifests, Markdown, walker
crates/surveyor-link      facts to nodes and edges; name resolution; unresolved gaps
crates/surveyor-store     Lance dataset behind LanceStore
crates/surveyor-graph     petgraph compute: neighbors, affected, paths, PageRank, communities
crates/surveyor-mcp       rmcp server
apps/surveyor             the CLI
languages/<name>/surveyor.scm   query packs
tests/fixtures/mini       a small polyglot repository used by the end-to-end test
```

## Develop

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

`mise run ci` runs the same gate plus `cargo deny` and `cargo machete`.

## Not yet

SCIP ingest, incremental update over `gix`, cards and embeddings, context packs, Rhai
invariants, the Causeway adapter, `serve` with streamable HTTP, personas and runtime
connectors are the following phases of the RFD. The `changed_paths_since` walker method
exists; nothing calls it yet.
