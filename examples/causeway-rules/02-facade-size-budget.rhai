// Ports `facade_size_budget` (bootstrap/cw-vsa/src/checks.rs): the ten server
// facade files stay under 220 lines; a facade that grows past that is a slice
// module in disguise.
//
// Graph facts: the File node of each facade (`file:<path>`) and its `lines`
// attribute, which the walker records beside `bytes`.
//
// Differences from cw-vsa, stated rather than hidden:
// - cw-vsa's `split_lines` drops trailing blank lines before counting; the
//   graph counts every newline (plus an unterminated last line). The two agree
//   on any file rustfmt has touched, which ends in exactly one newline.
// - A facade that does not exist is skipped here as there, since
//   `gate-staleness` already reports a vanished path.
const ID = "facade-size-budget";
const DESC = "server facades must stay under 220 lines (grow a slice module instead)";
const SEVERITY = "fail";

const BUDGET_LINES = 220;
const FACADES = [
    "apps/causeway-server/src/routes/mod.rs",
    "apps/causeway-server/src/routes/public.rs",
    "apps/causeway-server/src/routes/pages/governance.rs",
    "apps/causeway-server-web/src/services.rs",
    "apps/causeway-server/src/composition/remote_services.rs",
    "apps/causeway-server/src/composition/local_services/mock.rs",
    "apps/causeway-server/src/ws.rs",
    "apps/causeway-server/src/agent.rs",
    "apps/causeway-server/src/agui/agent_run.rs",
    "apps/causeway-server/src/metadata_admin.rs",
];

fn check(g) {
    let out = [];
    for path in FACADES {
        let file = g.node("file:" + path);
        if file == () || file.attributes.lines == () { continue; }
        let lines = parse_int(file.attributes.lines);
        if lines > BUDGET_LINES {
            out.push(violation(file, `${lines} lines (budget ${BUDGET_LINES})`));
        }
    }
    out
}
