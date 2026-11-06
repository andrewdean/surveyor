// Ports `server_rpc_deps` (bootstrap/cw-vsa/src/checks.rs): the server shell
// depends on no slice transport crate (`*-rpc`) except the four orchestrators
// ADR-0014/0016 allow.
//
// Graph facts: DEPENDS_ON edges from pkg:cargo/causeway-server, declared from
// apps/causeway-server/Cargo.toml; each edge cites the manifest line.
const ID = "server-rpc-contract-deps";
const DESC = "non-orchestrator *-rpc deps in the server shell";
const SEVERITY = "budget";
const BUDGET = 0;

const SERVER = "pkg:cargo/causeway-server";
// The five Causeway `main` allows at ae6bdbea (2026-11-05): checks.rs `server_rpc_deps`
// added explorer-session-rpc in #1050. This list follows that regex; a change there is
// a change here.
const ORCHESTRATORS = ["pkg:cargo/catalog-rpc", "pkg:cargo/metadata-rpc", "pkg:cargo/session-rpc", "pkg:cargo/explorer-session-rpc", "pkg:cargo/relay-rpc"];

fn check(g) {
    let out = [];
    for e in g.edges_from(SERVER, "DEPENDS_ON") {
        if e.dst.ends_with("-rpc") && !(e.dst in ORCHESTRATORS) {
            out.push(violation(e, "the server shell must not own a slice transport"));
        }
    }
    out
}
