# Context pack

Task: Workloads must stop migrating the database when they start. Make causeway-db-core's initialisation run no DDL: compare the applied migrations in the connection's schema with the embedded baseline, fail with an error that names the pending versions and the migrate step when the schema is behind, and accept a schema that is ahead so an older pod survives a rollout.

## Seeds

- `pkg:cargo/causeway-db-core` Package 1.00 (identifier `causeway-db-core`)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::StillPending` Variant 0.90 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::migrate_workspace` Function 0.81 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::pending_count` Function 0.77 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode::Apply` Variant 0.77 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationSummary::pending` Method 0.76 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::TargetReport::pending` Method 0.74 (text)
- `typescript:torrentfx/packages-node/causeway-data/src/errors.ts::MigrationDatabaseError` Class 0.71 (text)
- `task:mise/db:migrate` Task 0.71 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::CreateSchema` Variant 0.71 (text)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget::schema` Method 0.69 (text)

Budget 30000 tokens, used 29959. Seeds 11, candidates 1297, included 86, left out 1445.

## Context

### 1. Function migrate_workspace
`rust:crates/causeway-db-core/src/workspace_migrations.rs::migrate_workspace`  crates/causeway-db-core/src/workspace_migrations.rs:457-507  pkg:cargo/causeway-db-core  score 3.75  
`pub async fn migrate_workspace( database_url: &str, targets: &[MigrationTarget], workspace_uuid: &str, mode: MigrationMode, mut on_target: impl FnMut(&TargetReport<'_>), ) -> WorkspaceMigrationResult<MigrationSummary>`  
Migrates `targets` in order over `database_url`, with (doc continues, 421 bytes)  
callers 5: `migrate` (crates/causeway-testkit/src/infra.rs:281), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:527), `main` (jobs/causeway-migrate/src/main.rs:139) … 2 more  
calls 7: `connect` (crates/causeway-db-core/src/workspace_migrations.rs:469), `set_workspace_uuid` (crates/causeway-db-core/src/workspace_migrations.rs:471), `ensure_schema` (crates/causeway-db-core/src/workspace_migrations.rs:470) … 4 more  
references 2: `MigrationMode::Check` (crates/causeway-db-core/src/workspace_migrations.rs:483), `MigrationMode::Apply` (crates/causeway-db-core/src/workspace_migrations.rs:484)  
tests 2: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:95), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:527)  
uses_types 5: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:463), `TargetReport` (crates/causeway-db-core/src/workspace_migrations.rs:462), `MigrationMode` (crates/causeway-db-core/src/workspace_migrations.rs:461) … 2 more  
crates/causeway-db-core/src/workspace_migrations.rs:457-507
```rust
pub async fn migrate_workspace(
    database_url: &str,
    targets: &[MigrationTarget],
    workspace_uuid: &str,
    mode: MigrationMode,
    mut on_target: impl FnMut(&TargetReport<'_>),
) -> WorkspaceMigrationResult<MigrationSummary> {
    let mut summary = MigrationSummary {
        schemas: targets.len(),
        pending: 0,
    };
    for target in targets {
        let mut conn = connect(database_url).await?;
        ensure_schema(&mut conn, &target.schema).await?;
        set_workspace_uuid(&mut conn, workspace_uuid).await?;
        let mut migrator = Migrator::new(target.migrations.as_path())
            .await
            .map_err(|source| WorkspaceMigrationError::Load {
                path: target.migrations.clone(),
                source,
            })?;
        migrator.set_ignore_missing(target.ignore_missing);

        let pending = pending_count(&mut conn, &migrator).await?;
        summary.pending += pending;
        match mode {
            MigrationMode::Check => on_target(&TargetReport { target, pending }),
            MigrationMode::Apply => {
                migrator
                    .run(&mut conn)
                    .await
                    .map_err(|source| WorkspaceMigrationError::Apply {
                        schema: target.schema.clone(),
                        source,
                    })?;
                on_target(&TargetReport { target, pending });
                // Verify before moving on: apply mode must leave every
                // schema current or fail.
                let remaining = pending_count(&mut conn, &migrator).await?;
                if remaining != 0 {
                    return Err(WorkspaceMigrationError::StillPending {
                        schema: target.schema.clone(),
                        remaining,
                    });
                }
            }
        }
        conn.close().await.ok();
    }
    Ok(summary)
}
```

### 2. Variant MigrationMode::Apply
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode::Apply`  crates/causeway-db-core/src/workspace_migrations.rs:399  pkg:cargo/causeway-db-core  score 2.79  
`Apply`  
Apply pending migrations and verify each schema is current.  
referenced_by 8: `migrate` (crates/causeway-testkit/src/infra.rs:285), `main` (jobs/causeway-migrate/src/main.rs:168), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:531) … 5 more  

### 3. Package cargo/causeway-db-core
`pkg:cargo/causeway-db-core`  crates/causeway-db-core/Cargo.toml:2  pkg:cargo/causeway-db-core  score 2.58  
`cargo/causeway-db-core`  
members 130  
dependents 34: `cargo/governance-terms-component` (component/governance/terms/Cargo.toml:1-33), `cargo/marketplace-component` (component/marketplace/core/Cargo.toml:1-63), `cargo/catalog-component` (component/catalog/core/Cargo.toml:1-72) … 31 more  
dependencies 10: `cargo/tracing` (crates/causeway-db-core/Cargo.toml:1-23), `cargo/toml` (crates/causeway-db-core/Cargo.toml:1-23), `cargo/serde` (crates/causeway-db-core/Cargo.toml:1-23) … 7 more  
importers 99: `read_error` (component/governance/resolution/src/ports.rs:18), `component/catalog/core/src/bindings.rs` (component/catalog/core/src/bindings.rs:18), `component/metadata/core/src/content_source.rs` (component/metadata/core/src/content_source.rs:16) … 96 more  

### 4. Function pending_count
`rust:crates/causeway-db-core/src/workspace_migrations.rs::pending_count`  crates/causeway-db-core/src/workspace_migrations.rs:571-589  pkg:cargo/causeway-db-core  score 2.54  
`async fn pending_count( conn: &mut PgConnection, migrator: &Migrator, ) -> WorkspaceMigrationResult<usize>`  
Counts staged migrations not yet applied in the current schema.  
callers 2: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:480), `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:495)  
references 2: `WorkspaceMigrationError::ListApplied` (crates/causeway-db-core/src/workspace_migrations.rs:581), `WorkspaceMigrationError::MigrationsTable` (crates/causeway-db-core/src/workspace_migrations.rs:577)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:574)  
crates/causeway-db-core/src/workspace_migrations.rs:571-589
```rust
async fn pending_count(
    conn: &mut PgConnection,
    migrator: &Migrator,
) -> WorkspaceMigrationResult<usize> {
    conn.ensure_migrations_table()
        .await
        .map_err(WorkspaceMigrationError::MigrationsTable)?;
    let applied: HashSet<i64> = conn
        .list_applied_migrations()
        .await
        .map_err(WorkspaceMigrationError::ListApplied)?
        .into_iter()
        .map(|m| m.version)
        .collect();
    Ok(migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration() && !applied.contains(&m.version))
        .count())
}
```

### 5. File crates/causeway-db-core/src/workspace_migrations.rs
`file:crates/causeway-db-core/src/workspace_migrations.rs`  crates/causeway-db-core/src/workspace_migrations.rs  pkg:cargo/causeway-db-core  score 1.98  via `migrate_workspace` <- CONTAINS (crates/causeway-db-core/src/workspace_migrations.rs:457-507, 1.00)  
members 30  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:489)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2474)  
imports 1: `cargo/sqlx` (crates/causeway-db-core/src/workspace_migrations.rs:24)  

### 6. Variant WorkspaceMigrationError::StillPending
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::StillPending`  crates/causeway-db-core/src/workspace_migrations.rs:149  pkg:cargo/causeway-db-core  score 1.88  
`StillPending`  
Apply mode left a schema behind.  

### 7. Enum WorkspaceMigrationError
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError`  crates/causeway-db-core/src/workspace_migrations.rs:62-150  pkg:cargo/causeway-db-core  score 1.55  via `WorkspaceMigrationError::StillPending` <- DEFINES (crates/causeway-db-core/src/workspace_migrations.rs:149, 1.00)  
`pub enum WorkspaceMigrationError`  
What went wrong finding or applying a workspace's migrations. (doc continues, 177 bytes)  
members 17  
importers 1: `crates/causeway-testkit/src/error.rs` (crates/causeway-testkit/src/error.rs:7)  
used_by 2: `TestkitError::Migrate` (crates/causeway-testkit/src/error.rs:105), `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:153)  
crates/causeway-db-core/src/workspace_migrations.rs:62-141, cut
```rust
pub enum WorkspaceMigrationError {
    /// The staged layout has no `baseline/` directory.
    #[error("baseline migrations missing at {}", path.display())]
    BaselineMissing { path: PathBuf },

    /// A directory of migration sets could not be listed.
    #[error("reading {}", path.display())]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// An entry of a directory of migration sets could not be read.
    #[error(transparent)]
    DirEntry(std::io::Error),

    /// A component directory's name is not UTF-8, so it names no schema.
    #[error("non-UTF-8 component dir {}", name.display())]
    NonUtf8Component { name: std::ffi::OsString },

    /// A package's slice manifest does not name a slice as `area/slice`.
    #[error("{} must hold `slice = \"<area>/<slice>\"`: {message}", path.display())]
    SliceManifest { path: PathBuf, message: String },

