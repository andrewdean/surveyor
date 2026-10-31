// The invariant docs/rfd/0001-outbox.md states: nothing outside crates/outbox/ writes
// to the outbox table.
const ID = "outbox-written-only-by-outbox";
const DESC = "only code under crates/outbox/ writes cw_outbox.outbox";

fn check(g) {
    let out = [];
    for e in g.edges_to("sql:cw_outbox.outbox", "WRITES") {
        if e.file == () || !e.file.starts_with("crates/outbox/") {
            out.push(violation(e, "writes the outbox table from outside the outbox crate"));
        }
    }
    out
}
