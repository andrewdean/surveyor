// Ports `component_cross_deps` (bootstrap/cw-vsa/src/checks.rs, ADR-0014): a
// crate under component/ may take path dependencies only within its own area
// (the first directory under component/), or on crates under crates/ and
// services/.
//
// Graph facts: Package nodes carry a `dir` attribute (the manifest's
// directory) and DEPENDS_ON edges from every `[dependencies]`,
// `[dev-dependencies]` and `[build-dependencies]` entry, workspace-table
// entries included. A first-party dependency is one whose Package node has a
// `dir`; registry crates have none.
//
// Difference from cw-vsa: cw-vsa reads `cargo metadata --no-deps`, where a
// registry crate replaced through `[patch.crates-io]` keeps its registry
// identity. The graph resolves the patch to the vendored package, so
// component/Cargo.toml's `fastembed = { path = "../vendor/fastembed" }` shows
// as a dependency on vendor/. A vendored substitute is not slice code, so
// vendor/ is allowed alongside crates/ and services/.
const ID = "component-cross-deps";
const DESC = "components must not depend on sibling component implementation crates; depend on crates/ domain+contract crates or services/ wire crates instead (ADR-0014)";

const ALLOWED_OUTSIDE = ["crates/", "services/", "vendor/"];

fn area_of(dir) {
    dir.split("/")[1]
}

fn check(g) {
    let out = [];
    for p in g.nodes("Package") {
        let dir = p.attributes.dir;
        if dir == () || !dir.starts_with("component/") { continue; }
        let own = area_of(dir);
        for e in g.edges_from(p.id, "DEPENDS_ON") {
            let dep = g.node(e.dst);
            if dep == () { continue; }
            let dep_dir = dep.attributes.dir;
            if dep_dir == () { continue; }
            if dep_dir.starts_with("component/") {
                if area_of(dep_dir) != own {
                    out.push(violation(e, "-> " + dep.name + " (" + dep_dir + "): cross-slice component dependency"));
                }
            } else if !ALLOWED_OUTSIDE.some(|prefix| dep_dir.starts_with(prefix)) {
                out.push(violation(e, "-> " + dep.name + " (" + dep_dir + "): path dependency outside crates/ and services/"));
            }
        }
    }
    out
}