    /// The database URL does not parse.
    #[error("parsing database URL")]
    Url(#[source] sqlx::Error),

    /// The database could not be reached.
    #[error("connecting to Postgres")]
    Connect(#[source] sqlx::Error),

    /// The target names a schema the runner never manages.
    #[error("refusing to manage protected schema `{schema}`")]
    ProtectedSchema { schema: String },

    /// The target's schema is not a lower-case identifier.
    #[error("schema `{schema}` is not a valid lowercase identifier")]
    InvalidSchema { schema: String },

    /// `CREATE SCHEMA` failed.
    #[error("creating schema {schema}")]
    CreateSchema {
        schema: String,
        #[source]
        source: sqlx::Error,
    },

    /// `SET search_path` failed.
    #[error("setting search_path to {schema}")]
    SearchPath {
        schema: String,
        #[source]
        source: sqlx::Error,
    },

    /// `causeway.workspace_uuid` could not be set.
    #[error("configuring migration workspace UUID")]
    WorkspaceUuid(#[source] sqlx::Error),

    /// A migration directory could not be loaded.
    #[error("loading migrations from {}", path.display())]
    Load {
        path: PathBuf,
        #[source]
        source: MigrateError,
    },

    /// sqlx's bookkeeping table could not be created.
    #[error("ensuring _sqlx_migrations table")]
    MigrationsTable(#[source] MigrateError),

    /// The applied migrations could not be listed.
    #[error("listing applied migrations")]
    ListApplied(#[source] MigrateError),

    /// A migration failed to apply.
    #[error("migrating schema {schema}")]
    Apply {
```

### 8. Variant MigrationMode::Check
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode::Check`  crates/causeway-db-core/src/workspace_migrations.rs:397  pkg:cargo/causeway-db-core  score 1.52  via `migrate_workspace` -> REFERENCES (crates/causeway-db-core/src/workspace_migrations.rs:483, 1.00)  
`Check`  
Count pending migrations per schema and change nothing.  
referenced_by 7: `main` (jobs/causeway-migrate/src/main.rs:142), `tests::the_flags_parse_into_a_step_and_its_directories` (jobs/causeway-migrate/src/main.rs:403), `parse_args` (jobs/causeway-migrate/src/main.rs:84) … 4 more  

### 9. Method MigrationSummary::pending
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationSummary::pending`  crates/causeway-db-core/src/workspace_migrations.rs:441-443  pkg:cargo/causeway-db-core  score 1.48  
`pub const fn pending(&self) -> usize`  
Migrations pending across every schema when it was reached: in (doc continues, 109 bytes)  
callers 6: `main` (jobs/causeway-migrate/src/main.rs:164), `main` (jobs/causeway-migrate/src/main.rs:151), `main` (jobs/causeway-migrate/src/main.rs:162) … 3 more  
tests 1: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:105)  
crates/causeway-db-core/src/workspace_migrations.rs:441-443
```rust
    pub const fn pending(&self) -> usize {
        self.pending
    }
```

### 10. Method TargetReport::pending
`rust:crates/causeway-db-core/src/workspace_migrations.rs::TargetReport::pending`  crates/causeway-db-core/src/workspace_migrations.rs:419-421  pkg:cargo/causeway-db-core  score 1.45  
`pub const fn pending(&self) -> usize`  
Migrations that were pending when the schema was reached: in (doc continues, 107 bytes)  
callers 6: `main` (jobs/causeway-migrate/src/main.rs:164), `main` (jobs/causeway-migrate/src/main.rs:174), `main` (jobs/causeway-migrate/src/main.rs:162) … 3 more  
tests 1: `a_workspace_migrates_every_component_into_its_derived_schemas` (crates/causeway-testkit/tests/postgres.rs:105)  
crates/causeway-db-core/src/workspace_migrations.rs:419-421
```rust
    pub const fn pending(&self) -> usize {
        self.pending
    }
```

### 11. Class MigrationDatabaseError
`typescript:torrentfx/packages-node/causeway-data/src/errors.ts::MigrationDatabaseError`  torrentfx/packages-node/causeway-data/src/errors.ts:91-101  pkg:npm/@causeway/data  score 1.44  
`class MigrationDatabaseError extends Data.TaggedError("MigrationDatabaseError")<{ readonly schema: string; /** What the runner was doing: `connect`, `apply 3`, `list applied`… */ readonly step: string; readonly cause: unknown; }>`  
A statement of the runner, or of a migration, failed at the database.  
members 1  
importers 1: `torrentfx/packages-node/causeway-data/src/migrations.ts` (torrentfx/packages-node/causeway-data/src/migrations.ts:40-49)  
used_by 1: `migrate::failed` (torrentfx/packages-node/causeway-data/src/migrations.ts:263)  
torrentfx/packages-node/causeway-data/src/errors.ts:91-101
```typescript
export class MigrationDatabaseError extends Data.TaggedError("MigrationDatabaseError")<{
  readonly schema: string;
  /** What the runner was doing: `connect`, `apply 3`, `list applied`… */
  readonly step: string;
  readonly cause: unknown;
}> {
  override get message(): string {
    const reason = this.cause instanceof Error ? this.cause.message : String(this.cause);
    return `migrating schema ${this.schema}: ${this.step} failed: ${reason}`;
  }
}
```

### 12. Enum MigrationMode
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationMode`  crates/causeway-db-core/src/workspace_migrations.rs:395-400  pkg:cargo/causeway-db-core  score 1.40  via `MigrationMode::Apply` <- DEFINES (crates/causeway-db-core/src/workspace_migrations.rs:399, 1.00)  
`pub enum MigrationMode`  
Whether [`migrate_workspace`] applies the pending migrations or only (doc continues, 82 bytes)  
members 2  
importers 4: `crates/causeway-testkit/src/workspace.rs` (crates/causeway-testkit/src/workspace.rs:8), `crates/causeway-testkit/src/infra.rs` (crates/causeway-testkit/src/infra.rs:33), `jobs/causeway-migrate/src/main.rs` (jobs/causeway-migrate/src/main.rs:33) … 1 more  
used_by 2: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:461), `Step::Migrate` (jobs/causeway-migrate/src/main.rs:48)  
crates/causeway-db-core/src/workspace_migrations.rs:395-400
```rust
pub enum MigrationMode {
    /// Count pending migrations per schema and change nothing.
    Check,
    /// Apply pending migrations and verify each schema is current.
    Apply,
}
```

### 13. Function main
`rust:jobs/causeway-migrate/src/main.rs::main`  jobs/causeway-migrate/src/main.rs:97-185  pkg:cargo/causeway-migrate  score 1.39  via `migrate_workspace` <- CALLS (jobs/causeway-migrate/src/main.rs:139, 1.00)  
`async fn main() -> Result<()>`  
calls 43: `ResolvedConfig::from_resolved` (jobs/causeway-migrate/src/main.rs:103), `migrate_workspace` (jobs/causeway-migrate/src/main.rs:139), `base_schema` (jobs/causeway-migrate/src/main.rs:107) … 40 more  
references 7: `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:142), `Environment::Local` (jobs/causeway-migrate/src/main.rs:113), `MigrationMode::Apply` (jobs/causeway-migrate/src/main.rs:168) … 4 more  
jobs/causeway-migrate/src/main.rs:97-176, cut
```rust
async fn main() -> Result<()> {
    causeway_platform::install_crypto_provider();
    let args = parse_args(std::env::args().skip(1))?;

    let environment =
        ResolvedEnvironment::resolve().context("resolving the deployment environment")?;
    let config = ResolvedConfig::from_resolved(&environment, "causeway-migrate")
        .context("resolving the topology")?;
    let url = database_url(&config, environment.topology_file())?;
    let scope = &config.base.workspace;
    let base_schema = base_schema(scope)?;
    // The UUID component migrations mint stable identities under. It is
    // derived from the workspace id; `CAUSEWAY_WORKSPACE_UUID` stated it
    // before and is a checked alias for one release, so a value that names
    // another workspace stops the job before it migrates anything.
    let workspace_uuid = workspace_uuid(scope)?;
    if args.credentials.is_some() && config.base.deployment_environment != Environment::Local {
        bail!("--credentials writes passwords to files and is for `local` only");
    }

    // The loop is the library's (RFD-fpg4tnq2, "One migration path"): the
    // testkit migrates a test's workspace through the same function.
    let (targets, workloads_file) = layout(args.repository.as_deref(), &base_schema)?;
    // Planned before migrating, so a bad declaration stops the Job before
    // it changes anything.
    let roles = RolePlan::new(
        &targets,
        &RoleDeclarations::load(&workloads_file, &targets)?,
    )?;
    let mode = match args.step {
        Step::Migrate(mode) => mode,
        Step::RolesOnly => {
            let provisioned = slice_roles::provision_roles(&url, &roles).await?;
            let applied = matches!(provisioned, Provisioned::Applied { .. });
            report_roles(provisioned);
            if applied && let Some(dir) = &args.credentials {
                write_credentials(&url, &roles, dir).await?;
            }
            return Ok(());
        }
    };
    let summary =
        workspace_migrations::migrate_workspace(&url, &targets, &workspace_uuid, mode, |report| {
            let target = report.target();
            match mode {
                MigrationMode::Check => println!(
                    "check schema={} pending={} source={}",
                    target.schema(),
                    report.pending(),
                    target.migrations().display()
                ),
                MigrationMode::Apply => println!(
                    "migrated schema={} applied={}",
                    target.schema(),
                    report.pending()
                ),
            }
        })
        .await?;

    match mode {
        MigrationMode::Check => {
            println!(
                "check complete schemas={} total_pending={}",
                summary.schemas(),
                summary.pending()
            );
            if summary.pending() > 0 {
                std::process::exit(EXIT_PENDING);
            }
        }
        MigrationMode::Apply => {
            // The rollout greps this line to decide whether running
            // workloads must be restarted onto the new schema.
            println!(
                "migrate complete schemas={} total_applied={}",
                summary.schemas(),
                summary.pending()
            );
            let provisioned = slice_roles::provision_roles(&url, &roles).await?;
```

### 14. Method MigrationTarget::schema
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget::schema`  crates/causeway-db-core/src/workspace_migrations.rs:168-170  pkg:cargo/causeway-db-core  score 1.37  
`pub fn schema(&self) -> &str`  
The schema the set is applied into.  
callers 17: `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:787), `tests::schemas` (crates/causeway-db-core/src/workspace_migrations.rs:602), `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:789) … 14 more  
tests 2: `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:787), `tests::a_workspace_id_is_valid_and_leaves_room_for_every_schema` (crates/causeway-testkit/src/workspace.rs:513)  
crates/causeway-db-core/src/workspace_migrations.rs:168-170
```rust
    pub fn schema(&self) -> &str {
        &self.schema
    }
```

### 15. Task db:migrate
`task:mise/db:migrate`  .mise/toml/db.toml:4-7  pkg:npm/causeway-monorepo  score 1.36  
`mise run db:migrate`  
Apply all pending sqlx migrations to the workspace schemas (baseline + components). RESET_DB=1 drops every workspace schema first: it refuses a non-local KUBE_CONTEXT unless CAUSEWAY_OPS_CONTEXT_ALLOW names it exactly, and needs CONFIRM=1 (without it the task lists what it would drop and exits 1). Tasks that run db:migrate as a step clear RESET_DB for it.  
description: Apply all pending sqlx migrations to the workspace schemas (baseline + components). RESET_DB=1 drops every workspace schema first: it refuses a non-local KUBE_CONTEXT unless CAUSEWAY_OPS_CONTEXT_ALLOW names it exactly, and needs CONFIRM=1 (without it the task lists what it would drop and exits 1). Tasks that run db:migrate as a step clear RESET_DB for it.  
run: .mise/scripts/tasks/db/migrate.rhai  
dependents 4: `mise/deploy:search-sync-worker` (.mise/toml/deploy.toml:147), `mise/deploy:stuck-work-sweep` (.mise/toml/deploy.toml:97), `mise/deploy:server` (.mise/toml/deploy.toml:166) … 1 more  
run_by 2: `.mise/scripts/tasks/dev/run.rhai` (.mise/scripts/tasks/dev/run.rhai:336), `migrate` (.mise/rhai/causeway/devdata.rhai:67)  
runs 1: `.mise/scripts/tasks/db/migrate.rhai` (.mise/toml/db.toml:6)  
.mise/toml/db.toml:4-7
```toml
["db:migrate"]
description = "Apply all pending sqlx migrations to the workspace schemas (baseline + components). RESET_DB=1 drops every workspace schema first: it refuses a non-local KUBE_CONTEXT unless CAUSEWAY_OPS_CONTEXT_ALLOW names it exactly, and needs CONFIRM=1 (without it the task lists what it would drop and exits 1). Tasks that run db:migrate as a step clear RESET_DB for it."
run = ".mise/scripts/tasks/db/migrate.rhai"

```

### 16. TypeAlias WorkspaceMigrationResult
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationResult`  crates/causeway-db-core/src/workspace_migrations.rs:153  pkg:cargo/causeway-db-core  score 1.34  via `migrate_workspace` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:463, 1.00)  
`pub type WorkspaceMigrationResult<T> = std::result::Result<T, WorkspaceMigrationError>;`  
Result of a workspace migration step.  
used_by 13: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:463), `underscored` (crates/causeway-db-core/src/workspace_migrations.rs:383), `read_dir` (crates/causeway-db-core/src/workspace_migrations.rs:361) … 10 more  
uses_types 1: `WorkspaceMigrationError` (crates/causeway-db-core/src/workspace_migrations.rs:153)  
crates/causeway-db-core/src/workspace_migrations.rs:153-153
```rust
pub type WorkspaceMigrationResult<T> = std::result::Result<T, WorkspaceMigrationError>;
```

### 17. Variant WorkspaceMigrationError::CreateSchema
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::CreateSchema`  crates/causeway-db-core/src/workspace_migrations.rs:105-109  pkg:cargo/causeway-db-core  score 1.34  
`CreateSchema`  
`CREATE SCHEMA` failed.  
uses_types 5: `Audience::Error` (crates/causeway-db-core/src/workspace_migrations.rs:108), `ServiceEndpointTopology::Error` (crates/causeway-db-core/src/workspace_migrations.rs:108), `WebOrigin::Error` (crates/causeway-db-core/src/workspace_migrations.rs:108) … 2 more  

### 18. Function migrate
`rust:crates/causeway-testkit/src/infra.rs::migrate`  crates/causeway-testkit/src/infra.rs:270-299  pkg:cargo/causeway-testkit  score 1.23  via `migrate_workspace` <- CALLS (crates/causeway-testkit/src/infra.rs:281, 1.00)  
`async fn migrate(postgres: &EphemeralPostgres, scope: &WorkspaceScope) -> TestkitResult<String>`  
Applies the baseline and every component's migrations into the (doc continues, 121 bytes)  
callers 1: `InfraPlan::start` (crates/causeway-testkit/src/infra.rs:213)  
calls 13: `migrate_workspace` (crates/causeway-testkit/src/infra.rs:281), `repository_targets` (crates/causeway-testkit/src/infra.rs:274), `repository_root` (crates/causeway-testkit/src/infra.rs:274) … 10 more  
references 1: `MigrationMode::Apply` (crates/causeway-testkit/src/infra.rs:285)  
uses_types 3: `TestkitResult` (crates/causeway-testkit/src/infra.rs:270), `EphemeralPostgres` (crates/causeway-testkit/src/infra.rs:270), `WorkspaceScope` (crates/causeway-testkit/src/infra.rs:270)  
crates/causeway-testkit/src/infra.rs:270-299
```rust
async fn migrate(postgres: &EphemeralPostgres, scope: &WorkspaceScope) -> TestkitResult<String> {
    let started = std::time::Instant::now();
    let base_schema = scope.postgres_schema();
    let targets =
        workspace_migrations::repository_targets(&crate::repository_root(), &base_schema)?;
    let schemas: Vec<String> = targets
        .iter()
        .map(|target| target.schema().to_owned())
        .collect();
    check_identifier_lengths(&schemas)?;
    let owner_url = postgres.owner_url();
    workspace_migrations::migrate_workspace(
        &owner_url,
        &targets,
        &scope.workspace_uuid(),
        MigrationMode::Apply,
        |_| {},
    )
    .await?;
    println!(
        "  workspace {} migrated ({} schemas) in {:.2}s",
        scope.as_str(),
        schemas.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(with_search_path(
        &owner_url,
        &[base_schema.as_str(), "public"],
    ))
}
```

### 19. Struct TargetReport
`rust:crates/causeway-db-core/src/workspace_migrations.rs::TargetReport`  crates/causeway-db-core/src/workspace_migrations.rs:404-407  pkg:cargo/causeway-db-core  score 1.18  via `TargetReport#impl` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:409-422, 1.00)  
`pub struct TargetReport<'a>`  
What [`migrate_workspace`] found or did in one schema.  
members 2  
used_by 2: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:462), `TargetReport#impl` (crates/causeway-db-core/src/workspace_migrations.rs:409-422)  
uses_types 1: `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:405)  
crates/causeway-db-core/src/workspace_migrations.rs:404-407
```rust
pub struct TargetReport<'a> {
    target: &'a MigrationTarget,
    pending: usize,
}
```

### 20. Struct MigrationTarget
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget`  crates/causeway-db-core/src/workspace_migrations.rs:157-163  pkg:cargo/causeway-db-core  score 1.17  via `MigrationTarget#impl` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:165-221, 1.00)  
`pub struct MigrationTarget`  
One schema and the migration set that belongs in it.  
members 7  
importers 1: `crates/causeway-db-core/src/slice_roles.rs` (crates/causeway-db-core/src/slice_roles.rs:29)  
used_by 11: `TargetReport::target` (crates/causeway-db-core/src/workspace_migrations.rs:412), `tests::schemas` (crates/causeway-db-core/src/workspace_migrations.rs:599), `layout` (jobs/causeway-migrate/src/main.rs:218) … 8 more  
crates/causeway-db-core/src/workspace_migrations.rs:157-163
```rust
pub struct MigrationTarget {
    schema: String,
    slice: String,
    migrations: PathBuf,
    published: PathBuf,
    ignore_missing: bool,
}
```

### 21. Test a_workspace_migrates_every_component_into_its_derived_schemas
`rust:crates/causeway-testkit/tests/postgres.rs::a_workspace_migrates_every_component_into_its_derived_schemas`  crates/causeway-testkit/tests/postgres.rs:77-123  pkg:cargo/causeway-testkit  score 1.12  via `migrate_workspace` -> TESTED_BY (crates/causeway-testkit/tests/postgres.rs:95, 1.00)  
`async fn a_workspace_migrates_every_component_into_its_derived_schemas()`  
members 2  
calls 18: `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:95), `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:89), `schemas_like` (crates/causeway-testkit/tests/postgres.rs:82) … 15 more  
references 1: `MigrationMode::Check` (crates/causeway-testkit/tests/postgres.rs:99)  
tests_for 15: `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:95), `repository_targets` (crates/causeway-testkit/tests/postgres.rs:90), `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:89) … 12 more  
crates/causeway-testkit/tests/postgres.rs:77-123
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

### 22. Struct MigrationSummary
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationSummary`  crates/causeway-db-core/src/workspace_migrations.rs:426-429  pkg:cargo/causeway-db-core  score 1.05  via `MigrationSummary#impl` -> USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:431-444, 1.00)  
`pub struct MigrationSummary`  
Totals over every target of one [`migrate_workspace`] run.  
members 2  
used_by 2: `MigrationSummary#impl` (crates/causeway-db-core/src/workspace_migrations.rs:431-444), `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:463)  
crates/causeway-db-core/src/workspace_migrations.rs:426-429
```rust
pub struct MigrationSummary {
    schemas: usize,
    pending: usize,
}
```

### 23. Function ensure_schema
`rust:crates/causeway-db-core/src/workspace_migrations.rs::ensure_schema`  crates/causeway-db-core/src/workspace_migrations.rs:551-568  pkg:cargo/causeway-db-core  score 1.04  via `migrate_workspace` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:470, 1.00)  
`async fn ensure_schema(conn: &mut PgConnection, schema: &str) -> WorkspaceMigrationResult<()>`  
Creates the schema when absent and scopes the session to it, so (doc continues, 182 bytes)  
members 2  
callers 1: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:470)  
calls 1: `check_schema` (crates/causeway-db-core/src/workspace_migrations.rs:552)  
uses_types 2: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:551), `SearchPath` (crates/causeway-db-core/src/workspace_migrations.rs:563)  
crates/causeway-db-core/src/workspace_migrations.rs:551-568
```rust
async fn ensure_schema(conn: &mut PgConnection, schema: &str) -> WorkspaceMigrationResult<()> {
    check_schema(schema)?;
    sqlx::query(&format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#))
        .execute(&mut *conn)
        .await
        .map_err(|source| WorkspaceMigrationError::CreateSchema {
            schema: schema.to_owned(),
            source,
        })?;
    sqlx::query(&format!(r#"SET search_path = "{schema}", public"#))
        .execute(&mut *conn)
        .await
        .map_err(|source| WorkspaceMigrationError::SearchPath {
            schema: schema.to_owned(),
            source,
        })?;
    Ok(())
}
```

### 24. Method TestWorkspace::prepare
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::prepare`  crates/causeway-testkit/src/workspace.rs:84-153  pkg:cargo/causeway-testkit  score 0.96  via `migrate_workspace` <- CALLS (crates/causeway-testkit/src/workspace.rs:106, 1.00)  
`async fn prepare( containers: &Containers, repo_root: &Path, scope: WorkspaceScope, ) -> TestkitResult<Self>`  
[`Self::new`] over the migrations under `repo_root`, for `scope`. (doc continues, 269 bytes)  
callers 2: `tests::a_workspace_that_fails_partway_drops_what_it_created` (crates/causeway-testkit/src/workspace.rs:579), `TestWorkspace::new` (crates/causeway-testkit/src/workspace.rs:76)  
calls 18: `drop_workspace` (crates/causeway-testkit/src/workspace.rs:125), `repository_targets` (crates/causeway-testkit/src/workspace.rs:91), `with_search_path` (crates/causeway-testkit/src/workspace.rs:104) … 15 more  
references 2: `REPOSITORY_WORKLOAD_ROLES` (crates/causeway-testkit/src/workspace.rs:98), `MigrationMode::Apply` (crates/causeway-testkit/src/workspace.rs:110)  
tests 1: `tests::a_workspace_that_fails_partway_drops_what_it_created` (crates/causeway-testkit/src/workspace.rs:579)  
uses_types 4: `TestkitError` (crates/causeway-testkit/src/workspace.rs:119), `TestkitResult` (crates/causeway-testkit/src/workspace.rs:88), `WorkspaceScope` (crates/causeway-testkit/src/workspace.rs:87) … 1 more  
crates/causeway-testkit/src/workspace.rs:84-153
```rust
    async fn prepare(
        containers: &Containers,
        repo_root: &Path,
        scope: WorkspaceScope,
    ) -> TestkitResult<Self> {
        let started = std::time::Instant::now();
        let base_schema = scope.postgres_schema();
        let targets = workspace_migrations::repository_targets(repo_root, &base_schema)?;
        let schemas: Vec<String> = targets
            .iter()
            .map(|target| target.schema().to_owned())
            .collect();
        check_identifier_lengths(&schemas)?;
        let declarations = RoleDeclarations::load(
            &repo_root.join(slice_roles::REPOSITORY_WORKLOAD_ROLES),
            &targets,
        )?;
        let roles = RolePlan::new(&targets, &declarations)?;

        let owner_url = containers.postgres().owner_url();
        let database_url = with_search_path(&owner_url, &[base_schema.as_str(), "public"]);
        let prepared = async {
            workspace_migrations::migrate_workspace(
                &owner_url,
                &targets,
                &scope.workspace_uuid(),
                MigrationMode::Apply,
                |_| {},
            )
            .await?;
            provision(&owner_url, &roles).await?;
            let topology = LocalTopology::write(scope.as_str(), &database_url)?;
            let pool = PgPool::connect(&database_url).await.map_err(|source| {
                TestkitError::database("connecting the workspace pool", source)
            })?;
            Ok::<_, TestkitError>((topology, pool))
        }
        .await;
        let (topology, pool) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                if let Err(cleanup) = drop_workspace(&owner_url, &base_schema, &schemas).await {
                    eprintln!(
                        "causeway-testkit: dropping the schemas of failed workspace {}: {}",
                        scope.as_str(),
                        chain(&cleanup)
                    );
                }
                return Err(error);
            }
        };
        eprintln!(
            "causeway-testkit: workspace {} migrated ({} schemas) in {:.2}s",
            scope.as_str(),
            schemas.len(),
            started.elapsed().as_secs_f64()
        );

        Ok(Self {
            scope,
            schemas,
            roles,
            owner_url,
            database_url,
            pool,
            topology,
            service_env: Vec::new(),
            kept: false,
        })
    }
```

### 25. Test the_migrations_create_their_own_extensions_in_a_fresh_database
`rust:crates/causeway-testkit/tests/postgres.rs::the_migrations_create_their_own_extensions_in_a_fresh_database`  crates/causeway-testkit/tests/postgres.rs:507-555  pkg:cargo/causeway-testkit  score 0.95  via `migrate_workspace` -> TESTED_BY (crates/causeway-testkit/tests/postgres.rs:527, 1.00)  
`async fn the_migrations_create_their_own_extensions_in_a_fresh_database()`  
The migrations create the extensions they need on a database that has (doc continues, 167 bytes)  
members 1  
calls 8: `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:508), `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:527), `repository_targets` (crates/causeway-testkit/tests/postgres.rs:522) … 5 more  
references 1: `MigrationMode::Apply` (crates/causeway-testkit/tests/postgres.rs:531)  
tests_for 7: `repository_targets` (crates/causeway-testkit/tests/postgres.rs:522), `Containers::shared` (crates/causeway-testkit/tests/postgres.rs:508), `migrate_workspace` (crates/causeway-testkit/tests/postgres.rs:527) … 4 more  
crates/causeway-testkit/tests/postgres.rs:507-555
```rust
async fn the_migrations_create_their_own_extensions_in_a_fresh_database() {
    let containers = Containers::shared().await.expect("the shared containers");
    let admin = containers.postgres().admin_url();
    let mut server = PgConnection::connect(&admin).await.expect("connect");
    let database = format!("fresh_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {database} OWNER causeway"))
        .execute(&mut server)
        .await
        .expect("a fresh database");

    let owner_url = containers.postgres().owner_url();
    let owner_url = format!(
        "{}/{database}",
        owner_url.rsplit_once('/').expect("a database path").0
    );
    let targets = workspace_migrations::repository_targets(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        "cw_fresh",
    )
    .expect("targets");
    workspace_migrations::migrate_workspace(
        &owner_url,
        &targets,
        "00000000-0000-4000-8000-000000000000",
        MigrationMode::Apply,
        |_| {},
    )
    .await
    .expect("the migrations apply on a database with no extensions");

    let mut fresh = PgConnection::connect(&owner_url).await.expect("connect");
    let mut installed: Vec<String> =
        sqlx::query_scalar("SELECT extname::text FROM pg_extension WHERE extname <> 'plpgsql'")
            .fetch_all(&mut fresh)
            .await
            .expect("extensions");
    installed.sort();
    let mut allowed: Vec<String> = UPSTREAM_IMAGE_EXTENSIONS
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    allowed.sort();
    assert_eq!(installed, allowed);
    fresh.close().await.ok();
    sqlx::query(&format!("DROP DATABASE {database} WITH (FORCE)"))
        .execute(&mut server)
        .await
        .expect("drop the fresh database");
}
```

### 26. Function set_workspace_uuid
`rust:crates/causeway-db-core/src/workspace_migrations.rs::set_workspace_uuid`  crates/causeway-db-core/src/workspace_migrations.rs:510-520  pkg:cargo/causeway-db-core  score 0.91  via `migrate_workspace` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:471, 1.00)  
`async fn set_workspace_uuid( conn: &mut PgConnection, workspace_uuid: &str, ) -> WorkspaceMigrationResult<()>`  
Makes the workspace identity available to component migrations.  
members 1  
callers 1: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:471)  
references 1: `WorkspaceMigrationError::WorkspaceUuid` (crates/causeway-db-core/src/workspace_migrations.rs:518)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:513)  
crates/causeway-db-core/src/workspace_migrations.rs:510-520
```rust
async fn set_workspace_uuid(
    conn: &mut PgConnection,
    workspace_uuid: &str,
) -> WorkspaceMigrationResult<()> {
    sqlx::query("SELECT set_config('causeway.workspace_uuid', $1, false)")
        .bind(workspace_uuid)
        .execute(conn)
        .await
        .map_err(WorkspaceMigrationError::WorkspaceUuid)?;
    Ok(())
}
```

### 27. Function connect
`rust:crates/causeway-db-core/src/workspace_migrations.rs::connect`  crates/causeway-db-core/src/workspace_migrations.rs:522-528  pkg:cargo/causeway-db-core  score 0.84  via `migrate_workspace` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:469, 1.00)  
`async fn connect(url: &str) -> WorkspaceMigrationResult<PgConnection>`  
callers 54: `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:469), `connect` (crates/causeway-db-core/src/slice_roles.rs:956), `tests::policy_writer_registers_real_semantic_identities` (apps/causeway-policy-scan/src/main.rs:2296) … 51 more  
calls 1: `connect` (crates/causeway-db-core/src/workspace_migrations.rs:525)  
references 2: `WorkspaceMigrationError::Url` (crates/causeway-db-core/src/workspace_migrations.rs:523), `WorkspaceMigrationError::Connect` (crates/causeway-db-core/src/workspace_migrations.rs:527)  
tests 52: `database_tests::a_correction_replaces_the_payload_and_puts_the_work_back` (component/metadata/core/src/stuck_work.rs:800), `tests::literal_matches_are_empty_for_empty_sources_and_name_an_unreadable_one` (component/discovery/search/core/src/lib.rs:4566), `tests::ingest_writes_rows_and_binds_physical_columns` (component/semantic/pure/src/elements.rs:1248) … 49 more  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:522)  
crates/causeway-db-core/src/workspace_migrations.rs:522-528
```rust
async fn connect(url: &str) -> WorkspaceMigrationResult<PgConnection> {
    let options = PgConnectOptions::from_str(url).map_err(WorkspaceMigrationError::Url)?;
    options
        .connect()
        .await
        .map_err(WorkspaceMigrationError::Connect)
}
```

### 28. Function repository_targets
`rust:crates/causeway-db-core/src/workspace_migrations.rs::repository_targets`  crates/causeway-db-core/src/workspace_migrations.rs:282-312  pkg:cargo/causeway-db-core  score 0.79  via `MigrationTarget` <- USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:285, 1.00)  
`pub fn repository_targets( repo_root: &Path, base_schema: &str, ) -> WorkspaceMigrationResult<Vec<MigrationTarget>>`  
The targets of the repository checked out at `repo_root`: the baseline (doc continues, 549 bytes)  
callers 15: `tests::the_repository_layout_names_schemas_as_the_staging_script_does` (crates/causeway-db-core/src/workspace_migrations.rs:686), `provisioning_revokes_what_the_declarations_no_longer_list` (crates/causeway-testkit/tests/postgres.rs:372), `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:771) … 12 more  
calls 7: `MigrationTarget::component` (crates/causeway-db-core/src/workspace_migrations.rs:309), `package_slices` (crates/causeway-db-core/src/workspace_migrations.rs:305), `subdirectories` (crates/causeway-db-core/src/workspace_migrations.rs:296) … 4 more  
importers 2: `tests` (crates/causeway-testkit/src/parity.rs:200), `tests` (crates/causeway-db-core/src/slice_roles.rs:993)  
tests 10: `provisioning_revokes_what_the_declarations_no_longer_list` (crates/causeway-testkit/tests/postgres.rs:372), `tests::a_package_that_declares_a_slice_migrates_into_its_schema` (crates/causeway-db-core/src/workspace_migrations.rs:735), `the_migrations_create_their_own_extensions_in_a_fresh_database` (crates/causeway-testkit/tests/postgres.rs:522) … 7 more  
uses_types 2: `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:285), `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:285)  
crates/causeway-db-core/src/workspace_migrations.rs:282-312
```rust
pub fn repository_targets(
    repo_root: &Path,
    base_schema: &str,
) -> WorkspaceMigrationResult<Vec<MigrationTarget>> {
    let baseline = repo_root.join(REPOSITORY_BASELINE_DIR);
    if !baseline.is_dir() {
        return Err(WorkspaceMigrationError::BaselineMissing { path: baseline });
    }
    let published = repo_root
        .join(REPOSITORY_BASELINE_DIR)
        .with_file_name(PUBLISHED_VIEWS_FILE);
    let mut targets = vec![MigrationTarget::baseline(base_schema, baseline, published)];

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
    sets.extend(package_slices(repo_root)?);
    sets.sort();
    targets.extend(sets.into_iter().map(|(suffix, migrations)| {
        let published = migrations.with_file_name(PUBLISHED_VIEWS_FILE);
        MigrationTarget::component(base_schema, &suffix, migrations, published)
    }));
    Ok(targets)
}
```

