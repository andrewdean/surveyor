# Retrieval eval

`surveyor eval <questions.toml>` scores plain-English questions against the nodes they
should start from. Each `[[question]]` names the question as a user types it and the
identifiers of the nodes an engineer would open first; the eval asks the graph for the
question's starting nodes (what `query_graph` seeds on) and reports:

- **hit@k**: an expected node is in the top `k` (default 5);
- **recall@k**: the share of the expected nodes in the top `k`;
- **MRR@20**: the mean of `1 / rank` of the first expected node within the top 20;
- **unknown ids**: expected identifiers the store does not hold, so an expectation
  never silently names a node that does not exist.

`causeway-questions.toml` holds 29 questions over Causeway at commit `40f993d7`,
several in words the repository does not use (the `note` says which). Run it against
the Causeway store:

```bash
surveyor --repo-root ~/dev/trusted/causeway --store "$SURVEYOR_STORE" eval docs/eval/causeway-questions.toml
```

The mini fixture has a two-question set under `apps/surveyor/tests/data/`, which the
`eval` integration test runs.

## Results on Causeway

Debug build, the dataset at `~/Library/Application Support/Surveyor/causeway`, 60,969
nodes. The twelve unknown identifiers of the baseline are the mise tasks
(`task:mise/...`) the store did not hold before tasks were indexed.

| Stage | Ranking | hit@5 | recall@5 | MRR@20 | unknown ids |
|---|---|---:|---:|---:|---:|
| baseline (2026-11-05) | name match: exact, suffix, substring per word | 1/29 (3%) | 0.02 | 0.11 | 12 |
| 2. wider index | same matcher over 426 `Task` and 289 `Document` nodes more | 1/29 (3%) | 0.02 | 0.11 | 0 |
| 3. BM25F | `surveyor-search`: BM25F over name, scope, path, signature, doc and descriptive attributes | 17/29 (59%) | 0.41 | 0.43 | 0 |
| 4. hybrid (prototype, not shipped) | BM25F and dense `bge-small-en-v1.5` fused by reciprocal rank fusion, k=60 | 19/29 (66%) | 0.45 | 0.54 | 0 |
| 4. dense alone (prototype) | `bge-small-en-v1.5` cosine over the same node text | 16/29 (55%) | 0.34 | 0.41 | 0 |
| 5. `query_graph` seeds through the search | same ranking; the graph expansion and citations are unchanged | 17/29 (59%) | 0.41 | 0.43 | 0 |
| 6. RUNS edges | tasks, scripts and binaries linked; seeds unchanged | 17/29 (59%) | 0.41 | 0.43 | 0 |

Widening the index (stage 2) adds the mise tasks (345 from `.mise/toml/*.toml` and
`mise.toml`, 81 file tasks under `.mise/tasks/`), a `Document` node per Markdown file
outside the design and runbook globs, doc comments on Rhai functions and section
headings on every document. Every expected identifier now exists, but the name
matcher cannot reach the new text: it only sees names, so the scores do not move.

