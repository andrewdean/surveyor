// Ports `server_service_impls` (bootstrap/cw-vsa/src/checks.rs), the
// orchestrator count that cw-vsa reports under `server-orchestrators` with a
// budget of 99: the files among the four allowed orchestrators that implement
// a gRPC service. Reported, never failed, as in cw-vsa.
//
// Graph facts: as 11-server-service-impls.rhai.
const ID = "server-orchestrators";
const DESC = "allowed orchestrator residue (ADR-0014/0016) - reported only";
const SEVERITY = "warn";

const SERVER_SRC = "apps/causeway-server/src/";
const ORCHESTRATOR_FILES = [
    "apps/causeway-server/src/catalog.rs",
    "apps/causeway-server/src/metadata_admin/handlers.rs",
    "apps/causeway-server/src/transport/session_rpc.rs",
    "apps/causeway-server/src/transport/explorer_session_rpc.rs",
    "apps/causeway-server/src/transport/relay_gateway.rs",
];

fn check(g) {
    let out = [];
    let seen = [];
    for n in g.nodes_under("Impl", SERVER_SRC) {
        if !(n.qualified_name.contains("#impl(") && n.qualified_name.ends_with("Service)")) { continue; }
        if !(n.file in ORCHESTRATOR_FILES) || n.file in seen { continue; }
        seen.push(n.file);
        out.push(violation(n, "orchestrator implements a gRPC service (allowed residue)"));
    }
    out
}
