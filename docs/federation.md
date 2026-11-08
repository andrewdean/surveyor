# Federation: one store, several repositories, shared packages

A package is identified by its registry coordinate, so a manifest in one repository
that depends on `@torrentfx/core` and a manifest in another that declares it name the
same node identifier, `pkg:npm/@torrentfx/core`. Each repository is indexed into its
own partition of the store and knows nothing of the others; the federate pass pairs
the two by identifier and writes the pairing to a sixth table. This document says
what the pass computes, how a graph loads across partitions, and what it costs.
The RFD's section 7.1 states the rule this implements.

## The federate pass

`surveyor_link::federate` is a pure function over every repository's package nodes
and `DEPENDS_ON` edges, as the store holds them (`load_packages`). For each external
stub in repository B (a `Package` node with `external = "true"`, which the linker
creates for a dependency no manifest in B declares) that a package in B depends on,
the declared package nodes with the same identifier in other repositories A1..An are
its declarers. Each consumer edge yields one federated edge per declarer: from B's
consumer package to A's declared package, kind `DEPENDS_ON`, derivation `declared`,
confidence `1/n`, with the provenance of B's own edge (the manifest line) and its
attributes (the dependency as written, a `file:` path or a version range), and
`resolved_against` naming the declared node. A stub nobody declares, such as `react`
or `serde`, yields nothing; so does a declared package nobody outside its repository
consumes.

The pass runs at the end of `index` and `update` when the store holds more than one
repository, and `surveyor federate` runs it on demand and prints the edges. It
rewrites the `federated` table from the whole store rather than patching the rows of
the repository just written: a new declarer of a coordinate changes the confidence
of every consumer's row, whichever repository the consumer is in. The rows are few,
so the rewrite costs what the pass costs, under a second on the three repositories
below.

## The `federated` table

The edge columns plus `dst_repo`; `repo` is the consumer's repository. It is created
on the first pass and never on `open`, so a store that holds one repository has five
tables, as before, and no code path reads a sixth. `surveyor stats` reports its row
count.

## Loading across partitions

`load_graph(Some(repo))` is unchanged. The CLI loads `repo`'s partition and, only
when the store holds other repositories, the federated edges that touch it with
their far endpoints (`nodes_in` by identifier from the other partitions), or every
partition with `--all-repos`. `CodeGraph::federated` keys nodes by `(repo, id)`, so
two repositories' `file:README.md` coexist; a bare identifier resolves to the
primary repository first. An external stub whose identifier exactly one other loaded
partition declares is folded: the stub is dropped and its edges, the manifest's
`DEPENDS_ON` and the files' `IMPORTS`, point at the declared node, in memory only.
When two loaded partitions declare the coordinate, the stub stays as the ambiguous
hub and the federated edges reach each declarer at their stored `1/n`.

Every rendered node names its repository: `repo` in every JSON node, `[repo]` after a
node from another repository in `explain`, `affected` and the listings, and a
`repo` line on the explained node itself.

```bash
surveyor --store ~/federated --repo-id causeway explain pkg:npm/@torrentfx/core
surveyor --store ~/federated --repo-id cairn explain pkg:npm/@torrentfx/core
surveyor --store ~/federated --repo-id causeway --all-repos affected pkg:npm/@torrentfx/core
surveyor --store ~/federated federate
```

## What it does not do

Contracts, topics and tables linking by name inside a declared namespace wait for the
extractors that produce contract and topic nodes. Node identifiers stay unprefixed;
a collision between two repositories' code identifiers is not a federation problem,
because code nodes never fold and the graph keys them by `(repo, id)`.

## Measured

Causeway alone, release build, name tier, a scratch store, three runs each of a cold
`index` and an `update` with nothing changed, with the binary before this change and
after it; then Causeway, Cairn and torrentfx in one store. The numbers are in
`docs/exit-tests/raw/2026-11-07-phase-3-federation.txt` and summarized in the
commit that adds them.