Stage 3 replaces the per-word name matcher with a text index built when the graph
loads (2.5 s in a debug build for 61,779 nodes, on top of the 1.9 s graph load; the
index is in memory because the graph already is, see "Why not Lance's full-text
index" below). Identifiers are split at case changes and underscores and kept whole
as well, words are stemmed with Snowball English and stopwords dropped; each field is
normalized against its own average length (BM25F), which matters here: with one
document-wide length the nodes with the richest descriptions, the mise tasks and the
documented functions, sank below one-word matches. Kind priors are mild and
documented in the crate: impl blocks and SQL query nodes yield to the node they are
part of, tests a little to the code they test, prose documents a little to the code
they describe. Both questions from 2026-11-05 now hit: the advisory-lock question
ranks `advisory_lock_key` and `AdvisoryLockKey::for_workspace` first and second, the
development-data question puts `devdata:load` fifth behind four documents and RFDs
about development data. The misses left are lexical: "database migrations" against
a task that says "schemas" and "sqlx", "asleep" against "not answering", "schema of
its own" against "workspace".

Six expected sets grew after the BM25 run, each by a node the ranking surfaced that
is a right starting point on inspection (the `note` on each question says which):
`dev:data:load`, `PgPlacementRepo::default_home_for`, `upsert_catalog_asset_for`,
the `search:quality:check` task with its README, `SemanticCandidateLimit`, and the
two outbox `publish` methods. The baseline and stage 2 rows were measured before
those additions; rerun against the current file they are unchanged (the name matcher
finds none of the added nodes either).

### The vector half: measured, not shipped

`vector-prototype/` holds the measurement (`run.sh`, `proto.py`, the output of
2026-11-05). It embeds the same node text the BM25F index reads, with
`BAAI/bge-small-en-v1.5` through fastembed, the model Causeway's own hybrid search
uses, caches the vectors by content hash as an incremental implementation would,
and fuses the BM25 and cosine rankings by reciprocal rank fusion (k = 60, the
constant Causeway's ADR on hybrid search also settles on).

| system | hit@5 | recall@5 | MRR@20 |
|---|---:|---:|---:|
| BM25F (shipped) | 17/29 | 0.41 | 0.43 |
| dense alone | 16/29 | 0.34 | 0.41 |
| RRF of both | 19/29 | 0.45 | 0.54 |
| RRF, BM25 counted twice | 19/29 | 0.47 | 0.52 |

What the dense half buys: two more questions in the top five ("session cache keys
kept apart per workspace" lands on `SessionKeyspace::for_workspace` third, "what
checks that a loaded workspace is coherent" lands on `run_checks`' file fourth) and a
better first rank on six others, which is where the MRR gain comes from. What it
does not buy: the five questions both halves miss stay missed ("schema of its own",
"asleep", "glossary terms attached to columns", "contract fixtures", "canonical
identity minted"); the vocabulary gap there is between the question and the code,
and bge-small does not bridge it either.

What it costs: 28 minutes to embed 61,779 nodes on this laptop's CPU (1,704 s, eight
cores busy, 8 GB resident) against 55 s for the whole index today; the content-hash
cache makes later runs proportional to the change, but a fresh clone or an engine
bump pays the full price, so the vectors would have to be built in the background
and `query_graph` would run BM25 alone until they exist. In Rust the dependency is
`fastembed` over ONNX Runtime, which Causeway vendors after an ORT deadlock incident,
a 130 MB model download on first use, and an ONNX runtime binary per release target.

Recommendation: not now. Two questions and a tenth of MRR do not pay for a
half-hour cold start and a native runtime in a tool whose start-up `update` takes
0.4 s. Two cheaper things to try before any of that: a static embedding model
(`model2vec`-style, a token lookup table with no runtime, which embeds this corpus
in seconds) measured with the same script, and the five shared misses as more
questions, since they are where any gain would have to show. The prototype stays so
the measurement can be rerun when either is tried.

### Why not Lance's full-text index

Lance ships a BM25 inverted index, and it was the first candidate. Two things decided
against it for this stage. Every query already loads the whole graph into memory, so
an index built from the loaded nodes costs a few seconds once and nothing per query,
while a Lance index is a second artifact to keep in step with incremental `update`.
And the tokenization is the point: `AdvisoryLockKey` has to be `advisory`, `lock`,
`key` and the whole word at once, the path has to count less than the name, and a
task description has to be normalized against other descriptions, not against
one-word names; Lance's tokenizer would need the text pre-split and stored per
field. If the graph stops being memory-resident, the same documents and weights can
move into a Lance table; the crate boundary (`surveyor-search`) is where that change
would land.

The baseline's one hit was "how does the loader registry decide the run order", where
`order` is a function name. The two questions from the 2026-11-05 session that
prompted this work both miss: the development-data question returns
`ExplorerDatasets::loaded`, two `WasmActor::loaded` methods and two `.data` columns,
and the advisory-lock question returns `DERIVATION_PREFIX`, a `lock` host function
and three `key` methods.

## Context-pack eval

`surveyor eval <tasks.toml>` scores tasks instead of questions when the file holds
`[[task]]` entries. A task is the text an agent would be handed, with the answer key the
change that resolved it left behind: `expect`, the identifiers of nodes the change
edited, and `files`, the files it touched. The eval produces an answer for the task
text (`--source`, today `query-graph`: the MCP tool's exact output for the text, five
seeds at depth 1) and reports:

- **recall**: the share of the key (identifiers and files together) the answer carries;
  an identifier counts when the answer names the node, a file when the answer names any
  node in it;
- **hit**: any part of the key is carried;
- **tokens** and **bytes**: the answer's size, tokens under the `cl100k_base`
  encoding (`surveyor-context`), held against the task's `budget` (30,000 by default);
- **unknown ids**: expected identifiers the store does not hold.

`causeway-tasks.toml` holds the ten tasks of the Phase 2 exit test, drawn from merged
pull requests 1014 to 1073 (`PLAN-surveyor-phase-2.md` beside the RFD says how). Each
is scored against the tree at its `base` commit, the merge commit's parent, because a
pack is what an agent reads before it writes the change. `causeway-task-stores.sh
<scratch>` builds one store per base in a shared scratch clone of Causeway (name tier,
`examples/causeway-profile`), and `--stores <scratch>/stores` points the eval at them:

```bash
docs/eval/causeway-task-stores.sh "$SCRATCH"
surveyor --repo-root ~/dev/trusted/causeway eval docs/eval/causeway-tasks.toml --stores "$SCRATCH/stores"
```

### Results on the ten tasks

Table 1. The `query_graph` baseline, 2026-11-05: today's answer for each task text,
counted the same way a pack will be.

| Task | PR | Key found | Recall | Tokens | Bytes |
|---|---:|---:|---:|---:|---:|
| cwsh-user-agent | 1073 | 3/7 | 0.43 | 5,010 | 19,800 |
| drop-orphaned-base-tables | 1064 | 3/8 | 0.38 | 5,031 | 20,765 |
| schema-check-at-startup | 1068 | 1/11 | 0.09 | 8,167 | 34,023 |
| tasks-page-load-failed | 1066 | 6/10 | 0.60 | 3,674 | 15,128 |
| paused-tab-not-evicted | 1040 | 2/5 | 0.40 | 8,644 | 35,845 |
| dataset-load-waits-for-codec | 1027 | 1/11 | 0.09 | 4,202 | 17,790 |
| cookie-rpc-needs-preflight | 1021 | 7/13 | 0.54 | 7,572 | 31,482 |
| flightdeck-postgres-tls | 1014 | 4/13 | 0.31 | 5,010 | 20,303 |
| search-relay-addon-suite | 1039 | 4/8 | 0.50 | 1,825 | 7,101 |
| stuck-work-sources | 1071 | 5/14 | 0.36 | 11,510 | 46,499 |
| **all** | | **36/100** | **0.37** | median 5,031, max 11,510 | |

Every answer fits the 30 thousand token budget, and every one carries some part of its
key, because the text index finds the file or a neighbor of the symbols a task names.
What the baseline lacks is the rest of the key: `query_graph` returns five seeds and
their direct relations as JSON, so it names the seed's callers and callees and nothing
two hops away, carries no source text, and spends most of its tokens on citations and
repeated node views. The two tasks under 0.10 show the failure mode a pack has to
fix: the seeds are near the answer but not on it. `schema-check-at-startup` seeds on
five nodes of `workspace_migrations.rs` (`StillPending`, `migrate_workspace`,
`pending_count`), the module that applies migrations, one hop from the `init` that
has to stop calling it; `dataset-load-waits-for-codec` seeds on
`BrowserBus::registerActionCodec` and the codec registration functions, members of
the very class the key names, which the answer lists without the class itself. Both
are reachable by walking from the seeds, which is what a context pack does and
`query_graph` does not.
