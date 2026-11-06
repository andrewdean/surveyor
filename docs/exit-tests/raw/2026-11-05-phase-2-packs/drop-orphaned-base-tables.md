# Context pack

Task: The base workspace schema still creates policy_propagation_barriers and access_grants, which nothing reads or writes; the governance/core slice owns the live copy of the first. Add the base migration that drops both, register it with the embedded list, extend the vsa migration rules, and add a testkit test that keeps slice tables out of the base schema.

## Seeds

- `sql:public.access_grants` Table 1.00 (identifier `access_grants`)
- `sql:public.policy_propagation_barriers` Table 1.00 (identifier `policy_propagation_barriers`)
- `rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::schemas` Method 0.90 (text)
- `rust:jobs/causeway-migrate/src/main.rs::base_schema` Function 0.74 (text)
- `sql:public.policy_propagation_barriers#idx_policy_propagation_barriers_effective` Index 0.72 (text)
- `sql:public.policy_propagation_barriers#idx_policy_propagation_barriers_path` Index 0.72 (text)
- `sql:public.policy_propagation_barriers.created_at` Column 0.70 (text)
- `rust:component/orchestration/workflows/src/impls/read_repo.rs::database_tests::reads_find_the_slice_tables_from_the_workspace_pool` Test 0.69 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::repository_targets` Function 0.69 (text)
- `rust:jobs/causeway-migrate/src/main.rs::base_schema_of` Function 0.68 (text)
- `rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::database_url_for` Method 0.67 (text)
- `rust:component/governance/core/src/rights.rs::count` Function 0.40 (identifier `governance/core`)
- `rust:component/governance/core/src/rights.rs::invalid` Function 0.40 (identifier `governance/core`)
- `rust:component/governance/core/src/rights.rs::pretty` Function 0.40 (identifier `governance/core`)
- `rust:component/governance/core/src/rpc.rs::_` Const 0.40 (identifier `governance/core`)
- `rust:component/governance/core/src/taxonomy_rpc.rs::_` Const 0.40 (identifier `governance/core`)

Budget 30000 tokens, used 29964. Seeds 16, candidates 633, included 101, left out 532.

## Context

### 1. Table public.policy_propagation_barriers
`sql:public.policy_propagation_barriers`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 4.74  
declared in 4 places: component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5, crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql:1-29, component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5  
`-- Governance-owned policy propagation barriers.`  
members 12  
migrations 4: `crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql` (crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql:1-29), `public.policy_propagation_barriers.stopped_obligation_kinds` (component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5), `component/governance/core/migrations/0001_policy_propagation_barriers.sql` (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29) … 1 more  
readers 4: `PgPolicyBarrierRepo::effective_barriers_for_path::query#e3616183` (component/governance/core/src/policy_barrier_repo.rs:130-135), `PolicyScanner::attach_declassification_barriers::query#91b3d355` (apps/causeway-policy-scan/src/main.rs:1143-1147), `PgPolicyBarrierRepo::effective_barriers_for_path` (component/governance/core/src/policy_barrier_repo.rs:130-135) … 1 more  
writers 2: `PgPolicyBarrierRepo::insert_approved_barrier::query#48cb7bfd` (component/governance/core/src/policy_barrier_repo.rs:75-86), `PgPolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:75-86)  
component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29
```sql
-- Governance-owned policy propagation barriers.
--
-- These rows record approved, auditable reasons that policy should stop
-- propagating across a lineage path. Proposed/pending approvals live in the
-- workflow engine; this table contains only effective governance state.

CREATE TABLE IF NOT EXISTS policy_propagation_barriers (
    barrier_id   UUID PRIMARY KEY,
    barrier_kind TEXT NOT NULL CHECK (
        barrier_kind IN (
            'dropped_sensitive_column',
            'masked',
            'hashed',
            'anonymized',
            'aggregated',
            'declassified',
            'owner_override',
            'steward_exception',
            'temporary_bypass'
        )
    ),
    path         TEXT NOT NULL,
    actor        TEXT NOT NULL,
    reason       TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    approved_at  TIMESTAMPTZ NOT NULL,
    expires_at   TIMESTAMPTZ,
    event_json   JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

### 2. Table public.access_grants
`sql:public.access_grants`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 3.29  
declared in 2 places: crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, crates/causeway-db-core/migrations/0001_initial_schema.sql:53-62  
`CREATE TABLE IF NOT EXISTS access_grants (`  
members 10  
migrations 2: `crates/causeway-db-core/legacy-migrations/0004_governance.sql` (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43), `crates/causeway-db-core/migrations/0001_initial_schema.sql` (crates/causeway-db-core/migrations/0001_initial_schema.sql:53-62)  
crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43
```sql
CREATE TABLE IF NOT EXISTS access_grants (
    grant_id    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    principal   TEXT NOT NULL,
    resource    TEXT NOT NULL,
    actions     TEXT[] NOT NULL,
    granted_by  TEXT NOT NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ,
    revoked_at  TIMESTAMPTZ
);
```

### 3. Method TestWorkspace::schemas
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::schemas`  crates/causeway-testkit/src/workspace.rs:209-211  pkg:cargo/causeway-testkit  score 1.81  
`pub fn schemas(&self) -> &[String]`  
Every schema the workspace's migrations created, the base schema (doc continues, 72 bytes)  
callers 8: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:103), `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:103), `two_workspaces_share_the_server_and_nothing_else` (crates/causeway-testkit/tests/postgres.rs:147) … 5 more  
tests 2: `two_workspaces_share_the_server_and_nothing_else` (crates/causeway-testkit/tests/postgres.rs:147), `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:80)  
crates/causeway-testkit/src/workspace.rs:209-211
```rust
    pub fn schemas(&self) -> &[String] {
        &self.schemas
    }
```

### 4. Function base_schema
`rust:jobs/causeway-migrate/src/main.rs::base_schema`  jobs/causeway-migrate/src/main.rs:115-123  pkg:cargo/causeway-migrate  score 1.63  
`fn base_schema(scope: &WorkspaceScope) -> Result<String>`  
The base schema of this job's workspace, `cw_<id>`, derived from (doc continues, 511 bytes)  
callers 1: `main` (jobs/causeway-migrate/src/main.rs:51)  
calls 1: `base_schema_of` (jobs/causeway-migrate/src/main.rs:117)  
uses_types 1: `WorkspaceScope` (jobs/causeway-migrate/src/main.rs:115)  
jobs/causeway-migrate/src/main.rs:115-123
```rust
fn base_schema(scope: &WorkspaceScope) -> Result<String> {
    let alias = std::env::var("CAUSEWAY_PG_SCHEMA").ok();
    let (schema, warning) = base_schema_of(scope, alias.as_deref())?;
    // The job installs no log subscriber; its output is the Job's log.
    if let Some(warning) = warning {
        eprintln!("warning: {warning}");
    }
    Ok(schema)
}
```

### 5. Index public.policy_propagation_barriers#idx_policy_propagation_barriers_effective
`sql:public.policy_propagation_barriers#idx_policy_propagation_barriers_effective`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:34-35  pkg:cargo/governance-component  score 1.61  
`CREATE INDEX IF NOT EXISTS idx_policy_propagation_barriers_effective`  

### 6. Index public.policy_propagation_barriers#idx_policy_propagation_barriers_path
`sql:public.policy_propagation_barriers#idx_policy_propagation_barriers_path`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:31-32  pkg:cargo/governance-component  score 1.61  
`CREATE INDEX IF NOT EXISTS idx_policy_propagation_barriers_path`  

### 7. Test database_tests::reads_find_the_slice_tables_from_the_workspace_pool
`rust:component/orchestration/workflows/src/impls/read_repo.rs::database_tests::reads_find_the_slice_tables_from_the_workspace_pool`  component/orchestration/workflows/src/impls/read_repo.rs:752-816  pkg:cargo/workflows-component  score 1.54  
`async fn reads_find_the_slice_tables_from_the_workspace_pool()`  
The shell embeds this repository on its workspace pool, whose (doc continues, 186 bytes)  
members 1  
calls 16: `workflows_schema` (component/orchestration/workflows/src/impls/read_repo.rs:760), `PgWorkflowReadRepo::new` (component/orchestration/workflows/src/impls/read_repo.rs:780), `in_workspace` (component/orchestration/workflows/src/impls/read_repo.rs:753) … 13 more  
tests_for 16: `PgWorkflowReadRepo::new` (component/orchestration/workflows/src/impls/read_repo.rs:780), `workflows_schema` (component/orchestration/workflows/src/impls/read_repo.rs:760), `in_workspace` (component/orchestration/workflows/src/impls/read_repo.rs:753) … 13 more  
writes 5: `public.forms` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `public.workflow_definitions` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `public.workflow_instances` (component/orchestration/workflows/src/impls/read_repo.rs:762-774) … 2 more  
component/orchestration/workflows/src/impls/read_repo.rs:752-816
```rust
    async fn reads_find_the_slice_tables_from_the_workspace_pool() {
        let Some(workspace) = causeway_testkit::in_workspace(causeway_testkit::test_name!())
            .await
            .expect("a migrated workspace")
        else {
            return;
        };
        let pool = workspace.connect().await.expect("connect");
        let schema = workflows_schema();
        sqlx::raw_sql(&format!(
            "INSERT INTO {schema}.forms (name, version, body_toml, body_json, checksum) \
                 VALUES ('intake', '1', '', '{{\"title\": \"Intake\"}}', 'c'); \
             INSERT INTO {schema}.workflow_definitions \
                 (name, version, kind, body_toml, body_json, checksum) \
                 VALUES ('onboarding', '1', 'onboarding', '', '{{}}', 'c'); \
             INSERT INTO {schema}.workflow_instances \
                 (id, definition_name, definition_version, status) \
                 VALUES ('00000000-0000-0000-0000-000000000001', 'onboarding', '1', 'waiting_signal'); \
             INSERT INTO {schema}.workflow_events (instance_id, kind) \
                 VALUES ('00000000-0000-0000-0000-000000000001', 'started'); \
             INSERT INTO {schema}.workflow_tasks (id, instance_id, step, title, assignee) \
                 VALUES ('00000000-0000-0000-0000-000000000002', \
                         '00000000-0000-0000-0000-000000000001', 'review', 'Review it', 'user:reviewer');"
        ))
        .execute(&pool)
        .await
        .expect("the slice's rows insert");

        let repo = PgWorkflowReadRepo::new(pool);
        let instance = Uuid::from_u128(1);
        let task = Uuid::from_u128(2);

        let open = repo.list_open_tasks(None).await.expect("open tasks list");
        assert_eq!(
            open.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![task]
        );
        let loaded = repo
            .load_task_with_form(task)
            .await
            .expect("the task loads")
            .expect("the task is open");
        assert_eq!(loaded.instance_id, instance);
        let form = repo
            .find_form_body("intake", None)
            .await
            .expect("the form reads");
        assert_eq!(form, Some(serde_json::json!({"title": "Intake"})));
        let events = repo
            .replay_events_after(instance, 0)
            .await
            .expect("the events replay");
        assert_eq!(events.len(), 1);
        let instance_tasks = repo
            .workflow_tasks_for_instance(instance)
            .await
            .expect("the instance's tasks read");
        assert_eq!(
            instance_tasks
                .iter()
                .map(|row| (row.task_id, row.step.as_str()))
                .collect::<Vec<_>>(),
            vec![(task, "review")]
        );
    }
```

### 8. Column public.policy_propagation_barriers.created_at
`sql:public.policy_propagation_barriers.created_at`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 1.52  
`-- Governance-owned policy propagation barriers.`  

### 9. Function base_schema_of
`rust:jobs/causeway-migrate/src/main.rs::base_schema_of`  jobs/causeway-migrate/src/main.rs:149-158  pkg:cargo/causeway-migrate  score 1.36  
`fn base_schema_of( scope: &WorkspaceScope, alias: Option<&str>, ) -> Result<(String, Option<String>), ConfigError>`  
The base schema `scope` derives, checked against `alias`, the value of (doc continues, 137 bytes)  
callers 4: `tests::the_base_schema_is_derived_from_the_workspace_id` (jobs/causeway-migrate/src/main.rs:241), `tests::a_schema_alias_that_names_another_schema_stops_the_job` (jobs/causeway-migrate/src/main.rs:263), `tests::a_schema_alias_that_repeats_the_derived_schema_warns` (jobs/causeway-migrate/src/main.rs:250) … 1 more  
calls 3: `evaluate_companion` (jobs/causeway-migrate/src/main.rs:153), `AuthoritySource::warning` (jobs/causeway-migrate/src/main.rs:156), `AliasVerdict::warning` (jobs/causeway-migrate/src/main.rs:156)  
references 1: `Companion::PgSchema` (jobs/causeway-migrate/src/main.rs:153)  
tests 3: `tests::the_base_schema_is_derived_from_the_workspace_id` (jobs/causeway-migrate/src/main.rs:241), `tests::a_schema_alias_that_repeats_the_derived_schema_warns` (jobs/causeway-migrate/src/main.rs:250), `tests::a_schema_alias_that_names_another_schema_stops_the_job` (jobs/causeway-migrate/src/main.rs:263)  
uses_types 2: `ConfigError` (jobs/causeway-migrate/src/main.rs:152), `WorkspaceScope` (jobs/causeway-migrate/src/main.rs:150)  
jobs/causeway-migrate/src/main.rs:149-158
```rust
fn base_schema_of(
    scope: &WorkspaceScope,
    alias: Option<&str>,
) -> Result<(String, Option<String>), ConfigError> {
    let verdict = alias::evaluate_companion(scope, Companion::PgSchema, alias)?;
    Ok((
        scope.postgres_schema(),
        verdict.warning().map(str::to_owned),
    ))
}
```

