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
