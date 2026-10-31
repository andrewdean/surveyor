// Ports `island_cross_refs` (bootstrap/cw-vsa/src/checks.rs): a web island
// (`web/ui-*`, published as `@causeway/ui-*`) neither depends on nor imports a
// sibling island.
//
// Graph facts: npm Package nodes from each island's package.json, DEPENDS_ON
// edges from its dependency sections, and IMPORTS edges from the island's
// source files to the npm package an import specifier names. cw-vsa greps the
// manifest and sources for the text `@causeway/ui-*`, so it would also count a
// mention in a comment or a string; the graph counts declared dependencies
// and imports. Distinct `island -> reference` pairs, as cw-vsa counts them.
const ID = "island-cross-references";
const DESC = "web ui-* islands importing a sibling island";
const SEVERITY = "budget";
const BUDGET = 0;

const ISLAND_PKG = "pkg:npm/@causeway/ui-";

fn island_dir(file) {
    // web/ui-<name>/...
    let parts = file.split("/");
    if parts.len() < 2 || parts[0] != "web" || !parts[1].starts_with("ui-") { return (); }
    parts[0] + "/" + parts[1]
}

fn check(g) {
    let out = [];
    let seen = [];
    for e in g.edges("DEPENDS_ON") {
        if e.src.starts_with(ISLAND_PKG) && e.dst.starts_with(ISLAND_PKG) && e.src != e.dst {
            let src = g.node(e.src);
            let dir = if src == () || src.file == () { e.src } else { island_dir(src.file) };
            let key = dir + " -> " + e.dst;
            if !(key in seen) {
                seen.push(key);
                out.push(violation(e, key + ": island depends on a sibling island"));
            }
        }
    }
    for e in g.edges("IMPORTS") {
        if !e.dst.starts_with(ISLAND_PKG) || e.file == () { continue; }
        let dir = island_dir(e.file);
        if dir == () || g.package_of(e.src) == e.dst { continue; }
        let key = dir + " -> " + e.dst;
        if !(key in seen) {
            seen.push(key);
            out.push(violation(e, key + ": island imports a sibling island"));
        }
    }
    out
}