### 10. Function repository_targets
`rust:crates/causeway-db-core/src/workspace_migrations.rs::repository_targets`  crates/causeway-db-core/src/workspace_migrations.rs:238-264  pkg:cargo/causeway-db-core  score 1.21  
`pub fn repository_targets( repo_root: &Path, base_schema: &str, ) -> WorkspaceMigrationResult<Vec<MigrationTarget>>`  
The targets of the repository checked out at `repo_root`: the baseline (doc continues, 549 bytes)  
callers 9: `tests::a_workspace_id_is_valid_and_leaves_room_for_every_schema` (crates/causeway-testkit/src/workspace.rs:420), `migrate` (crates/causeway-testkit/src/infra.rs:274), `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:89) … 6 more  
calls 6: `MigrationTarget::component` (crates/causeway-db-core/src/workspace_migrations.rs:261), `MigrationTarget::baseline` (crates/causeway-db-core/src/workspace_migrations.rs:246), `subdirectories` (crates/causeway-db-core/src/workspace_migrations.rs:249) … 3 more  
importers 1: `tests` (crates/causeway-testkit/src/parity.rs:200)  
tests 7: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:89), `tests::the_repository_layout_names_schemas_as_the_staging_script_does` (crates/causeway-db-core/src/workspace_migrations.rs:575), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:222) … 4 more  
uses_types 2: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:241), `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:241)  
crates/causeway-db-core/src/workspace_migrations.rs:238-264
```rust
pub fn repository_targets(
    repo_root: &Path,
    base_schema: &str,
) -> WorkspaceMigrationResult<Vec<MigrationTarget>> {
    let baseline = repo_root.join(REPOSITORY_BASELINE_DIR);
    if !baseline.is_dir() {
        return Err(WorkspaceMigrationError::BaselineMissing { path: baseline });
    }
    let mut targets = vec![MigrationTarget::baseline(base_schema, baseline)];

    let mut sets: Vec<(String, PathBuf)> = Vec::new();
    for area in subdirectories(&repo_root.join(REPOSITORY_COMPONENT_DIR))? {
        for slice in subdirectories(&area)? {
            let migrations = slice.join("migrations");
            if migrations.is_dir() {
                let suffix = format!("{}_{}", underscored(&area)?, underscored(&slice)?);
                sets.push((suffix, migrations));
            }
        }
    }
    sets.sort();
    targets.extend(
        sets.into_iter()
            .map(|(suffix, path)| MigrationTarget::component(base_schema, &suffix, path)),
    );
    Ok(targets)
}
```

### 11. Method TestWorkspace::database_url_for
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::database_url_for`  crates/causeway-testkit/src/workspace.rs:225-234  pkg:cargo/causeway-testkit  score 1.16  
`pub fn database_url_for(&self, component: &str) -> String`  
A database URL that resolves `component`'s schema first, then the (doc continues, 230 bytes)  
callers 1: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:114)  
calls 3: `with_search_path` (crates/causeway-testkit/src/workspace.rs:226), `WorkspaceScope::component_schema` (crates/causeway-testkit/src/workspace.rs:229), `WorkspaceScope::postgres_schema` (crates/causeway-testkit/src/workspace.rs:230)  
tests 1: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:114)  
crates/causeway-testkit/src/workspace.rs:225-234
```rust
    pub fn database_url_for(&self, component: &str) -> String {
        with_search_path(
            &self.owner_url,
            &[
                self.scope.component_schema(component).as_str(),
                self.scope.postgres_schema().as_str(),
                "public",
            ],
        )
    }
```

### 12. Schema public
`sql:public`  score 1.14  via `public.access_grants` <- CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
members 253  

### 13. Struct TestWorkspace
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace`  crates/causeway-testkit/src/workspace.rs:29-39  pkg:cargo/causeway-testkit  score 1.12  via `TestWorkspace#impl` -> USES_TYPE (crates/causeway-testkit/src/workspace.rs:51-292, 1.00)  
`pub struct TestWorkspace`  
One test's workspace: its scope, its migrated schemas, the secret files (doc continues, 232 bytes)  
members 18  
importers 7: `crates/causeway-testkit/src/lib.rs` (crates/causeway-testkit/src/lib.rs:94), `crates/causeway-testkit/tests/kafka.rs` (crates/causeway-testkit/tests/kafka.rs:12-15), `crates/causeway-testkit/tests/valkey.rs` (crates/causeway-testkit/tests/valkey.rs:10-13) … 4 more  
used_by 5: `workspace` (crates/causeway-testkit/tests/postgres.rs:27), `TestWorkspace#impl` (crates/causeway-testkit/src/workspace.rs:51-292), `run_child` (crates/causeway-testkit/src/isolation.rs:325) … 2 more  
uses_types 2: `LocalTopology` (crates/causeway-testkit/src/workspace.rs:35), `WorkspaceScope` (crates/causeway-testkit/src/workspace.rs:30)  
crates/causeway-testkit/src/workspace.rs:29-39
```rust
pub struct TestWorkspace {
    scope: WorkspaceScope,
    schemas: Vec<String>,
    owner_url: String,
    database_url: String,
    pool: PgPool,
    topology: LocalTopology,
    // The variables a test's own process finds each attached service by.
    service_env: Vec<(&'static str, String)>,
    kept: bool,
}
```

### 14. Test a_workspace_migrates_every_component_into_its_derived_schemas
`rust:crates/causeway-testkit/tests/postgres.rs::a_workspace_migrates_every_component_into_its_derived_schemas`  crates/causeway-testkit/tests/postgres.rs:76-122  pkg:cargo/causeway-testkit  score 1.12  via `repository_targets` -> TESTED_BY (crates/causeway-testkit/tests/postgres.rs:89, 1.00)  
`async fn a_workspace_migrates_every_component_into_its_derived_schemas()`  
members 2  
calls 18: `repository_targets` (crates/causeway-testkit/tests/postgres.rs:89), `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:88), `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:94) … 15 more  
references 1: `MigrationMode::Check` (crates/causeway-testkit/tests/postgres.rs:98)  
tests_for 15: `repository_targets` (crates/causeway-testkit/tests/postgres.rs:89), `schemas_like` (crates/causeway-testkit/tests/postgres.rs:81), `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:88) … 12 more  
crates/causeway-testkit/tests/postgres.rs:76-122
```rust
async fn a_workspace_migrates_every_component_into_its_derived_schemas() {
    let workspace = workspace().await;
    let base = workspace.scope().postgres_schema();
    assert!(workspace.scope().as_str().starts_with("t-"));
    assert_eq!(workspace.schemas()[0], base);
    assert_eq!(schemas_like(&base).await, {
        let mut expected = workspace.schemas().to_vec();
        expected.sort();
        expected
    });

    // Every schema is current: the library's check mode finds nothing.
    let containers = Containers::shared().await.expect("the shared containers");
    let targets = workspace_migrations::repository_targets(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        &base,
    )
    .expect("targets");
    let summary = workspace_migrations::migrate_workspace(
        &containers.postgres().owner_url(),
        &targets,
        &workspace.scope().workspace_uuid(),
        MigrationMode::Check,
        |_| {},
    )
    .await
    .expect("check");
    assert_eq!(summary.schemas(), workspace.schemas().len());
    assert_eq!(summary.pending(), 0);

    // The pool resolves the base schema first.
    let current: String = sqlx::query_scalar("SELECT current_schema()::text")
        .fetch_one(workspace.pool())
        .await
        .expect("current schema");
    assert_eq!(current, base);

    // A slice's URL resolves its own schema first.
    let mut slice = PgConnection::connect(&workspace.database_url_for("platform/supervision"))
        .await
        .expect("connect");
    let current: String = sqlx::query_scalar("SELECT current_schema()::text")
        .fetch_one(&mut slice)
        .await
        .expect("current schema");
    assert_eq!(current, format!("{base}_platform_supervision"));
}
```

### 15. Column public.policy_propagation_barriers.stopped_obligation_kinds
`sql:public.policy_propagation_barriers.stopped_obligation_kinds`  component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5  pkg:cargo/governance-component  score 1.04  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5, 1.00)  
`-- Record the exact obligation kinds an approved propagation barrier stops.`  
migrates 1: `public.policy_propagation_barriers` (component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5)  

### 16. Column public.policy_propagation_barriers.barrier_id
`sql:public.policy_propagation_barriers.barrier_id`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.85  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 17. Column public.policy_propagation_barriers.barrier_kind
`sql:public.policy_propagation_barriers.barrier_kind`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.85  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 18. Column public.policy_propagation_barriers.actor
`sql:public.policy_propagation_barriers.actor`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 19. Column public.policy_propagation_barriers.approved_at
`sql:public.policy_propagation_barriers.approved_at`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 20. Column public.policy_propagation_barriers.event_json
`sql:public.policy_propagation_barriers.event_json`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 21. Column public.policy_propagation_barriers.expires_at
`sql:public.policy_propagation_barriers.expires_at`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 22. Column public.policy_propagation_barriers.path
`sql:public.policy_propagation_barriers.path`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 23. Column public.policy_propagation_barriers.reason
`sql:public.policy_propagation_barriers.reason`  component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29  pkg:cargo/governance-component  score 0.83  via `public.policy_propagation_barriers` -> CONTAINS (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
`-- Governance-owned policy propagation barriers.`  

### 24. Index public.access_grants#idx_access_grants_principal
`sql:public.access_grants#idx_access_grants_principal`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:45  pkg:cargo/causeway-db-core  score 0.82  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:45, 1.00)  
`CREATE INDEX IF NOT EXISTS idx_access_grants_principal ON access_grants(principal);`  

### 25. Index public.access_grants#idx_access_grants_resource
`sql:public.access_grants#idx_access_grants_resource`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:46  pkg:cargo/causeway-db-core  score 0.82  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:46, 1.00)  
`CREATE INDEX IF NOT EXISTS idx_access_grants_resource ON access_grants(resource);`  

### 26. Function count
`rust:component/governance/core/src/rights.rs::count`  component/governance/core/src/rights.rs:1429-1431  pkg:cargo/governance-component  score 0.80  
`fn count(value: usize) -> u32`  
callers 6: `summary` (component/governance/core/src/rights.rs:1290), `repair_to_pb` (component/governance/core/src/rights.rs:1329), `summary` (component/governance/core/src/rights.rs:1289) … 3 more  
component/governance/core/src/rights.rs:1429-1431
```rust
fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
```

### 27. Function invalid
`rust:component/governance/core/src/rights.rs::invalid`  component/governance/core/src/rights.rs:1473-1475  pkg:cargo/governance-component  score 0.80  
`fn invalid(message: impl Into<String>) -> ConnectError`  
callers 17: `required_timestamp` (component/governance/core/src/rights.rs:1441), `required_timestamp` (component/governance/core/src/rights.rs:1443), `slug_argument` (component/governance/core/src/rights.rs:1410) … 14 more  
component/governance/core/src/rights.rs:1473-1475
```rust
fn invalid(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}
```

### 28. Function pretty
`rust:component/governance/core/src/rights.rs::pretty`  component/governance/core/src/rights.rs:1391-1395  pkg:cargo/governance-component  score 0.80  
`fn pretty(document: &TermsDocument) -> Result<String, ConnectError>`  
callers 2: `run_import` (component/governance/core/src/rights.rs:1216), `RightsService::resolve_entities` (component/governance/core/src/rights.rs:909)  
calls 1: `internal` (component/governance/core/src/rights.rs:1394)  
uses_types 1: `TermsDocument` (component/governance/core/src/rights.rs:1391)  
component/governance/core/src/rights.rs:1391-1395
```rust
fn pretty(document: &TermsDocument) -> Result<String, ConnectError> {
    serde_json::to_string_pretty(document)
        .map(|json| format!("{json}\n"))
        .map_err(|error| internal(format!("cannot serialize document: {error}")))
}
```

