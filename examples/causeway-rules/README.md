# Causeway's VSA checks as Surveyor rules

The named checks of Causeway's vertical-slice-architecture gate
(`bootstrap/cw-vsa/src/checks.rs`, run by `mise run vsa:check`) ported to Rhai rules over
Surveyor's graph. One file per check, numbered in the order of the `Check` enum; each file's
header comment names the function it ports, the graph facts it reads and what it cannot see.
The checks that read facts the graph does not hold have no file and are listed below with
the missing fact.

Run them against a Causeway dataset (Causeway's checkout is never written to; the store
lives beside Surveyor):

```bash
cargo build --release -p surveyor
target/release/surveyor --repo-root ~/dev/trusted/causeway --store .surveyor/causeway-db index
target/release/surveyor --repo-root ~/dev/trusted/causeway --store .surveyor/causeway-db check --rules examples/causeway-rules
```

The gate's other 512 rules are regexes over file text (`forbid`, `require`, `require_path`);
they are not graph questions and are outside this port.

## Mapping

Causeway at `40f993d7` (2026-10-31). "cw-vsa" is what `vsa-gate` printed from the Causeway
checkout; "Surveyor" is what `surveyor check` printed from the dataset.

| `checks.rs` function | Rule file | cw-vsa | Surveyor | Agreement and gaps |
|---|---|---|---|---|
| `gate_staleness` | `01-gate-staleness.rhai` | 0 missing | 0 missing | Agree on the 337 paths outside dot-directories. The walker does not enter `.mise/` or `.github/`, so 9 paths there are skipped, not checked. An emptied directory still exists on disk but not in the graph. |
| `task_content` | none | pass | — | **Blocked**: a mise task's body (its TOML entry plus the scripts it names) is text; the graph has no Task nodes for Causeway's `mise.toml` and no bodies to match regexes against. |
| `msk_auth_topology` | none | pass | — | **Blocked**: the auth mode and secret mounts are lines of `config/topology/*.toml`; the graph holds no topology facts. |
| `runtime_env_resource_ctors` | `04-runtime-env-resource-ctors.rhai` | pass | pass | Agree. The graph knows which type a method belongs to; cw-vsa counts braces to find the impl block. |
| `relay_restate_staging_order` | none | pass | — | **Blocked**: the check is the order of two lines in one task script. |
| `component_cross_deps` | `06-component-cross-deps.rhai` | pass | pass | Agree, with one difference explained: the graph resolves component/Cargo.toml's `[patch]` of `fastembed` to `vendor/fastembed`, which `cargo metadata --no-deps` reports as a registry crate. The rule allows `vendor/` beside `crates/` and `services/`. |
| `crypto_provider_installs` | `07-crypto-provider-installs.rhai` | failures only under `.claude/worktrees/`; main tree passes | pass | Agree on the main tree (see below). **Partial**: which workspaces link both rustls backends is decided by registry crates' features (tonic, reqwest), which the graph does not hold, so the list is a constant in the rule (from `cargo metadata` on this commit). "Reaches rustls" is now what a default build of the binary's package pulls in (`cargo_build`), so dev-only and disabled optional paths do not count; on this commit that judges the same 48 binaries as plain reachability did. A binary reaching rustls through a registry crate alone is still not seen. |
| `cargo_feature_isolation` (×2: `causeway-sniffer`, `parquet`) | `09-cargo-feature-isolation.rhai` | pass | pass | Agree. DEPENDS_ON edges carry the entry's `section`, `optional`, `default_features` and `features`, Package nodes their `[features]` table, and `g.cargo_build(root, default_features)` resolves them as cargo does: the minimal build of causeway-server reaches neither crate, the default build reaches `parquet` (through the `tabular-explorer` default feature; the rule fires when run with defaults on), and `cargo tree` says the same for both. Registry crates are leaves, so a path through a registry crate's own dependencies would be missed; `workspace = true` entries are taken as written, without the `[workspace.dependencies]` definition. |
| `facade_size_budget` | `02-facade-size-budget.rhai` | pass (largest 219 of 220 lines) | pass (ws.rs 219, agent.rs 185) | Agree. File nodes carry a `lines` attribute beside `bytes`; the graph counts every newline plus an unterminated last line, cw-vsa drops trailing blank lines first, and the two agree on any rustfmt-formatted file. |
| `server_rpc_deps` | `10-server-rpc-deps.rhai` | 0 (budget 0) | 0 of 0 | Agree: the four orchestrator `*-rpc` crates and nothing else. |
| `server_service_impls` | `11-server-service-impls.rhai` | 0 (budget 0) | 0 of 0 | Agree. The graph names the trait an impl is for; cw-vsa's second regex (`impl pb::.*Service`) would also match an inherent impl of a generated type, which is not an implementation. |
| `server_service_impls` (orchestrator count) | `12-server-orchestrators.rhai` | 3 (budget 99) | 3 (warn) | Agree: `metadata_admin/handlers.rs`, `transport/relay_gateway.rs`, `transport/session_rpc.rs`; `catalog.rs` implements no service. |
| `island_cross_refs` | `13-island-cross-refs.rhai` | 0 (budget 0) | 0 of 0 | Agree. cw-vsa greps for the text `@causeway/ui-*` in manifests and sources (a comment would count); the graph counts declared dependencies and imports, both of which fire. |
| `schema_refs` → `cross-slice-schema-refs` | `14-cross-slice-table-access.rhai` | 0 (budget 0) | 0 of 0 | **Partial**: the literal in `actual_schema_name("<area>/<slice>")` is not a graph fact, and the SQL those schemas are spliced into is built at run time, so no READS/WRITES edge records a cross-schema query. The rule states the invariant in the graph's terms (table ownership by the slice whose migrations create it) and catches literal cross-slice SQL; it cannot see run-time splicing. |
| `schema_refs` → `search-projection-schema-refs` | none | 4 (exact) | — | **Blocked**: as above; the four seam references are string literals. |
| `schema_refs` → `coherence-check-schema-refs` | none | 6 (exact) | — | **Blocked**: as above. |
| `domain-modules` | `17-domain-modules.rhai` | 0 (budget 0) | 0 of 0 | Agree: no File node under `crates/causeway-domain/src/`. |
| `domain-dependents` | `18-domain-dependents.rhai` | 0 (budget 0) | 0 of 0 | Agree: no DEPENDS_ON edge into `pkg:cargo/causeway-domain` (the node does not exist). cw-vsa's regex would also match a commented-out dependency line. |
| `configuration_inputs` | none | pass | — | **Blocked**: a read of `CAUSEWAY_WORKSPACE_ID` and friends, a mention of `CAUSEWAY_ENV` and a comparison against `"prod"` are line-level text facts; the graph has no CONFIGURED_BY edges for Causeway. |
| `prefixed_settings` | none | pass | — | **Blocked**: as above, for `process.env.X` and `os.environ[...]` reads in Node and Python. |
| `image_copy_modes` | none | pass | — | **Blocked**: the `--chmod` of each `COPY` per Dockerfile stage; the graph has no Dockerfile nodes for Causeway. |

