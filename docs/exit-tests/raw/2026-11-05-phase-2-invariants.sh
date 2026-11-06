#!/bin/sh
# The invariant half of the Phase 2 exit test: five rules pass on Causeway `main` and
# three planted violations make three of them fail, each naming the planted line,
# while the other two stay green.
#
# Usage: docs/exit-tests/raw/2026-11-05-phase-2-invariants.sh <scratch-dir> [surveyor] [main-sha]
#
# <scratch-dir> holds a shared clone of Causeway (`repo`, made if missing) and a
# detached worktree of it at `main` (`planted`, recreated each run) whose working
# tree is edited and never committed; the store lives beside it. The Causeway checkout
# itself is read for the clone and never written. Exit 0 when every expectation holds.
set -u

scratch=${1:?scratch dir}
surveyor=${2:-target/release/surveyor}
here=$(cd "$(dirname "$0")" && pwd)
rules_dir="$here/../../../examples/causeway-rules"
causeway=${CAUSEWAY:-"$HOME/dev/trusted/causeway"}
tree="$scratch/planted"
store="$scratch/stores/planted-main"
out="$scratch/invariants.txt"

mkdir -p "$scratch/stores"
[ -d "$scratch/repo" ] || git clone --quiet --shared --no-checkout "$causeway" "$scratch/repo"
if [ -n "${3:-}" ]; then main=$3; else
    git -C "$scratch/repo" fetch --quiet https://github.com/andrewdean/causeway main
    main=$(git -C "$scratch/repo" rev-parse FETCH_HEAD)
fi
[ -d "$tree" ] && git -C "$scratch/repo" worktree remove --force "$tree"
git -C "$scratch/repo" worktree add --quiet --detach "$tree" "$main"
rm -rf "$store"

rules=""
for r in 06-component-cross-deps 10-server-rpc-deps 11-server-service-impls 13-island-cross-refs 04-runtime-env-resource-ctors; do
    rules="$rules --rules $rules_dir/$r.rhai"
done
# Runs the five rules, appends the report to the record and returns surveyor's exit.
check() {
    "$surveyor" --repo-root "$tree" --store "$store" check $rules > "$scratch/check.txt" 2>&1
    code=$?
    tee -a "$out" < "$scratch/check.txt"
    return $code
}

: > "$out"
printf 'Causeway main %s\n\n' "$main" | tee -a "$out"
"$surveyor" --repo-root "$tree" --store "$store" index >/dev/null 2>&1 || { echo "index failed"; exit 1; }

printf '## clean tree\n' | tee -a "$out"
check; clean=$?
printf 'exit %s\n\n' "$clean" | tee -a "$out"

# The three plantings: a cross-area path dependency in a component manifest, a gRPC
# service impl in a non-orchestrator shell file, an island importing a sibling.
sed -i.bak 's|^\[dependencies\]$|[dependencies]\ncatalog-component = { path = "../../catalog/core" }|' "$tree/component/governance/core/Cargo.toml" && rm "$tree/component/governance/core/Cargo.toml.bak"
printf '\n\n// planted for the Phase 2 exit test\nstruct Probe;\n\nimpl pb::SearchService for Probe {}\n' >> "$tree/apps/causeway-server/src/marketplace.rs"
printf 'import { requestDecider } from "@causeway/ui-request";\n' | cat - "$tree/web/ui-catalog-filter/src/index.ts" > "$tree/web/ui-catalog-filter/src/index.ts.new" && mv "$tree/web/ui-catalog-filter/src/index.ts.new" "$tree/web/ui-catalog-filter/src/index.ts"
printf '## planted\n' | tee -a "$out"
git -C "$tree" diff --stat | tee -a "$out"
"$surveyor" --repo-root "$tree" --store "$store" update >/dev/null 2>&1 || { echo "update failed"; exit 1; }
check; planted=$?
printf 'exit %s\n' "$planted" | tee -a "$out"

status=0
[ "$clean" -eq 0 ] || { echo "FAIL: the clean tree did not pass"; status=1; }
[ "$planted" -eq 1 ] || { echo "FAIL: the planted tree did not fail"; status=1; }
for expect in \
    'FAIL \[component-cross-deps\]' 'component/governance/core/Cargo.toml:10' \
    'FAIL \[server-rpc-service-impls\]' 'apps/causeway-server/src/marketplace.rs:[0-9]*  implements' \
    'FAIL \[island-cross-references\]' 'web/ui-catalog-filter/src/index.ts:1' \
    'ok   \[server-rpc-contract-deps\]' 'ok   \[runtime-env-resource-ctors\]' \
    '5 rules, 3 violations, 3 failed'; do
    grep -q "$expect" "$out" || { echo "FAIL: expected '$expect' in $out"; status=1; }
done
[ "$status" -eq 0 ] && echo "PASS: five rules pass on main, three planted violations fail three of them"
exit $status
