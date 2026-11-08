<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `36157f3ea844`, dataset at `ec36fd145df4`: **5 files, 12 changed symbols, 4 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (12)**

| Symbol | Kind | Where |
|---|---|---|
| `OpenTasks` | Enum | `apps/causeway-server-web/src/views/platform/tasks/list.rs:17-21` |
| `OpenTasks::Loaded` | Variant | `apps/causeway-server-web/src/views/platform/tasks/list.rs:18` |
| `OpenTasks::Unavailable` | Variant | `apps/causeway-server-web/src/views/platform/tasks/list.rs:20` |
| `tasks` | Function | `apps/causeway-server-web/src/views/platform/tasks/list.rs:23-71` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/tasks/list.rs:74-91` |
| `tests::a_failed_read_says_so_and_does_not_claim_there_are_no_tasks` | Test | `apps/causeway-server-web/src/views/platform/tasks/list.rs:78-83` |
| `tests::an_empty_read_says_there_are_no_open_tasks` | Test | `apps/causeway-server-web/src/views/platform/tasks/list.rs:86-90` |
| `tasks` | Function | `apps/causeway-server/src/routes/pages/tasks/list.rs:28-70` |
| `TaskListPageData` | Enum | `apps/causeway-server/src/workflow_tasks/page.rs:23-26` |
| `TaskListPageData::LoadFailed` | Variant | `apps/causeway-server/src/workflow_tasks/page.rs:25` |
| `TaskListPageData::Loaded` | Variant | `apps/causeway-server/src/workflow_tasks/page.rs:24` |
| `load_task_list_page` | Function | `apps/causeway-server/src/workflow_tasks/page.rs:28-39` |

Changed as whole files (no declaration at the changed lines): `apps/causeway-server-web/src/views/platform/tasks.rs`, `apps/causeway-server/src/workflow_tasks.rs`.

**Affected (4 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (2)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `routes` | Function | `apps/causeway-server/src/routes/app/govern.rs:10-74` | 1 | 0.50 | REFERENCES | `apps/causeway-server/src/routes/app/govern.rs:71 [syntactic 0.50]` |
| `routes` | Function | `apps/causeway-server/src/routes/app.rs:17-28` | 2 | 0.50 | CALLS | `apps/causeway-server/src/routes/app.rs:20` |

_Files and packages (2)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `apps/causeway-server/src/routes/pages.rs` | File | `apps/causeway-server/src/routes/pages.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages.rs:74` |
| `apps/causeway-server/src/routes/pages/tasks.rs` | File | `apps/causeway-server/src/routes/pages/tasks.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages/tasks.rs:10` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 66771 nodes, 286474 edges · 1655 ms</sub>