### 29. Variant WorkspaceMigrationError::ListApplied
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::ListApplied`  crates/causeway-db-core/src/workspace_migrations.rs:137  pkg:cargo/causeway-db-core  score 0.74  via `pending_count` -> REFERENCES (crates/causeway-db-core/src/workspace_migrations.rs:581, 1.00)  
`ListApplied`  
The applied migrations could not be listed.  
referenced_by 1: `pending_count` (crates/causeway-db-core/src/workspace_migrations.rs:581)  

### 30. Package cargo/causeway-platform
`pkg:cargo/causeway-platform`  torrentfx/crates/causeway-platform/Cargo.toml:2  pkg:cargo/causeway-platform  score 0.71  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:12, 1.00)  
`cargo/causeway-platform`  
members 19  
dependents 48: `cargo/causeway-metadata` (tools/causeway-metadata/Cargo.toml:11), `cargo/causeway-stuck-work-sweep` (jobs/causeway-stuck-work-sweep/Cargo.toml:17), `cargo/causeway-metadata-outbox-relay` (jobs/causeway-metadata-outbox-relay/Cargo.toml:16) … 45 more  
dependencies 22: `cargo/tokio-util` (torrentfx/crates/causeway-platform/Cargo.toml:30), `cargo/opentelemetry-semantic-conventions` (torrentfx/crates/causeway-platform/Cargo.toml:22), `cargo/opentelemetry-otlp` (torrentfx/crates/causeway-platform/Cargo.toml:13) … 19 more  
designs 2: `docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md` (docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md:100), `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md` (docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md:625)  
importers 121: `apps/causeway-server-tools/src/bin/sync_teams.rs` (apps/causeway-server-tools/src/bin/sync_teams.rs:11), `apps/causeway-server/src/routes/own_origins.rs` (apps/causeway-server/src/routes/own_origins.rs:21), `torrentfx/crates/causeway-platform/tests/conformance.rs` (torrentfx/crates/causeway-platform/tests/conformance.rs:27) … 118 more  

### 31. Variant WorkspaceMigrationError::MigrationsTable
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::MigrationsTable`  crates/causeway-db-core/src/workspace_migrations.rs:133  pkg:cargo/causeway-db-core  score 0.68  via `pending_count` -> REFERENCES (crates/causeway-db-core/src/workspace_migrations.rs:577, 1.00)  
`MigrationsTable`  
sqlx's bookkeeping table could not be created.  
referenced_by 1: `pending_count` (crates/causeway-db-core/src/workspace_migrations.rs:577)  

