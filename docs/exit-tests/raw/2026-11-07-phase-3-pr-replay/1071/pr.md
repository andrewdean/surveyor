<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `8647a8ff9a92`, dataset at `ac10ddb66395`: **14 files, 53 changed symbols, 54 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (53)**

| Symbol | Kind | Where |
|---|---|---|
| `App Twin-RPC Inventory — gap ledger for the /app port` | Document | `docs/plans/APP-TWIN-RPC-INVENTORY.md:1-292` |
| `DESCRIPTOR` | Const | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.py:30` |
| `PortListStuckWorkResponse` | Class | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:626-634` |
| `PortListStuckWorkResponse::__init__` | Method | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:634` |
| `StuckWorkRecord` | Class | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:562-600` |
| `StuckWorkRecord::__init__` | Method | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:600` |
| `StuckWorkSourceSummary` | Class | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:602-612` |
| `StuckWorkSourceSummary::__init__` | Method | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:612` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:525-586` |
| `tests::a_quiet_source_reads_zero_and_only_an_uncounted_record_reads_unknown` | Test | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:554-585` |
| `tests::record` | Function | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:530-540` |
| `tests::waiting` | Function | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:543-548` |
| `MetadataAdmin::port_list_stuck_work` | Method | `apps/causeway-server/src/metadata_admin/handlers.rs:542-565` |
| `listing_tests` | Module | `apps/causeway-server/src/metadata_admin/handlers.rs:1028` |
| `stuck_work_record` | Function | `apps/causeway-server/src/metadata_admin/handlers.rs:14-50` |
| `stuck_work_source` | Function | `apps/causeway-server/src/metadata_admin/handlers.rs:53-61` |
| `CALLER` | Const | `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:21` |
| `a_quarantine_action_naming_somebody_else_is_refused` | Test | `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:214-246` |
| `a_stuck_work_action_is_recorded_against_the_verified_caller` | Test | `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:138-175` |
| `ending_a_stuck_record_takes_the_seat_its_source_declares` | Test | `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:178-211` |
| `serve` | Function | `apps/causeway-server/src/metadata_admin/handlers/caller_tests.rs:90-124` |
| `Holding` | Struct | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:17-20` |
| `Holding#impl(StuckWorkSource)` | Impl | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:34-70` |
| `Holding::act` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:54-61` |
| `Holding::bounds` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:63-65` |
| `Holding::fencing` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:67-69` |
| `Holding::get` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:47-52` |
| `Holding::id` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:35-37` |
| `Holding::list` | Method | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:39-45` |
| `held` | Function | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:22-32` |
| `summary` | Function | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:72-80` |
| `the_listing_names_a_quiet_source_and_an_unknown_count` | Test | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:83-167` |
| `SourceCount` | Struct | `apps/causeway-server/src/metadata_admin/stuck_work.rs:66-76` |
| `SourceCount#impl` | Impl | `apps/causeway-server/src/metadata_admin/stuck_work.rs:78-92` |
| `SourceCount::of` | Method | `apps/causeway-server/src/metadata_admin/stuck_work.rs:81-91` |
| `StuckWorkListing` | Struct | `apps/causeway-server/src/metadata_admin/stuck_work.rs:19-27` |
| `counts` | Function | `apps/causeway-server/src/metadata_admin/stuck_work.rs:117-123` |
| `list` | Function | `apps/causeway-server/src/metadata_admin/stuck_work.rs:31-61` |
| `tests` | Module | `apps/causeway-server/src/metadata_admin/stuck_work.rs:331-792` |
| `tests::a_source_counts_its_open_work_and_sums_only_what_it_could_count` | Test | `apps/causeway-server/src/metadata_admin/stuck_work.rs:404-468` |

… and 13 more in the JSON.

Changed as whole files (no declaration at the changed lines): `apps-web/causeway-web/src/pages/data/wire.ts`, `proto/causeway/metadata/v2/admin.proto`.
Removed symbols: `countsOf` (Function, `apps-web/causeway-web/src/pages/data/stuck-work.ts:111-121` at base), `SourceCount` (Interface, `apps-web/causeway-web/src/pages/data/stuck-work.ts:38-42` at base), `the stuck-work filters, as the shell's admits::counts open and in-review work per source` (Test, `apps-web/causeway-web/test/app-operations.test.ts:194-205` at base).

