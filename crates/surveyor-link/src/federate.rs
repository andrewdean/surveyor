//! Cross-repository package edges: the federate pass.
//!
//! Packages are identified by registry coordinates, so a manifest in repository B that
//! depends on `@torrentfx/core` draws `DEPENDS_ON` to an external stub
//! `pkg:npm/@torrentfx/core` in B's own partition. When another repository A in the
//! same store declares that coordinate, the identifier is the join key: this pass
//! returns one edge from B's consumer package to A's declared package per declaring
//! repository, at confidence `1/n` for `n` declarers, citing B's manifest line (the
//! evidence B's own edge carries) and naming A's node in `resolved_against`. It is a
//! pure function of the package nodes and dependency edges it is given.

// Rust guideline compliant 2026-02-21

use std::collections::{BTreeMap, HashMap};

use surveyor_model::{
    Confidence, Edge, EdgeId, EdgeKind, FederatedEdge, Node, NodeId, NodeKind, RepoId,
};

/// One repository's packages and the dependencies between them, as the store holds them.
#[derive(Debug, Clone, PartialEq)]
pub struct RepoPackages<'a> {
    /// The repository.
    pub repo: RepoId,
    /// Its `Package` nodes, declared and external stubs alike (other kinds are ignored).
    pub packages: &'a [Node],
    /// Its `DEPENDS_ON` edges between packages (other edges are ignored).
    pub depends: &'a [Edge],
}

/// True for the external stub a manifest dependency on an undeclared package creates.
#[must_use]
pub fn is_external_package(node: &Node) -> bool {
    node.kind == NodeKind::Package && node.attributes.get("external").is_some_and(|v| v == "true")
}