### 32. Impl MigrationSummary#impl
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationSummary#impl`  crates/causeway-db-core/src/workspace_migrations.rs:431-444  pkg:cargo/causeway-db-core  score 0.65  via `MigrationSummary::pending` <- DEFINES (crates/causeway-db-core/src/workspace_migrations.rs:441-443, 1.00)  
`impl MigrationSummary`  
members 2  
uses_types 1: `MigrationSummary` (crates/causeway-db-core/src/workspace_migrations.rs:431-444)  

### 33. Impl TargetReport#impl
`rust:crates/causeway-db-core/src/workspace_migrations.rs::TargetReport#impl`  crates/causeway-db-core/src/workspace_migrations.rs:409-422  pkg:cargo/causeway-db-core  score 0.64  via `TargetReport::pending` <- DEFINES (crates/causeway-db-core/src/workspace_migrations.rs:419-421, 1.00)  
`impl TargetReport<'_>`  
members 2  
uses_types 1: `TargetReport` (crates/causeway-db-core/src/workspace_migrations.rs:409-422)  

### 34. Repository causeway
`repo:causeway`  score 0.62  via `cargo/causeway-db-core` <- CONTAINS (crates/causeway-db-core/Cargo.toml:2, 1.00)  
members 279  

### 35. Package cargo/causeway-workspace-scope
`pkg:cargo/causeway-workspace-scope`  torrentfx/crates/causeway-workspace-scope/Cargo.toml:2  pkg:cargo/causeway-workspace-scope  score 0.61  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:13, 1.00)  
`cargo/causeway-workspace-scope`  
members 6  
dependents 4: `cargo/causeway-testkit` (crates/causeway-testkit/Cargo.toml:25), `cargo/cw-workspace` (bootstrap/cw-workspace/Cargo.toml:14), `cargo/causeway-platform` (torrentfx/crates/causeway-platform/Cargo.toml:9) … 1 more  
dependencies 5: `cargo/temp-env` (torrentfx/crates/causeway-workspace-scope/Cargo.toml:19), `cargo/serde_json` (torrentfx/crates/causeway-workspace-scope/Cargo.toml:18), `cargo/sha2` (torrentfx/crates/causeway-workspace-scope/Cargo.toml:14) … 2 more  
designs 1: `docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md` (docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md:901)  
importers 20: `torrentfx/crates/causeway-platform/src/alias.rs` (torrentfx/crates/causeway-platform/src/alias.rs:31), `crates/causeway-testkit/src/valkey.rs` (crates/causeway-testkit/src/valkey.rs:11), `crates/causeway-testkit/tests/postgres.rs` (crates/causeway-testkit/tests/postgres.rs:15) … 17 more  

### 36. Impl MigrationTarget#impl
`rust:crates/causeway-db-core/src/workspace_migrations.rs::MigrationTarget#impl`  crates/causeway-db-core/src/workspace_migrations.rs:165-221  pkg:cargo/causeway-db-core  score 0.60  via `MigrationTarget::schema` <- DEFINES (crates/causeway-db-core/src/workspace_migrations.rs:168-170, 1.00)  
`impl MigrationTarget`  
members 7  
uses_types 1: `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:165-221)  

### 37. Package cargo/serde
`pkg:cargo/serde`  pkg:cargo/serde  score 0.60  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 137: `cargo/people-component` (component/people/core/Cargo.toml:1-27), `cargo/causeway-policy-bundle` (crates/causeway-policy-bundle/Cargo.toml:1-24), `cargo/conversations-component` (component/collaboration/conversations/Cargo.toml:1-49) … 134 more  
importers 377: `tools/prism-lsp/src/custom.rs` (tools/prism-lsp/src/custom.rs:10), `tests` (crates/drive-core/src/lib.rs:2466), `crates/ingest-core/src/error.rs` (crates/ingest-core/src/error.rs:30) … 374 more  

### 38. Package cargo/serde_json
`pkg:cargo/serde_json`  pkg:cargo/serde_json  score 0.60  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 145: `cargo/py-drive` (ffi/py-drive/Cargo.toml:22), `cargo/causeway-testkit` (crates/causeway-testkit/Cargo.toml:1-87), `cargo/coherence-component` (component/platform/coherence/Cargo.toml:1-44) … 142 more  
importers 136: `crates/platform-flags/tests/repository_group_rules.rs` (crates/platform-flags/tests/repository_group_rules.rs:19), `tests` (crates/causeway-metadata-ops/src/submit.rs:496), `tests` (crates/drive-runtime/src/schema_args.rs:178) … 133 more  

### 39. Package cargo/temp-env
`pkg:cargo/temp-env`  pkg:cargo/temp-env  score 0.60  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 13: `cargo/causeway-workspace-scope` (torrentfx/crates/causeway-workspace-scope/Cargo.toml:19), `cargo/causeway-server` (apps/causeway-server/Cargo.toml:190), `cargo/causeway-seed` (jobs/causeway-seed/Cargo.toml:61) … 10 more  
importers 1: `torrentfx/crates/causeway-relay/src/lib.rs` (torrentfx/crates/causeway-relay/src/lib.rs:23)  

### 40. Package cargo/thiserror
`pkg:cargo/thiserror`  pkg:cargo/thiserror  score 0.60  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 109: `cargo/causeway-oidc-keys` (torrentfx/crates/causeway-oidc-keys/Cargo.toml:21), `cargo/people-rpc` (crates/rpc/people-rpc/Cargo.toml:15), `cargo/metadata-rpc` (crates/rpc/metadata-rpc/Cargo.toml:15) … 106 more  
importers 60: `crates/causeway-testkit/tests/volumes.rs` (crates/causeway-testkit/tests/volumes.rs:35), `crates/causeway-testkit/tests/object_store.rs` (crates/causeway-testkit/tests/object_store.rs:29), `crates/causeway-runtime/src/resource.rs` (crates/causeway-runtime/src/resource.rs:23) … 57 more  

### 41. Test tests::the_repository_targets_cover_every_component_migration_directory
`rust:crates/causeway-db-core/src/workspace_migrations.rs::tests::the_repository_targets_cover_every_component_migration_directory`  crates/causeway-db-core/src/workspace_migrations.rs:769-791  pkg:cargo/causeway-db-core  score 0.59  via `MigrationTarget::schema` -> TESTED_BY (crates/causeway-db-core/src/workspace_migrations.rs:787, 1.00)  
`fn the_repository_targets_cover_every_component_migration_directory()`  
The repository's own sets, found the way the testkit finds them, (doc continues, 176 bytes)  
calls 9: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:771), `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:787), `package_slices` (crates/causeway-db-core/src/workspace_migrations.rs:785) … 6 more  
tests_for 5: `package_slices` (crates/causeway-db-core/src/workspace_migrations.rs:785), `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:787), `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:771) … 2 more  
crates/causeway-db-core/src/workspace_migrations.rs:769-791
```rust
    fn the_repository_targets_cover_every_component_migration_directory() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let targets = repository_targets(&repo_root, "cw_t").expect("targets");
        let mut expected = 0;
        for area in std::fs::read_dir(repo_root.join("component")).expect("component") {
            let area = area.expect("entry").path();
            if !area.is_dir() {
                continue;
            }
            for slice in std::fs::read_dir(&area).expect("area") {
                if slice.expect("entry").path().join("migrations").is_dir() {
                    expected += 1;
                }
            }
        }
        assert!(expected > 0, "the repository has component migrations");
        expected += package_slices(&repo_root).expect("package slices").len();
        assert_eq!(targets.len(), expected + 1);
        assert_eq!(targets[0].schema(), "cw_t");
        let mut sorted = targets[1..].to_vec();
        sorted.sort_by(|a, b| a.schema().cmp(b.schema()));
        assert_eq!(sorted, targets[1..], "components are in suffix order");
    }
```

### 42. Function migrate::failed
`typescript:torrentfx/packages-node/causeway-data/src/migrations.ts::migrate::failed`  torrentfx/packages-node/causeway-data/src/migrations.ts:262-263  pkg:npm/@causeway/data  score 0.59  via `MigrationDatabaseError` <- USES_TYPE (torrentfx/packages-node/causeway-data/src/migrations.ts:263, 1.00)  
`failed = (step: string) => (cause: unknown) => new MigrationDatabaseError({ schema, step, cause })`  
callers 3: `migrate::run` (torrentfx/packages-node/causeway-data/src/migrations.ts:288), `migrate` (torrentfx/packages-node/causeway-data/src/migrations.ts:275), `websocketTransport` (torrentfx/conformance/session/src/transport/websocket.ts:111)  
uses_types 1: `MigrationDatabaseError` (torrentfx/packages-node/causeway-data/src/migrations.ts:263)  
torrentfx/packages-node/causeway-data/src/migrations.ts:262-263
```typescript
  const failed = (step: string) => (cause: unknown) =>
    new MigrationDatabaseError({ schema, step, cause });
```