Totals: 21 named checks (22 rule instances). 11 ported with agreement, 2 ported partially
(`crypto_provider_installs`, `cross-slice-schema-refs`), 8 blocked on facts the graph does
not hold (task bodies, TOML content, Dockerfile flags, environment reads). Two checks were
blocked until the graph gained facts: `facade_size_budget` needed a `lines` attribute on File
nodes, and `cargo_feature_isolation` needed the optional, default-features and feature facts
on Cargo `DEPENDS_ON` edges and Package nodes, plus `g.cargo_build`, the one `GraphHandle`
addition, which resolves them the way cargo unifies features.

## Keeping the constants with Causeway

Several rules carry Causeway's allowlists as constants (`ORCHESTRATORS`,
`ORCHESTRATOR_FILES`, `PATHS`, `FACADES`, `WORKSPACES`), copied from
`bootstrap/cw-vsa/src/checks.rs` and `.mise/rhai/vsa/`. They drift: between
`40f993d7` and `ae6bdbea` (2026-11-05) Causeway's gate added `explorer-session-rpc`
as a fifth orchestrator and `transport/explorer_session_rpc.rs` as its file (#1050),
and the copies here flagged both on `main` until they were updated for the Phase 2
exit test. That is the argument for the rules living in Causeway's own
`surveyor-rules/` directory, where a change to the gate and a change to the rule are
one pull request; `docs/causeway-ci-check.md` in this repository proposes exactly that.

## The cw-vsa run itself

`vsa-gate` printed 159 failures and exit 1 on this checkout. Every one of them names a path
under `.claude/worktrees/adoring-newton-e3e68b/`, a git worktree checked out inside the
Causeway tree that the gate's walk does not prune: `crypto-provider-installs`,
`image-copy-modes`, `configuration-inputs` and the regex rule `hard-coded-subjects` all fire
on that stale copy and on nothing in the main tree. Surveyor's walker skips dot-directories,
so its dataset holds only the main tree, where every named check passes in both tools.

## Each rule fires

With all thirteen rules passing on the real tree, each was also run against a scratch clone of
Causeway with one violation planted per rule (a deleted required path, a `pub fn from_env`
on `KafkaBrokers`, a component depending on a sibling area, `apps/drive` without the crypto
provider call, a fifth `*-rpc` dependency in the server shell, an `impl pb::SearchService`
in `search.rs`, an island importing `@causeway/ui-request` by manifest and then by import
alone, an `INSERT INTO catalog_assets` from the articles slice, and `crates/causeway-domain`
restored with a dependent). All ten enforced rules failed on exactly the planted finding, with
a citation to its line; the warn rule kept reporting its three orchestrators. The facade rule
was shown to fire by lowering its budget to 150 lines in a scratch copy, which reported
`ws.rs` (219) and `agent.rs` (185) and nothing else. The feature-isolation rule was shown to
fire by resolving the default build instead of the minimal one in a scratch copy, which
reported parquet and not the sniffer, as `cargo tree -p causeway-server -i parquet` does.
`check` over the 58,550-node, 327,286-edge dataset takes about 0.6 s.
