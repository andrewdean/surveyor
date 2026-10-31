// Ports `domain-modules` (bootstrap/cw-vsa/src/checks.rs): the shared-domain
// god crate is dissolved; no module file may reappear directly under
// crates/causeway-domain/src/.
//
// Graph facts: File nodes under crates/causeway-domain/src/. cw-vsa matches
// `^crates/causeway-domain/src/[^/]+\.rs$`; the depth test is on the path.
const ID = "domain-god-crate-modules";
const DESC = "shared-domain god-crate modules (crate DISSOLVED)";
const SEVERITY = "budget";
const BUDGET = 0;

const SRC = "crates/causeway-domain/src/";

fn check(g) {
    let out = [];
    for f in g.nodes_under("File", SRC) {
        let rest = f.file.sub_string(SRC.len());
        if rest.ends_with(".rs") && !rest.contains("/") {
            out.push(violation(f, "module of the dissolved domain crate"));
        }
    }
    out
}
