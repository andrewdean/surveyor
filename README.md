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
- **Precise tier.** `surveyor index --scip index.scip` (or `[scip] indexes` in the
  profile) merges a SCIP index written by `rust-analyzer scip`, `scip-typescript` or
  `scip-python`. Each reference occurrence the index resolved becomes a `Resolved`
  fact that names the file and line of the declaration; the linker pairs it with the
  Tree-sitter reference at the same site, so the covered `CALLS`, `USES_TYPE` and
  `IMPLEMENTS` edges carry `resolved` provenance at confidence 1 while the uncovered
  ones keep their name-matched edges. A reference the index attributes to a dependency
  or a local binding is neither an edge nor a gap. No index, no change.
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
- **Incremental update.** `surveyor update` finds what changed since the recorded
  commit (`git diff` when both the index and the checkout are clean commits, content
  hashes otherwise), re-extracts the changed files, and relinks only the files whose
  resolution the change can affect: the changed files, files whose edges were resolved
  against a declaration whose text changed, files that reference a name whose set of
  declarations changed, and files whose edges lost an endpoint. Everything else stays
  as it was. A manifest change, or a change that would relink more than half the
  files, falls back to a full index. The result equals a fresh index of the same tree.
- **Compute.** Neighbors by edge kind, reverse impact to a depth, shortest paths with
  a tier that says how certain the hops are, hubs ranked by how many files depend on a
  node, label-propagation communities and dependency cycles over `petgraph`.
- **Dispatch.** A trait or interface method carries `DISPATCHES_TO` edges to every
  implementation that defines the same name, so a path through `dyn Trait` continues
  into the bodies that run and `affected` on an implementation climbs to the callers of
  its contract. Enum variants are `Variant` nodes; a path used as a value or in a
  pattern is a `REFERENCES` edge.
- **Rules.** Architectural intent as Rhai scripts over the graph: `surveyor check` runs
  every `*.rhai` under the profile's `[rules] dirs` (default `surveyor-rules/`), each a
  `fn check(g)` returning violations with citations, judged as `fail`, `warn` or a
  `budget` to burn down. Surveyor's own rules live in `surveyor-rules/`: crate
  layering, a pure model crate, Lance named only by the store, no crate cycles, and a
  budget of public functions without tests.
- **Interfaces.** A CLI and an MCP server whose first tools keep the names graphicle and
  Graphify publish (`query_graph`, `get_node`, `get_neighbors`, `shortest_path`,
  `god_nodes`, `get_community`, `read_project_source`) plus `affected`, `find_nodes`,
  `graph_stats` and `check_invariants`.

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

`surveyor update` on Causeway after one commit, same build (the cold index on the same
afternoon took 42 to 45 s):

| Change | Files relinked | Update |
|---|---:|---:|
| nothing (commit moved, content same) | 0 | 4.5 s |
| one statement added to `causeway_outbox::append` | 7 (the file and its 6 callers) | 7.4 s |
| a new public function whose name 4 other files reference | 5 | 8.0 s |
| one statement added to `Parser::expect`, a hub method | 31 | 9.7 s |
| one file deleted and one added, uncommitted (hash path) | 20 | 6.9 s |

About 2.5 s of each run is loading the stored graph and 1 to 2 s is the Lance delete
and append; the rest is extraction and relinking of the listed files. After the four
updates in sequence the dataset was compared node by node and edge by edge, including
citations, with a fresh index of the final tree: identical.

Cairn with and without a SCIP index from `rust-analyzer scip` (15 s to generate, 3.8 MB,
74 Rust documents; the checkout also holds a TypeScript extension and shell scripts the
index does not cover):

| | Tree-sitter only | with SCIP |
|---|---:|---:|
| `CALLS` edges, syntactic | 6,199 | 142 (8 Rust, 26 shell, 108 TypeScript) |
| `CALLS` edges, resolved | 0 | 2,383 |
| `CALLS` edges at confidence below 1 | 3,402 at 780 sites | 10 at 5 sites |
| `USES_TYPE` edges, syntactic / resolved | 1,017 / 0 | 178 / 831 |
| unresolved references | 1,276 | 155 (all in TypeScript) |
| index time | 0.7 s | 1.0 s |

