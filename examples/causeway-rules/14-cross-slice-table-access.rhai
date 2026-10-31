// Ports `schema_refs` / `cross-slice-schema-refs` (bootstrap/cw-vsa/src/checks.rs)
// as far as the graph can: code under component/<area>/<slice> must not read
// or write a table another slice owns.
//
// What cw-vsa checks is the string literal in every
// `actual_schema_name("<area>/<slice>")` call under component/. That literal
// is not a graph fact: the graph records the CALLS edge to actual_schema_name
// and not its argument, and the SQL those schemas are spliced into is built
// at run time (`format!("... FROM {schema}.catalog_entries_v1")`), which the
// SQL extractor cannot see, so no READS or WRITES edge records the cross-slice
// access either. The two seams cw-vsa holds at exact counts
// (search-projection-schema-refs = 4, coherence-check-schema-refs = 6) are
// therefore invisible here and have no rule.
//
// This rule states the same invariant in the graph's own terms: a table is
// owned by the slice whose migrations create it (the Table node's file under
// component/<area>/<slice>/migrations/), and a READS or WRITES edge from Rust
// code under another slice is a violation. Tables created by the baseline
// migrations under crates/ have no slice owner and are not judged.
//
// On Causeway 40f993d7 the graph sees 14 same-slice accesses from component
// code and 0 cross-slice, which agrees with cw-vsa's 0, but for the reason
// above the agreement is weaker than it looks: a cross-slice query written
// against a literal schema name would be caught, one spliced in at run time
// would not.
const ID = "cross-slice-schema-refs";
const DESC = "component slice reading or writing another slice's tables (literal SQL only; run-time schema splicing is not in the graph)";
const SEVERITY = "budget";
const BUDGET = 0;

fn slice_of(path) {
    let parts = path.split("/");
    if parts.len() < 3 || parts[0] != "component" { return (); }
    parts[1] + "/" + parts[2]
}

fn check(g) {
    let out = [];
    let owner = #{};
    for t in g.nodes("Table") {
        if t.file == () { continue; }
        let s = slice_of(t.file);
        if s != () { owner[t.id] = s; }
    }
    // The graph records each query twice: from the `query#hash` node and from
    // the function that holds it. The function's edge carries the same
    // citation, so only it is judged, once per (function, table).
    let seen = [];
    for kind in ["READS", "WRITES"] {
        for e in g.edges(kind) {
            if e.file == () || !e.file.starts_with("component/") || e.file.ends_with(".sql") { continue; }
            if e.src.contains("::query#") || !(e.dst in owner) { continue; }
            let own = slice_of(e.file);
            let theirs = owner[e.dst];
            let key = e.src + " " + e.dst;
            if own != theirs && !(key in seen) {
                seen.push(key);
                out.push(violation(e, own + " -> " + theirs + ": " + kind + " " + e.dst));
            }
        }
    }
    out
}
