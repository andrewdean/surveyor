#!/bin/sh
# `surveyor serve` measured on the Causeway scratch store: load time, the latency of
# every JSON route warm (one warm-up, then twenty requests each), an MCP tool call
# over streamable HTTP, eight clients at once, and the time from an `update` into the
# dataset to the server answering from the new graph.
#
# Usage: docs/exit-tests/raw/2026-11-08-phase-3-serve.sh <scratch-dir> <surveyor> [out-file]
#
# <scratch-dir>/federation is the detached Causeway worktree stage 3 left and
# <scratch-dir>/fed-stores/alone-after the store it indexed; both are made if missing.
# Andrew's Causeway checkout and store are never touched.
set -u

scratch=${1:?scratch dir}
surveyor=${2:?surveyor binary}
here=$(cd "$(dirname "$0")" && pwd)
out=${3:-"$here/2026-11-08-phase-3-serve.txt"}
profile="$here/../../../examples/causeway-profile/surveyor.toml"
tree="$scratch/federation"
store="$scratch/fed-stores/alone-after"
work="$scratch/serve-run"
mkdir -p "$work"
[ -d "$tree" ] || git -C "$scratch/repo" worktree add --quiet --detach "$tree" origin/main
[ -d "$store" ] || "$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" index >/dev/null 2>&1

: > "$out"
log() { printf '%s\n' "$*" | tee -a "$out"; }
ms_now() { python3 -c 'import time; print(int(time.time()*1000))'; }
# min, median and max of the numbers on stdin, in milliseconds.
summary() { sort -n | awk '{a[NR]=$1*1000} END {printf "min %.1f  median %.1f  max %.1f ms (n=%d)", a[1], a[int((NR+1)/2)], a[NR], NR}'; }

log "Causeway $(git -C "$tree" rev-parse HEAD) at $tree; $("$surveyor" --version)"
log ""

# Make sure the store is current for the tree, so the server's first load is the measured one.
"$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" update >/dev/null 2>&1

log "## start: load time of the Causeway graph, index and rules"
RUST_LOG=info "$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" --json \
    serve --listen 127.0.0.1:0 --reload-every 2 > "$work/announce.json" 2> "$work/server.log" &
server=$!
trap 'kill $server 2>/dev/null' EXIT INT TERM
for i in $(seq 1 600); do grep -q '"listen"' "$work/announce.json" 2>/dev/null && break; sleep 0.1; done
base="http://$(grep '"listen"' "$work/announce.json" | sed 's/.*: "\([^"]*\)".*/\1/')"
grep "surveyor::serve" "$work/server.log" | sed 's/\x1b\[[0-9;]*m//g; s/^[^ ]* *//' | tee -a "$out"
log "listening at $base"
log ""

hub1=$("$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" --json hubs --top 2 | jq -r '.[0].node.id')
hub2=$("$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" --json hubs --top 2 | jq -r '.[1].node.id')
log "hubs used below: $hub1 ; $hub2"
log "## JSON routes, warm: one warm-up request, then twenty timed (curl time_total)"
log "route	min / median / max"
time_route() {
    label=$1; url=$2
    code=$(curl -s -o "$work/warm.json" -w '%{http_code}' "$url")
    bytes=$(wc -c < "$work/warm.json" | tr -d ' ')
    times=$(for i in $(seq 1 20); do curl -s -o /dev/null -w '%{time_total}\n' "$url"; done | summary)
    log "$label	$times	http $code, $bytes bytes"
}
enc() { python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$1"; }
time_route "repos" "$base/v1/repos"
time_route "stats" "$base/v1/causeway/stats"
time_route "explain DbError" "$base/v1/causeway/explain?q=DbError"
time_route "card DbError" "$base/v1/causeway/card?q=DbError"
time_route "affected DbError depth 3" "$base/v1/causeway/affected?q=DbError&depth=3"
time_route "affected hub depth 3" "$base/v1/causeway/affected?q=$(enc "$hub1")&depth=3"
time_route "path hub1 -> hub2" "$base/v1/causeway/path?from=$(enc "$hub1")&to=$(enc "$hub2")"
time_route "query outbox" "$base/v1/causeway/query?q=outbox"
time_route "search" "$base/v1/causeway/search?q=$(enc "where is development data loaded")"
time_route "context 8000" "$base/v1/causeway/context?task=$(enc "add a retry to the outbox relay drain")&budget=8000"
time_route "source" "$base/v1/causeway/source?target=$(enc "$hub1")"
time_route "check" "$base/v1/causeway/check"
log ""

log "## MCP over streamable HTTP: initialize, then twenty get_node calls"
init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"measure","version":"0"}}}'
curl -s -D "$work/init.headers" -o "$work/init.body" -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -d "$init" "$base/mcp/causeway"
session=$(grep -i '^mcp-session-id:' "$work/init.headers" | sed 's/^[^:]*: *//' | tr -d '\r')
curl -s -o /dev/null -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "Mcp-Session-Id: $session" -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' "$base/mcp/causeway"
call='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_node","arguments":{"node_id":"DbError"}}}'
log "get_node DbError	$(for i in $(seq 1 20); do curl -s -o /dev/null -w '%{time_total}\n' -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "Mcp-Session-Id: $session" -d "$call" "$base/mcp/causeway"; done | summary)"
curl -s -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "Mcp-Session-Id: $session" -d "$call" "$base/mcp/causeway" | grep '^data: {' | head -1 | cut -c1-200 | sed 's/^/first bytes: /' | tee -a "$out"
log ""

log "## eight clients at once: 8 x 25 explain requests, wall time"
t0=$(ms_now)
clients=""
for c in 1 2 3 4 5 6 7 8; do
    (for i in $(seq 1 25); do curl -s -o /dev/null "$base/v1/causeway/explain?q=DbError"; done) &
    clients="$clients $!"
done
# Only the clients: a bare `wait` would also wait for the server.
wait $clients
t1=$(ms_now)
log "200 requests in $((t1 - t0)) ms ($(( 200000 / (t1 - t0 + 1) )) req/s)"
log ""

log "## reload: a document gains a line, update writes it, the server swaps the graph"
doc="$tree/README.md"
printf '\nA line added while the server runs, to measure the reload.\n' >> "$doc"
t0=$(ms_now)
"$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" update > "$work/update.txt" 2>&1
t1=$(ms_now)
log "update: $((t1 - t0)) ms; $(head -1 "$work/update.txt")"
for i in $(seq 1 600); do
    grep -q '"reloaded"\|reloaded' "$work/server.log" && break
    sleep 0.05
done
t2=$(ms_now)
log "server answered from the new graph $((t2 - t1)) ms after update returned (poll every 2 s); its own reload line:"
grep "surveyor::serve.*reloaded" "$work/server.log" | tail -1 | sed 's/\x1b\[[0-9;]*m//g; s/^[^ ]* *//' | tee -a "$out"
git -C "$tree" checkout --quiet -- README.md
"$surveyor" --repo-root "$tree" --store "$store" --profile "$profile" update >/dev/null 2>&1
log ""
log "## server log, load and reload lines"
grep "surveyor::serve" "$work/server.log" | sed 's/\x1b\[[0-9;]*m//g; s/^[^ ]* *//' | tee -a "$out"
kill $server 2>/dev/null
trap - EXIT
