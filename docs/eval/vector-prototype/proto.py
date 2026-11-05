"""Vector-half prototype: does dense retrieval fused with BM25 by RRF add recall?

Inputs (all produced by the Surveyor CLI, see run.sh):
  nodes.jsonl     one node per line: id, kind, name, qualified_name, file, doc, attributes
  questions.toml  the eval questions
  bm25/<n>.json   `surveyor search --json --limit 100` for question n (0-based)

Embeddings are cached by content hash of the node text in cache.npz, which is what
an incremental Rust implementation would do too.
"""
import hashlib, json, os, sys, time, tomllib
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
K = 5
DEEP = 20
RRF_K = 60
MODEL = "BAAI/bge-small-en-v1.5"


def node_text(n):
    attrs = n.get("attributes") or {}
    parts = [n["kind"], n["name"], n.get("qualified_name", ""), (n.get("file") or "").replace("/", " ")]
    for key in ("signature",):
        if n.get(key):
            parts.append(n[key][:200])
    if n.get("doc"):
        parts.append(n["doc"][:1000])
    for key in ("description", "headings", "run"):
        if attrs.get(key) and not (key == "description" and n.get("doc")):
            parts.append(attrs[key][:1000])
    return " . ".join(p for p in parts if p)


def load_nodes():
    nodes = []
    with open(os.path.join(HERE, "nodes.jsonl")) as f:
        for line in f:
            nodes.append(json.loads(line))
    return nodes


def embeddings(nodes):
    from fastembed import TextEmbedding
    texts = [node_text(n) for n in nodes]
    hashes = [hashlib.blake2b(t.encode(), digest_size=16).hexdigest() for t in texts]
    cache_path = os.path.join(HERE, "cache.npz")
    cache = {}
    if os.path.exists(cache_path):
        z = np.load(cache_path, allow_pickle=False)
        cache = dict(zip(z["hashes"].tolist(), z["vectors"]))
    missing = [i for i, h in enumerate(hashes) if h not in cache]
    model = TextEmbedding(MODEL)
    started = time.time()
    if missing:
        batch = [texts[i] for i in missing]
        for i, vec in zip(missing, model.embed(batch, batch_size=256)):
            cache[hashes[i]] = np.asarray(vec, dtype=np.float32)
        print(f"embedded {len(missing)} of {len(texts)} texts in {time.time()-started:.1f}s", file=sys.stderr)
        np.savez(cache_path, hashes=np.array(list(cache.keys())), vectors=np.stack(list(cache.values())))
    else:
        print(f"all {len(texts)} embeddings cached", file=sys.stderr)
    mat = np.stack([cache[h] for h in hashes])
    mat /= np.linalg.norm(mat, axis=1, keepdims=True) + 1e-9
    return model, mat


def rrf(lists, k=RRF_K):
    scores = {}
    for ranked in lists:
        for rank, node_id in enumerate(ranked, start=1):
            scores[node_id] = scores.get(node_id, 0.0) + 1.0 / (k + rank)
    return [i for i, _ in sorted(scores.items(), key=lambda kv: (-kv[1], kv[0]))]


def score(expect, ranked):
    exp = set(expect)
    top = ranked[:K]
    found = sum(1 for i in top if i in exp)
    first = next((r for r, i in enumerate(ranked[:DEEP], start=1) if i in exp), None)
    return found > 0, found / max(len(exp), 1), (1.0 / first if first else 0.0), first


def main():
    nodes = load_nodes()
    ids = [n["id"] for n in nodes]
    with open(os.path.join(HERE, "questions.toml"), "rb") as f:
        questions = tomllib.load(f)["question"]
    model, mat = embeddings(nodes)
    queries = [q["ask"] for q in questions]
    qvecs = np.stack([np.asarray(v, dtype=np.float32) for v in model.query_embed(queries)])
    qvecs /= np.linalg.norm(qvecs, axis=1, keepdims=True) + 1e-9
    systems = {"bm25": [], "dense": [], "rrf": [], "rrf-bm25x2": []}
    detail = []
    for qi, q in enumerate(questions):
        with open(os.path.join(HERE, "bm25", f"{qi}.json")) as f:
            bm25 = [r["node"]["id"] for r in json.load(f)]
        sims = mat @ qvecs[qi]
        order = np.argsort(-sims)[:100]
        dense = [ids[i] for i in order]
        fused = rrf([bm25, dense])
        fused2 = rrf([bm25, bm25, dense])
        row = {}
        for name, ranked in (("bm25", bm25), ("dense", dense), ("rrf", fused), ("rrf-bm25x2", fused2)):
            hit, rec, rr, first = score(q["expect"], ranked)
            systems[name].append((hit, rec, rr))
            row[name] = (hit, rec, first)
        detail.append((q["ask"], row, dense[:5]))
    print(f"{'system':<12}{'hit@5':>8}{'recall@5':>10}{'MRR@20':>8}")
    for name, rows in systems.items():
        n = len(rows)
        print(f"{name:<12}{sum(h for h,_,_ in rows):>5}/{n:<3}{sum(r for _,r,_ in rows)/n:>9.2f}{sum(m for _,_,m in rows)/n:>8.2f}")
    print()
    for ask, row, dense_top in detail:
        cells = "  ".join(f"{k}:{'hit' if v[0] else 'miss'}/{v[1]:.2f}/{'#'+str(v[2]) if v[2] else '--'}" for k, v in row.items())
        print(f"- {ask}\n    {cells}")
        if not row["bm25"][0] or not row["rrf"][0]:
            print("    dense top5: " + ", ".join(dense_top))


if __name__ == "__main__":
    main()
