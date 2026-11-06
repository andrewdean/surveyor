# Proposed: `surveyor check` as a Causeway CI step

A pull request for Andrew to open against andrewdean/causeway; nothing here has been
pushed to Causeway. It carries the five rules of the Phase 2 exit test into
Causeway's own `surveyor-rules/` directory, which its committed `surveyor.toml`
already names, and adds a workflow job that fails a pull request on a violation.

## Why the rules move

A rule belongs in the repository it constrains (RFD section 11). The port in
Surveyor's `examples/causeway-rules` copies Causeway's allowlists as constants, and
they drift: between `40f993d7` and `ae6bdbea` the gate in
`bootstrap/cw-vsa/src/checks.rs` added `explorer-session-rpc` as a fifth orchestrator
(#1050), and the copies flagged `main` until they were updated. In Causeway, the
change to the gate and the change to the rule are one pull request, and `surveyor
check` runs beside `vsa:check` rather than after the fact.

## What the pull request carries

1. `surveyor-rules/06-component-cross-deps.rhai`, `10-server-rpc-deps.rhai`,
   `11-server-service-impls.rhai`, `13-island-cross-refs.rhai` and
   `04-runtime-env-resource-ctors.rhai`, copied verbatim from Surveyor's
   `examples/causeway-rules` at the commit this document is in. Each header names the
   `checks.rs` function it ports and the graph facts it reads.
2. The workflow job below.
3. Optionally the other eight ported rules. `14-cross-slice-table-access` reports 45
   findings on `main` today (component code reading or writing tables that another
   slice's migrations create, the coherence test's write to `catalog_entries_v1`
   among them); it cannot ship as `fail` until those are either fixed or accepted, so
   it goes in as `warn` or stays out. `12-server-orchestrators` is `warn` already.

## The job

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ jobs:
+  surveyor-check:
+    name: surveyor check (architectural invariants)
+    needs: changes
+    if: needs.changes.outputs.code == 'true'
+    runs-on: ubuntu-latest
+    timeout-minutes: 10
+    steps:
+      - uses: actions/checkout@v4
+        with:
+          fetch-depth: 0   # update diffs the dataset's commit against HEAD
+      - name: Install surveyor
+        run: |
+          curl -fsSL https://github.com/andrewdean/surveyor/releases/download/v0.1.0/surveyor-0.1.0-x86_64-unknown-linux-gnu.tar.gz \
+            | tar -xz -C "$RUNNER_TEMP"
+          echo "$RUNNER_TEMP/surveyor-0.1.0-x86_64-unknown-linux-gnu" >> "$GITHUB_PATH"
+      - name: Restore the dataset
+        uses: actions/cache/restore@v4
+        with:
+          path: .surveyor/db
+          key: surveyor-db-${{ github.sha }}
+          restore-keys: surveyor-db-
+      - name: Bring the dataset to this commit
+        run: surveyor --store .surveyor/db update || surveyor --store .surveyor/db index
+      - name: Check the invariants
+        run: surveyor --store .surveyor/db check
+      - name: Save the dataset
+        if: github.ref == 'refs/heads/main'
+        uses: actions/cache/save@v4
+        with:
+          path: .surveyor/db
+          key: surveyor-db-${{ github.sha }}
```

Notes on the job:

- `surveyor update` relinks only what the commit touched (about a second on
  Causeway) when a dataset from an earlier `main` commit is restored; a cold `index`
  takes 4 to 10 s on the laptop that measured it and will take longer on a shared
  runner, so the first run and any run after a Surveyor release are the slow ones.
  The cache is written on `main` only, so pull requests read it and never race it.
- The dataset is the name tier alone. The precise tier (`[scip] indexes` in
  `surveyor.toml`) is absent on the runner, and the five rules read manifests, impl
  blocks and imports, which the name tier resolves; `check` is the same answer with
  or without the indexes for these five.
- `surveyor check` exits 1 on a failed rule with each finding's `file:lines`, and 2
  when a rule does not load. The job's log is the report; `--json` gives it as data
  for a comment bot later.
- Pin the Surveyor version in the download URL and bump it on purpose; the engine
  stamp in the dataset makes `update` re-index in full when the binary changes, so a
  bump costs one cold index on the next `main` run.
- The `changes` job and its `code` output already exist in `ci.yml` (#1055); the
  step needs no toolchain beyond the binary.

## The proof the rules give

`docs/exit-tests/raw/2026-11-05-phase-2-invariants.sh` in Surveyor reproduces the
exit test: on Causeway `main` (`ae6bdbea`) the five rules pass; with a cross-area
path dependency planted in `component/governance/core/Cargo.toml`, an `impl
pb::SearchService` planted in `apps/causeway-server/src/marketplace.rs` and an
`@causeway/ui-request` import planted in `web/ui-catalog-filter/src/index.ts`,
three rules fail naming those three lines and the other two stay green. The run's
output is beside the script.
