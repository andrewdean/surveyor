# `surveyor pr` as a Causeway pull-request comment

Opened as a draft on 2026-11-07, andrewdean/causeway#1088 (branch
`feat/ci-surveyor-pr-comment`, one commit on `.github/workflows/ci.yml`). It stays a
draft until a Surveyor release carries `pr`, because the job pins the release it
installs. The text below is the proposal as written before the pull request opened.

The `surveyor-check` job that landed as andrewdean/causeway#1078 fails a pull request
that breaks one of the five rules. This document proposes the step that turns the
same run into a comment: what the change touched, what it may break, and the rule
table with every finding cited, posted once per pull request and edited in place on
each push. The job stays one job; the comment step is added to it rather than a
second job restoring the same dataset. Opening the pull request is Andrew's call.

## What the comment shows

`surveyor pr` (Phase 3, stage 2; `docs/harness.md` has the command) measures the
working tree against the merge base of `HEAD` and the base branch. It maps each
changed line to the innermost declaration around it, walks the reverse impact three
hops from every changed node under the impact edge kinds with the 0.25 floor,
groups what it reaches by kind, runs the rules `check` runs, and exits 1 when a
`fail` rule fails. The Markdown starts with `<!-- surveyor-pr -->`, so the step
finds its earlier comment by that line and edits it.

On Causeway the contract and workload groups are empty, because the graph holds no
`Service`, `Rpc`, `Message` or workload nodes (the protobuf, OpenAPI, YAML workload
and Dockerfile extractors were carried out of Phase 1); the comment says so instead
of printing empty headings. The groups that do fill are code symbols, tables and
views (from the SQL migrations), tasks (the `.mise` Rhai tasks), files and packages
(manifests and imports) and documents (RFDs and runbooks that cite a symbol).

## The diff

Against `.github/workflows/ci.yml` at `bbec89ab1` (`main` on 2026-11-07). The job
keeps its name, its `needs`, its cache and its `update` step; the `check` step
becomes `pr`, which runs the same rules and adds the comment, and two steps post the
comment and then fail the job with the gate's exit code. The dataset cache is
unchanged, so a run costs what `check` cost plus the impact walk, which is under a
second on the Causeway graph.

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ jobs:
   surveyor-check:
     needs: changes
     if: needs.changes.outputs.code == 'true'
     runs-on: ubuntu-latest
     timeout-minutes: 10
+    permissions:
+      contents: read
+      pull-requests: write
     env:
       # Pinned on purpose. A bump costs one full index on the next main run.
-      SURVEYOR_VERSION: "0.1.0"
+      # `surveyor pr` needs 0.2.0; 0.1.0 has only `check`.
+      SURVEYOR_VERSION: "0.2.0"
     steps:
       - uses: actions/checkout@v4
         with:
-          # `update` diffs the dataset's recorded commit against HEAD.
+          # `update` diffs the dataset's recorded commit against HEAD, and `pr`
+          # needs the merge base with the base branch.
           fetch-depth: 0
@@
       - name: Bring the dataset to this commit
         run: surveyor update

-      - name: Check the invariants
-        run: surveyor check
+      # `pr` runs the five rules as `check` does and exits 1 when one fails, 2 when
+      # the report could not be made. The report is the pull request's comment and
+      # the job summary, posted either way, so this step records the code and the
+      # last step fails the job with it.
+      - name: Check the invariants and measure the change
+        id: pr
+        env:
+          BASE: ${{ github.event.pull_request.base.ref || 'main' }}
+        run: |
+          code=0
+          surveyor pr --base "origin/$BASE" --out "$RUNNER_TEMP/surveyor-pr" || code=$?
+          echo "code=$code" >> "$GITHUB_OUTPUT"
+          cat "$RUNNER_TEMP/surveyor-pr/pr.md" >> "$GITHUB_STEP_SUMMARY" || true
+
+      # One comment per pull request, found again by its first line and edited.
+      - name: Create or update the pull-request comment
+        if: github.event_name == 'pull_request'
+        env:
+          GH_TOKEN: ${{ github.token }}
+          PR: ${{ github.event.pull_request.number }}
+          REPO: ${{ github.repository }}
+        run: |
+          body="$RUNNER_TEMP/surveyor-pr/pr.md"
+          [ -s "$body" ] || printf '<!-- surveyor-pr -->\n### Surveyor\n\nThe report could not be made; see the surveyor-check job log.\n' > "$body"
+          existing=$(gh api "repos/$REPO/issues/$PR/comments" --paginate \
+            --jq '[.[] | select(.body | startswith("<!-- surveyor-pr -->"))][0].id // empty')
+          if [ -n "$existing" ]; then
+            gh api --method PATCH "repos/$REPO/issues/comments/$existing" -F body=@"$body" > /dev/null
+          else
+            gh api --method POST "repos/$REPO/issues/$PR/comments" -F body=@"$body" > /dev/null
+          fi
+
+      - name: Fail on a broken invariant
+        if: steps.pr.outputs.code != '0'
+        env:
+          CODE: ${{ steps.pr.outputs.code }}
+        run: |
+          echo "surveyor pr exited $CODE; the comment names the rule and the line"
+          exit "$CODE"

       - name: Save the dataset
         if: github.ref == 'refs/heads/main'
```

Notes on the diff:

- The version bump is the one real dependency: `surveyor pr` does not exist in the
  0.1.0 release the job pins today. The bump costs one full index on the next
  `main` run, as the pinning comment says. Until a release carries it, the job can
  be verified locally with a built binary, which is how this proposal was checked.
- On a push to `main` there is no pull request to comment on; `pr` still runs with
  `origin/main` as the base, measures nothing (the merge base is `HEAD`), runs the
  rules and keeps the gate, so `main` is checked as before. The comment step is
  gated on the event.
- `pull-requests: write` is granted to this job alone; the workflow's top-level
  `permissions` block stays read-only.
- The comment is one per pull request. Each push edits it rather than adding
  another, found by its first line.
- The job summary gets the same Markdown, so a reader without the pull request page
  (a self-hosted runner's log, say) sees it too.

## Verified locally

Causeway's self-hosted runners are offline, so the job cannot run in CI until Andrew
marks the pull request ready and GitHub's runners pick it up. What was run on this
machine against a scratch clone of Causeway, with the binary built from the commit
this document is in, is recorded in `docs/exit-tests/raw/2026-11-07-phase-3-pr-replay/`:
the ten Phase 2 merge commits, each measured against its parent, with the changed
symbols, the affected rows per group, the five rules' result and the time and size
of each report. The exit test document of Phase 3 reads from the same record.
