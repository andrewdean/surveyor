<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `1bdc4035b097`, dataset at `329206085616`: **7 files, 30 changed symbols, 1866 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (30)**

| Symbol | Kind | Where |
|---|---|---|
| `Design log` | Document | `docs/DESIGN-LOG.md:1-6018` |
| `TypeScript web tier workplan: execution ledger` | Document | `docs/plans/TYPESCRIPT-WEB-TIER-WORKPLAN.md:1-225` |
| `DbError` | Enum | `crates/causeway-db-core/src/error.rs:4-59` |
| `DbError::SchemaNotCurrent` | Variant | `crates/causeway-db-core/src/error.rs:18-21` |
| `AppliedMigration` | Struct | `crates/causeway-db-core/src/lib.rs:345-349` |
| `BaselineDrift` | Struct | `crates/causeway-db-core/src/lib.rs:289-293` |
| `BaselineDrift#impl` | Impl | `crates/causeway-db-core/src/lib.rs:295-319` |
| `BaselineDrift#impl(Display)` | Impl | `crates/causeway-db-core/src/lib.rs:321-341` |
| `BaselineDrift::changed` | Method | `crates/causeway-db-core/src/lib.rs:310-312` |
| `BaselineDrift::failed` | Method | `crates/causeway-db-core/src/lib.rs:304-306` |
| `BaselineDrift::fmt` | Method | `crates/causeway-db-core/src/lib.rs:322-340` |
| `BaselineDrift::is_current` | Method | `crates/causeway-db-core/src/lib.rs:316-318` |
| `BaselineDrift::pending` | Method | `crates/causeway-db-core/src/lib.rs:298-300` |
| `applied_migrations` | Function | `crates/causeway-db-core/src/lib.rs:383-408` |
| `applied_migrations::select #096104a5` | Query | `crates/causeway-db-core/src/lib.rs:396` |
| `applied_migrations::select #18b95f3b` | Query | `crates/causeway-db-core/src/lib.rs:385-386` |
| `baseline_drift` | Function | `crates/causeway-db-core/src/lib.rs:355-370` |
| `baseline_drift_tests` | Module | `crates/causeway-db-core/src/lib.rs:641-697` |
| `baseline_drift_tests::a_schema_at_or_ahead_of_the_build_is_current` | Test | `crates/causeway-db-core/src/lib.rs:653-664` |
| `baseline_drift_tests::an_empty_schema_has_the_whole_baseline_pending` | Test | `crates/causeway-db-core/src/lib.rs:687-696` |
| `baseline_drift_tests::applied` | Function | `crates/causeway-db-core/src/lib.rs:644-650` |
| `baseline_drift_tests::pending_failed_and_changed_migrations_are_each_named` | Test | `crates/causeway-db-core/src/lib.rs:667-684` |
| `embedded_baseline` | Function | `crates/causeway-db-core/src/lib.rs:373-379` |
| `init` | Function | `crates/causeway-db-core/src/lib.rs:260-284` |
| `MigratedPostgresConfig#impl` | Impl | `crates/causeway-runtime/src/resource.rs:1435-1472` |
| `startup_accepts_a_migrated_schema_and_refuses_one_behind` | Test | `crates/causeway-testkit/tests/postgres.rs:452-501` |
| `startup_accepts_a_migrated_schema_and_refuses_one_behind::delete #2ae051e7` | Query | `crates/causeway-testkit/tests/postgres.rs:485` |
| `startup_accepts_a_migrated_schema_and_refuses_one_behind::update #4403b84d` | Query | `crates/causeway-testkit/tests/postgres.rs:469` |
| `startup_accepts_a_migrated_schema_and_refuses_one_behind::select #f134261f` | Query | `crates/causeway-testkit/tests/postgres.rs:464` |
| `JobContext#impl` | Impl | `jobs/causeway-job-runtime/src/lib.rs:247-351` |


