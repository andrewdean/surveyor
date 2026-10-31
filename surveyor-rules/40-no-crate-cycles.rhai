const ID = "no-crate-cycles";
const DESC = "workspace crates form no dependency cycle";

fn check(g) {
    let out = [];
    for group in g.cycles(["DEPENDS_ON"]) {
        let ours = group.filter(|id| id.starts_with("pkg:cargo/surveyor"));
        if ours.len() > 1 {
            out.push(#{ message: "cycle among " + ours.len() + " crates", subject: ours[0] });
        }
    }
    out
}
