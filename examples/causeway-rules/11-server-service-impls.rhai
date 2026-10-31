// Ports `server_service_impls` (bootstrap/cw-vsa/src/checks.rs), the burndown
// leg: files under apps/causeway-server/src that implement a gRPC service
// trait, outside the four orchestrator files.
//
// Graph facts: Impl nodes under apps/causeway-server/src, named
// `Type#impl(Trait)` when the impl is for a trait. cw-vsa greps for
// `impl .*Service for` or `impl pb::.*Service`; here the trait's name must end
// in `Service`. The inherent-impl form `impl pb::FooServiceServer {` that the
// second cw-vsa regex also matches is not an implementation and is not
// counted here. Violations are one per file, as cw-vsa counts them.
const ID = "server-rpc-service-impls";
const DESC = "non-orchestrator gRPC service impls in the server shell";
const SEVERITY = "budget";
const BUDGET = 0;

const SERVER_SRC = "apps/causeway-server/src/";
const ORCHESTRATOR_FILES = [
    "apps/causeway-server/src/catalog.rs",
    "apps/causeway-server/src/metadata_admin/handlers.rs",
    "apps/causeway-server/src/transport/session_rpc.rs",
    "apps/causeway-server/src/transport/relay_gateway.rs",
];

fn implements_service(n) {
    n.qualified_name.contains("#impl(") && n.qualified_name.ends_with("Service)")
}

fn check(g) {
    let out = [];
    let seen = [];
    for n in g.nodes_under("Impl", SERVER_SRC) {
        if !implements_service(n) || n.file in ORCHESTRATOR_FILES || n.file in seen { continue; }
        seen.push(n.file);
        out.push(violation(n, "implements a gRPC service in the server shell"));
    }
    out
}