### 29. Const _
`rust:component/governance/core/src/rpc.rs::_`  component/governance/core/src/rpc.rs:40-43  pkg:cargo/governance-component  score 0.80  
`const _: fn() = || { fn assert_send_sync<T: Send + Sync + 'static>() {} assert_send_sync::<GovernanceService>(); };`  
members 1  
calls 1: `_::assert_send_sync` (component/governance/core/src/rpc.rs:42)  
uses_types 1: `GovernanceService` (component/governance/core/src/rpc.rs:42)  
component/governance/core/src/rpc.rs:40-43
```rust
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<GovernanceService>();
};
```

### 30. Const _
`rust:component/governance/core/src/taxonomy_rpc.rs::_`  component/governance/core/src/taxonomy_rpc.rs:32-35  pkg:cargo/governance-component  score 0.80  
`const _: fn() = || { fn assert_send_sync<T: Send + Sync + 'static>() {} assert_send_sync::<TaxonomyService>(); };`  
members 1  
calls 1: `_::assert_send_sync` (component/governance/core/src/taxonomy_rpc.rs:34)  
uses_types 1: `TaxonomyService` (component/governance/core/src/taxonomy_rpc.rs:34)  
component/governance/core/src/taxonomy_rpc.rs:32-35
```rust
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<TaxonomyService>();
};
```

### 31. File component/governance/core/migrations/0001_policy_propagation_barriers.sql
`file:component/governance/core/migrations/0001_policy_propagation_barriers.sql`  component/governance/core/migrations/0001_policy_propagation_barriers.sql  pkg:cargo/governance-component  score 0.78  via `public.policy_propagation_barriers` <- MIGRATES (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29, 1.00)  
migrates 1: `public.policy_propagation_barriers` (component/governance/core/migrations/0001_policy_propagation_barriers.sql:1-29)  

### 32. Impl TestWorkspace#impl
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace#impl`  crates/causeway-testkit/src/workspace.rs:51-292  pkg:cargo/causeway-testkit  score 0.77  via `TestWorkspace::schemas` <- DEFINES (crates/causeway-testkit/src/workspace.rs:209-211, 1.00)  
`impl TestWorkspace`  
members 16  
uses_types 1: `TestWorkspace` (crates/causeway-testkit/src/workspace.rs:51-292)  

### 33. File crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql
`file:crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql`  crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql  pkg:cargo/causeway-db-core  score 0.75  via `public.policy_propagation_barriers` <- MIGRATES (crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql:1-29, 1.00)  
migrates 1: `public.policy_propagation_barriers` (crates/causeway-db-core/migrations/0003_policy_propagation_barriers.sql:1-29)  

### 34. Method PgPolicyBarrierRepo::insert_approved_barrier
`rust:component/governance/core/src/policy_barrier_repo.rs::PgPolicyBarrierRepo::insert_approved_barrier`  component/governance/core/src/policy_barrier_repo.rs:54-119  pkg:cargo/governance-component  score 0.75  via `public.policy_propagation_barriers` <- WRITES (component/governance/core/src/policy_barrier_repo.rs:75-86, 1.00)  
`fn insert_approved_barrier( &self, barrier: &PolicyPropagationBarrier<ApprovedBarrier>, event: &PolicyBarrierEvent, ) -> BoxFuture<'_, Result<(), DbError>>`  
members 1  
callers 3: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:250), `tests::approved_barrier_stops_matching_path_only` (component/governance/core/src/policy_propagation.rs:167), `activate_manual_action` (crates/causeway-metadata-ops/src/manual_actions.rs:325)  
calls 3: `OutboxRecord::new` (component/governance/core/src/policy_barrier_repo.rs:103), `append` (component/governance/core/src/policy_barrier_repo.rs:101), `IngestObject::commit` (component/governance/core/src/policy_barrier_repo.rs:116)  
dispatched_from 1: `PolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:54-119)  
tests 2: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:250), `tests::approved_barrier_stops_matching_path_only` (component/governance/core/src/policy_propagation.rs:167)  
uses_types 6: `DbError` (component/governance/core/src/policy_barrier_repo.rs:58), `PolicyBarrierEvent` (component/governance/core/src/policy_barrier_repo.rs:57), `ApprovedBarrier` (component/governance/core/src/policy_barrier_repo.rs:56) … 3 more  
writes 1: `public.policy_propagation_barriers` (component/governance/core/src/policy_barrier_repo.rs:75-86)  
component/governance/core/src/policy_barrier_repo.rs:54-119
```rust
    fn insert_approved_barrier(
        &self,
        barrier: &PolicyPropagationBarrier<ApprovedBarrier>,
        event: &PolicyBarrierEvent,
    ) -> BoxFuture<'_, Result<(), DbError>> {
        let barrier = barrier.clone();
        let event = event.clone();
        let schema = self.schema.clone();
        let event_json = serde_json::to_value(&event).unwrap_or_else(|error| {
            serde_json::to_value(PolicyBarrierSerializationError {
                serialization_error: error.to_string(),
            })
            .expect("serializing policy barrier error payload should not fail")
        });
        Box::pin(async move {
            // The `policy_barrier_approved` notification rides in the SAME
            // transaction as the barrier upsert (this replaced the
            // standalone outbox publisher and its dual-write window).
            let outbox_idempotency_key = format!("outbox:policy-barrier:{}", barrier.barrier_id);
            let mut tx = self.pool.begin().await?;
            let insert = format!(
                "INSERT INTO {schema}.policy_propagation_barriers \
                 (barrier_id, barrier_kind, path, actor, reason, created_at, approved_at, expires_at, event_json) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                 ON CONFLICT (barrier_id) DO UPDATE SET \
                   barrier_kind = EXCLUDED.barrier_kind, \
                   path = EXCLUDED.path, \
                   actor = EXCLUDED.actor, \
                   reason = EXCLUDED.reason, \
                   created_at = EXCLUDED.created_at, \
                   approved_at = EXCLUDED.approved_at, \
                   expires_at = EXCLUDED.expires_at, \
                   event_json = EXCLUDED.event_json"
            );
            sqlx::query(&insert)
                .bind(barrier.barrier_id)
                .bind(barrier.kind.as_str())
                .bind(barrier.path.as_str())
                .bind(barrier.actor.as_str())
                .bind(barrier.reason.as_str())
                .bind(barrier.created_at)
                .bind(barrier.approved_at)
                .bind(barrier.expires_at)
                .bind(event_json.clone())
                .execute(&mut *tx)
                .await?;

            causeway_outbox::append(
                &mut *tx,
                &causeway_outbox::OutboxRecord::new(
                    event.event_id,
                    "policy",
                    event.path.as_str(),
                    "policy_barrier_approved",
                    event_json,
                    event.occurred_at,
                    outbox_idempotency_key.as_str(),
                ),
            )
            .await
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;

            tx.commit().await?;
            Ok(())
        })
    }
```

### 35. File jobs/causeway-migrate/src/main.rs
`file:jobs/causeway-migrate/src/main.rs`  jobs/causeway-migrate/src/main.rs  pkg:cargo/causeway-migrate  score 0.75  via `base_schema` <- CONTAINS (jobs/causeway-migrate/src/main.rs:115-123, 1.00)  
members 12  
designs 2: `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md` (docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md:636), `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:472)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2406)  
imports 16: `MigrationMode` (jobs/causeway-migrate/src/main.rs:26), `PostgresConnection` (jobs/causeway-migrate/src/main.rs:28-31), `ConfigError` (jobs/causeway-migrate/src/main.rs:28-31) … 13 more  

### 36. Method PgPolicyBarrierRepo::effective_barriers_for_path
`rust:component/governance/core/src/policy_barrier_repo.rs::PgPolicyBarrierRepo::effective_barriers_for_path`  component/governance/core/src/policy_barrier_repo.rs:121-144  pkg:cargo/governance-component  score 0.75  via `public.policy_propagation_barriers` <- READS (component/governance/core/src/policy_barrier_repo.rs:130-135, 1.00)  
`fn effective_barriers_for_path( &self, path: &LineagePathRef, now: DateTime<Utc>, ) -> BoxFuture<'_, Result<Vec<PolicyPropagationBarrier<ApprovedBarrier>>, DbError>>`  
members 1  
callers 3: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:255), `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:262), `evaluate_lineage_policy_propagation` (component/governance/core/src/policy_propagation.rs:51)  
dispatched_from 1: `PolicyBarrierRepo::effective_barriers_for_path` (component/governance/core/src/policy_barrier_repo.rs:121-144)  
reads 1: `public.policy_propagation_barriers` (component/governance/core/src/policy_barrier_repo.rs:130-135)  
tests 1: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:255)  
uses_types 4: `DbError` (component/governance/core/src/policy_barrier_repo.rs:125), `ApprovedBarrier` (component/governance/core/src/policy_barrier_repo.rs:125), `PolicyPropagationBarrier` (component/governance/core/src/policy_barrier_repo.rs:125) … 1 more  
component/governance/core/src/policy_barrier_repo.rs:121-144
```rust
    fn effective_barriers_for_path(
        &self,
        path: &LineagePathRef,
        now: DateTime<Utc>,
    ) -> BoxFuture<'_, Result<Vec<PolicyPropagationBarrier<ApprovedBarrier>>, DbError>> {
        let path = path.clone();
        let schema = self.schema.clone();
        Box::pin(async move {
            let query = format!(
                "SELECT barrier_id, barrier_kind, path, actor, reason, created_at, approved_at, expires_at \
                 FROM {schema}.policy_propagation_barriers \
                 WHERE path = $1 \
                   AND approved_at <= $2 \
                   AND (expires_at IS NULL OR expires_at > $2) \
                 ORDER BY approved_at DESC, created_at DESC"
            );
            let rows = sqlx::query(&query)
                .bind(path.as_str())
                .bind(now)
                .fetch_all(&self.pool)
                .await?;
            rows.iter().map(row_to_barrier).collect()
        })
    }
