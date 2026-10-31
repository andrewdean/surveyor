// The model crate is pure: types, hashing and serialization only.
const ID = "model-has-no-io";
const DESC = "surveyor-model depends on no crate that performs I/O or links a runtime";

const ALLOWED = ["pkg:cargo/serde", "pkg:cargo/serde_json", "pkg:cargo/blake3"];

fn check(g) {
    let out = [];
    for e in g.edges_from("pkg:cargo/surveyor-model", "DEPENDS_ON") {
        if !(e.dst in ALLOWED) {
            out.push(violation(e, "the model crate must stay free of I/O dependencies"));
        }
    }
    out
}