### 43. Test tests::a_workspace_that_fails_partway_drops_what_it_created
`rust:crates/causeway-testkit/src/workspace.rs::tests::a_workspace_that_fails_partway_drops_what_it_created`  crates/causeway-testkit/src/workspace.rs:554-598  pkg:cargo/causeway-testkit  score 0.58  via `TestWorkspace::prepare` <- CALLS (crates/causeway-testkit/src/workspace.rs:579, 1.00)  
`async fn a_workspace_that_fails_partway_drops_what_it_created()`  
A migration that fails partway leaves no schema behind: the baseline (doc continues, 132 bytes)  
members 1  
calls 13: `TestWorkspace::prepare` (crates/causeway-testkit/src/workspace.rs:579), `Containers::shared` (crates/causeway-testkit/src/workspace.rs:575), `random_workspace_id` (crates/causeway-testkit/src/workspace.rs:576) … 10 more  
tests_for 11: `random_workspace_id` (crates/causeway-testkit/src/workspace.rs:576), `connect_when_ready` (crates/causeway-testkit/src/workspace.rs:587), `WorkspaceScope::parse` (crates/causeway-testkit/src/workspace.rs:576) … 8 more  
crates/causeway-testkit/src/workspace.rs:554-598
```rust
    async fn a_workspace_that_fails_partway_drops_what_it_created() {
        let root = tempfile::tempdir().expect("tempdir");
        let write = |path: &str, sql: &str| {
            let path = root.path().join(path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
            std::fs::write(path, sql).expect("write");
        };
        write(
            "crates/causeway-db-core/migrations/0001_base.sql",
            "CREATE TABLE base_marker (id INTEGER);",
        );
        write(
            "component/aa/first/migrations/0001_first.sql",
            "CREATE TABLE first_marker (id INTEGER);",
        );
        write(
            "component/zz/broken/migrations/0001_broken.sql",
            "SELECT broken syntax here;",
        );
        // The role declarations are read before anything migrates.
        write("deploy/postgres/workload-roles.toml", "");
        let containers = Containers::shared().await.expect("the shared containers");
        let scope = WorkspaceScope::parse(random_workspace_id()).expect("an id");
        let base = scope.postgres_schema();

        let error = TestWorkspace::prepare(containers, root.path(), scope)
            .await
            .expect_err("the broken migration fails the workspace");
        assert!(
            error.to_string().contains("migrating the test workspace"),
            "{error}"
        );

        let mut connection = connect_when_ready(&containers.postgres().admin_url())
            .await
            .expect("connect");
        let left: Vec<String> = sqlx::query_scalar(
            "SELECT nspname::text FROM pg_namespace WHERE nspname = $1 OR nspname LIKE $1 || '\\_%'",
        )
        .bind(&base)
        .fetch_all(&mut connection)
        .await
        .expect("list schemas");
        assert_eq!(left, Vec::<String>::new(), "nothing of {base} is left");
    }
```

### 44. File torrentfx/packages-node/causeway-data/src/errors.ts
`file:torrentfx/packages-node/causeway-data/src/errors.ts`  torrentfx/packages-node/causeway-data/src/errors.ts  pkg:npm/@causeway/data  score 0.57  via `MigrationDatabaseError` <- CONTAINS (torrentfx/packages-node/causeway-data/src/errors.ts:91-101, 1.00)  
members 14  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1319)  
importers 5: `torrentfx/packages-node/causeway-data/src/config.ts` (torrentfx/packages-node/causeway-data/src/config.ts:26), `torrentfx/packages-node/causeway-data/src/outbox.ts` (torrentfx/packages-node/causeway-data/src/outbox.ts:16), `torrentfx/packages-node/causeway-data/src/rows.ts` (torrentfx/packages-node/causeway-data/src/rows.ts:13) … 2 more  
imports 1: `npm/effect` (torrentfx/packages-node/causeway-data/src/errors.ts:9)  

### 45. File .mise/toml/db.toml
`file:.mise/toml/db.toml`  .mise/toml/db.toml  pkg:npm/causeway-monorepo  score 0.57  via `mise/db:migrate` <- CONTAINS (.mise/toml/db.toml:4-7, 1.00)  
members 6  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:3469)  

### 46. Function migrate
`typescript:torrentfx/packages-node/causeway-data/src/migrations.ts::migrate`  torrentfx/packages-node/causeway-data/src/migrations.ts:260-378  pkg:npm/@causeway/data  score 0.57  via `migrate::failed` <- DEFINES (torrentfx/packages-node/causeway-data/src/migrations.ts:262-263, 1.00)  
`function migrate(options: MigrateOptions): Effect.Effect<MigrationReport, MigrationError>`  
Migrate one schema over one connection.  
members 3  
callers 5: `migrateSlice` (torrentfx/packages-node/causeway-data/src/migrations.ts:410), `packages/search-core/test/integration/search-log.test.ts` (packages/search-core/test/integration/search-log.test.ts:101), `torrentfx/packages-node/causeway-data/test/integration/data.test.ts` (torrentfx/packages-node/causeway-data/test/integration/data.test.ts:89) … 2 more  
calls 25: `migrate::run` (torrentfx/packages-node/causeway-data/src/migrations.ts:321), `migrate::run` (torrentfx/packages-node/causeway-data/src/migrations.ts:292), `migrate::run` (torrentfx/packages-node/causeway-data/src/migrations.ts:310) … 22 more  
tests 1: `migrations::refuses a protected schema` (torrentfx/packages-node/causeway-data/test/integration/data.test.ts:228)  
uses_types 4: `MigrationChecksumMismatch` (torrentfx/packages-node/causeway-data/src/migrations.ts:343), `MigrationVersionMissing` (torrentfx/packages-node/causeway-data/src/migrations.ts:336), `MigrationDirty` (torrentfx/packages-node/causeway-data/src/migrations.ts:329) … 1 more  
torrentfx/packages-node/causeway-data/src/migrations.ts:260-339, cut
```typescript
export function migrate(options: MigrateOptions): Effect.Effect<MigrationReport, MigrationError> {
  const { schema, set, mode } = options;
  const failed = (step: string) => (cause: unknown) =>
    new MigrationDatabaseError({ schema, step, cause });

  return Effect.scoped(
    Effect.gen(function* () {
      yield* checkSchema(schema);
      const client = yield* Effect.acquireRelease(
        Effect.tryPromise({
          try: async () => {
            const client = new Pg.Client({ connectionString: options.url.expose() });
            await client.connect();
            return client;
          },
          catch: failed("connect"),
        }),
        (client) => Effect.promise(() => client.end().catch(() => undefined)),
      );
      const run = <R extends Pg.QueryResultRow = Pg.QueryResultRow>(
        step: string,
        text: string,
        values?: readonly unknown[],
      ) =>
        Effect.tryPromise({
          // No values: the simple protocol, so a file may hold many statements.
          try: () =>
            values === undefined ? client.query<R>(text) : client.query<R>(text, [...values]),
          catch: failed(step),
        });

      yield* run("create schema", `CREATE SCHEMA IF NOT EXISTS "${schema}"`);
      yield* run("set search_path", `SET search_path = "${schema}", public`);
      yield* run("set workspace uuid", "SELECT set_config('causeway.workspace_uuid', $1, false)", [
        options.workspaceUuid,
      ]);

      const pendingOf = (applied: ReadonlyMap<bigint, Buffer>) =>
        set.migrations.filter((migration) => !applied.has(migration.version));
      const listApplied = run<AppliedRow>(
        "list applied",
        "SELECT version, checksum FROM _sqlx_migrations ORDER BY version",
      ).pipe(
        Effect.map(
          (result) =>
            new Map(result.rows.map((row) => [BigInt(row.version), row.checksum] as const)),
        ),
      );

      if (mode === "check") {
        yield* run("ensure history table", HISTORY_TABLE);
        const pending = pendingOf(yield* listApplied).map((m) => m.version);
        return { schema, pending, applied: [] };
      }

      const database = yield* run<{ name: string }>(
        "current database",
        "SELECT current_database() AS name",
      );
      const lockId = sqlxMigrationLockId(database.rows[0]?.name ?? "").toString();
      yield* run("lock", "SELECT pg_advisory_lock($1)", [lockId]);
      yield* run("ensure history table", HISTORY_TABLE);

      const dirty = yield* run<{ version: string }>(
        "dirty version",
        "SELECT version FROM _sqlx_migrations WHERE success = false ORDER BY version LIMIT 1",
      );
      const dirtyVersion = dirty.rows[0]?.version;
      if (dirtyVersion !== undefined) {
        return yield* new MigrationDirty({ schema, version: BigInt(dirtyVersion) });
      }

      const applied = yield* listApplied;
      const known = new Set(set.migrations.map((m) => m.version));
      const missing = [...applied.keys()].find((version) => !known.has(version));
      if (missing !== undefined) {
        return yield* new MigrationVersionMissing({ schema, version: missing });
      }
      const modified = set.migrations.find((migration) => {
        const recorded = applied.get(migration.version);
```

### 47. Function parse_args
`rust:jobs/causeway-migrate/src/main.rs::parse_args`  jobs/causeway-migrate/src/main.rs:67-94  pkg:cargo/causeway-migrate  score 0.56  via `MigrationMode::Apply` <- REFERENCES (jobs/causeway-migrate/src/main.rs:69, 1.00)  
`fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args>`  
callers 2: `main` (jobs/causeway-migrate/src/main.rs:99), `tests::args` (jobs/causeway-migrate/src/main.rs:388)  
calls 8: `tests::value` (jobs/causeway-migrate/src/main.rs:89), `AliasReplacement::value` (jobs/causeway-migrate/src/main.rs:89), `AliasReplacement::value` (jobs/causeway-migrate/src/main.rs:88) … 5 more  
references 5: `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:84), `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:82), `MigrationMode::Apply` (jobs/causeway-migrate/src/main.rs:69) … 2 more  
uses_types 1: `Args` (jobs/causeway-migrate/src/main.rs:67)  

### 48. Method TestWorkspace::new
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace::new`  crates/causeway-testkit/src/workspace.rs:74-77  pkg:cargo/causeway-testkit  score 0.56  via `TestWorkspace::prepare` <- CALLS (crates/causeway-testkit/src/workspace.rs:76, 1.00)  
`pub async fn new(containers: &Containers) -> TestkitResult<Self>`  
Prepares a new workspace in `containers`: (doc continues, 957 bytes)  
callers 6: `an_attached_store_is_the_topologys_s3` (crates/causeway-testkit/tests/object_store.rs:119), `an_attached_broker_is_the_topologys_kafka` (crates/causeway-testkit/tests/kafka.rs:138), `an_attached_server_is_the_topologys_valkey` (crates/causeway-testkit/tests/valkey.rs:116) … 3 more  
calls 4: `repository_root` (crates/causeway-testkit/src/workspace.rs:76), `TestWorkspace::prepare` (crates/causeway-testkit/src/workspace.rs:76), `random_workspace_id` (crates/causeway-testkit/src/workspace.rs:75) … 1 more  
tests 4: `an_attached_server_is_the_topologys_valkey` (crates/causeway-testkit/tests/valkey.rs:116), `an_attached_server_is_the_topologys_qdrant` (crates/causeway-testkit/tests/qdrant.rs:137), `an_attached_broker_is_the_topologys_kafka` (crates/causeway-testkit/tests/kafka.rs:138) … 1 more  
uses_types 2: `Containers` (crates/causeway-testkit/src/workspace.rs:74), `TestkitResult` (crates/causeway-testkit/src/workspace.rs:74)  
crates/causeway-testkit/src/workspace.rs:74-77
```rust
    pub async fn new(containers: &Containers) -> TestkitResult<Self> {
        let scope = WorkspaceScope::parse(random_workspace_id())?;
        Self::prepare(containers, &crate::repository_root(), scope).await
    }
