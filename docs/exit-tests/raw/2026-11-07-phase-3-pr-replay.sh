#!/bin/sh
# Replays the ten Phase 2 merge commits through `surveyor pr`, each measured against
# its parent (the base commit the task's pack was planned on), and records what the
# comment carries: changed symbols, affected rows per group, the five rules, time and
# size. Stage 2 of Phase 3 is measured by this run.
#
# Usage: docs/exit-tests/raw/2026-11-07-phase-3-pr-replay.sh <scratch-dir> [surveyor] [out-dir]
#
# <scratch-dir> is the Phase 2 scratch directory: `repo` is a shared clone of
# Causeway that holds the merge commits. A detached worktree `pr-replay` is made and
# reused; each store is built fresh at the merge commit with the Causeway profile
# from Surveyor's examples (name tier only; a full index, 10 to 60 s each on the
# laptop that wrote this). The Causeway checkout itself is never written.
set -u

scratch=${1:?scratch dir}
surveyor=${2:-target/release/surveyor}
here=$(cd "$(dirname "$0")" && pwd)
out=${3:-"$here/2026-11-07-phase-3-pr-replay"}
profile="$here/../../../examples/causeway-profile/surveyor.toml"
rules_dir="$here/../../../examples/causeway-rules"
tree="$scratch/pr-replay"
mkdir -p "$out" "$scratch/pr-stores"

rules=""
for r in 06-component-cross-deps 10-server-rpc-deps 11-server-service-impls 13-island-cross-refs 04-runtime-env-resource-ctors; do
    rules="$rules --rules $rules_dir/$r.rhai"
done

# pr, base, merge: the ten tasks of docs/eval/causeway-tasks.toml.
tasks="
1073 83dd9e101d9321b22b6a75fce5cbba4abdc12c3a ec266ebaf2c12cfb5bcbde9b2907204ac9b840a0
1064 08bc6c236da8c6c87f42775094233637b8191704 36157f3ea844fa6ba105fd68aa15f4ed9bc42562
1068 1bdc4035b097418722f4a93829cc6637aef84562 329206085616ac543a858abb9dbe348ae49de552
1066 36157f3ea844fa6ba105fd68aa15f4ed9bc42562 ec36fd145df4185446efa6d74d0b2ada710377b3
1040 c6028d8602733f8ee89e8e816b29b21e83612ec9 ed9a46459f408d8ae28ff3a8073879842b9de7e1
1027 33ced4d72079a23c6be09e811a542ccc076b31ce 904065a7cecb5dc1600b07ac3570d253b0513f52
1021 6b78aed1532aa53eef57d074e4c9b57834297143 e5ad13778ccd0a98cdd9ae81b6b9b95c100b578b
1014 dad1238120246c5f76275bd4e811c69728164386 a1d4cb0888496b898ecdc877dc5a9b5d685a7cd7
1039 9c15a151987e14a6bbf8e2a9ec82cc6e1d047197 a3779e8e4d284ee3543db1899e7db7ca796a31c5
1071 8647a8ff9a92d3917ddb130c9e3de27c3a048930 ac10ddb663950d1e913c535c2002f9111e58a6c1
"

[ -d "$tree" ] || git -C "$scratch/repo" worktree add --quiet --detach "$tree" HEAD
printf 'pr\tfiles\tsymbols\taffected\tcode\tdata\ttasks\tdocs\tfiles_pkgs\tother\trules_failed\texit\tindex_s\tpr_s\tmd_bytes\tjson_bytes\n' > "$out/summary.tsv"

echo "$tasks" | while read -r pr base merge; do
    [ -n "$pr" ] || continue
    store="$scratch/pr-stores/$pr"
    git -C "$tree" checkout --quiet --detach "$merge"
    rm -rf "$store"
    t0=$(date +%s.%N)
    "$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" index > "$out/$pr.index.log" 2>&1 || { echo "index failed for $pr"; continue; }
    t1=$(date +%s.%N)
    "$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" pr --base "$base" --out "$out/$pr" $rules > "$out/$pr.pr.log" 2>&1
    code=$?
    t2=$(date +%s.%N)
    j="$out/$pr/pr.json"
    group() { python3 -c "import json,sys; r=json.load(open(sys.argv[1])); print(sum(len(g['rows']) for g in r['impact']['groups'] if g['group']==sys.argv[2]))" "$j" "$1"; }
    files=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['change']['files_changed'])" "$j")
    symbols=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['change']['symbols_changed'])" "$j")
    affected=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['impact']['total'])" "$j")
    failed=$(python3 -c "import json,sys; r=json.load(open(sys.argv[1])); print(sum(1 for o in r['rules']['report']['outcomes'] if not o['passed']) if r['rules'] else 'none')" "$j")
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%.1f\t%.2f\t%s\t%s\n' \
        "$pr" "$files" "$symbols" "$affected" "$(group code)" "$(group data)" "$(group tasks)" "$(group documents)" "$(group files)" "$(group other)" \
        "$failed" "$code" "$(echo "$t1 - $t0" | bc)" "$(echo "$t2 - $t1" | bc)" \
        "$(wc -c < "$out/$pr/pr.md" | tr -d ' ')" "$(wc -c < "$j" | tr -d ' ')" | tee -a "$out/summary.tsv"
done
