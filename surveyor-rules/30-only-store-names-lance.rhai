// Lance and Arrow are the store's concern; every other crate goes through LanceStore.
const ID = "only-store-names-lance";
const DESC = "lancedb and arrow are dependencies of surveyor-store and of nothing else";

const STORE_ONLY = ["pkg:cargo/lancedb", "pkg:cargo/arrow-array", "pkg:cargo/arrow-schema"];

fn check(g) {
    let out = [];
    for dep in STORE_ONLY {
        for e in g.edges_to(dep, "DEPENDS_ON") {
            if e.src != "pkg:cargo/surveyor-store" {
                out.push(violation(e, "depends on the storage engine directly; use surveyor-store"));
            }
        }
    }
    out
}