```

### 49. Task deploy:stuck-work-sweep
`task:mise/deploy:stuck-work-sweep`  .mise/toml/deploy.toml:94-99  pkg:npm/causeway-monorepo  score 0.55  via `mise/db:migrate` <- DEPENDS_ON (.mise/toml/deploy.toml:97, 1.00)  
`mise run deploy:stuck-work-sweep`  
Build, load, and roll out the supervisory sweep (RFD-2n2tcfe2), which reclaims stuck work apart from the workers it watches. Its schema is component-owned, so db:migrate runs first.  
depends: db:migrate  
description: Build, load, and roll out the supervisory sweep (RFD-2n2tcfe2), which reclaims stuck work apart from the workers it watches. Its schema is component-owned, so db:migrate runs first.  
run: .mise/scripts/tasks/deploy/deploy-service.rhai stuck-work-sweep  
dependencies 1: `mise/db:migrate` (.mise/toml/deploy.toml:97)  
runs 1: `.mise/scripts/tasks/deploy/deploy-service.rhai` (.mise/toml/deploy.toml:98)  

### 50. Package cargo/sqlx
`pkg:cargo/sqlx`  pkg:cargo/sqlx  score 0.55  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 42: `cargo/causeway-testkit` (crates/causeway-testkit/Cargo.toml:1-87), `cargo/workflows-component` (component/orchestration/workflows/Cargo.toml:27), `cargo/causeway-catalog-host` (apps/causeway-catalog-host/Cargo.toml:68) … 39 more  
importers 147: `crates/causeway-testkit/tests/kafka.rs` (crates/causeway-testkit/tests/kafka.rs:27), `jobs/causeway-fibo-ingest/src/ingest_db.rs` (jobs/causeway-fibo-ingest/src/ingest_db.rs:17), `component/platform/coherence/src/store.rs` (component/platform/coherence/src/store.rs:7) … 144 more  

### 51. Function staged_targets
`rust:crates/causeway-db-core/src/workspace_migrations.rs::staged_targets`  crates/causeway-db-core/src/workspace_migrations.rs:232-267  pkg:cargo/causeway-db-core  score 0.54  via `MigrationTarget` <- USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:235, 1.00)  
`pub fn staged_targets( root: &Path, base_schema: &str, ) -> WorkspaceMigrationResult<Vec<MigrationTarget>>`  
The targets of the layout `causeway-migrate`'s image stages under (doc continues, 370 bytes)  
callers 4: `layout` (jobs/causeway-migrate/src/main.rs:228), `tests::a_layout_without_a_baseline_is_refused` (crates/causeway-db-core/src/workspace_migrations.rs:658), `tests::a_staged_layout_without_components_is_the_baseline_alone` (crates/causeway-db-core/src/workspace_migrations.rs:651) … 1 more  
calls 5: `MigrationTarget::component` (crates/causeway-db-core/src/workspace_migrations.rs:264), `read_dir` (crates/causeway-db-core/src/workspace_migrations.rs:246), `MigrationTarget::baseline` (crates/causeway-db-core/src/workspace_migrations.rs:241) … 2 more  
references 2: `WorkspaceMigrationError::DirEntry` (crates/causeway-db-core/src/workspace_migrations.rs:247), `WorkspaceMigrationError::DirEntry` (crates/causeway-db-core/src/workspace_migrations.rs:250)  
tests 3: `tests::a_staged_layout_without_components_is_the_baseline_alone` (crates/causeway-db-core/src/workspace_migrations.rs:651), `tests::the_staged_layout_is_the_baseline_then_components_by_suffix` (crates/causeway-db-core/src/workspace_migrations.rs:615), `tests::a_layout_without_a_baseline_is_refused` (crates/causeway-db-core/src/workspace_migrations.rs:658)  
uses_types 2: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:235), `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:235)  

### 52. Function check_schema
`rust:crates/causeway-db-core/src/workspace_migrations.rs::check_schema`  crates/causeway-db-core/src/workspace_migrations.rs:531-546  pkg:cargo/causeway-db-core  score 0.53  via `ensure_schema` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:552, 1.00)  
`fn check_schema(schema: &str) -> WorkspaceMigrationResult<()>`  
Refuses a schema the runner must not manage or cannot quote safely.  
callers 4: `tests::protected_and_unquotable_schemas_are_refused` (crates/causeway-db-core/src/workspace_migrations.rs:797), `ensure_schema` (crates/causeway-db-core/src/workspace_migrations.rs:552), `tests::protected_and_unquotable_schemas_are_refused` (crates/causeway-db-core/src/workspace_migrations.rs:803) … 1 more  
tests 1: `tests::protected_and_unquotable_schemas_are_refused` (crates/causeway-db-core/src/workspace_migrations.rs:797)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:531)  

### 53. Package cargo/tempfile
`pkg:cargo/tempfile`  pkg:cargo/tempfile  score 0.53  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 35: `cargo/causeway-metadata-admin-host` (apps/causeway-metadata-admin-host/Cargo.toml:57), `cargo/platform-flags` (crates/platform-flags/Cargo.toml:1-33), `cargo/causeway-code` (crates/causeway-code/Cargo.toml:1-37) … 32 more  
importers 24: `crates/causeway-testkit/tests/postgres.rs` (crates/causeway-testkit/tests/postgres.rs:24), `crates/causeway-principals/tests/repository_personas.rs` (crates/causeway-principals/tests/repository_personas.rs:18-21), `bootstrap/cw-vsa/src/main.rs` (bootstrap/cw-vsa/src/main.rs:18) … 21 more  

### 54. Package cargo/toml
`pkg:cargo/toml`  pkg:cargo/toml  score 0.53  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 20: `cargo/workflows-component` (component/orchestration/workflows/Cargo.toml:1-93), `cargo/flightdeck-auth` (crates/flightdeck-auth/Cargo.toml:29), `cargo/governance-policy-component` (component/governance/policy/Cargo.toml:1-59) … 17 more  
importers 8: `component/governance/policy/tests/phase3.rs` (component/governance/policy/tests/phase3.rs:41), `services/flightdeck/tests/content_test.rs` (services/flightdeck/tests/content_test.rs:48), `component/governance/policy/tests/phase4.rs` (component/governance/policy/tests/phase4.rs:45) … 5 more  

### 55. Package cargo/tracing
`pkg:cargo/tracing`  pkg:cargo/tracing  score 0.53  via `cargo/causeway-db-core` -> DEPENDS_ON (crates/causeway-db-core/Cargo.toml:1-23, 1.00)  
dependents 73: `cargo/causeway-capability` (crates/causeway-capability/Cargo.toml:1-24), `cargo/causeway-sniffer` (crates/causeway-sniffer/Cargo.toml:1-46), `cargo/catalog-query-understanding` (component/discovery/search/query-understanding/Cargo.toml:1-50) … 70 more  
importers 40: `torrentfx/crates/node-flight/src/trace.rs` (torrentfx/crates/node-flight/src/trace.rs:18), `apps/causeway-server-web/src/content/docs.rs` (apps/causeway-server-web/src/content/docs.rs:8), `torrentfx/crates/causeway-relay/src/app.rs` (torrentfx/crates/causeway-relay/src/app.rs:48) … 37 more  

### 56. Variant Step::Migrate
`rust:jobs/causeway-migrate/src/main.rs::Step::Migrate`  jobs/causeway-migrate/src/main.rs:48  pkg:cargo/causeway-migrate  score 0.53  via `MigrationMode` <- USES_TYPE (jobs/causeway-migrate/src/main.rs:48, 1.00)  
`Migrate`  
Migrate, or with [`MigrationMode::Check`] only count, then (doc continues, 95 bytes)  
referenced_by 1: `main` (jobs/causeway-migrate/src/main.rs:127)  
uses_types 1: `MigrationMode` (jobs/causeway-migrate/src/main.rs:48)  

### 57. Method RolePlan::new
`rust:crates/causeway-db-core/src/slice_roles.rs::RolePlan::new`  crates/causeway-db-core/src/slice_roles.rs:370-428  pkg:cargo/causeway-db-core  score 0.53  via `MigrationTarget` <- USES_TYPE (crates/causeway-db-core/src/slice_roles.rs:371, 1.00)  
`pub fn new( targets: &[MigrationTarget], declarations: &RoleDeclarations, ) -> SliceRoleResult<Self>`  
The roles for the workspace whose schemas are `targets`, the (doc continues, 289 bytes)  
callers 6: `tests::the_repository_declarations_name_real_slices` (crates/causeway-db-core/src/slice_roles.rs:1130), `provisioning_revokes_what_the_declarations_no_longer_list` (crates/causeway-testkit/tests/postgres.rs:373), `tests::a_workload_inherits_the_slices_it_names` (crates/causeway-db-core/src/slice_roles.rs:1014) … 3 more  
calls 13: `SliceRole::slice` (crates/causeway-db-core/src/slice_roles.rs:382), `SliceRole::slice` (crates/causeway-db-core/src/slice_roles.rs:378), `RoleName::new` (crates/causeway-db-core/src/slice_roles.rs:377) … 10 more  
tests 4: `tests::the_repository_declarations_name_real_slices` (crates/causeway-db-core/src/slice_roles.rs:1130), `provisioning_revokes_what_the_declarations_no_longer_list` (crates/causeway-testkit/tests/postgres.rs:373), `tests::a_workload_inherits_the_slices_it_names` (crates/causeway-db-core/src/slice_roles.rs:1014) … 1 more  
uses_types 6: `RoleDeclarations` (crates/causeway-db-core/src/slice_roles.rs:372), `RoleName` (crates/causeway-db-core/src/slice_roles.rs:396), `SliceRoleResult` (crates/causeway-db-core/src/slice_roles.rs:373) … 3 more  

### 58. Task deploy:server
`task:mise/deploy:server`  .mise/toml/deploy.toml:164-178  pkg:npm/causeway-monorepo  score 0.51  via `mise/db:migrate` <- DEPENDS_ON (.mise/toml/deploy.toml:166, 1.00)  
`mise run deploy:server`  
Build server image + rollout causeway-server. Auto-detects Colima (macOS) vs k3d (Linux). Builds for $TARGET_ARCH (default arm64 on Apple Silicon): cross-compiles the binaries on the host via cargo-zigbuild behind the compile cache CAUSEWAY_CARGO_CACHE selects, mbx by default (deploy:stage:server), then packages them into the thin runtime image (Dockerfile.runtime). No in-container compilation.  
depends: db:migrate,deploy:causeway-controller  
description: Build server image + rollout causeway-server. Auto-detects Colima (macOS) vs k3d (Linux). Builds for $TARGET_ARCH (default arm64 on Apple Silicon): cross-compiles the binaries on the host via cargo-zigbuild behind the compile cache CAUSEWAY_CARGO_CACHE selects, mbx by default (deploy:stage:server), then packages them into the thin runtime image (Dockerfile.runtime). No in-container compilation.  
run: .mise/scripts/tasks/deploy/server.rhai  
dependencies 2: `mise/db:migrate` (.mise/toml/deploy.toml:166), `mise/deploy:causeway-controller` (.mise/toml/deploy.toml:166)  
runs 1: `.mise/scripts/tasks/deploy/server.rhai` (.mise/toml/deploy.toml:177)  

### 59. Task deploy:sidecar
`task:mise/deploy:sidecar`  .mise/toml/deploy.toml:195-209  pkg:npm/causeway-monorepo  score 0.51  via `mise/db:migrate` <- DEPENDS_ON (.mise/toml/deploy.toml:197, 1.00)  
`mise run deploy:sidecar`  
Build sidecar image + rollout causeway-explorer. Auto-detects Colima (macOS) vs k3d (Linux). Builds for $TARGET_ARCH.  
depends: db:migrate,deploy:causeway-controller  
description: Build sidecar image + rollout causeway-explorer. Auto-detects Colima (macOS) vs k3d (Linux). Builds for $TARGET_ARCH.  
run: .mise/scripts/tasks/deploy/sidecar.rhai  
dependencies 2: `mise/deploy:causeway-controller` (.mise/toml/deploy.toml:197), `mise/db:migrate` (.mise/toml/deploy.toml:197)  
runs 1: `.mise/scripts/tasks/deploy/sidecar.rhai` (.mise/toml/deploy.toml:208)  

### 60. Method InfraPlan::start
`rust:crates/causeway-testkit/src/infra.rs::InfraPlan::start`  crates/causeway-testkit/src/infra.rs:177-246  pkg:cargo/causeway-testkit  score 0.51  via `migrate` <- CALLS (crates/causeway-testkit/src/infra.rs:213, 1.00)  
`pub async fn start(self) -> TestkitResult<RunningInfra>`  
Starts the containers concurrently, migrates the workspace into (doc continues, 396 bytes)  
callers 1: `optional` (crates/causeway-testkit/src/infra.rs:255)  
calls 74: `RunIdentity::as_str` (crates/causeway-testkit/src/infra.rs:210), `RunIdentity::as_str` (crates/causeway-testkit/src/infra.rs:207), `report` (crates/causeway-testkit/src/infra.rs:207) … 71 more  
references 9: `Service::Qdrant` (crates/causeway-testkit/src/infra.rs:193), `Service::Valkey` (crates/causeway-testkit/src/infra.rs:191), `EphemeralKafka::start` (crates/causeway-testkit/src/infra.rs:190) … 6 more  
uses_types 3: `RunningInfra` (crates/causeway-testkit/src/infra.rs:177), `Started` (crates/causeway-testkit/src/infra.rs:238), `TestkitResult` (crates/causeway-testkit/src/infra.rs:177)  

### 61. Task deploy:search-sync-worker
`task:mise/deploy:search-sync-worker`  .mise/toml/deploy.toml:145-163  pkg:npm/causeway-monorepo  score 0.50  via `mise/db:migrate` <- DEPENDS_ON (.mise/toml/deploy.toml:147, 1.00)  
`mise run deploy:search-sync-worker`  
Build causeway-search-sync-worker image + apply the durable catalog search queue worker Deployment.  
depends: db:migrate,deploy:causeway-controller  
description: Build causeway-search-sync-worker image + apply the durable catalog search queue worker Deployment.  
run: .mise/scripts/tasks/deploy/search-sync-worker.rhai  
dependencies 2: `mise/deploy:causeway-controller` (.mise/toml/deploy.toml:147), `mise/db:migrate` (.mise/toml/deploy.toml:147)  
runs 1: `.mise/scripts/tasks/deploy/search-sync-worker.rhai` (.mise/toml/deploy.toml:162)  

