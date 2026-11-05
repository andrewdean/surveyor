// Crates depend downward only: model < extract, link, graph < search, store < rules < mcp < app.
const ID = "crate-layering";
const DESC = "a Surveyor crate depends only on crates in lower layers";

const LAYERS = #{
    "pkg:cargo/surveyor-model": 0,
    "pkg:cargo/surveyor-extract": 1,
    "pkg:cargo/surveyor-link": 2,
    "pkg:cargo/surveyor-graph": 2,
    "pkg:cargo/surveyor-search": 3,
    "pkg:cargo/surveyor-store": 3,
    "pkg:cargo/surveyor-rules": 4,
    "pkg:cargo/surveyor-mcp": 5,
    "pkg:cargo/surveyor": 6,
};

fn check(g) {
    let out = [];
    for e in g.edges("DEPENDS_ON") {
        if !(e.src in LAYERS) || !(e.dst in LAYERS) { continue; }
        if LAYERS[e.dst] >= LAYERS[e.src] {
            out.push(violation(e, "depends on a crate in the same or a higher layer"));
        }
    }
    out
}