```

### 37. Column public.access_grants.granted_at
`sql:public.access_grants.granted_at`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.72  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 38. Column public.access_grants.granted_by
`sql:public.access_grants.granted_by`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.72  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 39. Column public.access_grants.grant_id
`sql:public.access_grants.grant_id`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.72  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 40. Column public.access_grants.actions
`sql:public.access_grants.actions`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.71  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 41. Column public.access_grants.expires_at
`sql:public.access_grants.expires_at`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.71  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 42. Column public.access_grants.principal
`sql:public.access_grants.principal`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.71  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 43. Column public.access_grants.resource
`sql:public.access_grants.resource`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.71  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 44. Column public.access_grants.revoked_at
`sql:public.access_grants.revoked_at`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43  pkg:cargo/causeway-db-core  score 0.71  via `public.access_grants` -> CONTAINS (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
`CREATE TABLE IF NOT EXISTS access_grants (`  

### 45. File crates/causeway-db-core/src/workspace_migrations.rs
`file:crates/causeway-db-core/src/workspace_migrations.rs`  crates/causeway-db-core/src/workspace_migrations.rs  pkg:cargo/causeway-db-core  score 0.71  via `repository_targets` <- CONTAINS (crates/causeway-db-core/src/workspace_migrations.rs:238-264, 1.00)  
members 24  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:489)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2388)  
imports 1: `cargo/sqlx` (crates/causeway-db-core/src/workspace_migrations.rs:24)  

### 46. File component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql
`file:component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql`  component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql  pkg:cargo/governance-component  score 0.70  via `public.policy_propagation_barriers` <- MIGRATES (component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5, 1.00)  
migrates 1: `public.policy_propagation_barriers` (component/governance/core/migrations/0004_policy_propagation_obligation_kinds.sql:1-5)  

### 47. Method PolicyScanner::attach_declassification_barriers
`rust:apps/causeway-policy-scan/src/main.rs::PolicyScanner::attach_declassification_barriers`  apps/causeway-policy-scan/src/main.rs:1136-1198  pkg:cargo/causeway-policy-scan  score 0.69  via `public.policy_propagation_barriers` <- READS (apps/causeway-policy-scan/src/main.rs:1143-1147, 1.00)  
`async fn attach_declassification_barriers( &self, edges: Vec<governance_policy_component::LineageEdge>, now: DateTime<Utc>, ) -> Result<Vec<governance_policy_component::LineageEdge>>`  
members 1  
callers 1: `PolicyScanner::refresh_effective_policy` (apps/causeway-policy-scan/src/main.rs:1039)  
calls 5: `DeclassificationBarrier::new` (apps/causeway-policy-scan/src/main.rs:1181), `PolicyPropagationBarrier::restore_approved` (apps/causeway-policy-scan/src/main.rs:1164), `actual_schema_name` (apps/causeway-policy-scan/src/main.rs:1141) … 2 more  
reads 1: `public.policy_propagation_barriers` (apps/causeway-policy-scan/src/main.rs:1143-1147)  
references 1: `LineageEdge::with_barrier` (apps/causeway-policy-scan/src/main.rs:1195)  
uses_types 5: `LineageEdge` (apps/causeway-policy-scan/src/main.rs:1138), `DeclassificationBarrier` (apps/causeway-policy-scan/src/main.rs:1154), `PolicyPropagationBarrier` (apps/causeway-policy-scan/src/main.rs:1163) … 2 more  
apps/causeway-policy-scan/src/main.rs:1136-1198
```rust
    async fn attach_declassification_barriers(
        &self,
        edges: Vec<governance_policy_component::LineageEdge>,
        now: DateTime<Utc>,
    ) -> Result<Vec<governance_policy_component::LineageEdge>> {
        let governance_schema = actual_schema_name("governance/core");
        let query = format!(
            "SELECT barrier_id, barrier_kind, path, actor, reason, created_at, \
                    approved_at, expires_at, stopped_obligation_kinds \
             FROM {governance_schema}.policy_propagation_barriers \
             WHERE approved_at <= $1 AND (expires_at IS NULL OR expires_at > $1) \
             ORDER BY path, approved_at"
        );
        let rows = sqlx::query(&query)
            .bind(now)
            .fetch_all(&self.pool)
            .await
            .context("loading effective declassification barriers")?;
        let mut by_path = BTreeMap::<String, Vec<DeclassificationBarrier>>::new();
        for row in rows {
            let kinds = serde_json::from_value::<BTreeSet<ObligationKind>>(
                row.try_get("stopped_obligation_kinds")?,
            )
            .context("parsing declassification obligation kinds")?;
            if kinds.is_empty() {
                continue;
            }
            let approval: PolicyPropagationBarrier<ApprovedBarrier> =
                PolicyPropagationBarrier::restore_approved(
                    row.try_get::<uuid::Uuid, _>("barrier_id")?,
                    PolicyPropagationBarrierKind::parse(
                        row.try_get::<String, _>("barrier_kind")?.as_str(),
                    )
                    .context("parsing declassification barrier kind")?,
                    policy_model::LineagePathRef::parse(row.try_get::<String, _>("path")?)
                        .context("parsing declassification path")?,
                    PolicyActor::parse(row.try_get::<String, _>("actor")?)
                        .context("parsing declassification actor")?,
                    PolicyReason::parse(row.try_get::<String, _>("reason")?)
                        .context("parsing declassification reason")?,
                    row.try_get("created_at")?,
                    row.try_get("approved_at")?,
                    row.try_get("expires_at")?,
                );
            by_path.entry(approval.path.to_string()).or_default().push(
                DeclassificationBarrier::new(approval, kinds)
                    .context("building scoped declassification barrier")?,
            );
        }

        edges
            .into_iter()
            .map(|edge| {
                let path = edge.path_ref()?.to_string();
                Ok(by_path
                    .get(&path)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .fold(edge, governance_policy_component::LineageEdge::with_barrier))
            })
            .collect()
    }
```

### 48. Module database_tests
`rust:component/orchestration/workflows/src/impls/read_repo.rs::database_tests`  component/orchestration/workflows/src/impls/read_repo.rs:745-817  pkg:cargo/workflows-component  score 0.68  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` <- DEFINES (component/orchestration/workflows/src/impls/read_repo.rs:752-816, 1.00)  
`mod database_tests`  
Against a workspace the testkit migrates in an ephemeral Postgres (doc continues, 216 bytes)  
members 1  

### 49. Test tests::the_base_schema_is_derived_from_the_workspace_id
`rust:jobs/causeway-migrate/src/main.rs::tests::the_base_schema_is_derived_from_the_workspace_id`  jobs/causeway-migrate/src/main.rs:233-245  pkg:cargo/causeway-migrate  score 0.67  via `base_schema_of` -> TESTED_BY (jobs/causeway-migrate/src/main.rs:241, 1.00)  
`fn the_base_schema_is_derived_from_the_workspace_id()`  
calls 2: `base_schema_of` (jobs/causeway-migrate/src/main.rs:241), `tests::scope` (jobs/causeway-migrate/src/main.rs:241)  
tests_for 2: `base_schema_of` (jobs/causeway-migrate/src/main.rs:241), `tests::scope` (jobs/causeway-migrate/src/main.rs:241)  
jobs/causeway-migrate/src/main.rs:233-245
```rust
    fn the_base_schema_is_derived_from_the_workspace_id() {
        for (id, schema) in [
            ("prod", "cw_prod"),
            ("cloud-local", "cw_cloud_local"),
            ("feat-x", "cw_feat_x"),
            ("local", "cw_local"),
        ] {
            assert_eq!(
                base_schema_of(&scope(id), None).expect("derives"),
                (schema.to_owned(), None)
            );
        }
    }
```

### 50. Variant Companion::PgSchema
`rust:torrentfx/crates/causeway-workspace-scope/src/companion.rs::Companion::PgSchema`  torrentfx/crates/causeway-workspace-scope/src/companion.rs:25  pkg:cargo/causeway-workspace-scope  score 0.61  via `base_schema_of` -> REFERENCES (jobs/causeway-migrate/src/main.rs:153, 1.00)  
`PgSchema`  
`CAUSEWAY_PG_SCHEMA`, the base schema.  
referenced_by 13: `tests::a_companion_that_repeats_the_derived_name_warns` (torrentfx/crates/causeway-platform/src/alias.rs:932), `tests::a_companion_that_disagrees_stops_startup` (torrentfx/crates/causeway-platform/src/alias.rs:955), `workspace_scope` (crates/causeway-db-core/src/lib.rs:46) … 10 more  

### 51. File crates/causeway-testkit/src/workspace.rs
`file:crates/causeway-testkit/src/workspace.rs`  crates/causeway-testkit/src/workspace.rs  pkg:cargo/causeway-testkit  score 0.61  via `TestWorkspace#impl` <- CONTAINS (crates/causeway-testkit/src/workspace.rs:51-292, 1.00)  
members 11  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:974)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2400)  
imports 20: `connect_when_ready` (crates/causeway-testkit/src/workspace.rs:15), `QdrantWorkspace` (crates/causeway-testkit/src/workspace.rs:16), `cargo/sqlx` (crates/causeway-testkit/src/workspace.rs:9) … 17 more  

### 52. File crates/causeway-db-core/migrations/0001_initial_schema.sql
`file:crates/causeway-db-core/migrations/0001_initial_schema.sql`  crates/causeway-db-core/migrations/0001_initial_schema.sql  pkg:cargo/causeway-db-core  score 0.60  via `public.access_grants` <- MIGRATES (crates/causeway-db-core/migrations/0001_initial_schema.sql:53-62, 1.00)  
designs 1: `docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md` (docs/rfd/RFD-2n2tcfe2-operational-resilience-and-the-stuck-work-surface.md:240)  
documents 1: `docs/METADATA-INGESTION-PATH.md` (docs/METADATA-INGESTION-PATH.md:234)  
migrates 52: `public.canonical_contract_controls` (crates/causeway-db-core/migrations/0001_initial_schema.sql:382-394), `public.agent_messages` (crates/causeway-db-core/migrations/0001_initial_schema.sql:103-109), `public.governance_policies` (crates/causeway-db-core/migrations/0001_initial_schema.sql:29-39) … 49 more  
reads 1: `public.governance_policies` (crates/causeway-db-core/migrations/0001_initial_schema.sql:1411-1428)  

### 53. File crates/causeway-db-core/legacy-migrations/0004_governance.sql
`file:crates/causeway-db-core/legacy-migrations/0004_governance.sql`  crates/causeway-db-core/legacy-migrations/0004_governance.sql  pkg:cargo/causeway-db-core  score 0.55  via `public.access_grants` <- MIGRATES (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43, 1.00)  
migrates 4: `public.access_grants` (crates/causeway-db-core/legacy-migrations/0004_governance.sql:34-43), `public.data_contracts` (crates/causeway-db-core/legacy-migrations/0004_governance.sql:22-32), `public.governance_policies` (crates/causeway-db-core/legacy-migrations/0004_governance.sql:12-20) … 1 more  

### 54. Struct PolicyPropagationBarrier
`rust:crates/policy-model/src/lib.rs::PolicyPropagationBarrier`  crates/policy-model/src/lib.rs:701-712  pkg:cargo/policy-model  score 0.54  via `PolicyScanner::attach_declassification_barriers` -> USES_TYPE (apps/causeway-policy-scan/src/main.rs:1163, 1.00)  
`pub struct PolicyPropagationBarrier<State>`  
Typestate policy propagation barrier. (doc continues, 280 bytes)  
members 4  
importers 7: `tests` (component/governance/core/src/policy_barrier_repo.rs:225-228), `component/governance/policy/tests/phase4.rs` (component/governance/policy/tests/phase4.rs:27-33), `crates/causeway-metadata-ops/src/manual_actions.rs` (crates/causeway-metadata-ops/src/manual_actions.rs:13-16) … 4 more  
used_by 18: `policy_barrier_from_action` (crates/causeway-metadata-ops/src/manual_actions.rs:507), `DeclassificationBarrier` (component/governance/policy/src/materialize.rs:50), `MockPolicyBarrierRepo` (component/governance/core/src/policy_barrier_repo.rs:150) … 15 more  
uses_types 1: `PolicyPropagationBarrierKind` (crates/policy-model/src/lib.rs:703)  

### 55. Test tests::a_schema_alias_that_repeats_the_derived_schema_warns
`rust:jobs/causeway-migrate/src/main.rs::tests::a_schema_alias_that_repeats_the_derived_schema_warns`  jobs/causeway-migrate/src/main.rs:248-259  pkg:cargo/causeway-migrate  score 0.53  via `base_schema_of` -> TESTED_BY (jobs/causeway-migrate/src/main.rs:250, 1.00)  
`fn a_schema_alias_that_repeats_the_derived_schema_warns()`  
calls 2: `tests::scope` (jobs/causeway-migrate/src/main.rs:250), `base_schema_of` (jobs/causeway-migrate/src/main.rs:250)  
tests_for 2: `base_schema_of` (jobs/causeway-migrate/src/main.rs:250), `tests::scope` (jobs/causeway-migrate/src/main.rs:250)  

### 56. Test tests::a_schema_alias_that_names_another_schema_stops_the_job
`rust:jobs/causeway-migrate/src/main.rs::tests::a_schema_alias_that_names_another_schema_stops_the_job`  jobs/causeway-migrate/src/main.rs:262-269  pkg:cargo/causeway-migrate  score 0.53  via `base_schema_of` -> TESTED_BY (jobs/causeway-migrate/src/main.rs:263, 1.00)  
`fn a_schema_alias_that_names_another_schema_stops_the_job()`  
calls 2: `tests::scope` (jobs/causeway-migrate/src/main.rs:263), `base_schema_of` (jobs/causeway-migrate/src/main.rs:263)  
tests_for 2: `tests::scope` (jobs/causeway-migrate/src/main.rs:263), `base_schema_of` (jobs/causeway-migrate/src/main.rs:263)  

