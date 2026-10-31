// Ports `runtime_env_resource_ctors` (bootstrap/cw-vsa/src/checks.rs): the
// causeway-runtime resource types expose typed topology constructors, never a
// `pub fn from_env(` inside their impl block.
//
// Graph facts: Method nodes under crates/causeway-runtime/src/resource.rs,
// each with its qualified name `Type::method` and its signature. cw-vsa finds
// the impl block by brace counting and greps it; the graph knows which type a
// method belongs to.
const ID = "runtime-env-resource-ctors";
const DESC = "causeway-runtime resource types must expose typed topology constructors, not env-first constructors";

const FILE = "crates/causeway-runtime/src/resource.rs";
const TYPES = [
    "KafkaBrokers",
    "KafkaProducerConfig",
    "KafkaAdminConfig",
    "KafkaConsumerConfig",
    "ValkeyUrl",
    "ValkeyConfig",
    "ObjectStoreConfig",
];

fn check(g) {
    let out = [];
    for m in g.nodes_under("Method", FILE) {
        if m.name != "from_env" { continue; }
        let sig = m.signature;
        if sig == () || !sig.starts_with("pub fn from_env(") { continue; }
        for t in TYPES {
            if m.qualified_name == t + "::from_env" {
                out.push(violation(m, t + " must expose typed topology constructors, not env-first constructors"));
            }
        }
    }
    out
}