The ambiguous method calls collapse to one target each: `run` in
`cairn-cli/src/commands/resolve.rs` called `as_str` on several receivers, name
matching drew every one of those edges to `FeedbackType::as_str` (the only `as_str` in
the same package, so at confidence 1), and the index draws them to `CommitSha::as_str`
and `SkillId::as_str` in `cairn-domain`. Calls into dependencies, previously 1,078
`no-declaration` gaps such as `create_dir_all` and `from_utf8_lossy`, are recognized
as external and reported as neither edges nor gaps.

## Use

```bash
cargo build --release
```

Index a checkout (the dataset lands in `<repo>/.surveyor`, or wherever `--store` points):

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway index
```

With a SCIP index from the language's own tooling, calls and type uses resolve
precisely where the index covers them:

```bash
rust-analyzer scip ~/dev/cairn --output /tmp/cairn.scip
surveyor --repo-root ~/dev/cairn --store ~/.surveyor/cairn index --scip /tmp/cairn.scip
```

After a commit, or while editing, bring the dataset up to date without a full index:

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway update
```

Ask it questions:

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway explain Relay::drain
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway affected cw_catalog.asset --depth 3
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway path crates/api/src/main.rs::main cw_outbox.outbox
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway hubs
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway source rust:crates/a/src/lib.rs::Relay::drain
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway nodes --kind Function --in crates/outbox --without TESTED_BY
```

A name that several declarations share is refused with their identifiers rather than
answered for whichever ranks first, and a qualifier (`path::`, `Type::`, `schema.`)
that matches no declaration of that name is an error, never another file's node.
`query` lists the candidates. `path` follows edge direction first and searches
against it only when no directed path exists; such a result is labeled a lead and
each reversed hop is marked `<-`. `--directed` and `--undirected` force one search.

`nodes` lists nodes by kind and location and by the edges they have or lack, which is
how to ask set questions such as "which functions in this crate have no test".

Check the repository's rules, or extra ones, and exit 1 on a failure:

```bash
surveyor --repo-root ~/dev/causeway --store ~/.surveyor/causeway check --rules ./my-rules
```

A rule is a Rhai file:

```rhai
const ID = "outbox-written-only-by-outbox";
const DESC = "only code under crates/outbox/ writes cw_outbox.outbox";

fn check(g) {
    let out = [];
    for e in g.edges_to("sql:cw_outbox.outbox", "WRITES") {
        if e.file == () || !e.file.starts_with("crates/outbox/") {
            out.push(violation(e, "writes the outbox table from outside the outbox crate"));
        }
    }
    out
}
```

`g` offers `nodes`, `nodes_under`, `node`, `edges`, `edges_from`, `edges_to`, `out`,
`inn`, `package_of`, `reaches`, `cycles`, `count` and `cargo_build` (what a build of a
Cargo package pulls in, from the optional, default-features and feature facts on its
manifest edges); `const SEVERITY` is `fail` (default), `warn` or `budget` with
`const BUDGET`. Scripts run sandboxed.

`examples/causeway-rules/` ports the named checks of Causeway's VSA gate to rules over
the graph, with the comparison against the original and the checks the graph cannot answer.

`nodes` lists nodes by kind and location and by the edges they have or lack, which is
how to ask set questions such as "which functions in this crate have no test".

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
default schema, point at migration and documentation directories, exclude paths, and
list SCIP indexes to merge when they exist (`[scip] indexes = ["index.scip"]`); every
field has a default. Exclude fixture trees, or their tables and symbols appear as the
repository's own. See `tests/fixtures/mini/surveyor.toml` and Surveyor's own
`surveyor.toml`.

Surveyor surveys itself: `docs/dogfood/` records what the graph said about the tool and
what changed because of it.

## Layout

```
crates/surveyor-model     nodes, edges, provenance, evidence, facts (no I/O)
crates/surveyor-extract   Tree-sitter host, query packs, sqlparser, rhai, manifests, Markdown, SCIP, walker
crates/surveyor-link      facts to nodes and edges; name resolution; unresolved gaps
crates/surveyor-store     Lance dataset behind LanceStore
crates/surveyor-graph     petgraph compute: neighbors, affected, paths, PageRank, communities
crates/surveyor-rules     Rhai rule engine: invariants over the graph, with citations
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

Cards and embeddings, context packs, the Causeway adapter, `serve` with streamable
HTTP, personas and runtime connectors are the following phases of the RFD. Incremental update does not watch the file system yet (`notify` is Phase 1 too),
treats a `surveyor.toml` edit like any other file (run `index` after changing the
profile), and relinks changed files syntactically: rerun `index --scip` when a fresh
SCIP index arrives, which the RFD expects to happen in CI on a slower cadence.