### 57. Method TestWorkspace::prepare
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::prepare`  crates/causeway-testkit/src/workspace.rs:78-140  pkg:cargo/causeway-testkit  score 0.53  via `repository_targets` <- CALLS (crates/causeway-testkit/src/workspace.rs:85, 1.00)  
`async fn prepare( containers: &Containers, repo_root: &Path, scope: WorkspaceScope, ) -> TestkitResult<Self>`  
[`Self::new`] over the migrations under `repo_root`, for `scope`. (doc continues, 269 bytes)  
callers 2: `TestWorkspace::new` (crates/causeway-testkit/src/workspace.rs:70), `tests::a_workspace_that_fails_partway_drops_what_it_created` (crates/causeway-testkit/src/workspace.rs:491)  
calls 15: `check_identifier_lengths` (crates/causeway-testkit/src/workspace.rs:90), `migrate_workspace` (crates/causeway-testkit/src/workspace.rs:95), `repository_targets` (crates/causeway-testkit/src/workspace.rs:85) … 12 more  
references 1: `MigrationMode::Apply` (crates/causeway-testkit/src/workspace.rs:99)  
tests 1: `tests::a_workspace_that_fails_partway_drops_what_it_created` (crates/causeway-testkit/src/workspace.rs:491)  
uses_types 4: `TestkitResult` (crates/causeway-testkit/src/workspace.rs:82), `Containers` (crates/causeway-testkit/src/workspace.rs:79), `TestkitError` (crates/causeway-testkit/src/workspace.rs:107) … 1 more  

### 58. Function underscored
`rust:crates/causeway-db-core/src/workspace_migrations.rs::underscored`  crates/causeway-db-core/src/workspace_migrations.rs:288-295  pkg:cargo/causeway-db-core  score 0.53  via `repository_targets` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:253, 1.00)  
`fn underscored(path: &Path) -> WorkspaceMigrationResult<String>`  
callers 2: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:253), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:253)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:288)  

### 59. Function subdirectories
`rust:crates/causeway-db-core/src/workspace_migrations.rs::subdirectories`  crates/causeway-db-core/src/workspace_migrations.rs:273-286  pkg:cargo/causeway-db-core  score 0.53  via `repository_targets` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:250, 1.00)  
`fn subdirectories(path: &Path) -> WorkspaceMigrationResult<Vec<PathBuf>>`  
callers 2: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:249), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:250)  
calls 3: `read_dir` (crates/causeway-db-core/src/workspace_migrations.rs:275), `PrincipalsDirectory::path` (crates/causeway-db-core/src/workspace_migrations.rs:282), `tests::Mount::path` (crates/causeway-db-core/src/workspace_migrations.rs:282)  
references 2: `WorkspaceMigrationError::DirEntry` (crates/causeway-db-core/src/workspace_migrations.rs:279), `WorkspaceMigrationError::DirEntry` (crates/causeway-db-core/src/workspace_migrations.rs:276)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:273)  

### 60. Struct ApprovedBarrier
`rust:crates/policy-model/src/lib.rs::ApprovedBarrier`  crates/policy-model/src/lib.rs:293  pkg:cargo/policy-model  score 0.52  via `PolicyScanner::attach_declassification_barriers` -> USES_TYPE (apps/causeway-policy-scan/src/main.rs:1163, 1.00)  
`pub struct ApprovedBarrier;`  
Approved policy propagation barrier state.  
importers 4: `component/governance/policy/src/materialize.rs` (component/governance/policy/src/materialize.rs:8-11), `component/governance/core/src/policy_barrier_repo.rs` (component/governance/core/src/policy_barrier_repo.rs:9-12), `component/governance/policy/tests/phase4.rs` (component/governance/policy/tests/phase4.rs:27-33) … 1 more  
used_by 16: `PolicyPropagationBarrier#impl#2` (crates/policy-model/src/lib.rs:761), `PgPolicyBarrierRepo::effective_barriers_for_path` (component/governance/core/src/policy_barrier_repo.rs:125), `PgPolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:56) … 13 more  
crates/policy-model/src/lib.rs:293-293
```rust
pub struct ApprovedBarrier;
```

### 61. Function migrate
`rust:crates/causeway-testkit/src/infra.rs::migrate`  crates/causeway-testkit/src/infra.rs:270-299  pkg:cargo/causeway-testkit  score 0.52  via `repository_targets` <- CALLS (crates/causeway-testkit/src/infra.rs:274, 1.00)  
`async fn migrate(postgres: &EphemeralPostgres, scope: &WorkspaceScope) -> TestkitResult<String>`  
Applies the baseline and every component's migrations into the (doc continues, 121 bytes)  
callers 1: `InfraPlan::start` (crates/causeway-testkit/src/infra.rs:213)  
calls 13: `migrate_workspace` (crates/causeway-testkit/src/infra.rs:281), `repository_targets` (crates/causeway-testkit/src/infra.rs:274), `repository_root` (crates/causeway-testkit/src/infra.rs:274) … 10 more  
references 1: `MigrationMode::Apply` (crates/causeway-testkit/src/infra.rs:285)  
uses_types 3: `TestkitResult` (crates/causeway-testkit/src/infra.rs:270), `EphemeralPostgres` (crates/causeway-testkit/src/infra.rs:270), `WorkspaceScope` (crates/causeway-testkit/src/infra.rs:270)  

### 62. Method MigrationTarget::baseline
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget::baseline`  crates/causeway-db-core/src/workspace_migrations.rs:162-168  pkg:cargo/causeway-db-core  score 0.51  via `repository_targets` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:246, 1.00)  
`fn baseline(base_schema: &str, migrations: PathBuf) -> Self`  
callers 2: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:246), `staged_targets` (crates/causeway-db-core/src/workspace_migrations.rs:196)  
crates/causeway-db-core/src/workspace_migrations.rs:162-168
```rust
    fn baseline(base_schema: &str, migrations: PathBuf) -> Self {
        Self {
            schema: base_schema.to_owned(),
            migrations,
            ignore_missing: true,
        }
    }
```

### 63. Test two_workspaces_share_the_server_and_nothing_else
`rust:crates/causeway-testkit/tests/postgres.rs::two_workspaces_share_the_server_and_nothing_else`  crates/causeway-testkit/tests/postgres.rs:144-152  pkg:cargo/causeway-testkit  score 0.51  via `TestWorkspace::schemas` -> TESTED_BY (crates/causeway-testkit/tests/postgres.rs:147, 0.75)  
`async fn two_workspaces_share_the_server_and_nothing_else()`  
calls 10: `workspace` (crates/causeway-testkit/tests/postgres.rs:145), `workspace` (crates/causeway-testkit/tests/postgres.rs:145), `TestWorkspace::schemas` (crates/causeway-testkit/tests/postgres.rs:147) … 7 more  
tests_for 5: `workspace` (crates/causeway-testkit/tests/postgres.rs:145), `TestWorkspace::schemas` (crates/causeway-testkit/tests/postgres.rs:147), `WorkspaceProcess::database_url` (crates/causeway-testkit/tests/postgres.rs:151) … 2 more  

### 64. Method MigrationTarget::component
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget::component`  crates/causeway-db-core/src/workspace_migrations.rs:170-176  pkg:cargo/causeway-db-core  score 0.51  via `repository_targets` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:261, 1.00)  
`fn component(base_schema: &str, suffix: &str, migrations: PathBuf) -> Self`  
callers 2: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:261), `staged_targets` (crates/causeway-db-core/src/workspace_migrations.rs:220)  

### 65. Struct WorkspaceScope
`rust:torrentfx/crates/causeway-workspace-scope/src/lib.rs::WorkspaceScope`  torrentfx/crates/causeway-workspace-scope/src/lib.rs:119  pkg:cargo/causeway-workspace-scope  score 0.50  via `base_schema` -> USES_TYPE (jobs/causeway-migrate/src/main.rs:115, 1.00)  
`pub struct WorkspaceScope`  
A validated workspace id and the names derived from it. (doc continues, 199 bytes)  
members 19  
importers 16: `torrentfx/crates/causeway-workspace-scope/src/companion.rs` (torrentfx/crates/causeway-workspace-scope/src/companion.rs:19), `crates/causeway-testkit/src/isolation.rs` (crates/causeway-testkit/src/isolation.rs:18), `crates/causeway-testkit/src/valkey.rs` (crates/causeway-testkit/src/valkey.rs:11) … 13 more  
used_by 94: `String::from` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:325), `SessionKeyspace::for_workspace` (apps/causeway-server/src/cache.rs:237), `term_governance_resolution_service_name_for` (component/governance/resolution/src/commands.rs:295) … 91 more  

### 66. Struct MigrationTarget
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget`  crates/causeway-db-core/src/workspace_migrations.rs:134-138  pkg:cargo/causeway-db-core  score 0.49  via `repository_targets` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:241, 1.00)  
`pub struct MigrationTarget`  
One schema and the migration set that belongs in it.  
members 5  
used_by 7: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:241), `MigrationTarget#impl` (crates/causeway-db-core/src/workspace_migrations.rs:140-177), `staged_targets` (crates/causeway-db-core/src/workspace_migrations.rs:191) … 4 more  

### 67. File component/orchestration/workflows/src/impls/read_repo.rs
`file:component/orchestration/workflows/src/impls/read_repo.rs`  component/orchestration/workflows/src/impls/read_repo.rs  pkg:cargo/workflows-component  score 0.48  via `database_tests` <- CONTAINS (component/orchestration/workflows/src/impls/read_repo.rs:745-817, 1.00)  
members 38  
imports 30: `TasksCommand` (component/orchestration/workflows/src/impls/read_repo.rs:28-31), `BoxFuture` (component/orchestration/workflows/src/impls/read_repo.rs:22-25), `WorkflowHealthError` (component/orchestration/workflows/src/impls/read_repo.rs:22-25) … 27 more  

### 68. Function workflows_schema
`rust:component/orchestration/workflows/src/impls/read_repo.rs::workflows_schema`  component/orchestration/workflows/src/impls/read_repo.rs:43-45  pkg:cargo/workflows-component  score 0.48  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> CALLS (component/orchestration/workflows/src/impls/read_repo.rs:760, 1.00)  
`fn workflows_schema() -> String`  
The schema this slice's tables live in. (doc continues, 370 bytes)  
callers 12: `PgWorkflowReadRepo::workflow_health_counts` (component/orchestration/workflows/src/impls/read_repo.rs:258), `PgWorkflowReadRepo::open_workflow_tasks` (component/orchestration/workflows/src/impls/read_repo.rs:282), `outbound_signals` (component/orchestration/workflows/src/impls/read_repo.rs:477) … 9 more  
calls 1: `actual_schema_name` (component/orchestration/workflows/src/impls/read_repo.rs:44)  
tests 1: `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` (component/orchestration/workflows/src/impls/read_repo.rs:760)  

### 69. Test the_migrations_create_their_own_extensions_in_a_fresh_database
`rust:crates/causeway-testkit/tests/postgres.rs::the_migrations_create_their_own_extensions_in_a_fresh_database`  crates/causeway-testkit/tests/postgres.rs:207-255  pkg:cargo/causeway-testkit  score 0.46  via `repository_targets` -> TESTED_BY (crates/causeway-testkit/tests/postgres.rs:222, 1.00)  
`async fn the_migrations_create_their_own_extensions_in_a_fresh_database()`  
The migrations create the extensions they need on a database that has (doc continues, 167 bytes)  
members 1  
calls 8: `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:208), `repository_targets` (crates/causeway-testkit/tests/postgres.rs:222), `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:227) … 5 more  
references 1: `MigrationMode::Apply` (crates/causeway-testkit/tests/postgres.rs:231)  
tests_for 7: `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:208), `repository_targets` (crates/causeway-testkit/tests/postgres.rs:222), `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:227) … 4 more  

### 70. TypeAlias WorkspaceMigrationResult
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationResult`  crates/causeway-db-core/src/workspace_migrations.rs:130  pkg:cargo/causeway-db-core  score 0.46  via `repository_targets` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:241, 1.00)  
`pub type WorkspaceMigrationResult<T> = std::result::Result<T, WorkspaceMigrationError>;`  
Result of a workspace migration step.  
used_by 11: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:241), `read_dir` (crates/causeway-db-core/src/workspace_migrations.rs:266), `connect` (crates/causeway-db-core/src/workspace_migrations.rs:427) … 8 more  
uses_types 1: `WorkspaceMigrationError` (crates/causeway-db-core/src/workspace_migrations.rs:130)  

### 71. Function main
`rust:jobs/causeway-migrate/src/main.rs::main`  jobs/causeway-migrate/src/main.rs:37-104  pkg:cargo/causeway-migrate  score 0.46  via `base_schema` <- CALLS (jobs/causeway-migrate/src/main.rs:51, 1.00)  
`async fn main() -> Result<()>`  
calls 34: `database_url` (jobs/causeway-migrate/src/main.rs:49), `ResolvedEnvironment::resolve` (jobs/causeway-migrate/src/main.rs:46), `ResolvedConfig::from_resolved` (jobs/causeway-migrate/src/main.rs:47) … 31 more  
references 6: `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:41), `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:67), `MigrationMode::Apply` (jobs/causeway-migrate/src/main.rs:73) … 3 more  

### 72. Test tests::the_repository_layout_names_schemas_as_the_staging_script_does
`rust:crates/causeway-db-core/src/workspace_migrations.rs::tests::the_repository_layout_names_schemas_as_the_staging_script_does`  crates/causeway-db-core/src/workspace_migrations.rs:562-590  pkg:cargo/causeway-db-core  score 0.45  via `repository_targets` -> TESTED_BY (crates/causeway-db-core/src/workspace_migrations.rs:575, 1.00)  
`fn the_repository_layout_names_schemas_as_the_staging_script_does()`  
calls 23: `tests::dir` (crates/causeway-db-core/src/workspace_migrations.rs:565), `tests::dir` (crates/causeway-db-core/src/workspace_migrations.rs:568), `MigrationTarget::migrations` (crates/causeway-db-core/src/workspace_migrations.rs:586) … 20 more  
tests_for 7: `MigrationTarget::migrations` (crates/causeway-db-core/src/workspace_migrations.rs:586), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:575), `tests::dir` (crates/causeway-db-core/src/workspace_migrations.rs:564) … 4 more  

### 73. Test tests::a_workspace_id_is_valid_and_leaves_room_for_every_schema
`rust:crates/causeway-testkit/src/workspace.rs::tests::a_workspace_id_is_valid_and_leaves_room_for_every_schema`  crates/causeway-testkit/src/workspace.rs:416-431  pkg:cargo/causeway-testkit  score 0.45  via `repository_targets` -> TESTED_BY (crates/causeway-testkit/src/workspace.rs:420, 1.00)  
`fn a_workspace_id_is_valid_and_leaves_room_for_every_schema()`  
calls 9: `repository_root` (crates/causeway-testkit/src/workspace.rs:421), `repository_targets` (crates/causeway-testkit/src/workspace.rs:420), `check_identifier_lengths` (crates/causeway-testkit/src/workspace.rs:429) … 6 more  
tests_for 8: `check_identifier_lengths` (crates/causeway-testkit/src/workspace.rs:429), `repository_root` (crates/causeway-testkit/src/workspace.rs:421), `WorkspaceScope::parse` (crates/causeway-testkit/src/workspace.rs:419) … 5 more  

### 74. Test tests::the_repository_targets_cover_every_component_migration_directory
`rust:crates/causeway-db-core/src/workspace_migrations.rs::tests::the_repository_targets_cover_every_component_migration_directory`  crates/causeway-db-core/src/workspace_migrations.rs:596-617  pkg:cargo/causeway-db-core  score 0.45  via `repository_targets` -> TESTED_BY (crates/causeway-db-core/src/workspace_migrations.rs:598, 1.00)  
`fn the_repository_targets_cover_every_component_migration_directory()`  
The repository's own sets, found the way the testkit finds them, (doc continues, 176 bytes)  
calls 8: `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:613), `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:615), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:598) … 5 more  
tests_for 4: `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:613), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:598), `PrincipalsDirectory::path` (crates/causeway-db-core/src/workspace_migrations.rs:601) … 1 more  

