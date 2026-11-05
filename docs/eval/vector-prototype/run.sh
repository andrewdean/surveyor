#!/bin/sh
# Reproduces the vector-half measurement: dumps the nodes and the BM25 rankings from a
# Surveyor store, embeds every node with bge-small through fastembed, fuses the two
# rankings by reciprocal rank fusion and scores them against the questions file.
#
#   docs/eval/vector-prototype/run.sh <repo-root> <store> <questions.toml> [work-dir]
#
# Needs `surveyor` on the path (a build of this repository), `uv`, Python 3.12 and
# network access for the first run (the model is downloaded once). Embedding 61,779
# Causeway nodes took 28 minutes on an Apple-silicon laptop CPU; the cache in the
# work directory keeps them by content hash, so a second run embeds only what changed.
set -eu
repo=$1; store=$2; questions=$3; work=${4:-/tmp/surveyor-vector-prototype}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/bm25"
cp "$questions" "$work/questions.toml"
cp "$here/proto.py" "$work/proto.py"
surveyor --repo-root "$repo" --store "$store" --json nodes --limit 200000 \
  | python3 -c 'import json,sys; [print(json.dumps(n)) for n in json.load(sys.stdin)]' > "$work/nodes.jsonl"
python3 - "$repo" "$store" "$work" <<'PY'
import json, subprocess, sys, tomllib
repo, store, work = sys.argv[1:4]
for i, q in enumerate(tomllib.load(open(f"{work}/questions.toml", "rb"))["question"]):
    out = subprocess.run(["surveyor", "--repo-root", repo, "--store", store, "--json",
                          "search", q["ask"], "--limit", "100"], capture_output=True, text=True).stdout
    open(f"{work}/bm25/{i}.json", "w").write(out)
PY
cd "$work"
[ -d .venv ] || uv venv --python 3.12 .venv >/dev/null
uv pip install --python .venv/bin/python --quiet fastembed numpy
.venv/bin/python -I proto.py