### 62. Function tests::schemas
`rust:crates/causeway-db-core/src/workspace_migrations.rs::tests::schemas`  crates/causeway-db-core/src/workspace_migrations.rs:599-604  pkg:cargo/causeway-db-core  score 0.49  via `MigrationTarget::schema` <- CALLS (crates/causeway-db-core/src/workspace_migrations.rs:602, 1.00)  
`fn schemas(targets: &[MigrationTarget]) -> Vec<(&str, bool)>`  
callers 6: `tests::a_package_that_declares_a_slice_migrates_into_its_schema` (crates/causeway-db-core/src/workspace_migrations.rs:737), `tests::the_repository_layout_names_schemas_as_the_staging_script_does` (crates/causeway-db-core/src/workspace_migrations.rs:688), `tests::the_staged_layout_is_the_baseline_then_components_by_suffix` (crates/causeway-db-core/src/workspace_migrations.rs:617) … 3 more  
calls 2: `MigrationTarget::schema` (crates/causeway-db-core/src/workspace_migrations.rs:602), `MigrationTarget::ignores_missing` (crates/causeway-db-core/src/workspace_migrations.rs:602)  
tests 4: `tests::the_repository_layout_names_schemas_as_the_staging_script_does` (crates/causeway-db-core/src/workspace_migrations.rs:688), `tests::a_package_that_declares_a_slice_migrates_into_its_schema` (crates/causeway-db-core/src/workspace_migrations.rs:737), `tests::the_staged_layout_is_the_baseline_then_components_by_suffix` (crates/causeway-db-core/src/workspace_migrations.rs:617) … 1 more  
uses_types 1: `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:599)  

### 63. File jobs/causeway-migrate/src/main.rs
`file:jobs/causeway-migrate/src/main.rs`  jobs/causeway-migrate/src/main.rs  pkg:cargo/causeway-migrate  score 0.48  via `main` <- CONTAINS (jobs/causeway-migrate/src/main.rs:97-185, 1.00)  
members 20  
designs 2: `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md` (docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md:636), `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:472)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2492)  
imports 19: `PostgresTopology` (jobs/causeway-migrate/src/main.rs:35-38), `cargo/causeway-db-core` (jobs/causeway-migrate/src/main.rs:32), `Environment` (jobs/causeway-migrate/src/main.rs:35-38) … 16 more  

### 64. Test tests::the_flags_parse_into_a_step_and_its_directories
`rust:jobs/causeway-migrate/src/main.rs::tests::the_flags_parse_into_a_step_and_its_directories`  jobs/causeway-migrate/src/main.rs:392-438  pkg:cargo/causeway-migrate  score 0.48  via `MigrationMode::Apply` <- REFERENCES (jobs/causeway-migrate/src/main.rs:396, 1.00)  
`fn the_flags_parse_into_a_step_and_its_directories()`  
calls 4: `tests::args` (jobs/causeway-migrate/src/main.rs:436), `tests::args` (jobs/causeway-migrate/src/main.rs:394), `tests::args` (jobs/causeway-migrate/src/main.rs:402) … 1 more  
references 3: `Step::RolesOnly` (jobs/causeway-migrate/src/main.rs:415), `MigrationMode::Check` (jobs/causeway-migrate/src/main.rs:403), `MigrationMode::Apply` (jobs/causeway-migrate/src/main.rs:396)  
tests_for 1: `tests::args` (jobs/causeway-migrate/src/main.rs:394)  
uses_types 1: `Args` (jobs/causeway-migrate/src/main.rs:395)  

### 65. Method MigrationDatabaseError::message
`typescript:torrentfx/packages-node/causeway-data/src/errors.ts::MigrationDatabaseError::message`  torrentfx/packages-node/causeway-data/src/errors.ts:97-100  pkg:npm/@causeway/data  score 0.48  via `MigrationDatabaseError` -> DEFINES (torrentfx/packages-node/causeway-data/src/errors.ts:97-100, 1.00)  
`override get message(): string`  

### 66. Variant Step::RolesOnly
`rust:jobs/causeway-migrate/src/main.rs::Step::RolesOnly`  jobs/causeway-migrate/src/main.rs:50  pkg:cargo/causeway-migrate  score 0.48  via `main` -> REFERENCES (jobs/causeway-migrate/src/main.rs:128, 1.00)  
`RolesOnly`  
Provision the roles of a workspace already migrated.  
referenced_by 4: `tests::the_flags_parse_into_a_step_and_its_directories` (jobs/causeway-migrate/src/main.rs:415), `main` (jobs/causeway-migrate/src/main.rs:128), `parse_args` (jobs/causeway-migrate/src/main.rs:85) … 1 more  

### 67. File crates/causeway-testkit/src/infra.rs
`file:crates/causeway-testkit/src/infra.rs`  crates/causeway-testkit/src/infra.rs  pkg:cargo/causeway-testkit  score 0.47  via `migrate` <- CONTAINS (crates/causeway-testkit/src/infra.rs:270-299, 1.00)  
members 20  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2667)  
imports 18: `LocalTopology` (crates/causeway-testkit/src/infra.rs:43), `TestkitError` (crates/causeway-testkit/src/infra.rs:36), `PostgresSecret` (crates/causeway-testkit/src/infra.rs:43) … 15 more  

### 68. Function migrate
`rhai:.mise/rhai/causeway/devdata.rhai::migrate`  .mise/rhai/causeway/devdata.rhai:64-68  pkg:npm/causeway-monorepo  score 0.43  via `mise/db:migrate` <- RUNS (.mise/rhai/causeway/devdata.rhai:67, 0.75)  
`fn migrate(repo, vars)`  
Run db:migrate as a step of another task. RESET_DB is cleared for it: the variable asks `mise run db:migrate` to drop the workspace first, and a value left in the caller's environment must not make every task that migrates on its way do the same.  
callers 1: `ensure_env` (.mise/rhai/causeway/devdata.rhai:137)  
calls 1: `task_run` (.mise/rhai/causeway/devdata.rhai:67)  
runs 1: `mise/db:migrate` (.mise/rhai/causeway/devdata.rhai:67)  

### 69. File .mise/scripts/tasks/db/migrate.rhai
`file:.mise/scripts/tasks/db/migrate.rhai`  .mise/scripts/tasks/db/migrate.rhai  pkg:npm/causeway-monorepo  score 0.42  via `mise/db:migrate` -> RUNS (.mise/toml/db.toml:6, 1.00)  
members 3  
calls 38: `copy` (.mise/scripts/tasks/db/migrate.rhai:127), `flag` (.mise/scripts/tasks/db/migrate.rhai:53), `kube_context` (.mise/scripts/tasks/db/migrate.rhai:57) … 35 more  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:472)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2520)  
run_by 1: `mise/db:migrate` (.mise/toml/db.toml:6)  

### 70. Method RoleDeclarations::load
`rust:crates/causeway-db-core/src/slice_roles.rs::RoleDeclarations::load`  crates/causeway-db-core/src/slice_roles.rs:225-240  pkg:cargo/causeway-db-core  score 0.41  via `MigrationTarget` <- USES_TYPE (crates/causeway-db-core/src/slice_roles.rs:225, 1.00)  
`pub fn load(workloads_file: &Path, targets: &[MigrationTarget]) -> SliceRoleResult<Self>`  
Reads `workloads_file` and each target's published-views file, when (doc continues, 234 bytes)  
callers 17: `main` (jobs/causeway-migrate/src/main.rs:124), `TestWorkspace::prepare` (crates/causeway-testkit/src/workspace.rs:97), `tests::the_repository_declarations_name_real_slices` (crates/causeway-db-core/src/slice_roles.rs:1128) … 14 more  
calls 11: `read` (crates/causeway-db-core/src/slice_roles.rs:226), `SliceRole::slice` (crates/causeway-db-core/src/slice_roles.rs:231), `read` (crates/causeway-db-core/src/slice_roles.rs:231) … 8 more  
tests 9: `tests::the_repository_declarations_name_real_slices` (crates/causeway-db-core/src/slice_roles.rs:1128), `tests::contract_refs_resolve_all_identities_in_one_batch` (component/governance/contracts/src/rpc.rs:763), `tests::query_embedding_cache_coalesces_concurrent_misses` (component/discovery/search/core/src/lib.rs:4003) … 6 more  
uses_types 2: `MigrationTarget` (crates/causeway-db-core/src/slice_roles.rs:225), `SliceRoleResult` (crates/causeway-db-core/src/slice_roles.rs:225)  

### 71. File torrentfx/packages-node/causeway-data/src/migrations.ts
`file:torrentfx/packages-node/causeway-data/src/migrations.ts`  torrentfx/packages-node/causeway-data/src/migrations.ts  pkg:npm/@causeway/data  score 0.40  via `MigrationDatabaseError` <- IMPORTS (torrentfx/packages-node/causeway-data/src/migrations.ts:40-49, 1.00)  
members 17  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1321)  
imports 14: `MigrationDatabaseError` (torrentfx/packages-node/causeway-data/src/migrations.ts:40-49), `MigrationError` (torrentfx/packages-node/causeway-data/src/migrations.ts:40-49), `npm/pg` (torrentfx/packages-node/causeway-data/src/migrations.ts:38) … 11 more  

### 72. Package cargo/causeway-server
`pkg:cargo/causeway-server`  apps/causeway-server/Cargo.toml:2  pkg:cargo/causeway-server  score 0.40  via `cargo/temp-env` <- DEPENDS_ON (apps/causeway-server/Cargo.toml:190, 1.00)  
`cargo/causeway-server`  
members 296  
dependencies 126: `cargo/object_store` (apps/causeway-server/Cargo.toml:140), `cargo/clap` (apps/causeway-server/Cargo.toml:1-225), `cargo/temp-env` (apps/causeway-server/Cargo.toml:190) … 123 more  
designs 4: `docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md` (docs/adrs/ADR-nbmt040e-cross-slice-read-boundaries.md:19), `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:322), `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:14) … 1 more  
documents 7: `docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md` (docs/archive/requirements/REQ-011-rust-binary-classification-and-target-workspace-layout.md:78), `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:43), `docs/plans/RESTATE-REMEDIATION-WORKPLAN.md` (docs/plans/RESTATE-REMEDIATION-WORKPLAN.md:553) … 4 more  
importers 1: `apps/causeway-server/src/main.rs` (apps/causeway-server/src/main.rs:4)  

### 73. Variant WorkspaceMigrationError::Connect
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::Connect`  crates/causeway-db-core/src/workspace_migrations.rs:93  pkg:cargo/causeway-db-core  score 0.38  via `connect` -> REFERENCES (crates/causeway-db-core/src/workspace_migrations.rs:527, 1.00)  
`Connect`  
The database could not be reached.  
referenced_by 1: `connect` (crates/causeway-db-core/src/workspace_migrations.rs:527)  
uses_types 5: `Audience::Error` (crates/causeway-db-core/src/workspace_migrations.rs:93), `CatalogServiceTopology::Error` (crates/causeway-db-core/src/workspace_migrations.rs:93), `WorkspaceScope::Error` (crates/causeway-db-core/src/workspace_migrations.rs:93) … 2 more  

### 74. Function package_slices
`rust:crates/causeway-db-core/src/workspace_migrations.rs::package_slices`  crates/causeway-db-core/src/workspace_migrations.rs:316-331  pkg:cargo/causeway-db-core  score 0.38  via `WorkspaceMigrationResult` <- USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:316, 1.00)  
`fn package_slices(repo_root: &Path) -> WorkspaceMigrationResult<Vec<(String, PathBuf)>>`  
The slices packages declare with a [`SLICE_MANIFEST`]: each one's schema (doc continues, 113 bytes)  
callers 2: `repository_targets` (crates/causeway-db-core/src/workspace_migrations.rs:305), `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:785)  
calls 2: `slice_suffix` (crates/causeway-db-core/src/workspace_migrations.rs:326), `subdirectories` (crates/causeway-db-core/src/workspace_migrations.rs:323)  
tests 1: `tests::the_repository_targets_cover_every_component_migration_directory` (crates/causeway-db-core/src/workspace_migrations.rs:785)  
uses_types 1: `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:316)  

### 75. Function run
`rust:crates/causeway-db-core/src/slice_roles.rs::run`  crates/causeway-db-core/src/slice_roles.rs:936-945  pkg:cargo/causeway-db-core  score 0.38  via `migrate_workspace` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:486, 0.75)  
`async fn run(conn: &mut PgConnection, sql: &str) -> SliceRoleResult<()>`  
callers 16: `revoke_stale_grants` (crates/causeway-db-core/src/slice_roles.rs:843), `provision_roles` (crates/causeway-db-core/src/slice_roles.rs:546), `revoke_stale_memberships` (crates/causeway-db-core/src/slice_roles.rs:893) … 13 more  
uses_types 2: `SliceRoleResult` (crates/causeway-db-core/src/slice_roles.rs:936), `Statement` (crates/causeway-db-core/src/slice_roles.rs:941)  