### 75. Test tests::the_migrations_create_only_extensions_the_upstream_image_ships
`rust:crates/causeway-testkit/src/parity.rs::tests::the_migrations_create_only_extensions_the_upstream_image_ships`  crates/causeway-testkit/src/parity.rs:252-280  pkg:cargo/causeway-testkit  score 0.45  via `repository_targets` -> TESTED_BY (crates/causeway-testkit/src/parity.rs:253, 1.00)  
`fn the_migrations_create_only_extensions_the_upstream_image_ships()`  
Every extension a migration creates is one the upstream image (doc continues, 157 bytes)  
calls 7: `repository_root` (crates/causeway-testkit/src/parity.rs:253), `tests::read` (crates/causeway-testkit/src/parity.rs:259), `created_extensions` (crates/causeway-testkit/src/parity.rs:259) … 4 more  
tests_for 7: `repository_root` (crates/causeway-testkit/src/parity.rs:253), `repository_targets` (crates/causeway-testkit/src/parity.rs:253), `tests::read` (crates/causeway-testkit/src/parity.rs:259) … 4 more  

### 76. Impl PgPolicyBarrierRepo#impl(PolicyBarrierRepo)
`rust:component/governance/core/src/policy_barrier_repo.rs::PgPolicyBarrierRepo#impl(PolicyBarrierRepo)`  component/governance/core/src/policy_barrier_repo.rs:53-145  pkg:cargo/governance-component  score 0.44  via `PgPolicyBarrierRepo::effective_barriers_for_path` <- DEFINES (component/governance/core/src/policy_barrier_repo.rs:121-144, 1.00)  
`impl PolicyBarrierRepo for PgPolicyBarrierRepo`  
members 2  
implements 1: `PolicyBarrierRepo` (component/governance/core/src/policy_barrier_repo.rs:53)  
uses_types 2: `PgPolicyBarrierRepo` (component/governance/core/src/policy_barrier_repo.rs:53-145), `PolicyBarrierRepo` (component/governance/core/src/policy_barrier_repo.rs:53)  

### 77. Method PolicyBarrierRepo::insert_approved_barrier
`rust:component/governance/core/src/policy_barrier_repo.rs::PolicyBarrierRepo::insert_approved_barrier`  component/governance/core/src/policy_barrier_repo.rs:24-28  pkg:cargo/governance-component  score 0.43  via `PgPolicyBarrierRepo::insert_approved_barrier` <- DISPATCHES_TO (component/governance/core/src/policy_barrier_repo.rs:54-119, 1.00)  
`fn insert_approved_barrier( &self, barrier: &PolicyPropagationBarrier<ApprovedBarrier>, event: &PolicyBarrierEvent, ) -> BoxFuture<'_, Result<(), DbError>>;`  
callers 3: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:250), `activate_manual_action` (crates/causeway-metadata-ops/src/manual_actions.rs:325), `tests::approved_barrier_stops_matching_path_only` (component/governance/core/src/policy_propagation.rs:167)  
dispatches_to 2: `PgPolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:54-119), `MockPolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:160-178)  
tests 2: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:250), `tests::approved_barrier_stops_matching_path_only` (component/governance/core/src/policy_propagation.rs:167)  
uses_types 5: `DbError` (component/governance/core/src/policy_barrier_repo.rs:28), `ApprovedBarrier` (component/governance/core/src/policy_barrier_repo.rs:26), `PolicyBarrierEvent` (component/governance/core/src/policy_barrier_repo.rs:27) … 2 more  

### 78. Method PolicyBarrierRepo::effective_barriers_for_path
`rust:component/governance/core/src/policy_barrier_repo.rs::PolicyBarrierRepo::effective_barriers_for_path`  component/governance/core/src/policy_barrier_repo.rs:30-34  pkg:cargo/governance-component  score 0.43  via `PgPolicyBarrierRepo::effective_barriers_for_path` <- DISPATCHES_TO (component/governance/core/src/policy_barrier_repo.rs:121-144, 1.00)  
`fn effective_barriers_for_path( &self, path: &LineagePathRef, now: DateTime<Utc>, ) -> BoxFuture<'_, Result<Vec<PolicyPropagationBarrier<ApprovedBarrier>>, DbError>>;`  
callers 3: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:262), `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:255), `evaluate_lineage_policy_propagation` (component/governance/core/src/policy_propagation.rs:51)  
dispatches_to 2: `MockPolicyBarrierRepo::effective_barriers_for_path` (component/governance/core/src/policy_barrier_repo.rs:180-196), `PgPolicyBarrierRepo::effective_barriers_for_path` (component/governance/core/src/policy_barrier_repo.rs:121-144)  
tests 1: `tests::mock_repo_filters_effective_barriers_by_path_and_time` (component/governance/core/src/policy_barrier_repo.rs:255)  
uses_types 4: `BoxFuture` (component/governance/core/src/policy_barrier_repo.rs:34), `DbError` (component/governance/core/src/policy_barrier_repo.rs:34), `PolicyPropagationBarrier` (component/governance/core/src/policy_barrier_repo.rs:34) … 1 more  

### 79. Variant MigrationMode::Check
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode::Check`  crates/causeway-db-core/src/workspace_migrations.rs:302  pkg:cargo/causeway-db-core  score 0.42  via `main` -> REFERENCES (jobs/causeway-migrate/src/main.rs:83, 1.00)  
`Check`  
Count pending migrations per schema and change nothing.  
referenced_by 5: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:98), `main` (jobs/causeway-migrate/src/main.rs:41), `main` (jobs/causeway-migrate/src/main.rs:67) … 2 more  

### 80. Enum DbError
`rust:crates/causeway-db-core/src/error.rs::DbError`  crates/causeway-db-core/src/error.rs:4-47  pkg:cargo/causeway-db-core  score 0.41  via `PgPolicyBarrierRepo::effective_barriers_for_path` -> USES_TYPE (component/governance/core/src/policy_barrier_repo.rs:125, 1.00)  
`pub enum DbError`  
Shared database error types for slice-owned persistence.  
members 16  
importers 70: `component/people/core/src/repo.rs` (component/people/core/src/repo.rs:10), `component/catalog/core/src/maintenance.rs` (component/catalog/core/src/maintenance.rs:4), `component/metadata/core/src/write.rs` (component/metadata/core/src/write.rs:11) … 67 more  
referenced_by 1: `CatalogPublicationCommands::approve` (apps/causeway-metadata-admin-host/src/approval.rs:230)  
used_by 1136: `PlacementRow::into_placement` (component/catalog/core/src/placements.rs:753), `PgMetadataRepo::pg_quarantine_materialization_work` (component/metadata/core/src/queue.rs:576), `PgMetadataRepo::pg_supersede_active_claims_except` (component/metadata/core/src/ingestion.rs:379) … 1133 more  

### 81. File component/governance/core/src/rights.rs
`file:component/governance/core/src/rights.rs`  component/governance/core/src/rights.rs  pkg:cargo/governance-component  score 0.41  via `count` <- CONTAINS (component/governance/core/src/rights.rs:1429-1431, 1.00)  
members 69  
designs 1: `docs/rfd/RFD-2x49z609-odrl-rights-policies.md` (docs/rfd/RFD-2x49z609-odrl-rights-policies.md:328)  
documents 2: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:532), `docs/plans/ODRL-RIGHTS-WORKPLAN.md` (docs/plans/ODRL-RIGHTS-WORKPLAN.md:79)  
imports 24: `EntityMatchReport` (component/governance/core/src/rights.rs:21-27), `ExportVariant` (component/governance/core/src/rights.rs:21-27), `Imported` (component/governance/core/src/rights.rs:21-27) … 21 more  

### 82. Macro test_name
`rust:crates/causeway-testkit/src/lib.rs::test_name`  crates/causeway-testkit/src/lib.rs:120-125  pkg:cargo/causeway-testkit  score 0.41  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> CALLS (component/orchestration/workflows/src/impls/read_repo.rs:753, 1.00)  
`macro_rules! test_name { () => {{ fn __causeway_testkit_here() {} $crate::__private::test_name_of(::std::any::type_name_of_val(&__causeway_testkit_here)) }}; }`  
The libtest name of the test this expands in, for [`in_workspace`]: (doc continues, 121 bytes)  
callers 72: `event_bus_recovery_is_hermetic` (crates/causeway-metadata-ops/tests/event_bus_recovery_acceptance.rs:65), `a_test_body_runs_in_a_process_scoped_to_its_workspace` (crates/causeway-testkit/tests/postgres.rs:261), `tests::postgres_superseded_alias_rebinds_without_deleting_history` (component/semantic/graph/src/lib.rs:2675) … 69 more  
importers 7: `crates/causeway-testkit/tests/qdrant.rs` (crates/causeway-testkit/tests/qdrant.rs:13-16), `crates/causeway-testkit/tests/object_store.rs` (crates/causeway-testkit/tests/object_store.rs:11-14), `crates/causeway-metadata-ops/tests/event_bus_recovery_acceptance.rs` (crates/causeway-metadata-ops/tests/event_bus_recovery_acceptance.rs:23-25) … 4 more  
tests 70: `a_test_body_runs_in_a_process_scoped_to_its_workspace` (crates/causeway-testkit/tests/postgres.rs:261), `tests::a_persona_token_approves_a_publication_and_the_approval_names_a_party` (apps/causeway-metadata-admin-host/src/access.rs:1325), `tests::policy_writer_registers_real_semantic_identities` (apps/causeway-policy-scan/src/main.rs:2290) … 67 more  

### 83. Function actual_schema_name
`rust:crates/causeway-db-core/src/lib.rs::actual_schema_name`  crates/causeway-db-core/src/lib.rs:72-74  pkg:cargo/causeway-db-core  score 0.41  via `PolicyScanner::attach_declassification_barriers` -> CALLS (apps/causeway-policy-scan/src/main.rs:1141, 1.00)  
`pub fn actual_schema_name(base_name: &str) -> String`  
Returns the schema a component owns in this process's workspace: (doc continues, 506 bytes)  
callers 323: `Schemas::for_workspace` (component/platform/coherence/src/run.rs:46), `load_name_matches` (apps/causeway-policy-scan/src/candidate_implementations.rs:55), `mark_outbox_event_failed` (crates/causeway-metadata-ops/src/event_outbox.rs:917) … 320 more  
calls 2: `scope_or_panic` (crates/causeway-db-core/src/lib.rs:73), `WorkspaceScope::component_schema` (crates/causeway-db-core/src/lib.rs:73)  
importers 41: `apps/causeway-policy-scan/src/implementation_governance.rs` (apps/causeway-policy-scan/src/implementation_governance.rs:15), `database_tests` (component/catalog/core/src/projection.rs:2198), `component/catalog/core/src/projection.rs` (component/catalog/core/src/projection.rs:16) … 38 more  
tests 28: `database_tests::observations_open_refresh_move_and_reidentify_bindings` (component/catalog/core/src/bindings.rs:1421), `database_tests::search_resource_identity_projection_maps_only_published_catalog_rows` (component/catalog/core/src/search_queue.rs:614), `tests::an_unset_id_is_the_local_workspace_in_local` (crates/causeway-db-core/src/lib.rs:360) … 25 more  

