// Known debt, held and burned down: public functions in the library crates with no
// test that reaches them. Lower BUDGET as tests land; a rise fails the check.
const ID = "public-functions-have-tests";
const DESC = "public functions in crates/ are exercised by a test (TESTED_BY)";
const SEVERITY = "budget";
const BUDGET = 20;

fn check(g) {
    let out = [];
    for n in g.nodes_under("Function", "crates/") {
        let sig = n.signature;
        if sig == () || !sig.starts_with("pub ") { continue; }
        if n.file.contains("/examples/") { continue; }
        if g.edges_from(n.id, "TESTED_BY").len() == 0 {
            out.push(violation(n, "no test reaches this public function"));
        }
    }
    out
}
