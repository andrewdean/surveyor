// Ports `domain-dependents` (bootstrap/cw-vsa/src/checks.rs): no first-party
// manifest depends on the dissolved causeway-domain crate.
//
// Graph facts: DEPENDS_ON edges into pkg:cargo/causeway-domain, declared from
// each dependent's Cargo.toml. cw-vsa greps every Cargo.toml for
// `causeway-domain\s*=`, which would also match a commented-out line; the
// graph counts declared dependencies. When the crate is absent the node does
// not exist and there are no edges, which is the passing case.
const ID = "domain-god-crate-dependents";
const DESC = "first-party Cargo.toml dependents of the dissolved domain crate";
const SEVERITY = "budget";
const BUDGET = 0;

fn check(g) {
    let out = [];
    for e in g.edges_to("pkg:cargo/causeway-domain", "DEPENDS_ON") {
        out.push(violation(e, "depends on the dissolved domain crate"));
    }
    out
}