### 84. Function in_workspace
`rust:crates/causeway-testkit/src/isolation.rs::in_workspace`  crates/causeway-testkit/src/isolation.rs:317-319  pkg:cargo/causeway-testkit  score 0.40  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> CALLS (component/orchestration/workflows/src/impls/read_repo.rs:753, 1.00)  
`pub async fn in_workspace(test: impl AsRef<str>) -> TestkitResult<Option<WorkspaceProcess>>`  
Runs the calling test in a workspace of its own, with Postgres alone: (doc continues, 150 bytes)  
callers 63: `database_tests::postgres_feedback_commands_complete_one_round_trip` (component/discovery/search-feedback/src/lib.rs:971), `tests::postgres_generation_publishes_cross_domain_traversal` (component/semantic/graph/src/projection_source.rs:3298), `database_tests::a_finding_is_listed_reviewed_and_re_asked` (component/platform/coherence/src/source.rs:498) … 60 more  
calls 2: `WorkspaceSpec::run` (crates/causeway-testkit/src/isolation.rs:318), `WorkspaceSpec::new` (crates/causeway-testkit/src/isolation.rs:318)  
importers 2: `crates/causeway-testkit/tests/postgres.rs` (crates/causeway-testkit/tests/postgres.rs:10-13), `crates/causeway-testkit/src/lib.rs` (crates/causeway-testkit/src/lib.rs:73-76)  
tests 62: `database_tests::a_quarantined_row_reads_and_acts_through_the_port` (component/metadata/core/src/stuck_work.rs:722), `tests::postgres_registry_isolates_aliases_and_reads_by_workspace` (component/semantic/graph/src/lib.rs:2456), `database_tests::a_second_sweep_steps_aside_rather_than_queueing` (component/platform/supervision/src/store.rs:311) … 59 more  
uses_types 2: `TestkitResult` (crates/causeway-testkit/src/isolation.rs:317), `WorkspaceProcess` (crates/causeway-testkit/src/isolation.rs:317)  

### 85. Table public.governance_policies
`sql:public.governance_policies`  crates/causeway-db-core/legacy-migrations/0004_governance.sql:12-20  pkg:cargo/causeway-db-core  score 0.40  via `crates/causeway-db-core/legacy-migrations/0004_governance.sql` -> MIGRATES (crates/causeway-db-core/legacy-migrations/0004_governance.sql:12-20, 1.00)  
declared in 5 places: crates/causeway-db-core/legacy-migrations/0004_governance.sql:12-20, crates/causeway-db-core/legacy-migrations/0006_governance_seed.sql:10-15, crates/causeway-db-core/legacy-migrations/0021_test_seed.sql:31, crates/causeway-db-core/migrations/0001_initial_schema.sql:29-39 and 1 more  
`CREATE TABLE IF NOT EXISTS governance_policies (`  
members 9  
migrations 6: `crates/causeway-db-core/migrations/0001_initial_schema.sql` (crates/causeway-db-core/migrations/0001_initial_schema.sql:29-39), `crates/causeway-db-core/migrations/0009_policy_control_plane.sql` (crates/causeway-db-core/migrations/0009_policy_control_plane.sql:1-7), `crates/causeway-db-core/legacy-migrations/0004_governance.sql` (crates/causeway-db-core/legacy-migrations/0004_governance.sql:12-20) … 3 more  
readers 25: `identity_tests::database_tests::policy_write_rolls_back_when_identity_registration_fails::query#766fa181` (component/governance/policy/src/repo.rs:2416), `crates/causeway-db-core/migrations/0001_initial_schema.sql` (crates/causeway-db-core/migrations/0001_initial_schema.sql:1411-1428), `public.policy_entries_v1` (component/governance/core/migrations/0002_projection_views.sql:61-69) … 22 more  
writers 7: `upsert_registry::query#23c1db95` (component/governance/policy/src/repo.rs:1790-1792), `tests::seed_fixture::query#8600072f` (component/semantic/graph/src/projection_source.rs:3569-3581), `tests::seed_fixture` (component/semantic/graph/src/projection_source.rs:3569-3581) … 4 more  

### 86. Impl PolicyScanner#impl
`rust:apps/causeway-policy-scan/src/main.rs::PolicyScanner#impl`  apps/causeway-policy-scan/src/main.rs:482-1769  pkg:cargo/causeway-policy-scan  score 0.40  via `PolicyScanner::attach_declassification_barriers` <- DEFINES (apps/causeway-policy-scan/src/main.rs:1136-1198, 1.00)  
`impl PolicyScanner`  
members 32  
uses_types 1: `PolicyScanner` (apps/causeway-policy-scan/src/main.rs:482-1769)  

### 87. Function summary
`rust:component/governance/core/src/rights.rs::summary`  component/governance/core/src/rights.rs:1262-1297  pkg:cargo/governance-component  score 0.40  via `count` <- CALLS (component/governance/core/src/rights.rs:1290, 1.00)  
`fn summary( slug: &str, document: &TermsDocument, head: Option<&RightsDraft>, ) -> pb::ContractPolicySummary`  
callers 2: `RightsService::list_contract_policies` (component/governance/core/src/rights.rs:722), `RightsService::get_contract_policy` (component/governance/core/src/rights.rs:752)  
calls 9: `count` (component/governance/core/src/rights.rs:1290), `CheckStatus::as_str` (component/governance/core/src/rights.rs:1279), `count` (component/governance/core/src/rights.rs:1289) … 6 more  
references 1: `TermTier::Confirmed` (component/governance/core/src/rights.rs:1275)  
uses_types 2: `TermsDocument` (component/governance/core/src/rights.rs:1264), `RightsDraft` (component/governance/core/src/rights.rs:1265)  

### 88. Method WorkspaceScope::postgres_schema
`rust:torrentfx/crates/causeway-workspace-scope/src/lib.rs::WorkspaceScope::postgres_schema`  torrentfx/crates/causeway-workspace-scope/src/lib.rs:186-188  pkg:cargo/causeway-workspace-scope  score 0.40  via `TestWorkspace::database_url_for` -> CALLS (crates/causeway-testkit/src/workspace.rs:230, 0.75)  
`pub fn postgres_schema(&self) -> String`  
The base `PostgreSQL` schema: `cw_<id_>`.  
callers 22: `WorkspaceScope::qdrant_prefix` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:228), `tests::names` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:443), `WorkspaceScope::notify_channel` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:221) … 19 more  
calls 1: `WorkspaceScope::underscored` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:187)  
tests 9: `tests::only_a_hyphenated_ids_valkey_and_restate_names_change` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:517), `tests::an_id_without_a_hyphen_is_used_verbatim_in_every_name` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:376), `tests::every_name_of_table_4_for_a_hyphenated_id` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:347) … 6 more  

### 89. Variant MigrationMode::Apply
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode::Apply`  crates/causeway-db-core/src/workspace_migrations.rs:304  pkg:cargo/causeway-db-core  score 0.39  via `main` -> REFERENCES (jobs/causeway-migrate/src/main.rs:93, 1.00)  
`Apply`  
Apply pending migrations and verify each schema is current.  
referenced_by 7: `migrate` (crates/causeway-testkit/src/infra.rs:285), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:231), `main` (jobs/causeway-migrate/src/main.rs:73) … 4 more  

### 90. Function evaluate_companion
`rust:torrentfx/crates/causeway-platform/src/alias.rs::evaluate_companion`  torrentfx/crates/causeway-platform/src/alias.rs:591-602  pkg:cargo/causeway-platform  score 0.39  via `base_schema_of` -> CALLS (jobs/causeway-migrate/src/main.rs:153, 1.00)  
`pub fn evaluate_companion( scope: &WorkspaceScope, companion: Companion<'_>, value: Option<&str>, ) -> Result<AliasVerdict, ConfigError>`  
Judges the `value` of a companion scope variable (`CAUSEWAY_PG_SCHEMA` (doc continues, 360 bytes)  
callers 9: `check_companion` (torrentfx/crates/causeway-platform/src/alias.rs:620), `companion_case` (torrentfx/crates/causeway-platform/tests/conformance.rs:382), `tests::a_companion_that_repeats_the_derived_name_warns` (torrentfx/crates/causeway-platform/src/alias.rs:943) … 6 more  
calls 5: `RemovedVariable::checked` (torrentfx/crates/causeway-platform/src/alias.rs:596), `companion_replacement` (torrentfx/crates/causeway-platform/src/alias.rs:599), `RemovedVariable::judge` (torrentfx/crates/causeway-platform/src/alias.rs:596) … 2 more  
tests 3: `tests::a_companion_that_disagrees_stops_startup` (torrentfx/crates/causeway-platform/src/alias.rs:953), `tests::an_object_prefix_companion_names_the_topology_prefix_it_derives_from` (torrentfx/crates/causeway-platform/src/alias.rs:979), `tests::a_companion_that_repeats_the_derived_name_warns` (torrentfx/crates/causeway-platform/src/alias.rs:932)  
uses_types 4: `AliasVerdict` (torrentfx/crates/causeway-platform/src/alias.rs:595), `WorkspaceScope` (torrentfx/crates/causeway-platform/src/alias.rs:592), `ConfigError` (torrentfx/crates/causeway-platform/src/alias.rs:595) … 1 more  

### 91. Method WorkspaceScope::component_schema
`rust:torrentfx/crates/causeway-workspace-scope/src/lib.rs::WorkspaceScope::component_schema`  torrentfx/crates/causeway-workspace-scope/src/lib.rs:196-205  pkg:cargo/causeway-workspace-scope  score 0.39  via `TestWorkspace::database_url_for` -> CALLS (crates/causeway-testkit/src/workspace.rs:229, 0.75)  
`pub fn component_schema(&self, component: &str) -> String`  
The schema a component owns: `cw_<id_>_<component>`. (doc continues, 222 bytes)  
callers 18: `tests::names` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:444), `tests::an_id_without_a_hyphen_is_used_verbatim_in_every_name` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:378), `tests::a_component_path_becomes_a_bare_identifier` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:402) … 15 more  
calls 3: `with_separator` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:203), `WorkspaceScope::postgres_schema` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:202), `WorkspaceScope::postgres_schema` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:198)  
tests 7: `tests::an_id_without_a_hyphen_is_used_verbatim_in_every_name` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:378), `tests::only_a_hyphenated_ids_valkey_and_restate_names_change` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:519), `tests::every_name_of_table_4_for_a_hyphenated_id` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:349) … 4 more  

### 92. Enum ConfigError
`rust:torrentfx/crates/causeway-platform/src/config.rs::ConfigError`  torrentfx/crates/causeway-platform/src/config.rs:29-135  pkg:cargo/causeway-platform  score 0.39  via `base_schema_of` -> USES_TYPE (jobs/causeway-migrate/src/main.rs:152, 1.00)  
`pub enum ConfigError`  
Error returned when config loading or validation fails.  
members 14  
importers 20: `torrentfx/crates/causeway-platform/src/lib.rs` (torrentfx/crates/causeway-platform/src/lib.rs:44-51), `torrentfx/crates/causeway-platform/src/stated.rs` (torrentfx/crates/causeway-platform/src/stated.rs:18-21), `crates/causeway-runtime/src/identity.rs` (crates/causeway-runtime/src/identity.rs:21) … 17 more  
referenced_by 4: `tests::a_selector_that_names_another_environment_stops_the_scoped_names` (crates/causeway-runtime/src/restate.rs:1013), `tests::a_selector_that_names_another_environment_stops_the_scoped_names` (crates/causeway-runtime/src/restate.rs:1038), `tests::a_selector_that_names_another_environment_has_no_keyspace` (apps/causeway-server/src/cache.rs:552) … 1 more  
used_by 123: `seed_data_alias` (services/mokta/src/topology.rs:191), `TopologySource::verifier_with` (torrentfx/crates/causeway-platform/src/stated.rs:228), `Environment::Err` (torrentfx/crates/causeway-platform/src/config.rs:342) … 120 more  
uses_types 1: `ConfigError` (torrentfx/crates/causeway-platform/src/config.rs:29)  

### 93. Query PgPolicyBarrierRepo::effective_barriers_for_path::select #e3616183
`rust:component/governance/core/src/policy_barrier_repo.rs::PgPolicyBarrierRepo::effective_barriers_for_path::query#e3616183`  component/governance/core/src/policy_barrier_repo.rs:130-135  pkg:cargo/governance-component  score 0.38  via `public.policy_propagation_barriers` <- READS (component/governance/core/src/policy_barrier_repo.rs:130-135, 1.00)  
`SELECT barrier_id, barrier_kind, path, actor, reason, created_at, approved_at, expires_at                  FROM {schema}.policy_propagation_barriers            …`  
reads 1: `public.policy_propagation_barriers` (component/governance/core/src/policy_barrier_repo.rs:130-135)  

