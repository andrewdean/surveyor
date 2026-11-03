"""Check that every span a Surveyor answer cites contains the text the edge claims.

Usage: python3 cite_check.py <repo-root> <answer.json>...

For a member: the member's name inside its own lines. For a relation: the name of
one endpoint (subject at a call or import site, declaration at a definition) inside
the evidence span. For an affected hit or a path step: the name of the edge's
target or source inside the evidence span.
"""

import json
import sys
from functools import lru_cache
from pathlib import Path

root = Path(sys.argv[1])


@lru_cache(maxsize=None)
def lines_of(path):
    return (root / path).read_text(errors="replace").splitlines()


def span_text(evidence):
    path, _, rng = evidence.rpartition(":")
    a, _, b = rng.partition("-")
    a, b = int(a), int(b or a)
    return "\n".join(lines_of(path)[a - 1 : b])


def simple(name):
    return name.split("#")[0].rsplit("::", 1)[-1].rsplit(".", 1)[-1]


def names_of_id(node_id):
    return simple(node_id)


checked = failed = 0
failures = []


def check(evidence, candidates, where):
    global checked, failed
    checked += 1
    try:
        text = span_text(evidence)
    except (OSError, ValueError) as e:
        failed += 1
        failures.append((where, evidence, f"unreadable: {e}"))
        return
    if not any(c and c in text for c in candidates):
        failed += 1
        failures.append((where, evidence, f"none of {sorted(set(candidates))} in span"))


for file in sys.argv[2:]:
    text = open(file).read()
    if not text.strip():
        continue
    d = json.loads(text)
    tag = Path(file).stem
    if "members" in d:
        subject = d["node"]["name"]
        for m in d.get("members", []):
            if m.get("file") and m.get("lines"):
                check(f"{m['file']}:{m['lines']}", [m["name"]], f"{tag} member")
        for direction in ("incoming", "outgoing"):
            for r in d.get(direction, []):
                other = r["node"]["name"]
                for ev in r.get("evidence", []):
                    check(ev, [subject, other, simple(r["node"]["id"])], f"{tag} {direction} {r['kind']}")
    elif "affected" in d:
        for h in d["affected"]:
            via = h["via"]
            for ev in via.get("evidence", []):
                check(ev, [names_of_id(via["dst"]), names_of_id(via["src"])], f"{tag} affected {via['kind']}")
    elif "steps" in d:
        for s in d["steps"]:
            edge = s.get("edge")
            if edge:
                for ev in edge.get("evidence", []):
                    check(ev, [names_of_id(edge["dst"]), names_of_id(edge["src"])], f"{tag} path {edge['kind']}")

print(f"citations checked: {checked}, verified: {checked - failed}, failed: {failed}")
for where, ev, why in failures:
    print(f"  {where}: {ev}: {why}")
