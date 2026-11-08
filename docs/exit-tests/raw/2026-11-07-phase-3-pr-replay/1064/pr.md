<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `08bc6c236da8`, dataset at `36157f3ea844`: **4 files, 5 changed symbols, 3 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (5)**

| Symbol | Kind | Where |
|---|---|---|
| `migration_rules` | Function | `.mise/rhai/vsa/rules.rhai:40-1310` |
| `EMBEDDED_MIGRATIONS` | Const | `crates/causeway-db-core/src/migrations.rs:19-49` |
| `no_base_table_is_shadowed_by_a_component_schema` | Test | `crates/causeway-testkit/tests/postgres.rs:131-158` |
| `no_base_table_is_shadowed_by_a_component_schema::SHARED_BY_DESIGN` | Const | `crates/causeway-testkit/tests/postgres.rs:132-136` |
| `no_base_table_is_shadowed_by_a_component_schema::select #9e13a97d` | Query | `crates/causeway-testkit/tests/postgres.rs:141-146` |

Changed as whole files (no declaration at the changed lines): `crates/causeway-db-core/migrations/0029_drop_orphaned_base_tables.sql`.

**Affected (3 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (1)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `all_rules` | Function | `.mise/rhai/vsa/rules.rhai:1419-1437` | 1 | 1.00 | CALLS | `.mise/rhai/vsa/rules.rhai:1422` |

_Files and packages (2)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `tests` | Module | `crates/causeway-db-core/src/migrations.rs:52-82` | 1 | 1.00 | IMPORTS | `crates/causeway-db-core/src/migrations.rs:53` |
| `.mise/rhai/vsa/all.rhai` | File | `.mise/rhai/vsa/all.rhai` | 2 | 1.00 | CALLS | `.mise/rhai/vsa/all.rhai:7` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 66762 nodes, 286447 edges · 1743 ms</sub>