**Affected (54 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (40)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `PortActOnStuckWorkResponse` | Class | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:652-656` | 1 | 1.00 | USES_TYPE | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:655` |
| `PortActOnStuckWorkResponse::__init__` | Method | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:656` | 1 | 1.00 | USES_TYPE | `services-py/relay-grpc/gen/causeway/metadata/v2/admin_pb2.pyi:656` |
| `SourceCount` | Struct | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:16-23` | 1 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:16` |
| `stuck_work_indication` | Function | `apps/causeway-server/src/governance/issues.rs:111-124` | 1 | 1.00 | CALLS | `apps/causeway-server/src/governance/issues.rs:114` |
| `MetadataAdmin::port_act_on_stuck_work` | Method | `apps/causeway-server/src/metadata_admin/handlers.rs:567-623` | 1 | 1.00 | CALLS | `apps/causeway-server/src/metadata_admin/handlers.rs:618` |
| `StuckWorkCounts` | Struct | `apps/causeway-server/src/metadata_admin/stuck_work.rs:95-100` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/metadata_admin/stuck_work.rs:97` |
| `tests::every_class_of_stuck_work_reaches_one_surface` | Test | `apps/causeway-server/src/metadata_admin/stuck_work.rs:568-624` | 1 | 1.00 | CALLS | `apps/causeway-server/src/metadata_admin/stuck_work.rs:616` |
| `stuck_work_metrics` | Function | `apps/causeway-server/src/system_status.rs:74-96` | 1 | 1.00 | CALLS | `apps/causeway-server/src/system_status.rs:75` |
| `StuckWorkSource::act` | Method | `crates/stuck-work/src/port.rs:860-865` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:54-61 [resolved]` |
| `StuckWorkSource::bounds` | Method | `crates/stuck-work/src/port.rs:910` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:63-65 [resolved]` |
| `StuckWorkSource::fencing` | Method | `crates/stuck-work/src/port.rs:953` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:67-69 [resolved]` |
| `StuckWorkSource::get` | Method | `crates/stuck-work/src/port.rs:854-857` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:47-52 [resolved]` |
| `StuckWorkSource::id` | Method | `crates/stuck-work/src/port.rs:845` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:35-37 [resolved]` |
| `StuckWorkSource::list` | Method | `crates/stuck-work/src/port.rs:848-851` | 1 | 1.00 | DISPATCHES_TO | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:39-45 [resolved]` |
| `createPages` | Function | `apps-web/causeway-web/src/pages/routes.tsx:314-592` | 1 | 1.00 | CALLS | `apps-web/causeway-web/src/pages/routes.tsx:567` |
| `the stuck-work filters, as the shell's admits::formats and links as the shell does` | Test | `apps-web/causeway-web/test/app-operations.test.ts:219-224` | 1 | 1.00 | CALLS | `apps-web/causeway-web/test/app-operations.test.ts:221` |
| `the stuck-work filters, as the shell's admits::narrows by class, subject and age` | Test | `apps-web/causeway-web/test/app-operations.test.ts:198-210` | 1 | 1.00 | CALLS | `apps-web/causeway-web/test/app-operations.test.ts:203` |
| `StuckWorkListPage` | Struct | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:26-47` | 2 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:30` |
| `StuckWorkListPage::new` | Method | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:52-69` | 2 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:54` |
| `counts_header` | Function | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:208-231` | 2 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:208` |
| `load_issues` | Function | `apps/causeway-server/src/governance/issues.rs:23-103` | 2 | 1.00 | CALLS | `apps/causeway-server/src/governance/issues.rs:101` |
| `StuckWorkCounts#impl` | Impl | `apps/causeway-server/src/metadata_admin/stuck_work.rs:102-112` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/metadata_admin/stuck_work.rs:102-112` |
| `stuck_work` | Function | `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:132-212` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:181` |
| `metrics_json_payload` | Function | `apps/causeway-server/src/system_status.rs:44-69` | 2 | 1.00 | CALLS | `apps/causeway-server/src/system_status.rs:66` |
| `StuckWorkSource::detail` | Method | `crates/stuck-work/src/port.rs:872-890` | 2 | 1.00 | CALLS | `crates/stuck-work/src/port.rs:878` |
| `StuckWorkListPage#impl` | Impl | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100` | 3 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:49-100` |
| `filters` | Function | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:249-287` | 3 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:249` |
| `stuck_work_list` | Function | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:144-172` | 3 | 1.00 | CALLS | `apps/causeway-server-web/src/views/platform/operations/stuck_work.rs:158` |
| `issues` | Function | `apps/causeway-server/src/routes/pages/governance/issues.rs:15-56` | 3 | 1.00 | CALLS | `apps/causeway-server/src/routes/pages/governance/issues.rs:38` |
| `metrics_json` | Function | `apps/causeway-server/src/routes/system.rs:108-110` | 3 | 1.00 | CALLS | `apps/causeway-server/src/routes/system.rs:109` |
| `tests::a_source_that_shares_no_payload_still_has_a_detail` | Test | `crates/stuck-work/src/port.rs:1117-1178` | 3 | 1.00 | CALLS | `crates/stuck-work/src/port.rs:1165` |
| `startPages::appFor` | Function | `apps-web/causeway-web/src/pages/runtime.ts:175-181` | 2 | 0.75 | CALLS | `apps-web/causeway-web/src/pages/runtime.ts:178 [syntactic 0.75]` |
| `startPages::page` | Function | `apps-web/causeway-web/src/pages/runtime.ts:192-209` | 3 | 0.75 | CALLS | `apps-web/causeway-web/src/pages/runtime.ts:207` |
| `tests::unheld_groups` | Function | `component/platform/coherence/src/checks.rs:805-814` | 1 | 0.50 | CALLS | `component/platform/coherence/src/checks.rs:811 [syntactic 0.50]` |
| `tests::a_policy_group_with_no_holder_is_reported` | Test | `component/platform/coherence/src/checks.rs:819-846` | 2 | 0.50 | CALLS | `component/platform/coherence/src/checks.rs:843` |
| `tests::every_group_a_policy_names_is_held_by_a_principal` | Test | `component/platform/coherence/src/checks.rs:788-802` | 2 | 0.50 | CALLS | `component/platform/coherence/src/checks.rs:795` |
| `routes` | Function | `apps/causeway-server/src/routes/app/operations.rs:10-42` | 3 | 0.50 | REFERENCES | `apps/causeway-server/src/routes/app/operations.rs:34 [syntactic 0.50]` |
| `edgeMeters` | Function | `apps-web/causeway-web/src/edge/telemetry.ts:107-153` | 1 | 0.38 | CALLS | `apps-web/causeway-web/src/edge/telemetry.ts:135 [syntactic 0.38]` |
| `startEdge` | Function | `apps-web/causeway-web/src/server.ts:105-284` | 2 | 0.38 | CALLS | `apps-web/causeway-web/src/server.ts:113` |
| `startTestEdge` | Function | `apps-web/causeway-web/test/harness.ts:112-119` | 3 | 0.38 | CALLS | `apps-web/causeway-web/test/harness.ts:114` |

_Files and packages (14)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `apps-web/causeway-web/src/pages/data/governance.ts` | File | `apps-web/causeway-web/src/pages/data/governance.ts` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/data/governance.ts:14` |
| `apps-web/causeway-web/src/pages/data/marketplace.ts` | File | `apps-web/causeway-web/src/pages/data/marketplace.ts` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/data/marketplace.ts:14` |
| `apps-web/causeway-web/src/pages/data/people.ts` | File | `apps-web/causeway-web/src/pages/data/people.ts` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/data/people.ts:9` |
| `apps-web/causeway-web/src/pages/data/stuck-work.ts` | File | `apps-web/causeway-web/src/pages/data/stuck-work.ts` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/data/stuck-work.ts:15` |
| `apps-web/causeway-web/src/pages/routes.tsx` | File | `apps-web/causeway-web/src/pages/routes.tsx` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/routes.tsx:52` |
| `apps-web/causeway-web/src/pages/routes/Glossary.tsx` | File | `apps-web/causeway-web/src/pages/routes/Glossary.tsx` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/routes/Glossary.tsx:22` |
| `apps-web/causeway-web/src/pages/routes/Product.tsx` | File | `apps-web/causeway-web/src/pages/routes/Product.tsx` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/routes/Product.tsx:12` |
| `apps-web/causeway-web/src/pages/routes/StuckWork.tsx` | File | `apps-web/causeway-web/src/pages/routes/StuckWork.tsx` | 1 | 1.00 | IMPORTS | `apps-web/causeway-web/src/pages/routes/StuckWork.tsx:9-15` |
| `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs` | File | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/metadata_admin/handlers/listing_tests.rs:13` |
| `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` | File | `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages/metadata_ops/stuck_work.rs:15-17` |
| `apps/causeway-server/src/governance.rs` | File | `apps/causeway-server/src/governance.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/governance.rs:20` |
| `apps/causeway-server/src/routes/pages.rs` | File | `apps/causeway-server/src/routes/pages.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages.rs:53-58` |
| `apps-web/causeway-web/src/server.ts` | File | `apps-web/causeway-web/src/server.ts` | 2 | 0.38 | IMPORTS | `apps-web/causeway-web/src/server.ts:31-37` |
| `apps-web/causeway-web/test/harness.ts` | File | `apps-web/causeway-web/test/harness.ts` | 3 | 0.38 | IMPORTS | `apps-web/causeway-web/test/harness.ts:15` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 67105 nodes, 288543 edges · 2126 ms</sub>
