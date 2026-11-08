#!/bin/sh
# Open a pull request only when Surveyor's gate passes.
#
# Usage: examples/harness/open-pr.sh [--base <ref>] [gh pr create arguments...]
#
# Runs `surveyor update`, then `surveyor pr` against the base (the merge base of HEAD
# and the default branch unless --base is given) with the current session's ledger
# embedded, writes the comment to a directory, and refuses to open the pull request
# when a `fail` rule failed (exit 1) or the report could not be made (exit 2). On
# exit 0 it opens a draft whose body is the report, followed by whatever the
# repository's template adds. Surveyor never calls GitHub; this script does.
#
# SURVEYOR_SESSION names the ledger (set by `surveyor session start`, or by the
# Claude Code hook); SURVEYOR_PR_DIR is where the report lands (default .surveyor-pr).
set -eu

base=""
if [ "${1:-}" = "--base" ]; then
    base=$2
    shift 2
fi
out=${SURVEYOR_PR_DIR:-.surveyor-pr}

surveyor update >&2
if [ -n "$base" ]; then
    surveyor pr --base "$base" --out "$out"
else
    surveyor pr --out "$out"
fi
# `set -e` has stopped the script on exit 1 (a failed invariant) or 2 (no report);
# the comment that explains why is in "$out/pr.md".

gh pr create --draft --body-file "$out/pr.md" "$@"