**Affected (1866 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (1753)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `CatalogPublicationCommands::approve` | Method | `apps/causeway-metadata-admin-host/src/approval.rs:195-244` | 1 | 1.00 | REFERENCES | `apps/causeway-metadata-admin-host/src/approval.rs:230` |
| `propose_error` | Function | `apps/causeway-metadata-admin-host/src/placement.rs:110-122` | 1 | 1.00 | USES_TYPE | `apps/causeway-metadata-admin-host/src/placement.rs:110` |
| `run` | Function | `apps/causeway-policy-host/src/bin/policy-lifecycle-executor.rs:244-303` | 1 | 1.00 | CALLS | `apps/causeway-policy-host/src/bin/policy-lifecycle-executor.rs:280` |
| `run` | Function | `apps/causeway-policy-scan/src/main.rs:280-373` | 1 | 1.00 | CALLS | `apps/causeway-policy-scan/src/main.rs:293` |
| `catalog_projection_bundle_from_entry` | Function | `component/catalog/core/src/application.rs:40-77` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/application.rs:43` |
| `count_catalog_assets` | Function | `component/catalog/core/src/application.rs:243-248` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/application.rs:246` |
| `filter_catalog_assets` | Function | `component/catalog/core/src/application.rs:231-241` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/application.rs:237` |
| `find_catalog_asset_by_slug` | Function | `component/catalog/core/src/application.rs:195-209` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/application.rs:198` |
| `list_catalog_assets_by_content_class` | Function | `component/catalog/core/src/application.rs:217-229` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/application.rs:221` |
| `CatalogPublicationApprovalStore::list_catalog_publication_approvals` | Method | `component/catalog/core/src/approval.rs:299-303` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:303` |
| `CatalogPublicationApprovalStore::record_catalog_publication_approval` | Method | `component/catalog/core/src/approval.rs:293-296` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:296` |
| `CatalogPublicationReassessmentQueue::enqueue_catalog_publication_reassessment` | Method | `component/catalog/core/src/approval.rs:313-316` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:316` |
| `MockCatalogPublicationApprovalRepo::list_catalog_publication_approvals` | Method | `component/catalog/core/src/approval.rs:539-569` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:543` |
| `MockCatalogPublicationApprovalRepo::record_catalog_publication_approval` | Method | `component/catalog/core/src/approval.rs:520-537` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:523` |
| `MockCatalogPublicationReassessmentQueue::enqueue_catalog_publication_reassessment` | Method | `component/catalog/core/src/approval.rs:497-506` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:500` |
| `PgCatalogPublicationApprovalRepo::list_catalog_publication_approvals` | Method | `component/catalog/core/src/approval.rs:392-427` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:396` |
| `PgCatalogPublicationApprovalRepo::record_catalog_publication_approval` | Method | `component/catalog/core/src/approval.rs:334-390` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:337` |
| `approval_from_row` | Function | `component/catalog/core/src/approval.rs:430-449` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:430` |
| `policy_evidence_from_row` | Function | `component/catalog/core/src/approval.rs:451-472` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/approval.rs:451` |
| `BindingRow::into_open` | Method | `component/catalog/core/src/bindings.rs:1284-1304` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:1284` |
| `ContainerRow::into_collection` | Method | `component/catalog/core/src/bindings.rs:1320-1334` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:1320` |
| `PgBindingRepo::adopt_store_instance` | Method | `component/catalog/core/src/bindings.rs:297-353` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:304` |
| `PgBindingRepo::advance_route_events_since` | Method | `component/catalog/core/src/bindings.rs:950-973` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:956` |
| `PgBindingRepo::backfill_from_locators` | Method | `component/catalog/core/src/bindings.rs:1081-1086` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:1081` |
| `PgBindingRepo::choose_default_route` | Method | `component/catalog/core/src/bindings.rs:983-1041` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:987` |
| `PgBindingRepo::close_route` | Method | `component/catalog/core/src/bindings.rs:897-920` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:901` |
| `PgBindingRepo::close_routes_except` | Method | `component/catalog/core/src/bindings.rs:866-889` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:870` |
| `PgBindingRepo::close_share_grants_except` | Method | `component/catalog/core/src/bindings.rs:693-714` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:697` |
| `PgBindingRepo::close_unseen_bindings` | Method | `component/catalog/core/src/bindings.rs:725-757` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:731` |
| `PgBindingRepo::collection_by_slug` | Method | `component/catalog/core/src/bindings.rs:515-535` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:518` |
| `PgBindingRepo::collection_members` | Method | `component/catalog/core/src/bindings.rs:544-574` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:547` |
| `PgBindingRepo::describe_container` | Method | `component/catalog/core/src/bindings.rs:256-285` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:262` |
| `PgBindingRepo::link_container_entity` | Method | `component/catalog/core/src/bindings.rs:490-506` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:494` |
| `PgBindingRepo::open_binding_at_locator` | Method | `component/catalog/core/src/bindings.rs:385-406` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:388` |
| `PgBindingRepo::open_bindings` | Method | `component/catalog/core/src/bindings.rs:465-483` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:465` |
| `PgBindingRepo::open_bindings_by_slug` | Method | `component/catalog/core/src/bindings.rs:413-432` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:413` |
| `PgBindingRepo::open_primary` | Method | `component/catalog/core/src/bindings.rs:360-377` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:360` |
| `PgBindingRepo::open_routes` | Method | `component/catalog/core/src/bindings.rs:1049-1072` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:1052` |
| `PgBindingRepo::open_routes_by_container_id` | Method | `component/catalog/core/src/bindings.rs:439-458` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:442` |
| `PgBindingRepo::record_derivation` | Method | `component/catalog/core/src/bindings.rs:585-626` | 1 | 1.00 | USES_TYPE | `component/catalog/core/src/bindings.rs:589` |

… and 1713 more in the JSON.

_Files and packages (113)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `component/catalog/core/src/application.rs` | File | `component/catalog/core/src/application.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/application.rs:14` |
| `component/catalog/core/src/approval.rs` | File | `component/catalog/core/src/approval.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/approval.rs:15` |
| `component/catalog/core/src/bindings.rs` | File | `component/catalog/core/src/bindings.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/bindings.rs:18` |
| `component/catalog/core/src/concepts.rs` | File | `component/catalog/core/src/concepts.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/concepts.rs:9` |
| `component/catalog/core/src/contract_publish.rs` | File | `component/catalog/core/src/contract_publish.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/contract_publish.rs:5` |
| `component/catalog/core/src/glossary.rs` | File | `component/catalog/core/src/glossary.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/glossary.rs:6` |
| `component/catalog/core/src/identity.rs` | File | `component/catalog/core/src/identity.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/identity.rs:18` |
| `component/catalog/core/src/ingest.rs` | File | `component/catalog/core/src/ingest.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/ingest.rs:8` |
| `component/catalog/core/src/lineage.rs` | File | `component/catalog/core/src/lineage.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/lineage.rs:7` |
| `component/catalog/core/src/maintenance.rs` | File | `component/catalog/core/src/maintenance.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/maintenance.rs:4` |
| `component/catalog/core/src/mock_projection.rs` | File | `component/catalog/core/src/mock_projection.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/mock_projection.rs:17` |
| `component/catalog/core/src/placements.rs` | File | `component/catalog/core/src/placements.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/placements.rs:12` |
| `component/catalog/core/src/projection.rs` | File | `component/catalog/core/src/projection.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/projection.rs:16` |
| `component/catalog/core/src/read_port.rs` | File | `component/catalog/core/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/read_port.rs:15` |
| `component/catalog/core/src/read_repo.rs` | File | `component/catalog/core/src/read_repo.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/read_repo.rs:7` |
| `component/catalog/core/src/repo.rs` | File | `component/catalog/core/src/repo.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/repo.rs:10` |
| `component/catalog/core/src/review.rs` | File | `component/catalog/core/src/review.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/review.rs:15` |
| `component/catalog/core/src/search_queue.rs` | File | `component/catalog/core/src/search_queue.rs` | 1 | 1.00 | IMPORTS | `component/catalog/core/src/search_queue.rs:4` |
| `component/collaboration/conversations/src/conversation_repo.rs` | File | `component/collaboration/conversations/src/conversation_repo.rs` | 1 | 1.00 | IMPORTS | `component/collaboration/conversations/src/conversation_repo.rs:11` |
| `component/collaboration/conversations/src/read_port.rs` | File | `component/collaboration/conversations/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/collaboration/conversations/src/read_port.rs:8` |
| `component/content/articles/src/article_repo.rs` | File | `component/content/articles/src/article_repo.rs` | 1 | 1.00 | IMPORTS | `component/content/articles/src/article_repo.rs:16` |
| `component/content/articles/src/read_port.rs` | File | `component/content/articles/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/content/articles/src/read_port.rs:8` |
| `component/governance/contracts/src/canonical_contract_repo.rs` | File | `component/governance/contracts/src/canonical_contract_repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/canonical_contract_repo.rs:22` |
| `component/governance/contracts/src/contract_projection_repo.rs` | File | `component/governance/contracts/src/contract_projection_repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/contract_projection_repo.rs:21` |
| `component/governance/contracts/src/drift.rs` | File | `component/governance/contracts/src/drift.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/drift.rs:11` |
| `component/governance/contracts/src/read_port.rs` | File | `component/governance/contracts/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/read_port.rs:16` |
| `component/governance/contracts/src/review.rs` | File | `component/governance/contracts/src/review.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/review.rs:13` |
| `component/governance/contracts/src/source_evidence.rs` | File | `component/governance/contracts/src/source_evidence.rs` | 1 | 1.00 | IMPORTS | `component/governance/contracts/src/source_evidence.rs:10` |
| `component/governance/core/src/governance_projection_repo.rs` | File | `component/governance/core/src/governance_projection_repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/governance_projection_repo.rs:19` |
| `component/governance/core/src/policy_barrier_repo.rs` | File | `component/governance/core/src/policy_barrier_repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/policy_barrier_repo.rs:7` |
| `component/governance/core/src/policy_propagation.rs` | File | `component/governance/core/src/policy_propagation.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/policy_propagation.rs:4` |
| `component/governance/core/src/policy_repo.rs` | File | `component/governance/core/src/policy_repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/policy_repo.rs:10` |
| `component/governance/core/src/read_port.rs` | File | `component/governance/core/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/read_port.rs:13` |
| `component/governance/core/src/seed.rs` | File | `component/governance/core/src/seed.rs` | 1 | 1.00 | IMPORTS | `component/governance/core/src/seed.rs:2` |
| `component/governance/ownership/src/repo.rs` | File | `component/governance/ownership/src/repo.rs` | 1 | 1.00 | IMPORTS | `component/governance/ownership/src/repo.rs:2` |
| `component/governance/resolution/src/adapters.rs` | File | `component/governance/resolution/src/adapters.rs` | 1 | 1.00 | IMPORTS | `component/governance/resolution/src/adapters.rs:4` |
| `component/governance/resolution/src/maintenance.rs` | File | `component/governance/resolution/src/maintenance.rs` | 1 | 1.00 | IMPORTS | `component/governance/resolution/src/maintenance.rs:4` |
| `component/ingestion/feed/src/read_port.rs` | File | `component/ingestion/feed/src/read_port.rs` | 1 | 1.00 | IMPORTS | `component/ingestion/feed/src/read_port.rs:7` |
| `component/ingestion/feed/src/read_repo.rs` | File | `component/ingestion/feed/src/read_repo.rs` | 1 | 1.00 | IMPORTS | `component/ingestion/feed/src/read_repo.rs:4` |
| `component/marketplace/core/src/marketplace_projection_repo.rs` | File | `component/marketplace/core/src/marketplace_projection_repo.rs` | 1 | 1.00 | IMPORTS | `component/marketplace/core/src/marketplace_projection_repo.rs:13` |

… and 73 more in the JSON.

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 66974 nodes, 287705 edges · 3009 ms</sub>
