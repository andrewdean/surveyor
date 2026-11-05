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

The baseline's one hit is "how does the loader registry decide the run order", where
`order` is a function name. The two questions from the 2026-11-05 session that
prompted this work both miss: the development-data question returns
`ExplorerDatasets::loaded`, two `WasmActor::loaded` methods and two `.data` columns,
and the advisory-lock question returns `DERIVATION_PREFIX`, a `lock` host function
and three `key` methods.
