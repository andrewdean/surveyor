"""Replay the last N commits of the Causeway clone through `surveyor update`.

Usage: python3 replay.py <store> <profile> <N> [label]

Checks out main~N, indexes cold, then steps forward one commit at a time running
`surveyor update`, timing every step. Writes a TSV beside the store.
"""

import re
import subprocess
import sys
import time
from pathlib import Path

S = Path("/private/tmp/claude-501/-Users-andrew-dev-trusted-acme/d1212ed1-03a3-4926-a428-0e9793681cd8/scratchpad")
B = "/Users/andrew/dev/trusted/surveyor/target/release/surveyor"
REPO = S / "cw-replay"

store, profile, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
label = sys.argv[4] if len(sys.argv) > 4 else Path(store).name
out = S / "p1" / f"replay-{label}.tsv"


def git(*args):
    return subprocess.run(["git", "-C", str(REPO), *args], check=True, capture_output=True, text=True).stdout.strip()


def surveyor(*args):
    t0 = time.perf_counter()
    p = subprocess.run([B, "--repo-root", str(REPO), "--store", store, "--profile", profile, *args],
                       capture_output=True, text=True)
    return time.perf_counter() - t0, p.stdout + p.stderr


def counts(text):
    m = re.search(r"files (\d+)\s+nodes (\d+)\s+edges (\d+)\s+unresolved (\d+)", text)
    return m.groups() if m else ("", "", "", "")


def changed(text):
    m = re.search(r"changed: (\d+) modified, (\d+) added, (\d+) deleted; relinked (\d+) files", text)
    return m.groups() if m else ("", "", "", "")


def mode(text):
    if "(full:" in text:
        return "full"
    if "(incremental" in text or "relinked" in text:
        return "incremental"
    return "?"


rows = []
git("checkout", "-q", "--detach", f"main~{n}")
subprocess.run(["rm", "-rf", store], check=True)
wall, text = surveyor("index")
f, nd, ed, un = counts(text)
rows.append(["cold", git("rev-parse", "--short", "HEAD"), "", "", "", "", "", "", f"{wall:.2f}", "index", f, nd, ed, un, ""])
print(f"cold index main~{n}: {wall:.2f}s  {text.strip().splitlines()[0]}", flush=True)

for i in range(n - 1, -1, -1):
    git("checkout", "-q", "--detach", f"main~{i}")
    sha = git("rev-parse", "--short", "HEAD")
    subject = git("log", "-1", "--format=%s")
    touched = git("diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").splitlines()
    wall, text = surveyor("update")
    f, nd, ed, un = counts(text)
    mo, ad, de, rl = changed(text)
    first = text.strip().splitlines()[0] if text.strip() else ""
    rows.append([str(n - i), sha, subject, str(len(touched)), mo, ad, de, rl, f"{wall:.2f}", mode(text), f, nd, ed, un, first])
    print(f"{n - i:>2} {sha} {wall:6.2f}s {mode(text):11} commit touched {len(touched):3} | {first}", flush=True)

header = ["step", "commit", "subject", "commit_files", "modified", "added", "deleted", "relinked", "wall_s", "mode", "files", "nodes", "edges", "unresolved", "line"]
out.write_text("\n".join("\t".join(r) for r in [header, *rows]) + "\n")
git("checkout", "-q", "main")
print(f"wrote {out}")
