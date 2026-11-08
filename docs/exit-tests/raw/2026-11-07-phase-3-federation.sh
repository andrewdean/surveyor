#!/bin/sh
# Federation measured: Causeway alone before and after stage 3 (cold index and empty
# update, three runs each), then Causeway, Cairn and torrentfx in one scratch store with
# the federated edges, `explain` from both sides and `affected --all-repos`.
#
# Usage: docs/exit-tests/raw/2026-11-07-phase-3-federation.sh <scratch-dir> <surveyor-before> <surveyor-after> [out-file]
#
# <scratch-dir>/repo is a shared clone of Causeway; a detached worktree `federation` of
# it at origin/main is made if missing. Cairn is read from CAIRN (default
# ~/dev/cairn/main) and torrentfx from TORRENTFX (default ~/dev/trusted/torrentfx);
# neither is written. Andrew's Causeway checkout and store are never touched.
set -u

scratch=${1:?scratch dir}
before=${2:?surveyor binary before stage 3}
after=${3:?surveyor binary after stage 3}
here=$(cd "$(dirname "$0")" && pwd)
out=${4:-"$here/2026-11-07-phase-3-federation.txt"}
profile="$here/../../../examples/causeway-profile/surveyor.toml"
cairn=${CAIRN:-"$HOME/dev/cairn/main"}
torrentfx=${TORRENTFX:-"$HOME/dev/trusted/torrentfx"}
tree="$scratch/federation"
[ -d "$tree" ] || git -C "$scratch/repo" worktree add --quiet --detach "$tree" origin/main

: > "$out"
log() { printf '%s\n' "$*" | tee -a "$out"; }
seconds() { t0=$(date +%s.%N); "$@" >/dev/null 2>&1; code=$?; t1=$(date +%s.%N); printf '%.2f' "$(echo "$t1 - $t0" | bc)"; return $code; }

log "Causeway $(git -C "$tree" rev-parse HEAD) at $tree"
log "Cairn $(git -C "$cairn" rev-parse HEAD); torrentfx $(git -C "$torrentfx" rev-parse HEAD)"
log ""
log "## Causeway alone: seconds, three runs each (cold index; update with nothing changed)"
log "label	command	run	seconds"
for label in before after; do
    if [ "$label" = before ]; then S=$before; else S=$after; fi
    store="$scratch/fed-stores/alone-$label"
    for i in 1 2 3; do
        rm -rf "$store"
        log "$label	index	$i	$(seconds "$S" --repo-root "$tree" --store "$store" --profile "$profile" index)"
    done
    for i in 1 2 3; do
        log "$label	update	$i	$(seconds "$S" --repo-root "$tree" --store "$store" --profile "$profile" update)"
    done
    ls "$store" | tr '\n' ' ' | sed 's/^/tables: /' | tee -a "$out"; echo | tee -a "$out"
done

log ""
log "## Causeway, Cairn and torrentfx in one store (binary after)"
store="$scratch/fed-stores/three"
rm -rf "$store"
log "index causeway: $(seconds "$after" --repo-root "$tree" --store "$store" --profile "$profile" index) s"
log "index cairn: $(seconds "$after" --repo-root "$cairn" --store "$store" --repo-id cairn index) s (pass runs: 2 repositories)"
log "## stats with two repositories"
"$after" --store "$store" --repo-id causeway stats 2>&1 | tee -a "$out"
log "## federated edges with two repositories"
"$after" --store "$store" federate 2>&1 | tee -a "$out"
log "## explain from Causeway's side"
"$after" --store "$store" --repo-id causeway explain pkg:npm/@torrentfx/core 2>&1 | tee -a "$out"
log "## explain from Cairn's side"
"$after" --store "$store" --repo-id cairn explain pkg:npm/@torrentfx/core 2>&1 | tee -a "$out"
log "## affected across both from the declared package (depth 2)"
"$after" --store "$store" --repo-id causeway --all-repos affected pkg:npm/@torrentfx/core --depth 2 2>&1 | tee -a "$out"
log "## update with nothing changed, two repositories, three runs"
for i in 1 2 3; do
    log "update causeway	$i	$(seconds "$after" --repo-root "$tree" --store "$store" --profile "$profile" update)"
done
log ""
log "index torrentfx: $(seconds "$after" --repo-root "$torrentfx" --store "$store" --repo-id torrentfx index) s (pass runs: 3 repositories)"
log "## federated edges with three repositories"
"$after" --store "$store" federate 2>&1 | tee -a "$out"
log "## explain from Cairn's side with two declarers"
"$after" --store "$store" --repo-id cairn explain pkg:npm/@torrentfx/core 2>&1 | tee -a "$out"
log "## Cairn's extension package with two declarers"
"$after" --store "$store" --repo-id cairn explain pkg:npm/cairn-vscode 2>&1 | tee -a "$out"
log "## the correct negative: Surveyor's own repository with Cairn shares only registry stubs"
neg="$scratch/fed-stores/negative"
rm -rf "$neg"
"$after" --repo-root "$here/../../.." --store "$neg" index >/dev/null 2>&1
"$after" --repo-root "$cairn" --store "$neg" --repo-id cairn index >/dev/null 2>&1
"$after" --store "$neg" federate 2>&1 | tee -a "$out"
