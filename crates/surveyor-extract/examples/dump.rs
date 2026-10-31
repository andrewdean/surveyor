//! Debug helper: dump facts for a file given on the command line.
use surveyor_extract::{RhaiExtractor, TreeSitterExtractor};
use surveyor_model::{CommitId, Fact, RepoId, RepoPath, SourceUnit};

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let content = std::fs::read(&path).expect("read");
    let unit = SourceUnit::new(
        RepoId::new("r"),
        RepoPath::new(&path),
        CommitId::new("c"),
        content,
    );
    let set = if std::path::Path::new(&path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("rhai"))
    {
        RhaiExtractor::new().extract(&unit).expect("extract")
    } else {
        TreeSitterExtractor::new().extract(&unit).expect("extract")
    };
    for f in &set.facts {
        match f {
            Fact::Declaration(d) => println!(
                "DECL {:?} {} scope={:?} span={:?} rule={}",
                d.kind, d.name, d.scope, d.span, d.rule
            ),
            Fact::Reference(r) => println!(
                "REF  {:?} {} path={:?} scope={:?}",
                r.role, r.name, r.path, r.scope
            ),
            Fact::Literal(l) => println!("LIT  {:?} {}", l.role, l.value),
            Fact::Relation(r) => println!("REL  {:?} {} -> {}", r.kind, r.src_name, r.dst_name),
        }
    }
}