### 76. Method DirectoryPath::as_path
`rust:torrentfx/crates/causeway-platform/src/config.rs::DirectoryPath::as_path`  torrentfx/crates/causeway-platform/src/config.rs:1356-1358  pkg:cargo/causeway-platform  score 0.38  via `migrate_workspace` -> CALLS (crates/causeway-db-core/src/workspace_migrations.rs:472, 0.75)  
`pub fn as_path(&self) -> &Path`  
The directory.  
callers 19: `tests::in_local_a_relative_directory_is_taken_from_the_topology_file` (torrentfx/crates/causeway-platform/src/config.rs:4362), `ResolvedConfig::from_topology_file` (torrentfx/crates/causeway-platform/src/config.rs:1601), `migrate_workspace` (crates/causeway-db-core/src/workspace_migrations.rs:472) … 16 more  
tests 10: `tests::in_local_a_relative_directory_is_taken_from_the_topology_file` (torrentfx/crates/causeway-platform/src/config.rs:4362), `tests::development_reviewer_is_authorized_by_the_real_cedar_bundle` (jobs/causeway-seed/src/main.rs:2242), `tests::a_package_that_declares_a_slice_migrates_into_its_schema` (crates/causeway-db-core/src/workspace_migrations.rs:750) … 7 more  
torrentfx/crates/causeway-platform/src/config.rs:1356-1358
```rust
    pub fn as_path(&self) -> &Path {
        &self.0
    }
```

### 77. Function slice_suffix
`rust:crates/causeway-db-core/src/workspace_migrations.rs::slice_suffix`  crates/causeway-db-core/src/workspace_migrations.rs:335-359  pkg:cargo/causeway-db-core  score 0.38  via `WorkspaceMigrationResult` <- USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:335, 1.00)  
`fn slice_suffix(manifest: &Path) -> WorkspaceMigrationResult<String>`  
The schema suffix a slice manifest names: `microsite/search` is (doc continues, 126 bytes)  
members 1  
callers 1: `package_slices` (crates/causeway-db-core/src/workspace_migrations.rs:326)  
uses_types 2: `slice_suffix::Manifest` (crates/causeway-db-core/src/workspace_migrations.rs:346), `WorkspaceMigrationResult` (crates/causeway-db-core/src/workspace_migrations.rs:335)  

### 78. Package cargo/causeway-testkit
`pkg:cargo/causeway-testkit`  crates/causeway-testkit/Cargo.toml:2  pkg:cargo/causeway-testkit  score 0.36  via `cargo/causeway-workspace-scope` <- DEPENDS_ON (crates/causeway-testkit/Cargo.toml:25, 1.00)  
`cargo/causeway-testkit`  
members 26  
dependents 18: `cargo/contracts-component` (component/governance/contracts/Cargo.toml:1-70), `cargo/causeway-migrate` (jobs/causeway-migrate/Cargo.toml:22), `cargo/pure-component` (component/semantic/pure/Cargo.toml:1-60) … 15 more  
dependencies 16: `cargo/serde_json` (crates/causeway-testkit/Cargo.toml:1-87), `cargo/sqlx` (crates/causeway-testkit/Cargo.toml:1-87), `cargo/object_store` (crates/causeway-testkit/Cargo.toml:1-87) … 13 more  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:181)  
importers 34: `tests` (apps/causeway-policy-scan/src/main.rs:2122), `tests` (component/semantic/pure/src/elements.rs:1068), `component/governance/policy/tests/phase3.rs` (component/governance/policy/tests/phase3.rs:16) … 31 more  

### 79. Method TargetReport::target
`rust:crates/causeway-db-core/src/workspace_migrations.rs::TargetReport::target`  crates/causeway-db-core/src/workspace_migrations.rs:412-414  pkg:cargo/causeway-db-core  score 0.36  via `MigrationTarget` <- USES_TYPE (crates/causeway-db-core/src/workspace_migrations.rs:412, 1.00)  
`pub const fn target(&self) -> &MigrationTarget`  
The schema and its migration set.  
callers 3: `main` (jobs/causeway-migrate/src/main.rs:140), `tests::a_declared_volume_mounts_a_capped_tmpfs_at_its_path` (crates/causeway-testkit/src/scratch.rs:134), `unresolved_references` (component/governance/core/src/rights.rs:687)  
tests 1: `tests::a_declared_volume_mounts_a_capped_tmpfs_at_its_path` (crates/causeway-testkit/src/scratch.rs:134)  
uses_types 1: `MigrationTarget` (crates/causeway-db-core/src/workspace_migrations.rs:412)  

### 80. Variant WorkspaceMigrationError::Url
`rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::Url`  crates/causeway-db-core/src/workspace_migrations.rs:89  pkg:cargo/causeway-db-core  score 0.36  via `connect` -> REFERENCES (crates/causeway-db-core/src/workspace_migrations.rs:523, 1.00)  
`Url`  
The database URL does not parse.  
referenced_by 1: `connect` (crates/causeway-db-core/src/workspace_migrations.rs:523)  
uses_types 5: `ServiceEndpointTopology::Error` (crates/causeway-db-core/src/workspace_migrations.rs:89), `WebOrigin::Error` (crates/causeway-db-core/src/workspace_migrations.rs:89), `CatalogServiceTopology::Error` (crates/causeway-db-core/src/workspace_migrations.rs:89) … 2 more  

### 81. Package cargo/causeway-runtime
`pkg:cargo/causeway-runtime`  crates/causeway-runtime/Cargo.toml:2  pkg:cargo/causeway-runtime  score 0.36  via `cargo/temp-env` <- DEPENDS_ON (crates/causeway-runtime/Cargo.toml:1-57, 1.00)  
`cargo/causeway-runtime`  
members 10  
dependents 29: `cargo/causeway-code` (crates/causeway-code/Cargo.toml:14), `cargo/atlas` (apps/atlas/Cargo.toml:25), `cargo/marketplace-component` (component/marketplace/core/Cargo.toml:12) … 26 more  
dependencies 24: `cargo/tokio` (crates/causeway-runtime/Cargo.toml:1-57), `cargo/url` (crates/causeway-runtime/Cargo.toml:1-57), `cargo/sqlx` (crates/causeway-runtime/Cargo.toml:1-57) … 21 more  
designs 2: `docs/rfd/RFD-p22b4acw-developer-workspaces-and-state-isolation.md` (docs/rfd/RFD-p22b4acw-developer-workspaces-and-state-isolation.md:80), `docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md` (docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md:98)  
documents 1: `docs/METADATA-INGESTION-PATH.md` (docs/METADATA-INGESTION-PATH.md:6)  
importers 51: `crates/causeway-code-mirror-sync/src/cli.rs` (crates/causeway-code-mirror-sync/src/cli.rs:8), `crates/causeway-code/src/mirror.rs` (crates/causeway-code/src/mirror.rs:14), `component/marketplace/core/src/main.rs` (component/marketplace/core/src/main.rs:44) … 48 more  

### 82. Package cargo/causeway-metadata-admin-host
`pkg:cargo/causeway-metadata-admin-host`  apps/causeway-metadata-admin-host/Cargo.toml:2  pkg:cargo/causeway-metadata-admin-host  score 0.35  via `cargo/temp-env` <- DEPENDS_ON (apps/causeway-metadata-admin-host/Cargo.toml:59, 1.00)  
`cargo/causeway-metadata-admin-host`  
members 8  
dependents 1: `cargo/causeway-server` (apps/causeway-server/Cargo.toml:59)  
dependencies 35: `cargo/tempfile` (apps/causeway-metadata-admin-host/Cargo.toml:57), `cargo/connect-server-ext` (apps/causeway-metadata-admin-host/Cargo.toml:35), `cargo/contracts-component` (apps/causeway-metadata-admin-host/Cargo.toml:19) … 32 more  
designs 1: `docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md` (docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md:291)  
documents 1: `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:471)  
importers 3: `apps/causeway-metadata-admin-host/src/bin/causeway-metadata-admin-host.rs` (apps/causeway-metadata-admin-host/src/bin/causeway-metadata-admin-host.rs:20-24), `apps/causeway-metadata-admin-host/tests/auth_requirement.rs` (apps/causeway-metadata-admin-host/tests/auth_requirement.rs:23), `apps/causeway-server/src/metadata_admin/auth.rs` (apps/causeway-server/src/metadata_admin/auth.rs:7)  

### 83. Impl TestWorkspace#impl
`rust:crates/causeway-testkit/src/workspace.rs::TestWorkspace#impl`  crates/causeway-testkit/src/workspace.rs:53-344  pkg:cargo/causeway-testkit  score 0.35  via `TestWorkspace::prepare` <- DEFINES (crates/causeway-testkit/src/workspace.rs:84-153, 1.00)  
`impl TestWorkspace`  
members 18  
uses_types 1: `TestWorkspace` (crates/causeway-testkit/src/workspace.rs:53-344)  

### 84. Package cargo/flightdeck-server
`pkg:cargo/flightdeck-server`  services/flightdeck/Cargo.toml:2  pkg:cargo/flightdeck-server  score 0.35  via `cargo/temp-env` <- DEPENDS_ON (services/flightdeck/Cargo.toml:90, 1.00)  
`cargo/flightdeck-server`  
members 28  
dependents 3: `cargo/pure-flight-server` (services/pure-flight/Cargo.toml:49), `cargo/pure-flight-server` (services/pure-flight/Cargo.toml:29), `cargo/flightdeck-server` (services/flightdeck/Cargo.toml:75)  
dependencies 69: `cargo/drive-manifest` (services/flightdeck/Cargo.toml:67), `cargo/tonic-health` (services/flightdeck/Cargo.toml:40), `cargo/reqwest` (services/flightdeck/Cargo.toml:42) … 66 more  
documents 1: `docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md` (docs/plans/SERVICE-BOUNDARIES-WORKPLAN.md:43)  
importers 6: `services/flightdeck/src/main.rs` (services/flightdeck/src/main.rs:38), `services/pure-flight/src/main.rs` (services/pure-flight/src/main.rs:43), `services/flightdeck/tests/common/mod.rs` (services/flightdeck/tests/common/mod.rs:28) … 3 more  

### 85. TypeAlias TestkitResult
`rust:crates/causeway-testkit/src/error.rs::TestkitResult`  crates/causeway-testkit/src/error.rs:161  pkg:cargo/causeway-testkit  score 0.35  via `migrate` -> USES_TYPE (crates/causeway-testkit/src/infra.rs:270, 1.00)  
`pub type TestkitResult<T> = std::result::Result<T, TestkitError>;`  
Result of a testkit operation.  
importers 13: `crates/causeway-testkit/src/containers.rs` (crates/causeway-testkit/src/containers.rs:16), `crates/causeway-testkit/src/qdrant.rs` (crates/causeway-testkit/src/qdrant.rs:24), `crates/causeway-testkit/src/infra.rs` (crates/causeway-testkit/src/infra.rs:36) … 10 more  
used_by 70: `PinnedImage::ensure_present` (crates/causeway-testkit/src/images.rs:87), `drop_workspace` (crates/causeway-testkit/src/workspace.rs:420), `EphemeralPostgres::start` (crates/causeway-testkit/src/postgres.rs:75) … 67 more  
uses_types 1: `TestkitError` (crates/causeway-testkit/src/error.rs:161)  

### 86. Struct WorkspaceScope
`rust:torrentfx/crates/causeway-workspace-scope/src/lib.rs::WorkspaceScope`  torrentfx/crates/causeway-workspace-scope/src/lib.rs:119  pkg:cargo/causeway-workspace-scope  score 0.35  via `migrate` -> USES_TYPE (crates/causeway-testkit/src/infra.rs:270, 1.00)  
`pub struct WorkspaceScope`  
A validated workspace id and the names derived from it. (doc continues, 199 bytes)  
members 19  
importers 16: `torrentfx/crates/causeway-workspace-scope/src/companion.rs` (torrentfx/crates/causeway-workspace-scope/src/companion.rs:19), `crates/causeway-testkit/src/isolation.rs` (crates/causeway-testkit/src/isolation.rs:18), `crates/causeway-testkit/src/valkey.rs` (crates/causeway-testkit/src/valkey.rs:11) … 13 more  
used_by 94: `String::from` (torrentfx/crates/causeway-workspace-scope/src/lib.rs:325), `SessionKeyspace::for_workspace` (apps/causeway-server/src/cache.rs:237), `term_governance_resolution_service_name_for` (component/governance/resolution/src/commands.rs:295) … 91 more  

## Left out

20 included nodes have no source window for want of budget.  
- `rust:jobs/causeway-migrate/src/main.rs::layout` Function 0.34 (over budget, 267 tokens)
- `rust:crates/causeway-db-core/src/workspace_migrations.rs::WorkspaceMigrationError::WorkspaceUuid` Variant 0.34 (over budget, 217 tokens)
- `pkg:cargo/causeway-seed` Package 0.33 (over budget, 163 tokens)
- and 1442 more

## Unresolved references in the seeds' files

- `is_dir` at crates/causeway-db-core/src/workspace_migrations.rs:237 (no-declaration)
- `is_dir` at crates/causeway-db-core/src/workspace_migrations.rs:245 (no-declaration)
- `file_type` at crates/causeway-db-core/src/workspace_migrations.rs:249 (no-declaration)
- `is_dir` at crates/causeway-db-core/src/workspace_migrations.rs:251 (no-declaration)
- `file_name` at crates/causeway-db-core/src/workspace_migrations.rs:254 (no-declaration)
- `into_string` at crates/causeway-db-core/src/workspace_migrations.rs:255 (ambiguous-receiver)
- `is_dir` at crates/causeway-db-core/src/workspace_migrations.rs:287 (no-declaration)
- `with_file_name` at crates/causeway-db-core/src/workspace_migrations.rs:292 (no-declaration)
- `is_dir` at crates/causeway-db-core/src/workspace_migrations.rs:299 (no-declaration)
- `with_file_name` at crates/causeway-db-core/src/workspace_migrations.rs:308 (no-declaration)

