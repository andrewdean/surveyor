// Ports `cargo_feature_isolation` (bootstrap/cw-vsa/src/checks.rs), which the gate
// runs twice (server-sniffer-isolation, server-parquet-isolation): a minimal build of
// causeway-server, `cargo tree -p causeway-server --no-default-features -i <crate>`,
// must reach neither causeway-sniffer nor parquet.
//
// Graph facts: the DEPENDS_ON edges of every first-party manifest carry the entry's
// `section`, `key`, `optional`, `default_features` and `features`, and each Package
// node carries its `[features]` table as `feature.<name>` attributes.
// `g.cargo_build(root, default_features)` resolves them the way cargo unifies
// features and returns the packages a build reaches (`packages`) and how each was
// first reached (`via`).
//
// Differences from cw-vsa, stated rather than hidden:
// - cw-vsa asks cargo, which resolves the whole lock file; the graph holds first-party
//   manifests only, so a registry crate is a leaf. A path to parquet through a registry
//   crate's own dependencies would be found by cargo and missed here.
// - An entry with `workspace = true` inherits features and default-features from
//   `[workspace.dependencies]`, which the extractor does not read; the rule takes what
//   the entry itself says.
// - cargo tree follows the root's dev-dependencies and no one else's; so does the walk.
const ID = "cargo-feature-isolation";
const DESC = "causeway-server minimal builds must not depend on causeway-sniffer (keep sniffer worker code behind causeway-metadata-ops/sniffer) or parquet (keep parquet preview behind tabular-explorer)";
const SEVERITY = "fail";

const ROOT = "pkg:cargo/causeway-server";
const ISOLATED = [
    ["pkg:cargo/causeway-sniffer", "causeway-server minimal builds must not depend on causeway-sniffer"],
    ["pkg:cargo/parquet", "causeway-server minimal builds must not depend on parquet"],
];

// The dependency chain from the root to `pkg`, as crate names.
fn chain_to(build, pkg) {
    let names = [pkg.sub_string(10)]; // strip "pkg:cargo/"
    let at = pkg;
    while build.via.contains(at) {
        at = build.via[at];
        names.insert(0, at.sub_string(10));
    }
    let text = "";
    for n in names { text += if text == "" { n } else { " -> " + n }; }
    text
}

fn check(g) {
    let out = [];
    let root = g.node(ROOT);
    if root == () { return out; }
    let build = g.cargo_build(ROOT, false);
    for pair in ISOLATED {
        if build.packages.contains(pair[0]) {
            out.push(violation(root, pair[1] + " (" + chain_to(build, pair[0]) + ")"));
        }
    }
    out
}