### 94. Query PgPolicyBarrierRepo::insert_approved_barrier::insert #48cb7bfd
`rust:component/governance/core/src/policy_barrier_repo.rs::PgPolicyBarrierRepo::insert_approved_barrier::query#48cb7bfd`  component/governance/core/src/policy_barrier_repo.rs:75-86  pkg:cargo/governance-component  score 0.38  via `public.policy_propagation_barriers` <- WRITES (component/governance/core/src/policy_barrier_repo.rs:75-86, 1.00)  
`INSERT INTO {schema}.policy_propagation_barriers                  (barrier_id, barrier_kind, path, actor, reason, created_at, approved_at, expires_at, event_jso…`  
writes 1: `public.policy_propagation_barriers` (component/governance/core/src/policy_barrier_repo.rs:75-86)  

### 95. Table public.forms
`sql:public.forms`  component/orchestration/workflows/migrations/0001_initial_workflows.sql:1-33  pkg:cargo/workflows-component  score 0.35  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> WRITES (component/orchestration/workflows/src/impls/read_repo.rs:762-774, 1.00)  
declared in 2 places: component/orchestration/workflows/migrations/0001_initial_workflows.sql:1-33, crates/causeway-db-core/legacy-migrations/0018_workflows.sql:1-33  
`-- Workflow engine schema.`  
members 7  
migrations 2: `component/orchestration/workflows/migrations/0001_initial_workflows.sql` (component/orchestration/workflows/migrations/0001_initial_workflows.sql:1-33), `crates/causeway-db-core/legacy-migrations/0018_workflows.sql` (crates/causeway-db-core/legacy-migrations/0018_workflows.sql:1-33)  
readers 9: `PgWorkflowReadRepo::find_form_body::query#9f9c240a` (component/orchestration/workflows/src/impls/read_repo.rs:73-74), `load_pinned_form` (component/orchestration/workflows/src/impls/read_repo.rs:454), `main` (apps/causeway-server-tools/src/bin/sync_workflows.rs:76) … 6 more  
writers 4: `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `database_tests::reads_find_the_slice_tables_from_the_workspace_pool::query#301b1b24` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `main::query#1cdc659b` (apps/causeway-server-tools/src/bin/sync_forms.rs:65-66) … 1 more  

### 96. Table public.workflow_definitions
`sql:public.workflow_definitions`  component/orchestration/workflows/migrations/0001_initial_workflows.sql:35-46  pkg:cargo/workflows-component  score 0.35  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> WRITES (component/orchestration/workflows/src/impls/read_repo.rs:762-774, 1.00)  
declared in 2 places: component/orchestration/workflows/migrations/0001_initial_workflows.sql:35-46, crates/causeway-db-core/legacy-migrations/0018_workflows.sql:35-46  
`-- ─── Workflow definitions ───────────────────────────────────────`  
members 9  
migrations 2: `crates/causeway-db-core/legacy-migrations/0018_workflows.sql` (crates/causeway-db-core/legacy-migrations/0018_workflows.sql:35-46), `component/orchestration/workflows/migrations/0001_initial_workflows.sql` (component/orchestration/workflows/migrations/0001_initial_workflows.sql:35-46)  
readers 7: `outbound_signals::query#5cde8616` (component/orchestration/workflows/src/impls/read_repo.rs:475-476), `main::query#5efc5794` (apps/causeway-server-tools/src/bin/sync_workflows.rs:87), `main` (apps/causeway-server-tools/src/bin/sync_workflows.rs:87) … 4 more  
writers 6: `ensure_task` (component/orchestration/workflows/src/impls/managed_tasks.rs:192-195), `database_tests::reads_find_the_slice_tables_from_the_workspace_pool::query#301b1b24` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` (component/orchestration/workflows/src/impls/read_repo.rs:762-774) … 3 more  

### 97. Table public.workflow_events
`sql:public.workflow_events`  component/orchestration/workflows/migrations/0001_initial_workflows.sql:125-150  pkg:cargo/workflows-component  score 0.35  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> WRITES (component/orchestration/workflows/src/impls/read_repo.rs:762-774, 1.00)  
declared in 4 places: component/orchestration/workflows/migrations/0001_initial_workflows.sql:125-150, component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:64-67, component/orchestration/workflows/migrations/0003_admin_resume_and_force_cancel.sql:1-7, crates/causeway-db-core/legacy-migrations/0018_workflows.sql:125-150  
`-- ─── Events (append-only audit log) ─────────────────────────────`  
members 10  
migrations 4: `crates/causeway-db-core/legacy-migrations/0018_workflows.sql` (crates/causeway-db-core/legacy-migrations/0018_workflows.sql:125-150), `component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql` (component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:64-67), `component/orchestration/workflows/migrations/0003_admin_resume_and_force_cancel.sql` (component/orchestration/workflows/migrations/0003_admin_resume_and_force_cancel.sql:1-7) … 1 more  
readers 4: `load_workflow_event` (component/orchestration/workflows/src/impls/read_repo.rs:544-545), `replay_workflow_events_after` (component/orchestration/workflows/src/impls/read_repo.rs:525-528), `replay_workflow_events_after::query#ee40be4b` (component/orchestration/workflows/src/impls/read_repo.rs:525-528) … 1 more  
writers 6: `PgSink::emit::query#3e76d2da` (component/orchestration/workflows/src/impls/sink.rs:21-23), `database_tests::reads_find_the_slice_tables_from_the_workspace_pool::query#301b1b24` (component/orchestration/workflows/src/impls/read_repo.rs:762-774), `settle_task` (component/orchestration/workflows/src/impls/managed_tasks.rs:288-289) … 3 more  

### 98. Table public.workflow_instances
`sql:public.workflow_instances`  component/orchestration/workflows/migrations/0001_initial_workflows.sql:74-115  pkg:cargo/workflows-component  score 0.35  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> WRITES (component/orchestration/workflows/src/impls/read_repo.rs:762-774, 1.00)  
declared in 4 places: component/orchestration/workflows/migrations/0001_initial_workflows.sql:74-115, component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:35-40, crates/causeway-db-core/legacy-migrations/0018_workflows.sql:74-115, component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:35-40  
`-- ─── Workflow instances ─────────────────────────────────────────`  
members 24  
migrations 4: `crates/causeway-db-core/legacy-migrations/0018_workflows.sql` (crates/causeway-db-core/legacy-migrations/0018_workflows.sql:74-115), `public.workflow_instances.attention_resumed_state` (component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:35-40), `component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql` (component/orchestration/workflows/migrations/0002_idempotent_start_and_attention_required.sql:35-40) … 1 more  
readers 10: `load_open_tasks` (component/orchestration/workflows/src/impls/read_repo.rs:369-376), `load_open_tasks::query#39ca63f8` (component/orchestration/workflows/src/impls/read_repo.rs:369-376), `PgStore::load::query#853017c9` (component/orchestration/workflows/src/impls/store.rs:57-62) … 7 more  
writers 10: `settle_task::query#1205ef73` (component/orchestration/workflows/src/impls/managed_tasks.rs:278-279), `PgStore::create` (component/orchestration/workflows/src/impls/store.rs:195-201), `ensure_task::query#022fed6d` (component/orchestration/workflows/src/impls/managed_tasks.rs:213-216) … 7 more  

### 99. Table public.workflow_tasks
`sql:public.workflow_tasks`  component/orchestration/workflows/migrations/0001_initial_workflows.sql:157-177  pkg:cargo/workflows-component  score 0.35  via `database_tests::reads_find_the_slice_tables_from_the_workspace_pool` -> WRITES (component/orchestration/workflows/src/impls/read_repo.rs:762-774, 1.00)  
declared in 2 places: component/orchestration/workflows/migrations/0001_initial_workflows.sql:157-177, crates/causeway-db-core/legacy-migrations/0018_workflows.sql:157-177  
`-- ─── Tasks (UI queue) ───────────────────────────────────────────`  
members 15  
migrations 2: `crates/causeway-db-core/legacy-migrations/0018_workflows.sql` (crates/causeway-db-core/legacy-migrations/0018_workflows.sql:157-177), `component/orchestration/workflows/migrations/0001_initial_workflows.sql` (component/orchestration/workflows/migrations/0001_initial_workflows.sql:157-177)  
readers 14: `supersede_tasks` (component/orchestration/workflows/src/impls/managed_tasks.rs:309-313), `reconcile_tasks` (component/orchestration/workflows/src/tasks.rs:68-69), `load_task_with_form` (component/orchestration/workflows/src/impls/read_repo.rs:391-396) … 11 more  
writers 9: `reconcile_tasks` (component/orchestration/workflows/src/tasks.rs:46-50), `ensure_task::query#1813ec4f` (component/orchestration/workflows/src/impls/managed_tasks.rs:226-231), `reconcile_tasks::query#d6585814` (component/orchestration/workflows/src/tasks.rs:46-50) … 6 more  

### 100. Method PolicyPropagationBarrier::restore_approved
`rust:crates/policy-model/src/lib.rs::PolicyPropagationBarrier::restore_approved`  crates/policy-model/src/lib.rs:768-789  pkg:cargo/policy-model  score 0.35  via `PolicyScanner::attach_declassification_barriers` -> CALLS (apps/causeway-policy-scan/src/main.rs:1164, 1.00)  
`pub fn restore_approved( barrier_id: Uuid, kind: PolicyPropagationBarrierKind, path: LineagePathRef, actor: PolicyActor, reason: PolicyReason, created_at: DateTime<Utc>, approved_at: DateTime<Utc>, expires_at: Option<DateTime<Utc>>, ) -> Self`  
Restores an approved barrier from durable storage.  
callers 2: `row_to_barrier` (component/governance/core/src/policy_barrier_repo.rs:210), `PolicyScanner::attach_declassification_barriers` (apps/causeway-policy-scan/src/main.rs:1164)  
uses_types 1: `PolicyPropagationBarrierKind` (crates/policy-model/src/lib.rs:770)  

### 101. Struct PolicyBarrierEvent
`rust:crates/policy-model/src/lib.rs::PolicyBarrierEvent`  crates/policy-model/src/lib.rs:1109-1118  pkg:cargo/policy-model  score 0.35  via `PgPolicyBarrierRepo::insert_approved_barrier` -> USES_TYPE (component/governance/core/src/policy_barrier_repo.rs:57, 1.00)  
`pub struct PolicyBarrierEvent`  
Auditable event emitted by a propagation barrier transition.  
members 1  
importers 1: `component/governance/core/src/policy_barrier_repo.rs` (component/governance/core/src/policy_barrier_repo.rs:9-12)  
used_by 7: `AuditedBarrierTransition` (crates/policy-model/src/lib.rs:1142), `AuditedBarrierTransition::new` (crates/policy-model/src/lib.rs:1147), `PgPolicyBarrierRepo::insert_approved_barrier` (component/governance/core/src/policy_barrier_repo.rs:57) … 4 more  
uses_types 2: `PolicyBarrierEventKind` (crates/policy-model/src/lib.rs:1112), `PolicyPropagationBarrierKind` (crates/policy-model/src/lib.rs:1113)  

## Left out

38 included nodes have no source window for want of budget.  
- `rust:component/governance/policy/src/materialize.rs::DeclassificationBarrier::new` Method 0.34 (over budget, 397 tokens)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::tests::a_layout_without_a_baseline_is_refused` Test 0.34 (over budget, 267 tokens)
- `rust:crates/causeway-testkit/src/parity.rs::tests` Module 0.34 (over budget, 161 tokens)
- and 529 more

## Unresolved references in the seeds' files

- `id` at component/governance/core/src/rights.rs:1007 (ambiguous)
- `subject` at component/governance/core/src/rights.rs:1007 (ambiguous)
- `to_owned_message` at component/governance/core/src/rights.rs:1023 (no-declaration)
- `to_owned_message` at component/governance/core/src/rights.rs:1077 (no-declaration)
- `to_owned_message` at component/governance/core/src/rights.rs:1110 (no-declaration)
- `Right` at component/governance/core/src/rights.rs:1270 (qualifier-mismatch)
- `split_once` at component/governance/core/src/rights.rs:1376 (no-declaration)
- `is_ascii_digit` at component/governance/core/src/rights.rs:1403 (no-declaration)
- `is_ascii_lowercase` at component/governance/core/src/rights.rs:1403 (no-declaration)
- `is_some_and` at component/governance/core/src/rights.rs:1403 (no-declaration)