/// The cross-repository edges of a store, sorted by consumer repository, consumer,
/// declaring repository.
///
/// For every external stub in a repository with at least one consumer edge, the
/// declared (non-external) package nodes with the same identifier in the other
/// repositories are its declarers. With `n` declarers each consumer edge yields `n`
/// federated edges at confidence `1/n`, carrying the consumer edge's provenance and
/// attributes (the dependency as written: a path, a version range), with
/// `resolved_against` naming the declared node. A stub nobody declares, and a
/// declared package nobody outside its repository consumes, yield nothing.
#[must_use]
pub fn federate(repos: &[RepoPackages<'_>]) -> Vec<FederatedEdge> {
    // Declarers by coordinate: (repo, node id), one per repository at most.
    let mut declarers: HashMap<&NodeId, Vec<&RepoId>> = HashMap::new();
    for r in repos {
        for n in r
            .packages
            .iter()
            .filter(|n| n.kind == NodeKind::Package && !is_external_package(n))
        {
            let list = declarers.entry(&n.id).or_default();
            if !list.contains(&&r.repo) {
                list.push(&r.repo);
            }
        }
    }
    let mut out: Vec<FederatedEdge> = Vec::new();
    for r in repos {
        let stubs: BTreeMap<&NodeId, &Node> = r
            .packages
            .iter()
            .filter(|n| is_external_package(n))
            .map(|n| (&n.id, n))
            .collect();
        let packages: HashMap<&NodeId, &Node> = r.packages.iter().map(|n| (&n.id, n)).collect();
        for e in r.depends.iter().filter(|e| e.kind == EdgeKind::DependsOn) {
            let Some(stub) = stubs.get(&e.dst) else {
                continue;
            };
            if packages
                .get(&e.src)
                .is_none_or(|n| n.kind != NodeKind::Package)
            {
                continue;
            }
            let foreign: Vec<&RepoId> = declarers
                .get(&stub.id)
                .map(|list| list.iter().copied().filter(|d| **d != r.repo).collect())
                .unwrap_or_default();
            if foreign.is_empty() {
                continue;
            }
            let share = Confidence::new(1.0 / foreign.len() as f32);
            for dst_repo in foreign {
                let mut edge = Edge::new(
                    e.src.clone(),
                    EdgeKind::DependsOn,
                    e.dst.clone(),
                    e.provenance.clone().with_confidence(share),
                )
                .with_attributes(e.attributes.clone());
                edge.resolved_against = vec![e.dst.clone()];
                edge.id = EdgeId::from(format!("{}@{dst_repo}", edge.id.as_str()));
                out.push(FederatedEdge {
                    src_repo: r.repo.clone(),
                    dst_repo: dst_repo.clone(),
                    edge,
                });
            }
        }
    }
    out.sort_by(|a, b| {
        (&a.src_repo, a.edge.src.as_str(), &a.dst_repo).cmp(&(
            &b.src_repo,
            b.edge.src.as_str(),
            &b.dst_repo,
        ))
    });
    out
}

#[cfg(test)]
mod tests {
    use surveyor_model::{
        ByteSpan, CommitId, ContentHash, DeclaredSource, Evidence, LineSpan, Provenance, RepoPath,
        SourceLocation, TextSpan,
    };

    use super::*;

    fn pkg(repo: &str, coordinate: &str, external: bool) -> Node {
        let n = Node::synthetic(
            NodeId::new(format!("pkg:{coordinate}")),
            NodeKind::Package,
            coordinate.rsplit('/').next().unwrap(),
            RepoId::new(repo),
        )
        .with_qualified_name(coordinate);
        if external {
            n.with_attribute("external", "true")
        } else {
            n
        }
    }

    fn depends(repo: &str, from: &str, to: &str, file: &str, line: u32, spec: &str) -> Edge {
        let span = TextSpan::new(ByteSpan::new(0, 10), LineSpan::new(line, line));
        let evidence = Evidence {
            repo: RepoId::new(repo),
            location: SourceLocation::new(RepoPath::new(file), span),
            commit: CommitId::new("c"),
            content_hash: ContentHash::of(spec.as_bytes()),
        };
        let mut attributes = BTreeMap::new();
        attributes.insert("spec".to_owned(), spec.to_owned());
        Edge::new(
            NodeId::new(format!("pkg:{from}")),
            EdgeKind::DependsOn,
            NodeId::new(format!("pkg:{to}")),
            Provenance::declared(DeclaredSource::Manifest, evidence),
        )
        .with_attributes(attributes)
    }

    #[test]
    fn a_consumer_pairs_with_the_one_declarer_at_full_confidence() {
        let causeway = [pkg("causeway", "npm/@torrentfx/core", false)];
        let cairn = [
            pkg("cairn", "npm/cairn-vscode", false),
            pkg("cairn", "npm/@torrentfx/core", true),
            pkg("cairn", "npm/react", true),
        ];
        let cairn_deps = [
            depends(
                "cairn",
                "npm/cairn-vscode",
                "npm/@torrentfx/core",
                "vscode/package.json",
                99,
                "file:../x",
            ),
            depends(
                "cairn",
                "npm/cairn-vscode",
                "npm/react",
                "vscode/package.json",
                100,
                "^18",
            ),
        ];
        let edges = federate(&[
            RepoPackages {
                repo: RepoId::new("causeway"),
                packages: &causeway,
                depends: &[],
            },
            RepoPackages {
                repo: RepoId::new("cairn"),
                packages: &cairn,
                depends: &cairn_deps,
            },
        ]);
        assert_eq!(edges.len(), 1, "{edges:#?}");
        let f = &edges[0];
        assert_eq!(f.src_repo.as_str(), "cairn");
        assert_eq!(f.dst_repo.as_str(), "causeway");
        assert_eq!(f.edge.src.as_str(), "pkg:npm/cairn-vscode");
        assert_eq!(f.edge.dst.as_str(), "pkg:npm/@torrentfx/core");
        assert!((f.edge.provenance.confidence.get() - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            f.edge.provenance.evidence[0].citation(),
            "vscode/package.json:99"
        );
        assert_eq!(
            f.edge.attributes.get("spec").map(String::as_str),
            Some("file:../x")
        );
        assert_eq!(
            f.edge.resolved_against,
            vec![NodeId::new("pkg:npm/@torrentfx/core")]
        );
        assert!(f.edge.id.as_str().ends_with("@causeway"));
    }

    #[test]
    fn two_declarers_split_the_confidence_and_own_stubs_do_not_count() {
        let causeway = [pkg("causeway", "npm/@torrentfx/core", false)];
        let torrentfx = [pkg("torrentfx", "npm/@torrentfx/core", false)];
        let cairn = [
            pkg("cairn", "npm/cairn-vscode", false),
            pkg("cairn", "npm/@torrentfx/core", true),
        ];
        let deps = [depends(
            "cairn",
            "npm/cairn-vscode",
            "npm/@torrentfx/core",
            "vscode/package.json",
            99,
            "0.1.0",
        )];
        // Causeway also consumes its own declared package: no stub, so nothing federates from it.
        let causeway_deps = [depends(
            "causeway",
            "npm/cw-ui",
            "npm/@torrentfx/core",
            "web/package.json",
            5,
            "workspace:*",
        )];
        let edges = federate(&[
            RepoPackages {
                repo: RepoId::new("causeway"),
                packages: &causeway,
                depends: &causeway_deps,
            },
            RepoPackages {
                repo: RepoId::new("torrentfx"),
                packages: &torrentfx,
                depends: &[],
            },
            RepoPackages {
                repo: RepoId::new("cairn"),
                packages: &cairn,
                depends: &deps,
            },
        ]);
        let shape: Vec<(String, String, f32)> = edges
            .iter()
            .map(|f| {
                (
                    f.src_repo.to_string(),
                    f.dst_repo.to_string(),
                    f.edge.provenance.confidence.get(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            vec![
                ("cairn".to_owned(), "causeway".to_owned(), 0.5),
                ("cairn".to_owned(), "torrentfx".to_owned(), 0.5),
            ]
        );
    }

    #[test]
    fn a_stub_nobody_declares_yields_nothing() {
        let a = [pkg("a", "cargo/serde", true), pkg("a", "cargo/mine", false)];
        let b = [
            pkg("b", "cargo/serde", true),
            pkg("b", "cargo/yours", false),
        ];
        let deps_a = [depends(
            "a",
            "cargo/mine",
            "cargo/serde",
            "Cargo.toml",
            9,
            "1",
        )];
        let deps_b = [depends(
            "b",
            "cargo/yours",
            "cargo/serde",
            "Cargo.toml",
            9,
            "1",
        )];
        assert!(
            federate(&[
                RepoPackages {
                    repo: RepoId::new("a"),
                    packages: &a,
                    depends: &deps_a
                },
                RepoPackages {
                    repo: RepoId::new("b"),
                    packages: &b,
                    depends: &deps_b
                },
            ])
            .is_empty()
        );
    }
}
