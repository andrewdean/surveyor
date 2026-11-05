#!/bin/sh
# Builds one Surveyor store per base commit of docs/eval/causeway-tasks.toml, so the
# task eval sees the tree an agent would have been handed before each change.
#
# Usage: docs/eval/causeway-task-stores.sh <scratch-dir> [surveyor-binary]
#
# <scratch-dir> is a scratch area outside the Causeway checkout; the script makes a
# shared clone of Causeway under it (`repo`), one detached worktree per base commit
# (`<sha8>`) and one store per base (`stores/<sha8>`). Causeway's own checkout is read,
# never written. Re-running skips what exists.
set -eu

scratch=${1:?scratch dir}
surveyor=${2:-target/release/surveyor}
here=$(cd "$(dirname "$0")" && pwd)
tasks="$here/causeway-tasks.toml"
causeway=${CAUSEWAY:-"$HOME/dev/trusted/causeway"}
profile="$here/../../examples/causeway-profile/surveyor.toml"

mkdir -p "$scratch/stores"
[ -d "$scratch/repo" ] || git clone --quiet --shared --no-checkout "$causeway" "$scratch/repo"

grep -E '^base *= *"' "$tasks" | sed -E 's/.*"([0-9a-f]+)".*/\1/' | sort -u | while read -r base; do
    short=$(printf '%s' "$base" | cut -c1-8)
    tree="$scratch/$short"
    store="$scratch/stores/$short"
    [ -d "$tree" ] || git -C "$scratch/repo" worktree add --quiet --detach "$tree" "$base"
    if [ ! -d "$store" ]; then
        printf '%s ' "$short"
        "$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" index --json \
            | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("nodes"), "nodes", d.get("edges"), "edges", round(d.get("seconds",0),1), "s")' \
            || true
    fi
done
