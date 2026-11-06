# Context pack

Task: flightdeck's PostgreSQL remote connects without TLS, and the password it sends is the caller's delegated credential, so any DSN that leaves the host sends it in cleartext. Parse the DSN when the remote config loads, honour libpq's sslmode values, require TLS for any host that is not local, and keep the plain path for the local test database.

## Seeds

- `rust:bootstrap/cw-host/src/modules/pg.rs::dsn_config` Function 0.90 (text)
- `rust:services/flightdeck/src/catalog_refresh.rs::CatalogTableFormat::PostgresRemote` Variant 0.62 (text)
- `rust:bootstrap/cw-host/src/modules/pg.rs::connect_dsn` Function 0.47 (text)
- `rust:services/flightdeck/src/remote.rs::RemoteSource::PostgreSql` Variant 0.45 (text)
- `rust:services/flightdeck/src/remote.rs::RemoteConfig::load` Method 0.44 (text)
- `rust:services/flightdeck/src/remote.rs::PostgreSqlSource` Struct 0.44 (text)
- `rhai:.mise/rhai/causeway/cluster_preset_test.rhai::test_a_preset_naming_a_remote_host_still_needs_a_local_context` Function 0.42 (text)
- `rust:crates/catalog-domain/src/binding.rs::StoreKind::Postgresql` Variant 0.40 (identifier `PostgreSQL`)
- `rust:torrentfx/crates/causeway-platform/src/config.rs::ELASTICACHE_WITHOUT_TLS` Const 0.40 (text)
- `rust:services/flightdeck/src/remote.rs::send_postgres_batch` Function 0.40 (text)
- `rust:crates/causeway-testkit/src/topology.rs::PostgresSecret::LocalInline` Variant 0.39 (text)

Budget 30000 tokens, used 29910. Seeds 11, candidates 404, included 111, left out 293.

## Context

### 1. Function dsn_config
`rust:bootstrap/cw-host/src/modules/pg.rs::dsn_config`  bootstrap/cw-host/src/modules/pg.rs:62-81  pkg:cargo/cw-host  score 3.01  
`fn dsn_config(dsn: &str, pgpassword: Option<String>) -> Res<tokio_postgres::Config>`  
A connection string as libpq takes it, a `postgres://` URL (percent- (doc continues, 314 bytes)  
callers 6: `tests::a_url_without_a_password_takes_pgpassword` (bootstrap/cw-host/src/modules/pg.rs:320), `tests::a_url_is_percent_decoded_and_keeps_its_timeout` (bootstrap/cw-host/src/modules/pg.rs:304), `tests::tls_and_garbage_are_refused` (bootstrap/cw-host/src/modules/pg.rs:329) … 3 more  
calls 2: `err` (bootstrap/cw-host/src/modules/pg.rs:65), `err` (bootstrap/cw-host/src/modules/pg.rs:78)  
importers 1: `tests` (bootstrap/cw-host/src/modules/pg.rs:298)  
tests 3: `tests::tls_and_garbage_are_refused` (bootstrap/cw-host/src/modules/pg.rs:329), `tests::a_url_is_percent_decoded_and_keeps_its_timeout` (bootstrap/cw-host/src/modules/pg.rs:304), `tests::a_url_without_a_password_takes_pgpassword` (bootstrap/cw-host/src/modules/pg.rs:320)  
uses_types 1: `Res` (bootstrap/cw-host/src/modules/pg.rs:62)  
bootstrap/cw-host/src/modules/pg.rs:62-81
```rust
fn dsn_config(dsn: &str, pgpassword: Option<String>) -> Res<tokio_postgres::Config> {
    let mut config: tokio_postgres::Config = dsn
        .parse()
        .or_else(|e| err(format!("bad postgres connection string: {e}")))?;
    if config.get_password().is_none()
        && let Some(password) = pgpassword
    {
        config.password(password);
    }
    if config.get_connect_timeout().is_none() {
        config.connect_timeout(std::time::Duration::from_secs(10));
    }
    if config.get_application_name().is_none() {
        config.application_name("cwsh");
    }
    if config.get_ssl_mode() == tokio_postgres::config::SslMode::Require {
        return err("sslmode=require: the task layer connects without TLS");
    }
    Ok(config)
}
```

### 2. Variant StoreKind::Postgresql
`rust:crates/catalog-domain/src/binding.rs::StoreKind::Postgresql`  crates/catalog-domain/src/binding.rs:19  pkg:cargo/catalog-domain  score 1.25  
`Postgresql`  
referenced_by 10: `SourceProfile::table_object` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:228), `SourceProfile::postgres` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:101), `SourceProfile::locator` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:155) … 7 more  

### 3. Function connect_dsn
`rust:bootstrap/cw-host/src/modules/pg.rs::connect_dsn`  bootstrap/cw-host/src/modules/pg.rs:83-85  pkg:cargo/cw-host  score 1.17  
`fn connect_dsn(dsn: &str) -> Res<Pg>`  
callers 1: `connect` (bootstrap/cw-host/src/modules/pg.rs:281)  
calls 2: `open` (bootstrap/cw-host/src/modules/pg.rs:84), `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:84)  
uses_types 2: `Pg` (bootstrap/cw-host/src/modules/pg.rs:83), `Res` (bootstrap/cw-host/src/modules/pg.rs:83)  
bootstrap/cw-host/src/modules/pg.rs:83-85
```rust
fn connect_dsn(dsn: &str) -> Res<Pg> {
    open(dsn_config(dsn, std::env::var("PGPASSWORD").ok())?)
}
```

### 4. Variant CatalogTableFormat::PostgresRemote
`rust:services/flightdeck/src/catalog_refresh.rs::CatalogTableFormat::PostgresRemote`  services/flightdeck/src/catalog_refresh.rs:61-63  pkg:cargo/flightdeck-server  score 1.12  
`PostgresRemote`  
A `PostgreSQL` table (`postgres://instance/database.schema.table`), (doc continues, 187 bytes)  

### 5. Variant RemoteSource::PostgreSql
`rust:services/flightdeck/src/remote.rs::RemoteSource::PostgreSql`  services/flightdeck/src/remote.rs:92  pkg:cargo/flightdeck-server  score 1.10  
`PostgreSql`  
referenced_by 6: `RemoteSource::default_mode` (services/flightdeck/src/remote.rs:110), `RemoteRuntime::execute` (services/flightdeck/src/remote.rs:452), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:392) … 3 more  
uses_types 1: `PostgreSqlSource` (services/flightdeck/src/remote.rs:92)  

### 6. File bootstrap/cw-host/src/modules/pg.rs
`file:bootstrap/cw-host/src/modules/pg.rs`  bootstrap/cw-host/src/modules/pg.rs  pkg:cargo/cw-host  score 1.07  via `dsn_config` <- CONTAINS (bootstrap/cw-host/src/modules/pg.rs:62-81, 1.00)  
members 18  
imports 10: `cargo/rhai` (bootstrap/cw-host/src/modules/pg.rs:27), `Res` (bootstrap/cw-host/src/modules/pg.rs:31), `opt_str` (bootstrap/cw-host/src/modules/pg.rs:31) … 7 more  

### 7. Struct PostgreSqlSource
`rust:services/flightdeck/src/remote.rs::PostgreSqlSource`  services/flightdeck/src/remote.rs:137-147  pkg:cargo/flightdeck-server  score 0.95  
`pub struct PostgreSqlSource`  
`PostgreSQL` remote source.  
used_by 3: `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:92), `execute_postgres` (services/flightdeck/src/remote.rs:751), `tests::runtime` (services/flightdeck/src/remote.rs:1341)  
uses_types 2: `RemoteExecutionMode` (services/flightdeck/src/remote.rs:141), `DownstreamTokenConfig` (services/flightdeck/src/remote.rs:144)  
services/flightdeck/src/remote.rs:137-147
```rust
pub struct PostgreSqlSource {
    pub name: String,
    pub dsn: String,
    #[serde(default = "default_remote_modes")]
    pub allowed_modes: Vec<RemoteExecutionMode>,
    #[serde(default = "default_translated_mode")]
    pub default_mode: RemoteExecutionMode,
    pub auth: DownstreamTokenConfig,
    #[serde(default = "default_remote_batch_rows")]
    pub batch_rows: usize,
}
```

### 8. Function err
`rust:bootstrap/cw-host/src/convert.rs::err`  bootstrap/cw-host/src/convert.rs:13-15  pkg:cargo/cw-host  score 0.88  via `dsn_config` -> CALLS (bootstrap/cw-host/src/modules/pg.rs:78, 1.00)  
`pub fn err<T>(msg: impl Into<String>) -> Res<T>`  
A runtime error carrying a plain message.  
callers 108: `param` (bootstrap/cw-host/src/modules/pg.rs:136), `send` (bootstrap/cw-host/src/modules/proc.rs:530), `param` (bootstrap/cw-host/src/modules/pg.rs:142) … 105 more  
importers 12: `bootstrap/cw-host/src/modules/re.rs` (bootstrap/cw-host/src/modules/re.rs:25), `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `bootstrap/cw-host/src/modules/hash.rs` (bootstrap/cw-host/src/modules/hash.rs:14) … 9 more  
tests 10: `tests::a_reservation_of_nothing_is_refused` (crates/ingest-store/src/budget.rs:351), `outside_local_an_issuer_variable_that_differs_from_the_topology_stops_startup` (apps/causeway-metadata-admin-host/tests/auth_requirement.rs:256), `the_removed_selector_cannot_make_a_prod_process_local` (apps/causeway-metadata-admin-host/tests/auth_requirement.rs:338) … 7 more  
uses_types 1: `Res` (bootstrap/cw-host/src/convert.rs:13)  
bootstrap/cw-host/src/convert.rs:13-15
```rust
pub fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(msg.into().into())
}
```

### 9. Test tests::tls_and_garbage_are_refused
`rust:bootstrap/cw-host/src/modules/pg.rs::tests::tls_and_garbage_are_refused`  bootstrap/cw-host/src/modules/pg.rs:328-331  pkg:cargo/cw-host  score 0.87  via `dsn_config` -> TESTED_BY (bootstrap/cw-host/src/modules/pg.rs:329, 1.00)  
`fn tls_and_garbage_are_refused()`  
calls 2: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:329), `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:330)  
tests_for 1: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:329)  
bootstrap/cw-host/src/modules/pg.rs:328-331
```rust
    fn tls_and_garbage_are_refused() {
        assert!(dsn_config("postgres://u:p@h/d?sslmode=require", None).is_err());
        assert!(dsn_config("postgres://u:p@h/d?nonsense=1", None).is_err());
    }
```

### 10. Test tests::a_url_without_a_password_takes_pgpassword
`rust:bootstrap/cw-host/src/modules/pg.rs::tests::a_url_without_a_password_takes_pgpassword`  bootstrap/cw-host/src/modules/pg.rs:319-325  pkg:cargo/cw-host  score 0.80  via `dsn_config` -> TESTED_BY (bootstrap/cw-host/src/modules/pg.rs:320, 1.00)  
`fn a_url_without_a_password_takes_pgpassword()`  
calls 2: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:320), `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:323)  
tests_for 1: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:320)  
bootstrap/cw-host/src/modules/pg.rs:319-325
```rust
    fn a_url_without_a_password_takes_pgpassword() {
        let c = dsn_config("postgres://causeway@127.0.0.1/causeway", Some("pw".into())).unwrap();
        assert_eq!(c.get_password(), Some(&b"pw"[..]));
        assert_eq!(c.get_connect_timeout(), Some(&Duration::from_secs(10)));
        let c = dsn_config("postgres://causeway@127.0.0.1/causeway", None).unwrap();
        assert_eq!(c.get_password(), None);
    }
```

### 11. Function test_a_preset_naming_a_remote_host_still_needs_a_local_context
`rhai:.mise/rhai/causeway/cluster_preset_test.rhai::test_a_preset_naming_a_remote_host_still_needs_a_local_context`  .mise/rhai/causeway/cluster_preset_test.rhai:203-210  pkg:npm/causeway-monorepo  score 0.78  
`fn test_a_preset_naming_a_remote_host_still_needs_a_local_context()`  
calls 4: `guard_refusal` (.mise/rhai/causeway/cluster_preset_test.rhai:205), `origin_at` (.mise/rhai/causeway/cluster_preset_test.rhai:204), `guard_refusal` (.mise/rhai/causeway/cluster_preset_test.rhai:207) … 1 more  
.mise/rhai/causeway/cluster_preset_test.rhai:203-210
```rhai
fn test_a_preset_naming_a_remote_host_still_needs_a_local_context() {
    let remote = origin_at("db.example.internal");
    assert_eq(cluster::guard_refusal(remote, "causeway", "", "seed"), ());
    assert_eq(cluster::guard_refusal(remote, "eks", "eks", "seed"), (), "the named override still admits it");
    let refusal = cluster::guard_refusal(remote, "prod", "", "seed the database");
    assert(refusal.contains("KUBE_CONTEXT is 'prod'") && refusal.contains("refusing to seed the database"), refusal);
    assert(refusal.contains("'db.example.internal' is not a loopback address"), refusal);
}
```

### 12. TypeAlias Res
`rust:bootstrap/cw-host/src/convert.rs::Res`  bootstrap/cw-host/src/convert.rs:10  pkg:cargo/cw-host  score 0.76  via `dsn_config` -> USES_TYPE (bootstrap/cw-host/src/modules/pg.rs:62, 1.00)  
`pub type Res<T> = Result<T, Box<EvalAltResult>>;`  
importers 14: `bootstrap/cw-host/src/modules/pg.rs` (bootstrap/cw-host/src/modules/pg.rs:31), `bootstrap/cw-host/src/globals.rs` (bootstrap/cw-host/src/globals.rs:28), `bootstrap/cw-host/src/modules/hash.rs` (bootstrap/cw-host/src/modules/hash.rs:14) … 11 more  
used_by 87: `Res` (bootstrap/cw-devdata/src/manifest.rs:35), `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:62), `for_path` (bootstrap/cw-host/src/modules/wait.rs:70) … 84 more  
uses_types 1: `Res` (bootstrap/cw-host/src/convert.rs:10)  
bootstrap/cw-host/src/convert.rs:10-10
```rust
pub type Res<T> = Result<T, Box<EvalAltResult>>;
```

### 13. Method RemoteConfig::load
`rust:services/flightdeck/src/remote.rs::RemoteConfig::load`  services/flightdeck/src/remote.rs:77-81  pkg:cargo/flightdeck-server  score 0.70  
`pub fn load(path: &std::path::Path) -> anyhow::Result<Self>`  
Load remote backend configuration from TOML. (doc continues, 114 bytes)  
callers 1: `build_remote_routing` (services/flightdeck/src/startup.rs:728)  
services/flightdeck/src/remote.rs:77-81
```rust
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let src = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&src)?;
        Ok(config)
    }
```

### 14. File services/flightdeck/src/remote.rs
`file:services/flightdeck/src/remote.rs`  services/flightdeck/src/remote.rs  pkg:cargo/flightdeck-server  score 0.69  via `PostgreSqlSource` <- CONTAINS (services/flightdeck/src/remote.rs:137-147, 1.00)  
members 46  
designs 1: `docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md` (docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md:265)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:133)  
imports 19: `cargo/arrow-schema` (services/flightdeck/src/remote.rs:10), `cargo/tokio-postgres` (services/flightdeck/src/remote.rs:32), `cargo/tonic` (services/flightdeck/src/remote.rs:35) … 16 more  

### 15. Module tests
`rust:bootstrap/cw-host/src/modules/pg.rs::tests`  bootstrap/cw-host/src/modules/pg.rs:297-332  pkg:cargo/cw-host  score 0.67  via `dsn_config` <- IMPORTS (bootstrap/cw-host/src/modules/pg.rs:298, 1.00)  
`mod tests`  
members 3  
imports 2: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:298), `cargo/tokio-postgres` (bootstrap/cw-host/src/modules/pg.rs:300)  

### 16. Function send_postgres_batch
`rust:services/flightdeck/src/remote.rs::send_postgres_batch`  services/flightdeck/src/remote.rs:873-899  pkg:cargo/flightdeck-server  score 0.64  
`async fn send_postgres_batch( tx: &mpsc::Sender<Result<RecordBatch, Status>>, schema: &Arc<Schema>, rows: &mut Vec<tokio_postgres::Row>, ) -> Result<(), ()>`  
callers 2: `execute_postgres` (services/flightdeck/src/remote.rs:863), `execute_postgres` (services/flightdeck/src/remote.rs:843)  
calls 1: `postgres_cell_to_string` (services/flightdeck/src/remote.rs:886)  
uses_types 1: `Row` (services/flightdeck/src/remote.rs:876)  
services/flightdeck/src/remote.rs:873-899
```rust
async fn send_postgres_batch(
    tx: &mpsc::Sender<Result<RecordBatch, Status>>,
    schema: &Arc<Schema>,
    rows: &mut Vec<tokio_postgres::Row>,
) -> Result<(), ()> {
    let mut builders = schema
        .fields()
        .iter()
        .map(|_| StringBuilder::new())
        .collect::<Vec<StringBuilder>>();

    for row in rows.drain(..) {
        for (index, builder) in builders.iter_mut().enumerate() {
            match postgres_cell_to_string(&row, index) {
                Some(value) => builder.append_value(value),
                None => builder.append_null(),
            }
        }
    }

    let columns = builders
        .into_iter()
        .map(|mut builder| Arc::new(builder.finish()) as arrow::array::ArrayRef)
        .collect::<Vec<_>>();
    let batch = RecordBatch::try_new(Arc::clone(schema), columns).map_err(|_| ())?;
    tx.send(Ok(batch)).await.map_err(|_| ())
}
```

### 17. Test tests::a_url_is_percent_decoded_and_keeps_its_timeout
`rust:bootstrap/cw-host/src/modules/pg.rs::tests::a_url_is_percent_decoded_and_keeps_its_timeout`  bootstrap/cw-host/src/modules/pg.rs:303-316  pkg:cargo/cw-host  score 0.64  via `dsn_config` -> TESTED_BY (bootstrap/cw-host/src/modules/pg.rs:304, 1.00)  
`fn a_url_is_percent_decoded_and_keeps_its_timeout()`  
calls 1: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:304)  
tests_for 1: `dsn_config` (bootstrap/cw-host/src/modules/pg.rs:304)  
bootstrap/cw-host/src/modules/pg.rs:303-316
```rust
    fn a_url_is_percent_decoded_and_keeps_its_timeout() {
        let c = dsn_config(
            "postgres://caus%40eway:p%2Fw@10.43.0.1:5433/my%20db?sslmode=disable&connect_timeout=5",
            Some("ignored".into()),
        )
        .unwrap();
        assert_eq!(c.get_user(), Some("caus@eway"));
        assert_eq!(c.get_password(), Some(&b"p/w"[..]));
        assert_eq!(c.get_dbname(), Some("my db"));
        assert_eq!(c.get_hosts(), &[Host::Tcp("10.43.0.1".into())]);
        assert_eq!(c.get_ports(), &[5433]);
        assert_eq!(c.get_connect_timeout(), Some(&Duration::from_secs(5)));
        assert_eq!(c.get_application_name(), Some("cwsh"));
    }
```

### 18. Method RemoteRuntime::plan_query
`rust:services/flightdeck/src/remote.rs::RemoteRuntime::plan_query`  services/flightdeck/src/remote.rs:335-430  pkg:cargo/flightdeck-server  score 0.60  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:392, 1.00)  
`pub fn plan_query( &self, sql: &str, preferred_mode: Option<RemoteExecutionMode>, ) -> Result<QueryPlan, Status>`  
Routes and normalizes a query before centralized authorization. (doc continues, 142 bytes)  
callers 9: `tests::local_query_stays_local` (services/flightdeck/src/remote.rs:1443), `tests::catalog_alias_routes_to_the_warehouse_under_its_catalog_name` (services/flightdeck/src/remote.rs:1424), `tests::mixed_local_and_remote_query_is_rejected` (services/flightdeck/src/remote.rs:1492) … 6 more  
calls 14: `rewrite_strip_source_prefix` (services/flightdeck/src/remote.rs:393), `RemoteSource::backend_kind` (services/flightdeck/src/remote.rs:396), `RemoteSource::name` (services/flightdeck/src/remote.rs:412) … 11 more  
references 4: `RemoteSource::DatabricksSqlWarehouse` (services/flightdeck/src/remote.rs:392), `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:392), `QueryPlan::Local` (services/flightdeck/src/remote.rs:365) … 1 more  
tests 6: `tests::catalog_alias_routes_to_the_warehouse_under_its_catalog_name` (services/flightdeck/src/remote.rs:1396), `tests::mixed_local_and_remote_query_is_rejected` (services/flightdeck/src/remote.rs:1492), `tests::postgres_query_rewrites_source_prefix` (services/flightdeck/src/remote.rs:1452) … 3 more  
uses_types 3: `QueryPlan` (services/flightdeck/src/remote.rs:339), `RemoteQueryPlan` (services/flightdeck/src/remote.rs:413), `RemoteExecutionMode` (services/flightdeck/src/remote.rs:338)  
services/flightdeck/src/remote.rs:335-414, cut
```rust
    pub fn plan_query(
        &self,
        sql: &str,
        preferred_mode: Option<RemoteExecutionMode>,
    ) -> Result<QueryPlan, Status> {
        let mut df_statements = DFParser::parse_sql(sql)
            .map_err(|e| Status::invalid_argument(format!("SQL syntax error: {e}")))?;
        let _ = df_statements
            .pop_front()
            .ok_or_else(|| Status::invalid_argument("query is empty"))?;
        if !df_statements.is_empty() {
            return Err(Status::invalid_argument(
                "only a single SQL statement is supported per request",
            ));
        }
        let mut statements = Parser::parse_sql(&GenericDialect {}, sql)
            .map_err(|e| Status::invalid_argument(format!("SQL syntax error: {e}")))?;
        let mut statement = statements
            .pop()
            .ok_or_else(|| Status::invalid_argument("query is empty"))?;

        // A catalog-registered alias is a bare local name that stands for a
        // qualified relation on a source; expanding it here lets the rest of
        // the planner treat the query as an ordinary remote query. The alias
        // itself is the authorization resource, because that is the name the
        // catalog published and the gateway binds.
        let alias_resources = expand_catalog_aliases(&mut statement, &self.catalog_tables())?;
        let relations = collect_relations(&statement);
        let source_name = classify_source(&self.config, &relations)?;
        let Some(source_name) = source_name else {
            return Ok(QueryPlan::Local);
        };
        // A remote source runs what it is sent with the caller's own
        // credential, and the gateway authorizes the relations as reads, so
        // only a query may be sent: never DDL, DML or a session statement.
        if !matches!(statement, Statement::Query(_)) {
            return Err(Status::permission_denied(
                "a remote source accepts only queries over Flight SQL",
            ));
        }

        let source = self
            .config
            .source_by_name(&source_name)
            .ok_or_else(|| Status::internal(format!("unknown remote source: {source_name}")))?;

        let mode = preferred_mode.unwrap_or_else(|| source.default_mode());
        if !source.allows_mode(mode) {
            return Err(Status::invalid_argument(format!(
                "remote source '{}' does not allow {:?} execution",
                source.name(),
                mode
            )));
        }

        let mut resources = Vec::new();
        match source {
            RemoteSource::PostgreSql(_) | RemoteSource::DatabricksSqlWarehouse(_) => {
                rewrite_strip_source_prefix(
                    &mut statement,
                    source.name(),
                    source.backend_kind(),
                    &mut resources,
                )?;
                if !alias_resources.is_empty() {
                    // Alias-routed relations are authorized under their
                    // catalog name, not the backend-qualified one.
                    resources.retain(|resource| {
                        !alias_resources
                            .iter()
                            .any(|(_, qualified)| resource == qualified)
                    });
                    resources.extend(alias_resources.iter().map(|(alias, _)| alias.clone()));
                }
            }
            RemoteSource::UnityCatalog(_) => {
                let unity_bindings =
                    rewrite_unity_aliases(&mut statement, source.name(), &mut resources)?;
                return Ok(QueryPlan::Remote(Box::new(RemoteQueryPlan {
                    source: source.clone(),
```

### 19. Const ELASTICACHE_WITHOUT_TLS
`rust:torrentfx/crates/causeway-platform/src/config.rs::ELASTICACHE_WITHOUT_TLS`  torrentfx/crates/causeway-platform/src/config.rs:2301-2302  pkg:cargo/causeway-platform  score 0.58  
`pub const ELASTICACHE_WITHOUT_TLS: &str = "valkey.elasticache.tls_enabled is false, and \ ElastiCache is reached over TLS in every environment: remove the key, or set it to true";`  
Why an `ElastiCache` that states `tls_enabled = false` fails to load: no (doc continues, 165 bytes)  
torrentfx/crates/causeway-platform/src/config.rs:2301-2302
```rust
pub const ELASTICACHE_WITHOUT_TLS: &str = "valkey.elasticache.tls_enabled is false, and \
     ElastiCache is reached over TLS in every environment: remove the key, or set it to true";
```

### 20. Struct RemoteConfig
`rust:services/flightdeck/src/remote.rs::RemoteConfig`  services/flightdeck/src/remote.rs:66-69  pkg:cargo/flightdeck-server  score 0.57  via `RemoteConfig#impl` -> USES_TYPE (services/flightdeck/src/remote.rs:71-86, 1.00)  
`pub struct RemoteConfig`  
Remote source configuration loaded from TOML.  
members 2  
importers 1: `services/flightdeck/src/startup.rs` (services/flightdeck/src/startup.rs:22)  
used_by 7: `RemoteRuntime` (services/flightdeck/src/remote.rs:224), `tests::runtime` (services/flightdeck/src/remote.rs:1339), `unity_catalog_query_executes_delta_remote_source` (services/flightdeck/tests/flight_test.rs:229) … 4 more  
uses_types 1: `RemoteSource` (services/flightdeck/src/remote.rs:68)  
services/flightdeck/src/remote.rs:66-69
```rust
pub struct RemoteConfig {
    #[serde(default)]
    pub sources: Vec<RemoteSource>,
}
```

### 21. Variant PostgresSecret::LocalInline
`rust:crates/causeway-testkit/src/topology.rs::PostgresSecret::LocalInline`  crates/causeway-testkit/src/topology.rs:139  pkg:cargo/causeway-testkit  score 0.57  
`LocalInline`  
The reference keeps the committed in-cluster mount, which no host (doc continues, 324 bytes)  
referenced_by 4: `InfraPlan::start` (crates/causeway-testkit/src/infra.rs:217), `tests::a_local_inline_topology_mounts_no_postgres_secret` (crates/causeway-testkit/src/topology.rs:473), `LocalTopology::write_in` (crates/causeway-testkit/src/topology.rs:207) … 1 more  

### 22. Function execute_postgres
`rust:services/flightdeck/src/remote.rs::execute_postgres`  services/flightdeck/src/remote.rs:750-871  pkg:cargo/flightdeck-server  score 0.54  via `PostgreSqlSource` <- USES_TYPE (services/flightdeck/src/remote.rs:751, 1.00)  
`async fn execute_postgres( source: &PostgreSqlSource, sql: &str, caller: &Access<Authenticated>, http: &reqwest::Client, shutdown: CancellationToken, deadline: Option<tokio::time::Instant>, ) -> Result<RemoteExecution, Status>`  
callers 1: `RemoteRuntime::execute` (services/flightdeck/src/remote.rs:453)  
calls 15: `RemoteSource::name` (services/flightdeck/src/remote.rs:788), `send_postgres_batch` (services/flightdeck/src/remote.rs:863), `acquire_downstream_token` (services/flightdeck/src/remote.rs:758) … 12 more  
uses_types 4: `PostgreSqlSource` (services/flightdeck/src/remote.rs:751), `Access` (services/flightdeck/src/remote.rs:753), `RemoteExecution` (services/flightdeck/src/remote.rs:757) … 1 more  
services/flightdeck/src/remote.rs:750-829, cut
```rust
async fn execute_postgres(
    source: &PostgreSqlSource,
    sql: &str,
    caller: &Access<Authenticated>,
    http: &reqwest::Client,
    shutdown: CancellationToken,
    deadline: Option<tokio::time::Instant>,
) -> Result<RemoteExecution, Status> {
    let token = acquire_downstream_token(http, caller, &source.auth)
        .await
        .map_err(|e| {
            Status::unauthenticated(format!("postgres credential acquisition failed: {e}"))
        })?;
    let dsn = source
        .dsn
        .replace("{subject_id}", caller.identity().subject_id());
    let mut config = tokio_postgres::Config::from_str(&dsn)
        .map_err(|e| Status::invalid_argument(format!("invalid PostgreSQL DSN: {e}")))?;
    config.password(&token);

    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|e| Status::unavailable(format!("PostgreSQL connect failed: {e}")))?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            warn!(error = %error, "PostgreSQL connection ended");
        }
    });

    let statement = client
        .prepare(sql)
        .await
        .map_err(|e| Status::invalid_argument(format!("PostgreSQL prepare failed: {e}")))?;
    let schema = Arc::new(Schema::new(
        statement
            .columns()
            .iter()
            .map(|column| Field::new(column.name(), DataType::Utf8, true))
            .collect::<Vec<Field>>(),
    ));
    let batch_rows = source.batch_rows.max(1);
    let (tx, rx) = mpsc::channel(8);
    let schema_for_task = Arc::clone(&schema);

    tokio::spawn(async move {
        let row_stream = match client
            .query_raw(&statement, std::iter::empty::<&str>())
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                let _ = tx
                    .send(Err(Status::invalid_argument(format!(
                        "PostgreSQL query failed: {error}"
                    ))))
                    .await;
                return;
            }
        };
        tokio::pin!(row_stream);
        let mut rows = Vec::with_capacity(batch_rows);
        loop {
            let next_row = match deadline {
                Some(deadline) => {
                    tokio::select! {
                        () = shutdown.cancelled() => {
                            let _ = tx.send(Err(Status::cancelled("server is shutting down"))).await;
                            return;
                        }
                        result = tokio::time::timeout_at(deadline, row_stream.next()) => {
                            if let Ok(value) = result { value } else {
                                let _ = tx.send(Err(Status::deadline_exceeded("remote PostgreSQL query timed out"))).await;
                                return;
                            }
                        }
                    }
                }
                None => {
                    tokio::select! {
```

### 23. Enum CatalogTableFormat
`rust:services/flightdeck/src/catalog_refresh.rs::CatalogTableFormat`  services/flightdeck/src/catalog_refresh.rs:37-64  pkg:cargo/flightdeck-server  score 0.50  via `CatalogTableFormat::PostgresRemote` <- DEFINES (services/flightdeck/src/catalog_refresh.rs:61-63, 1.00)  
`enum CatalogTableFormat`  
members 7  
used_by 3: `infer_format` (services/flightdeck/src/catalog_refresh.rs:524), `CatalogTableSpec` (services/flightdeck/src/catalog_refresh.rs:81), `normalized_storage_uri` (services/flightdeck/src/catalog_refresh.rs:616)  
services/flightdeck/src/catalog_refresh.rs:37-64
```rust
enum CatalogTableFormat {
    Csv,
    Json,
    Parquet,
    #[cfg(feature = "delta")]
    Delta,
    /// A Unity Catalog table (`unity-catalog://catalog.schema.table`), served
    /// by emitting Databricks SQL to the configured warehouse source rather
    /// than by reading its managed storage.
    DatabricksWarehouse {
        relation: Vec<String>,
    },
    /// A table in the Iceberg catalog this flightdeck has registered
    /// (`iceberg-rest://instance/warehouse/namespace/table`), read
    /// locally through that catalog's provider rather than pushed
    /// anywhere.
    #[cfg(feature = "iceberg")]
    IcebergCatalog {
        namespace: String,
        table: String,
    },
    /// A `PostgreSQL` table (`postgres://instance/database.schema.table`),
    /// served by the configured `PostgreSQL` source; the source's DSN fixes
    /// the database, so the relation is `schema.table`.
    PostgresRemote {
        relation: Vec<String>,
    },
}
```

### 24. Function open
`rust:bootstrap/cw-host/src/modules/pg.rs::open`  bootstrap/cw-host/src/modules/pg.rs:87-103  pkg:cargo/cw-host  score 0.49  via `connect_dsn` -> CALLS (bootstrap/cw-host/src/modules/pg.rs:84, 1.00)  
`fn open(config: tokio_postgres::Config) -> Res<Pg>`  
callers 2: `connect_dsn` (bootstrap/cw-host/src/modules/pg.rs:84), `connect` (bootstrap/cw-host/src/modules/pg.rs:54)  
calls 6: `err` (bootstrap/cw-host/src/modules/pg.rs:101), `with_causes` (bootstrap/cw-host/src/modules/pg.rs:101), `block_on` (bootstrap/cw-host/src/modules/pg.rs:88) … 3 more  
uses_types 3: `Error` (bootstrap/cw-host/src/modules/pg.rs:99), `Res` (bootstrap/cw-host/src/modules/pg.rs:87), `Pg` (bootstrap/cw-host/src/modules/pg.rs:87)  
bootstrap/cw-host/src/modules/pg.rs:87-103
```rust
fn open(config: tokio_postgres::Config) -> Res<Pg> {
    let client = runtime::block_on(async move {
        let (client, connection) = config.connect(NoTls).await?;
        runtime::spawn(async move {
            // A connection closing (the script ended, or a forward it ran
            // through was stopped) is not worth a line; anything else is.
            if let Err(e) = connection.await
                && !e.is_closed()
            {
                eprintln!("cwsh: postgres connection: {e}");
            }
        });
        Ok::<_, tokio_postgres::Error>(client)
    })
    .or_else(|e| err(format!("connecting to postgres: {}", with_causes(&e))))?;
    Ok(Pg(Rc::new(client)))
}
```

### 25. Enum RemoteSource
`rust:services/flightdeck/src/remote.rs::RemoteSource`  services/flightdeck/src/remote.rs:91-95  pkg:cargo/flightdeck-server  score 0.49  via `RemoteSource::PostgreSql` <- DEFINES (services/flightdeck/src/remote.rs:92, 1.00)  
`pub enum RemoteSource`  
Concrete remote source definition.  
members 7  
used_by 4: `RemoteQueryPlan` (services/flightdeck/src/remote.rs:498), `RemoteConfig` (services/flightdeck/src/remote.rs:68), `RemoteConfig::source_by_name` (services/flightdeck/src/remote.rs:83) … 1 more  
services/flightdeck/src/remote.rs:91-95
```rust
pub enum RemoteSource {
    PostgreSql(PostgreSqlSource),
    DatabricksSqlWarehouse(DatabricksSqlWarehouseSource),
    UnityCatalog(UnityCatalogSource),
}
```

### 26. Struct Pg
`rust:bootstrap/cw-host/src/modules/pg.rs::Pg`  bootstrap/cw-host/src/modules/pg.rs:36  pkg:cargo/cw-host  score 0.46  via `connect_dsn` -> USES_TYPE (bootstrap/cw-host/src/modules/pg.rs:83, 1.00)  
`pub struct Pg`  
An open connection.  
callers 1: `open` (bootstrap/cw-host/src/modules/pg.rs:102)  
used_by 11: `query` (bootstrap/cw-host/src/modules/pg.rs:256), `module` (bootstrap/cw-host/src/modules/pg.rs:278), `connect_dsn` (bootstrap/cw-host/src/modules/pg.rs:83) … 8 more  
bootstrap/cw-host/src/modules/pg.rs:36-36
```rust
pub struct Pg(Rc<Client>);
```

### 27. Method RemoteSource::name
`rust:services/flightdeck/src/remote.rs::RemoteSource::name`  services/flightdeck/src/remote.rs:99-105  pkg:cargo/flightdeck-server  score 0.46  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:101, 1.00)  
`pub fn name(&self) -> &str`  
callers 16: `execute_postgres` (services/flightdeck/src/remote.rs:788), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:412), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:385) … 13 more  
references 3: `RemoteSource::UnityCatalog` (services/flightdeck/src/remote.rs:103), `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:101), `RemoteSource::DatabricksSqlWarehouse` (services/flightdeck/src/remote.rs:102)  
tests 2: `tests::catalog_alias_routes_to_the_warehouse_under_its_catalog_name` (services/flightdeck/src/remote.rs:1404), `live_query_streams_rows` (services/flightdeck/tests/live_query_test.rs:48)  
services/flightdeck/src/remote.rs:99-105
```rust
    pub fn name(&self) -> &str {
        match self {
            Self::PostgreSql(source) => &source.name,
            Self::DatabricksSqlWarehouse(source) => &source.name,
            Self::UnityCatalog(source) => &source.name,
        }
    }
```

### 28. File bootstrap/cw-host/src/convert.rs
`file:bootstrap/cw-host/src/convert.rs`  bootstrap/cw-host/src/convert.rs  pkg:cargo/cw-host  score 0.44  via `Res` <- CONTAINS (bootstrap/cw-host/src/convert.rs:10, 1.00)  
members 12  
imports 2: `cargo/rhai` (bootstrap/cw-host/src/convert.rs:7), `cargo/serde_json` (bootstrap/cw-host/src/convert.rs:8)  

### 29. File services/flightdeck/src/catalog_refresh.rs
`file:services/flightdeck/src/catalog_refresh.rs`  services/flightdeck/src/catalog_refresh.rs  pkg:cargo/flightdeck-server  score 0.43  via `CatalogTableFormat` <- CONTAINS (services/flightdeck/src/catalog_refresh.rs:37-64, 1.00)  
members 25  
designs 1: `docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md` (docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md:265)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:507)  
imports 22: `TransportProtocol` (services/flightdeck/src/catalog_refresh.rs:14), `cargo/tokio-util` (services/flightdeck/src/catalog_refresh.rs:24), `CatalogRefreshConfig` (services/flightdeck/src/catalog_refresh.rs:7) … 19 more  

### 30. Impl RemoteConfig#impl
`rust:services/flightdeck/src/remote.rs::RemoteConfig#impl`  services/flightdeck/src/remote.rs:71-86  pkg:cargo/flightdeck-server  score 0.42  via `RemoteConfig::load` <- DEFINES (services/flightdeck/src/remote.rs:77-81, 1.00)  
`impl RemoteConfig`  
members 2  
uses_types 1: `RemoteConfig` (services/flightdeck/src/remote.rs:71-86)  

### 31. Enum RemoteExecutionMode
`rust:services/flightdeck/src/remote.rs::RemoteExecutionMode`  services/flightdeck/src/remote.rs:47-50  pkg:cargo/flightdeck-server  score 0.42  via `PostgreSqlSource` -> USES_TYPE (services/flightdeck/src/remote.rs:141, 1.00)  
`pub enum RemoteExecutionMode`  
Remote execution mode.  
members 3  
importers 1: `services/flightdeck/src/service.rs` (services/flightdeck/src/service.rs:43)  
used_by 12: `remote_mode_from_metadata` (services/flightdeck/src/service.rs:758), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:338), `StatementHandleEntry` (services/flightdeck/src/service.rs:54) … 9 more  
services/flightdeck/src/remote.rs:47-50
```rust
pub enum RemoteExecutionMode {
    Translated,
    PassThrough,
}
```

### 32. Method RemoteRuntime::execute
`rust:services/flightdeck/src/remote.rs::RemoteRuntime::execute`  services/flightdeck/src/remote.rs:437-478  pkg:cargo/flightdeck-server  score 0.41  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:452, 1.00)  
`pub async fn execute( &self, plan: &RemoteQueryPlan, caller: &Access<Authenticated>, base_ctx: &GovernedSession, shutdown: CancellationToken, execution_timeout: Duration, ) -> Result<RemoteExecution, Status>`  
Execute a routed remote query. (doc continues, 107 bytes)  
callers 58: `an_analyst_displays_public_tables_only` (services/flightdeck/tests/principal_policy_test.rs:189), `unlisted_table_denied_by_fail_closed_metadata` (services/flightdeck/tests/auth_test.rs:718), `query_count_returns_single_row` (services/flightdeck/tests/flight_test.rs:183) … 55 more  
calls 5: `execute_databricks_sql` (services/flightdeck/src/remote.rs:464), `execute_postgres` (services/flightdeck/src/remote.rs:453), `ByteLen::is_zero` (services/flightdeck/src/remote.rs:445) … 2 more  
references 3: `RemoteSource::UnityCatalog` (services/flightdeck/src/remote.rs:466), `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:452), `RemoteSource::DatabricksSqlWarehouse` (services/flightdeck/src/remote.rs:463)  
tests 33: `query_csv_table` (services/flightdeck/tests/flight_test.rs:144), `statements_that_name_a_location_are_refused_when_every_write_is_permitted` (services/flightdeck/tests/write_test.rs:132), `a_request_without_authenticated_evidence_is_refused` (services/flightdeck/tests/flight_test.rs:453) … 30 more  
uses_types 5: `RemoteExecution` (services/flightdeck/src/remote.rs:444), `Authenticated` (services/flightdeck/src/remote.rs:440), `RemoteQueryPlan` (services/flightdeck/src/remote.rs:439) … 2 more  
services/flightdeck/src/remote.rs:437-478
```rust
    pub async fn execute(
        &self,
        plan: &RemoteQueryPlan,
        caller: &Access<Authenticated>,
        base_ctx: &GovernedSession,
        shutdown: CancellationToken,
        execution_timeout: Duration,
    ) -> Result<RemoteExecution, Status> {
        let deadline = if execution_timeout.is_zero() {
            None
        } else {
            Some(tokio::time::Instant::now() + execution_timeout)
        };

        match &plan.source {
            RemoteSource::PostgreSql(source) => {
                execute_postgres(
                    source,
                    &plan.rewritten_sql,
                    caller,
                    &self.http,
                    shutdown,
                    deadline,
                )
                .await
            }
            RemoteSource::DatabricksSqlWarehouse(source) => {
                execute_databricks_sql(source, &plan.rewritten_sql, caller, &self.http).await
            }
            RemoteSource::UnityCatalog(source) => {
                execute_unity_catalog(
                    source,
                    &plan.rewritten_sql,
                    plan.unity_bindings.as_ref(),
                    base_ctx,
                    shutdown,
                    deadline,
                )
                .await
            }
        }
    }
```

### 33. Method RemoteSource::default_mode
`rust:services/flightdeck/src/remote.rs::RemoteSource::default_mode`  services/flightdeck/src/remote.rs:108-114  pkg:cargo/flightdeck-server  score 0.41  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:110, 1.00)  
`pub fn default_mode(&self) -> RemoteExecutionMode`  
callers 1: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:381)  
references 4: `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:110), `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:112), `RemoteSource::DatabricksSqlWarehouse` (services/flightdeck/src/remote.rs:111) … 1 more  
uses_types 1: `RemoteExecutionMode` (services/flightdeck/src/remote.rs:108)  
services/flightdeck/src/remote.rs:108-114
```rust
    pub fn default_mode(&self) -> RemoteExecutionMode {
        match self {
            Self::PostgreSql(source) => source.default_mode,
            Self::DatabricksSqlWarehouse(source) => source.default_mode,
            Self::UnityCatalog(_) => RemoteExecutionMode::Translated,
        }
    }
```

### 34. Method RemoteSource::allows_mode
`rust:services/flightdeck/src/remote.rs::RemoteSource::allows_mode`  services/flightdeck/src/remote.rs:117-123  pkg:cargo/flightdeck-server  score 0.41  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:119, 1.00)  
`pub fn allows_mode(&self, mode: RemoteExecutionMode) -> bool`  
callers 1: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:382)  
references 4: `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:121), `RemoteSource::UnityCatalog` (services/flightdeck/src/remote.rs:121), `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:119) … 1 more  
uses_types 1: `RemoteExecutionMode` (services/flightdeck/src/remote.rs:117)  
services/flightdeck/src/remote.rs:117-123
```rust
    pub fn allows_mode(&self, mode: RemoteExecutionMode) -> bool {
        match self {
            Self::PostgreSql(source) => source.allowed_modes.contains(&mode),
            Self::DatabricksSqlWarehouse(source) => source.allowed_modes.contains(&mode),
            Self::UnityCatalog(_) => mode == RemoteExecutionMode::Translated,
        }
    }
```

### 35. Function connect
`rhai:bootstrap/cw-host/src/modules/pg.rs::connect`  bootstrap/cw-host/src/modules/pg.rs:280  pkg:cargo/cw-host  score 0.40  via `connect_dsn` <- CALLS (bootstrap/cw-host/src/modules/pg.rs:281, 1.00)  
`m.set_native_fn("connect", |spec: Map| connect(&spec))`  
callers 1: `connect` (.mise/rhai/causeway/db.rhai:43)  
calls 2: `connect` (bootstrap/cw-host/src/modules/pg.rs:280), `connect_dsn` (bootstrap/cw-host/src/modules/pg.rs:281)  
bootstrap/cw-host/src/modules/pg.rs:280-280
```rhai
    m.set_native_fn("connect", |spec: Map| connect(&spec));
```

### 36. Function guard_refusal
`rhai:.mise/rhai/causeway/cluster.rhai::guard_refusal`  .mise/rhai/causeway/cluster.rhai:412-427  pkg:npm/causeway-monorepo  score 0.38  via `test_a_preset_naming_a_remote_host_still_needs_a_local_context` -> CALLS (.mise/rhai/causeway/cluster_preset_test.rhai:207, 1.00)  
`fn guard_refusal(o, ctx, allow, what)`  
Why the local-cluster guard refuses WHAT, given the origin of the task's endpoints, or () when it passes. Under a preset whose database is on the loopback no cluster is reached, so the context is not consulted; a preset naming another host is checked as the cluster is.  
callers 9: `test_a_preset_naming_a_remote_host_still_needs_a_local_context` (.mise/rhai/causeway/cluster_preset_test.rhai:205), `test_a_loopback_preset_passes_the_guard_whatever_the_context` (.mise/rhai/causeway/cluster_preset_test.rhai:199), `test_a_preset_naming_a_remote_host_still_needs_a_local_context` (.mise/rhai/causeway/cluster_preset_test.rhai:207) … 6 more  
calls 4: `local_context_refusal` (.mise/rhai/causeway/cluster.rhai:419), `unknown_origin` (.mise/rhai/causeway/cluster.rhai:425), `is_loopback` (.mise/rhai/causeway/cluster.rhai:416) … 1 more  
.mise/rhai/causeway/cluster.rhai:412-427
```rhai
fn guard_refusal(o, ctx, allow, what) {
    switch o.kind {
        "preset" => {
            let host = o.preset.database.host;
            if is_loopback(host) {
                return ();
            }
            let refusal = local_context_refusal(ctx, allow, what, true);
            if refusal == () { () } else {
                refusal + "\n" + `  The preset's database host '${host}' is not a loopback address, so the preset does not stand in for a local cluster.`
            }
        }
        "cluster" => local_context_refusal(ctx, allow, what, true),
        _ => { throw unknown_origin(o); }
    }
}
```

### 37. Method RemoteSource::backend_kind
`rust:services/flightdeck/src/remote.rs::RemoteSource::backend_kind`  services/flightdeck/src/remote.rs:126-132  pkg:cargo/flightdeck-server  score 0.37  via `RemoteSource::PostgreSql` <- REFERENCES (services/flightdeck/src/remote.rs:128, 1.00)  
`fn backend_kind(&self) -> &'static str`  
callers 1: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:396)  
references 3: `RemoteSource::UnityCatalog` (services/flightdeck/src/remote.rs:130), `RemoteSource::PostgreSql` (services/flightdeck/src/remote.rs:128), `RemoteSource::DatabricksSqlWarehouse` (services/flightdeck/src/remote.rs:129)  
services/flightdeck/src/remote.rs:126-132
```rust
    fn backend_kind(&self) -> &'static str {
        match self {
            Self::PostgreSql(_) => "postgres",
            Self::DatabricksSqlWarehouse(_) => "databricks_sql",
            Self::UnityCatalog(_) => "unity",
        }
    }
```

### 38. File .mise/rhai/causeway/cluster_preset_test.rhai
`file:.mise/rhai/causeway/cluster_preset_test.rhai`  .mise/rhai/causeway/cluster_preset_test.rhai  pkg:npm/causeway-monorepo  score 0.36  via `test_a_preset_naming_a_remote_host_still_needs_a_local_context` <- CONTAINS (.mise/rhai/causeway/cluster_preset_test.rhai:203-210, 1.00)  
members 33  
designs 1: `docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md` (docs/rfd/RFD-fpg4tnq2-ephemeral-infrastructure-for-ci-integration-tests.md:807)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1249)  

### 39. Variant RemoteSource::DatabricksSqlWarehouse
`rust:services/flightdeck/src/remote.rs::RemoteSource::DatabricksSqlWarehouse`  services/flightdeck/src/remote.rs:93  pkg:cargo/flightdeck-server  score 0.36  via `RemoteRuntime::execute` -> REFERENCES (services/flightdeck/src/remote.rs:463, 1.00)  
`DatabricksSqlWarehouse`  
referenced_by 7: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:392), `RemoteSource::default_mode` (services/flightdeck/src/remote.rs:111), `RemoteRuntime::databricks_workspace_url` (services/flightdeck/src/remote.rs:302) … 4 more  
uses_types 1: `DatabricksSqlWarehouseSource` (services/flightdeck/src/remote.rs:93)  

### 40. Variant RemoteSource::UnityCatalog
`rust:services/flightdeck/src/remote.rs::RemoteSource::UnityCatalog`  services/flightdeck/src/remote.rs:94  pkg:cargo/flightdeck-server  score 0.34  via `RemoteRuntime::execute` -> REFERENCES (services/flightdeck/src/remote.rs:466, 1.00)  
`UnityCatalog`  
referenced_by 6: `RemoteRuntime::execute` (services/flightdeck/src/remote.rs:466), `RemoteSource::default_mode` (services/flightdeck/src/remote.rs:112), `RemoteSource::backend_kind` (services/flightdeck/src/remote.rs:130) … 3 more  
uses_types 1: `UnityCatalogSource` (services/flightdeck/src/remote.rs:94)  

### 41. Function connect
`rust:bootstrap/cw-host/src/modules/pg.rs::connect`  bootstrap/cw-host/src/modules/pg.rs:38-55  pkg:cargo/cw-host  score 0.33  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:38-55, 1.00)  
`fn connect(spec: &Map) -> Res<Pg>`  
callers 2: `connect` (bootstrap/cw-host/src/modules/pg.rs:280), `open` (bootstrap/cw-host/src/modules/pg.rs:89)  
calls 8: `opt_str` (bootstrap/cw-host/src/modules/pg.rs:49), `opt_str` (bootstrap/cw-host/src/modules/pg.rs:40), `err` (bootstrap/cw-host/src/modules/pg.rs:42) … 5 more  
uses_types 2: `Res` (bootstrap/cw-host/src/modules/pg.rs:38), `Pg` (bootstrap/cw-host/src/modules/pg.rs:38)  
bootstrap/cw-host/src/modules/pg.rs:38-55
```rust
fn connect(spec: &Map) -> Res<Pg> {
    let mut config = tokio_postgres::Config::new();
    config.host(opt_str(spec, "host")?.as_deref().unwrap_or("127.0.0.1"));
    config
        .port(u16::try_from(opt_int(spec, "port")?.unwrap_or(5432)).or_else(|_| err("bad port"))?);
    if let Some(user) = opt_str(spec, "user")? {
        config.user(user);
    }
    if let Some(password) = opt_str(spec, "password")? {
        config.password(password);
    }
    if let Some(dbname) = opt_str(spec, "dbname")? {
        config.dbname(dbname);
    }
    config.application_name("cwsh");
    config.connect_timeout(std::time::Duration::from_secs(10));
    open(config)
}
```

### 42. Enum StoreKind
`rust:crates/catalog-domain/src/binding.rs::StoreKind`  crates/catalog-domain/src/binding.rs:16-26  pkg:cargo/catalog-domain  score 0.32  via `StoreKind::Postgresql` <- DEFINES (crates/catalog-domain/src/binding.rs:19, 1.00)  
`pub enum StoreKind`  
The kind of system a store is. Closed: a new kind is a design change.  
members 11  
importers 5: `component/catalog/core/src/bindings.rs` (component/catalog/core/src/bindings.rs:13-17), `tests` (apps/causeway-server-web/src/views/platform/governance/placements.rs:323), `crates/catalog-domain/src/lib.rs` (crates/catalog-domain/src/lib.rs:17-23) … 2 more  
used_by 14: `ContainerPlacements` (crates/catalog-domain/src/binding.rs:882), `PhysicalContainer::store_kind` (crates/catalog-domain/src/binding.rs:1195), `PhysicalObject::store_kind` (crates/catalog-domain/src/binding.rs:1568) … 11 more  
crates/catalog-domain/src/binding.rs:16-26
```rust
pub enum StoreKind {
    UnityCatalog,
    Snowflake,
    Postgresql,
    S3,
    IcebergRest,
    HttpApi,
    McpServer,
    /// A third party's system Causeway describes but does not reach.
    External,
}
```

### 43. File torrentfx/crates/causeway-platform/src/config.rs
`file:torrentfx/crates/causeway-platform/src/config.rs`  torrentfx/crates/causeway-platform/src/config.rs  pkg:cargo/causeway-platform  score 0.32  via `ELASTICACHE_WITHOUT_TLS` <- CONTAINS (torrentfx/crates/causeway-platform/src/config.rs:2301-2302, 1.00)  
members 180  
designs 3: `docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md` (docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md:578), `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md` (docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md:632), `docs/rfd/RFD-wshvsxjy-dev-run-without-the-cluster-and-a-keyed-data-snapshot.md` (docs/rfd/RFD-wshvsxjy-dev-run-without-the-cluster-and-a-keyed-data-snapshot.md:281)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:250)  
imports 17: `ScopeError` (torrentfx/crates/causeway-platform/src/config.rs:17), `Companion` (torrentfx/crates/causeway-platform/src/config.rs:17), `VerifierValueError` (torrentfx/crates/causeway-platform/src/config.rs:20-23) … 14 more  

### 44. Enum PostgresSecret
`rust:crates/causeway-testkit/src/topology.rs::PostgresSecret`  crates/causeway-testkit/src/topology.rs:128-140  pkg:cargo/causeway-testkit  score 0.32  via `PostgresSecret::LocalInline` <- DEFINES (crates/causeway-testkit/src/topology.rs:139, 1.00)  
`pub enum PostgresSecret<'a>`  
Where a process reading the topology finds the `postgres-url` secret, (doc continues, 368 bytes)  
members 2  
importers 2: `crates/causeway-testkit/src/infra.rs` (crates/causeway-testkit/src/infra.rs:43), `crates/causeway-testkit/src/lib.rs` (crates/causeway-testkit/src/lib.rs:92)  
used_by 1: `LocalTopology::write_in` (crates/causeway-testkit/src/topology.rs:205)  

### 45. Method LocalTopology::write_in
`rust:crates/causeway-testkit/src/topology.rs::LocalTopology::write_in`  crates/causeway-testkit/src/topology.rs:202-234  pkg:cargo/causeway-testkit  score 0.31  via `PostgresSecret::LocalInline` <- REFERENCES (crates/causeway-testkit/src/topology.rs:225, 1.00)  
`pub fn write_in( parent: &Path, label: &str, postgres: PostgresSecret<'_>, ) -> TestkitResult<Self>`  
As [`Self::write`], in a new directory under `parent`, with the (doc continues, 354 bytes)  
callers 3: `tests::a_local_inline_topology_mounts_no_postgres_secret` (crates/causeway-testkit/src/topology.rs:473), `LocalTopology::write` (crates/causeway-testkit/src/topology.rs:186), `InfraPlan::start` (crates/causeway-testkit/src/infra.rs:214)  
calls 4: `LocalTopology::render` (crates/causeway-testkit/src/topology.rs:232), `LocalTopology::secrets_dir` (crates/causeway-testkit/src/topology.rs:230), `LocalTopology::write` (crates/causeway-testkit/src/topology.rs:230) … 1 more  
references 6: `PostgresSource::LocalInline` (crates/causeway-testkit/src/topology.rs:225), `PostgresSource::MountedFile` (crates/causeway-testkit/src/topology.rs:224), `PostgresSecret::MountedFile` (crates/causeway-testkit/src/topology.rs:224) … 3 more  
tests 1: `tests::a_local_inline_topology_mounts_no_postgres_secret` (crates/causeway-testkit/src/topology.rs:473)  
uses_types 2: `PostgresSecret` (crates/causeway-testkit/src/topology.rs:205), `TestkitResult` (crates/causeway-testkit/src/topology.rs:206)  

### 46. Function tests::runtime
`rust:services/flightdeck/src/remote.rs::tests::runtime`  services/flightdeck/src/remote.rs:1338-1361  pkg:cargo/flightdeck-server  score 0.30  via `PostgreSqlSource` <- USES_TYPE (services/flightdeck/src/remote.rs:1341, 1.00)  
`fn runtime() -> RemoteRuntime`  
callers 5: `tests::unity_query_rewrites_to_aliases` (services/flightdeck/src/remote.rs:1508), `tests::local_query_stays_local` (services/flightdeck/src/remote.rs:1442), `tests::mixed_local_and_remote_query_is_rejected` (services/flightdeck/src/remote.rs:1491) … 2 more  
calls 2: `RemoteRuntime::new` (services/flightdeck/src/remote.rs:1339), `default_remote_modes` (services/flightdeck/src/remote.rs:1344)  
references 2: `DownstreamTokenConfig::Passthrough` (services/flightdeck/src/remote.rs:1346), `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:1345)  
tests 5: `tests::mixed_local_and_remote_query_is_rejected` (services/flightdeck/src/remote.rs:1491), `tests::local_query_stays_local` (services/flightdeck/src/remote.rs:1442), `tests::postgres_query_rewrites_source_prefix` (services/flightdeck/src/remote.rs:1451) … 2 more  
uses_types 5: `RemoteConfig` (services/flightdeck/src/remote.rs:1339), `PostgreSqlSource` (services/flightdeck/src/remote.rs:1341), `RemoteRuntime` (services/flightdeck/src/remote.rs:1338) … 2 more  

### 47. Package cargo/tokio-postgres
`pkg:cargo/tokio-postgres`  pkg:cargo/tokio-postgres  score 0.30  via `tests` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:300, 1.00)  
dependents 2: `cargo/flightdeck-server` (services/flightdeck/Cargo.toml:60), `cargo/cw-host` (bootstrap/cw-host/Cargo.toml:29)  
importers 4: `services/flightdeck/src/remote.rs` (services/flightdeck/src/remote.rs:32), `tests` (bootstrap/cw-host/src/modules/pg.rs:300), `bootstrap/cw-host/src/modules/pg.rs` (bootstrap/cw-host/src/modules/pg.rs:28) … 1 more  

### 48. Function dynamic_to_json
`rust:bootstrap/cw-host/src/convert.rs::dynamic_to_json`  bootstrap/cw-host/src/convert.rs:54-93  pkg:cargo/cw-host  score 0.30  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:31, 1.00)  
`pub fn dynamic_to_json(value: &Dynamic) -> Res<Json>`  
Convert a Rhai value to JSON. (doc continues, 131 bytes)  
callers 12: `same` (bootstrap/cw-host/src/globals.rs:158), `debug` (bootstrap/cw-host/src/globals.rs:165), `param` (bootstrap/cw-host/src/modules/pg.rs:160) … 9 more  
calls 6: `err` (bootstrap/cw-host/src/convert.rs:67), `err` (bootstrap/cw-host/src/convert.rs:92), `Value::as_float` (bootstrap/cw-host/src/convert.rs:64) … 3 more  
importers 6: `bootstrap/cw-host/src/modules/data.rs` (bootstrap/cw-host/src/modules/data.rs:17), `bootstrap/cw-host/src/modules/http.rs` (bootstrap/cw-host/src/modules/http.rs:48-50), `bootstrap/cw-host/src/globals.rs` (bootstrap/cw-host/src/globals.rs:28) … 3 more  
references 1: `Json::Null` (bootstrap/cw-host/src/convert.rs:56)  
uses_types 3: `Res` (bootstrap/cw-host/src/convert.rs:54), `Json` (bootstrap/cw-host/src/convert.rs:54), `Json` (bootstrap/cw-host/src/convert.rs:54)  

### 49. Method PhysicalContainer::postgres_database
`rust:crates/catalog-domain/src/binding.rs::PhysicalContainer::postgres_database`  crates/catalog-domain/src/binding.rs:1036-1045  pkg:cargo/catalog-domain  score 0.29  via `StoreKind::Postgresql` <- REFERENCES (crates/catalog-domain/src/binding.rs:1038, 1.00)  
`pub fn postgres_database(instance: &str, database: &str) -> Self`  
A PostgreSQL database in one server instance: the container the (doc continues, 129 bytes)  
callers 2: `PhysicalContainer::postgres_schema` (crates/catalog-domain/src/binding.rs:1050), `SourceProfile::catalog_container` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:270)  
references 2: `ContainerKind::Database` (crates/catalog-domain/src/binding.rs:1041), `StoreKind::Postgresql` (crates/catalog-domain/src/binding.rs:1038)  
uses_types 1: `ContainerLevel` (crates/catalog-domain/src/binding.rs:1040)  

### 50. Function build_remote_routing
`rust:services/flightdeck/src/startup.rs::build_remote_routing`  services/flightdeck/src/startup.rs:713-770  pkg:cargo/flightdeck-server  score 0.28  via `RemoteConfig::load` <- CALLS (services/flightdeck/src/startup.rs:728, 1.00)  
`fn build_remote_routing( cfg: &ServerConfig, report: &mut StartupReport, ) -> anyhow::Result<( Option<RemoteRuntime>, Option<crate::catalog_refresh::CatalogRemoteRouting>, )>`  
Builds the remote runtime from `--remote-config` and, when the catalog (doc continues, 336 bytes)  
callers 1: `build_query_service` (services/flightdeck/src/startup.rs:779)  
calls 5: `StartupReport::record` (services/flightdeck/src/startup.rs:721), `RemoteConfig::load` (services/flightdeck/src/startup.rs:728), `StartupReport::record` (services/flightdeck/src/startup.rs:729) … 2 more  
references 4: `FailureProfile::Optional` (services/flightdeck/src/startup.rs:723), `FailureProfile::Optional` (services/flightdeck/src/startup.rs:731), `catalog_refresh` (services/flightdeck/src/startup.rs:763) … 1 more  
uses_types 4: `RemoteRuntime` (services/flightdeck/src/startup.rs:717), `CatalogRemoteRouting` (services/flightdeck/src/startup.rs:718), `StartupReport` (services/flightdeck/src/startup.rs:715) … 1 more  

### 51. Method SourceProfile::catalog_container
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::catalog_container`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:253-279  pkg:cargo/causeway-metadata-ops  score 0.28  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:269, 1.00)  
`fn catalog_container( &self, store_instance: &str, catalog: &str, shared: bool, ) -> catalog_domain::PhysicalContainer`  
The catalog-level container: a Unity Catalog catalog or share, a (doc continues, 86 bytes)  
callers 2: `run_source_sync` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:794), `run_source_sync` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:703)  
calls 6: `PhysicalContainer::postgres_database` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:270), `PhysicalContainer::iceberg_warehouse` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:261), `PhysicalContainer::unity_catalog_share` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:273) … 3 more  
references 4: `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:263), `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:266), `StoreKind::IcebergRest` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:260) … 1 more  
uses_types 1: `PhysicalContainer` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:258)  

### 52. File crates/catalog-domain/src/binding.rs
`file:crates/catalog-domain/src/binding.rs`  crates/catalog-domain/src/binding.rs  pkg:cargo/catalog-domain  score 0.28  via `StoreKind` <- CONTAINS (crates/catalog-domain/src/binding.rs:16-26, 1.00)  
members 55  
designs 1: `docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md` (docs/rfd/RFD-q10sm0wq-logical-assets-physical-bindings-and-taxonomy-placement.md:251)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:977)  
imports 2: `cargo/causeway-principal` (crates/catalog-domain/src/binding.rs:12), `RecordedBy` (crates/catalog-domain/src/binding.rs:12)  

### 53. Variant StoreKind::Snowflake
`rust:crates/catalog-domain/src/binding.rs::StoreKind::Snowflake`  crates/catalog-domain/src/binding.rs:18  pkg:cargo/catalog-domain  score 0.28  via `StoreKind` -> DEFINES (crates/catalog-domain/src/binding.rs:18, 1.00)  
`Snowflake`  
referenced_by 15: `SourceProfile::table_object` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:220), `SourceProfile::locator_for_name` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:177), `SourceProfile::snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:122) … 12 more  

### 54. File bootstrap/cw-host/src/globals.rs
`file:bootstrap/cw-host/src/globals.rs`  bootstrap/cw-host/src/globals.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/globals.rs:28, 1.00)  
members 8  
imports 4: `Res` (bootstrap/cw-host/src/globals.rs:28), `cargo/rhai` (bootstrap/cw-host/src/globals.rs:26), `dynamic_to_json` (bootstrap/cw-host/src/globals.rs:28) … 1 more  

### 55. File bootstrap/cw-host/src/modules/data.rs
`file:bootstrap/cw-host/src/modules/data.rs`  bootstrap/cw-host/src/modules/data.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/data.rs:17, 1.00)  
members 6  
imports 7: `dynamic_to_json` (bootstrap/cw-host/src/modules/data.rs:17), `cargo/rhai` (bootstrap/cw-host/src/modules/data.rs:13), `json_to_dynamic` (bootstrap/cw-host/src/modules/data.rs:17) … 4 more  

### 56. File bootstrap/cw-host/src/modules/fs.rs
`file:bootstrap/cw-host/src/modules/fs.rs`  bootstrap/cw-host/src/modules/fs.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/fs.rs:40, 1.00)  
members 14  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2123)  
imports 5: `cargo/rhai` (bootstrap/cw-host/src/modules/fs.rs:38), `opt_bool` (bootstrap/cw-host/src/modules/fs.rs:40), `err` (bootstrap/cw-host/src/modules/fs.rs:40) … 2 more  

### 57. File bootstrap/cw-host/src/modules/hash.rs
`file:bootstrap/cw-host/src/modules/hash.rs`  bootstrap/cw-host/src/modules/hash.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/hash.rs:14, 1.00)  
members 3  
imports 6: `cargo/sha1` (bootstrap/cw-host/src/modules/hash.rs:11), `cargo/rhai` (bootstrap/cw-host/src/modules/hash.rs:10), `err` (bootstrap/cw-host/src/modules/hash.rs:14) … 3 more  

### 58. File bootstrap/cw-host/src/modules/http.rs
`file:bootstrap/cw-host/src/modules/http.rs`  bootstrap/cw-host/src/modules/http.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/http.rs:48-50, 1.00)  
members 9  
imports 14: `opt_map` (bootstrap/cw-host/src/modules/http.rs:48-50), `opt_int` (bootstrap/cw-host/src/modules/http.rs:48-50), `dynamic_to_json` (bootstrap/cw-host/src/modules/http.rs:48-50) … 11 more  

### 59. File bootstrap/cw-host/src/modules/k8s.rs
`file:bootstrap/cw-host/src/modules/k8s.rs`  bootstrap/cw-host/src/modules/k8s.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/k8s.rs:70-72, 1.00)  
members 30  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2472)  
imports 17: `opt_str` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `string_vec` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `cargo/tokio` (bootstrap/cw-host/src/modules/k8s.rs:67) … 14 more  

### 60. File bootstrap/cw-host/src/modules/proc.rs
`file:bootstrap/cw-host/src/modules/proc.rs`  bootstrap/cw-host/src/modules/proc.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/proc.rs:81-83, 1.00)  
members 26  
imports 14: `cargo/nix` (bootstrap/cw-host/src/modules/proc.rs:77), `supervise` (bootstrap/cw-host/src/modules/proc.rs:85), `err_map` (bootstrap/cw-host/src/modules/proc.rs:81-83) … 11 more  

### 61. File bootstrap/cw-host/src/modules/re.rs
`file:bootstrap/cw-host/src/modules/re.rs`  bootstrap/cw-host/src/modules/re.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/re.rs:25, 1.00)  
members 3  
imports 4: `cargo/rhai` (bootstrap/cw-host/src/modules/re.rs:23), `err` (bootstrap/cw-host/src/modules/re.rs:25), `cargo/regex` (bootstrap/cw-host/src/modules/re.rs:22) … 1 more  

### 62. File bootstrap/cw-host/src/modules/s3.rs
`file:bootstrap/cw-host/src/modules/s3.rs`  bootstrap/cw-host/src/modules/s3.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/s3.rs:22, 1.00)  
members 17  
imports 8: `cargo/sha2` (bootstrap/cw-host/src/modules/s3.rs:20), `runtime` (bootstrap/cw-host/src/modules/s3.rs:23), `rfc3339_utc` (bootstrap/cw-host/src/modules/s3.rs:25) … 5 more  

### 63. File bootstrap/cw-host/src/modules/workspace.rs
`file:bootstrap/cw-host/src/modules/workspace.rs`  bootstrap/cw-host/src/modules/workspace.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/modules/workspace.rs:40, 1.00)  
members 5  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1854)  
imports 3: `Res` (bootstrap/cw-host/src/modules/workspace.rs:40), `cargo/rhai` (bootstrap/cw-host/src/modules/workspace.rs:38), `err` (bootstrap/cw-host/src/modules/workspace.rs:40)  

### 64. File bootstrap/cw-host/src/stream.rs
`file:bootstrap/cw-host/src/stream.rs`  bootstrap/cw-host/src/stream.rs  pkg:cargo/cw-host  score 0.27  via `err` <- IMPORTS (bootstrap/cw-host/src/stream.rs:16, 1.00)  
members 6  
imports 2: `err` (bootstrap/cw-host/src/stream.rs:16), `Res` (bootstrap/cw-host/src/stream.rs:16)  

### 65. Method SourceProfile::postgres
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::postgres`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:97-111  pkg:cargo/causeway-metadata-ops  score 0.27  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:101, 1.00)  
`pub fn postgres(schema_scope: Option<String>) -> Self`  
A PostgreSQL database read through its own catalogs.  
callers 3: `profile` (crates/causeway-metadata-ops/src/openapi_import.rs:82), `profile` (crates/causeway-metadata-ops/src/mcp_registry.rs:102), `run_postgres_sync_from_env` (crates/causeway-metadata-ops/src/postgres_sync.rs:74)  
references 1: `StoreKind::Postgresql` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:101)  

### 66. TypeAlias Res
`rust:bootstrap/cw-devdata/src/manifest.rs::Res`  bootstrap/cw-devdata/src/manifest.rs:35  pkg:cargo/cw-devdata  score 0.27  via `Res` -> USES_TYPE (bootstrap/cw-host/src/convert.rs:10, 1.00)  
`type Res<T> = Result<T, String>;`  
used_by 11: `read_records` (bootstrap/cw-devdata/src/manifest.rs:93), `Res` (bootstrap/cw-host/src/convert.rs:10), `read_json` (bootstrap/cw-devdata/src/manifest.rs:56) … 8 more  
uses_types 1: `Res` (bootstrap/cw-devdata/src/manifest.rs:35)  

### 67. Impl RemoteSource#impl
`rust:services/flightdeck/src/remote.rs::RemoteSource#impl`  services/flightdeck/src/remote.rs:97-133  pkg:cargo/flightdeck-server  score 0.27  via `RemoteSource::allows_mode` <- DEFINES (services/flightdeck/src/remote.rs:117-123, 1.00)  
`impl RemoteSource`  
members 4  
uses_types 1: `RemoteSource` (services/flightdeck/src/remote.rs:97-133)  

### 68. Package cargo/cw-host
`pkg:cargo/cw-host`  bootstrap/cw-host/Cargo.toml:2  pkg:cargo/cw-host  score 0.26  via `bootstrap/cw-host/src/modules/pg.rs` <- CONTAINS (bootstrap/cw-host/Cargo.toml:2, 1.00)  
`cargo/cw-host`  
members 22  
dependents 2: `cargo/cw-vsa` (bootstrap/cw-vsa/Cargo.toml:15), `cargo/cwsh` (bootstrap/cwsh/Cargo.toml:11)  
dependencies 22: `cargo/tokio` (bootstrap/cw-host/Cargo.toml:28), `cargo/rustls` (bootstrap/cw-host/Cargo.toml:20), `cargo/sha1` (bootstrap/cw-host/Cargo.toml:24) … 19 more  
importers 4: `bootstrap/cw-vsa/src/main.rs` (bootstrap/cw-vsa/src/main.rs:19), `bootstrap/cwsh/tests/cli.rs` (bootstrap/cwsh/tests/cli.rs:14), `bootstrap/cw-vsa/tests/gate.rs` (bootstrap/cw-vsa/tests/gate.rs:10) … 1 more  

### 69. Test tests::mixed_local_and_remote_query_is_rejected
`rust:services/flightdeck/src/remote.rs::tests::mixed_local_and_remote_query_is_rejected`  services/flightdeck/src/remote.rs:1490-1504  pkg:cargo/flightdeck-server  score 0.26  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1492, 1.00)  
`fn mixed_local_and_remote_query_is_rejected()`  
calls 6: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1492), `tests::runtime` (services/flightdeck/src/remote.rs:1491), `ExpressionError::message` (services/flightdeck/src/remote.rs:1501) … 3 more  
tests_for 6: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1492), `tests::runtime` (services/flightdeck/src/remote.rs:1491), `ExpressionError::message` (services/flightdeck/src/remote.rs:1501) … 3 more  

### 70. Test tests::local_query_stays_local
`rust:services/flightdeck/src/remote.rs::tests::local_query_stays_local`  services/flightdeck/src/remote.rs:1441-1447  pkg:cargo/flightdeck-server  score 0.26  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1443, 1.00)  
`fn local_query_stays_local()`  
calls 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1443), `tests::runtime` (services/flightdeck/src/remote.rs:1442)  
references 1: `QueryPlan::Local` (services/flightdeck/src/remote.rs:1446)  
tests_for 2: `tests::runtime` (services/flightdeck/src/remote.rs:1442), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1443)  

### 71. Function module
`rust:bootstrap/cw-host/src/modules/pg.rs::module`  bootstrap/cw-host/src/modules/pg.rs:277-294  pkg:cargo/cw-host  score 0.25  via `connect` <- DEFINES (bootstrap/cw-host/src/modules/pg.rs:280, 1.00)  
`pub fn module(engine: &mut Engine) -> Module`  
members 4  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:35)  
uses_types 1: `Pg` (bootstrap/cw-host/src/modules/pg.rs:278)  

### 72. File crates/causeway-testkit/src/topology.rs
`file:crates/causeway-testkit/src/topology.rs`  crates/causeway-testkit/src/topology.rs  pkg:cargo/causeway-testkit  score 0.25  via `PostgresSecret` <- CONTAINS (crates/causeway-testkit/src/topology.rs:128-140, 1.00)  
members 26  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1218)  
imports 9: `QdrantGrpcUrl` (crates/causeway-testkit/src/topology.rs:12), `ValkeyUrl` (crates/causeway-testkit/src/topology.rs:13), `S3Endpoint` (crates/causeway-testkit/src/topology.rs:10) … 6 more  

### 73. Function postgres_cell_to_string
`rust:services/flightdeck/src/remote.rs::postgres_cell_to_string`  services/flightdeck/src/remote.rs:901-941  pkg:cargo/flightdeck-server  score 0.25  via `send_postgres_batch` -> CALLS (services/flightdeck/src/remote.rs:886, 1.00)  
`fn postgres_cell_to_string(row: &tokio_postgres::Row, index: usize) -> Option<String>`  
callers 1: `send_postgres_batch` (services/flightdeck/src/remote.rs:886)  
uses_types 1: `Row` (services/flightdeck/src/remote.rs:901)  

### 74. Variant StoreKind::IcebergRest
`rust:crates/catalog-domain/src/binding.rs::StoreKind::IcebergRest`  crates/catalog-domain/src/binding.rs:21  pkg:cargo/catalog-domain  score 0.25  via `StoreKind` -> DEFINES (crates/catalog-domain/src/binding.rs:21, 1.00)  
`IcebergRest`  
referenced_by 10: `StoreKind::as_str` (crates/catalog-domain/src/binding.rs:37), `SourceProfile::catalog_container` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:260), `SourceProfile::iceberg_rest` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:140) … 7 more  

### 75. Enum DownstreamTokenConfig
`rust:crates/flightdeck-auth/src/oauth.rs::DownstreamTokenConfig`  crates/flightdeck-auth/src/oauth.rs:15-32  pkg:cargo/flightdeck-auth  score 0.25  via `PostgreSqlSource` -> USES_TYPE (services/flightdeck/src/remote.rs:144, 1.00)  
`pub enum DownstreamTokenConfig`  
Bearer acquisition strategy for downstream backends.  
members 3  
importers 2: `services/flightdeck/src/remote.rs` (services/flightdeck/src/remote.rs:19), `apps/causeway-server/src/query_flight_auth.rs` (apps/causeway-server/src/query_flight_auth.rs:52)  
referenced_by 1: `RemoteCatalogClient::bearer_for` (apps/causeway-server/src/composition/remote_services/catalog.rs:174)  
used_by 4: `acquire_downstream_token` (crates/flightdeck-auth/src/oauth.rs:52), `PostgreSqlSource` (services/flightdeck/src/remote.rs:144), `FlightdeckAuth` (apps/causeway-server/src/query_flight_auth.rs:74) … 1 more  

### 76. Function opt_int
`rust:bootstrap/cw-host/src/convert.rs::opt_int`  bootstrap/cw-host/src/convert.rs:108-117  pkg:cargo/cw-host  score 0.25  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:31, 1.00)  
`pub fn opt_int(opts: &Map, key: &str) -> Res<Option<INT>>`  
callers 9: `poll` (bootstrap/cw-host/src/modules/wait.rs:39), `for_port` (bootstrap/cw-host/src/modules/wait.rs:55), `delete` (bootstrap/cw-host/src/modules/k8s.rs:400) … 6 more  
calls 1: `err` (bootstrap/cw-host/src/convert.rs:115)  
importers 4: `bootstrap/cw-host/src/modules/http.rs` (bootstrap/cw-host/src/modules/http.rs:48-50), `bootstrap/cw-host/src/modules/wait.rs` (bootstrap/cw-host/src/modules/wait.rs:18), `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:70-72) … 1 more  
uses_types 1: `Res` (bootstrap/cw-host/src/convert.rs:108)  

### 77. Function opt_str
`rust:bootstrap/cw-host/src/convert.rs::opt_str`  bootstrap/cw-host/src/convert.rs:96-106  pkg:cargo/cw-host  score 0.25  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:31, 1.00)  
`pub fn opt_str(opts: &Map, key: &str) -> Res<Option<String>>`  
Read an optional string option. `()` and a missing key are both "absent".  
callers 26: `port_forward` (bootstrap/cw-host/src/modules/k8s.rs:567), `connect` (bootstrap/cw-host/src/modules/pg.rs:49), `start` (bootstrap/cw-host/src/modules/proc.rs:337) … 23 more  
calls 3: `err` (bootstrap/cw-host/src/convert.rs:101), `is_string` (bootstrap/cw-host/src/convert.rs:100), `FieldDto::type_name` (bootstrap/cw-host/src/convert.rs:103)  
importers 5: `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `bootstrap/cw-host/src/modules/http.rs` (bootstrap/cw-host/src/modules/http.rs:48-50), `bootstrap/cw-host/src/modules/pg.rs` (bootstrap/cw-host/src/modules/pg.rs:31) … 2 more  
uses_types 1: `Res` (bootstrap/cw-host/src/convert.rs:96)  

### 78. Struct RemoteQueryPlan
`rust:services/flightdeck/src/remote.rs::RemoteQueryPlan`  services/flightdeck/src/remote.rs:497-503  pkg:cargo/flightdeck-server  score 0.25  via `RemoteRuntime::execute` -> USES_TYPE (services/flightdeck/src/remote.rs:439, 1.00)  
`pub struct RemoteQueryPlan`  
Routed remote query metadata.  
used_by 3: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:413), `RemoteRuntime::execute` (services/flightdeck/src/remote.rs:439), `QueryPlan::Remote` (services/flightdeck/src/remote.rs:492)  
uses_types 2: `RemoteSource` (services/flightdeck/src/remote.rs:498), `RemoteExecutionMode` (services/flightdeck/src/remote.rs:499)  

### 79. Test tests::postgres_query_rewrites_source_prefix
`rust:services/flightdeck/src/remote.rs::tests::postgres_query_rewrites_source_prefix`  services/flightdeck/src/remote.rs:1450-1467  pkg:cargo/flightdeck-server  score 0.24  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1452, 1.00)  
`fn postgres_query_rewrites_source_prefix()`  
calls 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1452), `tests::runtime` (services/flightdeck/src/remote.rs:1451)  
references 2: `QueryPlan::Remote` (services/flightdeck/src/remote.rs:1458), `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:1454)  
tests_for 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1452), `tests::runtime` (services/flightdeck/src/remote.rs:1451)  

### 80. Variant RemoteExecutionMode::Translated
`rust:services/flightdeck/src/remote.rs::RemoteExecutionMode::Translated`  services/flightdeck/src/remote.rs:48  pkg:cargo/flightdeck-server  score 0.24  via `RemoteSource::allows_mode` -> REFERENCES (services/flightdeck/src/remote.rs:121, 1.00)  
`Translated`  
referenced_by 9: `default_remote_modes` (services/flightdeck/src/remote.rs:192), `RemoteSource::default_mode` (services/flightdeck/src/remote.rs:112), `RemoteSource::allows_mode` (services/flightdeck/src/remote.rs:121) … 6 more  

### 81. Test tests::a_statement_with_effects_is_never_sent_to_a_remote_source
`rust:services/flightdeck/src/remote.rs::tests::a_statement_with_effects_is_never_sent_to_a_remote_source`  services/flightdeck/src/remote.rs:1470-1487  pkg:cargo/flightdeck-server  score 0.24  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1479, 1.00)  
`fn a_statement_with_effects_is_never_sent_to_a_remote_source()`  
calls 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1479), `tests::runtime` (services/flightdeck/src/remote.rs:1478)  
references 1: `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:1479)  
tests_for 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1479), `tests::runtime` (services/flightdeck/src/remote.rs:1478)  

### 82. Function bind
`rust:bootstrap/cw-host/src/modules/pg.rs::bind`  bootstrap/cw-host/src/modules/pg.rs:232-247  pkg:cargo/cw-host  score 0.24  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:232-247, 1.00)  
`fn bind(pg: &Pg, sql: &str, params: &Array) -> Res<(tokio_postgres::Statement, Vec<Param>)>`  
callers 2: `query` (bootstrap/cw-host/src/modules/pg.rs:258), `execute` (bootstrap/cw-host/src/modules/pg.rs:266)  
calls 4: `block_on` (bootstrap/cw-host/src/modules/pg.rs:233), `err` (bootstrap/cw-host/src/modules/pg.rs:235), `param` (bootstrap/cw-host/src/modules/pg.rs:244) … 1 more  
uses_types 4: `Param` (bootstrap/cw-host/src/modules/pg.rs:232), `Res` (bootstrap/cw-host/src/modules/pg.rs:232), `Pg` (bootstrap/cw-host/src/modules/pg.rs:232) … 1 more  

### 83. Package cargo/rhai
`pkg:cargo/rhai`  pkg:cargo/rhai  score 0.23  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:27, 1.00)  
dependents 3: `cargo/cw-vsa` (bootstrap/cw-vsa/Cargo.toml:17), `cargo/cw-host` (bootstrap/cw-host/Cargo.toml:19), `cargo/cwsh` (bootstrap/cwsh/Cargo.toml:12)  
importers 22: `bootstrap/cw-host/src/modules/fs.rs` (bootstrap/cw-host/src/modules/fs.rs:38), `bootstrap/cw-host/src/modules/re.rs` (bootstrap/cw-host/src/modules/re.rs:23), `bootstrap/cw-vsa/src/main.rs` (bootstrap/cw-vsa/src/main.rs:19) … 19 more  

### 84. Function json_to_dynamic
`rust:bootstrap/cw-host/src/convert.rs::json_to_dynamic`  bootstrap/cw-host/src/convert.rs:23-47  pkg:cargo/cw-host  score 0.23  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:31, 1.00)  
`pub fn json_to_dynamic(value: Json) -> Dynamic`  
callers 3: `parse` (bootstrap/cw-host/src/modules/data.rs:87), `yaml_docs` (bootstrap/cw-host/src/modules/data.rs:40), `request` (bootstrap/cw-host/src/modules/http.rs:145)  
importers 5: `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:70-72), `bootstrap/cw-host/src/modules/pg.rs` (bootstrap/cw-host/src/modules/pg.rs:31), `bootstrap/cw-host/src/modules/data.rs` (bootstrap/cw-host/src/modules/data.rs:17) … 2 more  
references 2: `Json::Null` (bootstrap/cw-host/src/convert.rs:25), `Json::Bool` (bootstrap/cw-host/src/convert.rs:26)  
uses_types 2: `Json` (bootstrap/cw-host/src/convert.rs:23), `Json` (bootstrap/cw-host/src/convert.rs:23)  

### 85. Module runtime
`rust:bootstrap/cw-host/src/lib.rs::runtime`  bootstrap/cw-host/src/lib.rs:35  pkg:cargo/cw-host  score 0.23  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:32, 1.00)  
`mod runtime;`  
importers 4: `bootstrap/cw-host/src/modules/pg.rs` (bootstrap/cw-host/src/modules/pg.rs:32), `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:73), `bootstrap/cw-host/src/modules/http.rs` (bootstrap/cw-host/src/modules/http.rs:51) … 1 more  

### 86. Function runtime
`rust:bootstrap/cw-host/src/runtime.rs::runtime`  bootstrap/cw-host/src/runtime.rs:11-23  pkg:cargo/cw-host  score 0.23  via `bootstrap/cw-host/src/modules/pg.rs` -> IMPORTS (bootstrap/cw-host/src/modules/pg.rs:32, 1.00)  
`fn runtime() -> &'static tokio::runtime::Runtime`  
callers 3: `spawn` (bootstrap/cw-host/src/runtime.rs:33), `spawn_abortable` (bootstrap/cw-host/src/runtime.rs:41), `block_on` (bootstrap/cw-host/src/runtime.rs:26)  
calls 1: `build` (bootstrap/cw-host/src/runtime.rs:20)  
importers 4: `bootstrap/cw-host/src/modules/s3.rs` (bootstrap/cw-host/src/modules/s3.rs:23), `bootstrap/cw-host/src/modules/k8s.rs` (bootstrap/cw-host/src/modules/k8s.rs:73), `bootstrap/cw-host/src/modules/http.rs` (bootstrap/cw-host/src/modules/http.rs:51) … 1 more  
uses_types 1: `Runtime` (bootstrap/cw-host/src/runtime.rs:11)  

### 87. Impl RemoteRuntime#impl
`rust:services/flightdeck/src/remote.rs::RemoteRuntime#impl`  services/flightdeck/src/remote.rs:231-479  pkg:cargo/flightdeck-server  score 0.23  via `RemoteRuntime::execute` <- DEFINES (services/flightdeck/src/remote.rs:437-478, 1.00)  
`impl RemoteRuntime`  
members 9  
uses_types 1: `RemoteRuntime` (services/flightdeck/src/remote.rs:231-479)  

### 88. Method RemoteConfig::source_by_name
`rust:services/flightdeck/src/remote.rs::RemoteConfig::source_by_name`  services/flightdeck/src/remote.rs:83-85  pkg:cargo/flightdeck-server  score 0.23  via `RemoteSource` <- USES_TYPE (services/flightdeck/src/remote.rs:83, 1.00)  
`fn source_by_name(&self, name: &str) -> Option<&RemoteSource>`  
callers 4: `RemoteRuntime::has_source` (services/flightdeck/src/remote.rs:245), `RemoteRuntime::databricks_workspace_url` (services/flightdeck/src/remote.rs:301), `classify_source` (services/flightdeck/src/remote.rs:600) … 1 more  
calls 1: `RemoteSource::name` (services/flightdeck/src/remote.rs:84)  
uses_types 1: `RemoteSource` (services/flightdeck/src/remote.rs:83)  

### 89. Test tests::unity_query_rewrites_to_aliases
`rust:services/flightdeck/src/remote.rs::tests::unity_query_rewrites_to_aliases`  services/flightdeck/src/remote.rs:1507-1525  pkg:cargo/flightdeck-server  score 0.23  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1509, 1.00)  
`fn unity_query_rewrites_to_aliases()`  
calls 2: `tests::runtime` (services/flightdeck/src/remote.rs:1508), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1509)  
references 2: `QueryPlan::Remote` (services/flightdeck/src/remote.rs:1515), `RemoteExecutionMode::Translated` (services/flightdeck/src/remote.rs:1511)  
tests_for 2: `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1509), `tests::runtime` (services/flightdeck/src/remote.rs:1508)  

### 90. Test tests::catalog_alias_routes_to_the_warehouse_under_its_catalog_name
`rust:services/flightdeck/src/remote.rs::tests::catalog_alias_routes_to_the_warehouse_under_its_catalog_name`  services/flightdeck/src/remote.rs:1364-1428  pkg:cargo/flightdeck-server  score 0.22  via `RemoteRuntime::plan_query` -> TESTED_BY (services/flightdeck/src/remote.rs:1396, 1.00)  
`fn catalog_alias_routes_to_the_warehouse_under_its_catalog_name()`  
calls 8: `default_statement_timeout_secs` (services/flightdeck/src/remote.rs:1380), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1424), `RemoteRuntime::register_catalog_table` (services/flightdeck/src/remote.rs:1385) … 5 more  
references 3: `RemoteExecutionMode::PassThrough` (services/flightdeck/src/remote.rs:1376), `QueryPlan::Local` (services/flightdeck/src/remote.rs:1426), `QueryPlan::Remote` (services/flightdeck/src/remote.rs:1401)  
tests_for 7: `default_remote_modes` (services/flightdeck/src/remote.rs:1375), `RemoteRuntime::register_catalog_table` (services/flightdeck/src/remote.rs:1385), `RemoteRuntime::plan_query` (services/flightdeck/src/remote.rs:1396) … 4 more  
uses_types 2: `RemoteConfig` (services/flightdeck/src/remote.rs:1369), `DatabricksSqlWarehouseSource` (services/flightdeck/src/remote.rs:1371)  

### 91. Function with_causes
`rust:bootstrap/cw-host/src/modules/pg.rs::with_causes`  bootstrap/cw-host/src/modules/pg.rs:107-119  pkg:cargo/cw-host  score 0.22  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:107-119, 1.00)  
`fn with_causes(e: &dyn std::error::Error) -> String`  
An error and its causes, `: `-joined: tokio-postgres's own message for a (doc continues, 130 bytes)  
callers 1: `open` (bootstrap/cw-host/src/modules/pg.rs:101)  
uses_types 1: `Error` (bootstrap/cw-host/src/modules/pg.rs:107)  

### 92. Module tests
`rust:services/flightdeck/src/remote.rs::tests`  services/flightdeck/src/remote.rs:1335-1526  pkg:cargo/flightdeck-server  score 0.21  via `tests::runtime` <- DEFINES (services/flightdeck/src/remote.rs:1338-1361, 1.00)  
`mod tests`  
members 8  

### 93. Function origin_at
`rhai:.mise/rhai/causeway/cluster_preset_test.rhai::origin_at`  .mise/rhai/causeway/cluster_preset_test.rhai:192-194  pkg:npm/causeway-monorepo  score 0.21  via `test_a_preset_naming_a_remote_host_still_needs_a_local_context` -> CALLS (.mise/rhai/causeway/cluster_preset_test.rhai:204, 1.00)  
`fn origin_at(host)`  
callers 3: `test_a_preset_naming_a_remote_host_still_needs_a_local_context` (.mise/rhai/causeway/cluster_preset_test.rhai:204), `test_a_loopback_preset_passes_the_guard_whatever_the_context` (.mise/rhai/causeway/cluster_preset_test.rhai:198), `test_a_loopback_preset_passes_the_guard_whatever_the_context` (.mise/rhai/causeway/cluster_preset_test.rhai:199)  

### 94. Test tests::a_local_inline_topology_mounts_no_postgres_secret
`rust:crates/causeway-testkit/src/topology.rs::tests::a_local_inline_topology_mounts_no_postgres_secret`  crates/causeway-testkit/src/topology.rs:470-494  pkg:cargo/causeway-testkit  score 0.21  via `PostgresSecret::LocalInline` <- REFERENCES (crates/causeway-testkit/src/topology.rs:473, 1.00)  
`fn a_local_inline_topology_mounts_no_postgres_secret()`  
A local-inline topology leaves the Postgres reference at the (doc continues, 193 bytes)  
calls 7: `LocalTopology::directory` (crates/causeway-testkit/src/topology.rs:475), `LocalTopology::secrets_dir` (crates/causeway-testkit/src/topology.rs:478), `host_root` (crates/causeway-testkit/src/topology.rs:485) … 4 more  
references 1: `PostgresSecret::LocalInline` (crates/causeway-testkit/src/topology.rs:473)  
tests_for 5: `LocalTopology::directory` (crates/causeway-testkit/src/topology.rs:475), `LocalTopology::path` (crates/causeway-testkit/src/topology.rs:473), `LocalTopology::write_in` (crates/causeway-testkit/src/topology.rs:473) … 2 more  

### 95. Impl SourceProfile#impl
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile#impl`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:76-327  pkg:cargo/causeway-metadata-ops  score 0.21  via `SourceProfile::catalog_container` <- DEFINES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:253-279, 1.00)  
`impl SourceProfile`  
members 10  
uses_types 1: `SourceProfile` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:76-327)  

### 96. Function param
`rust:bootstrap/cw-host/src/modules/pg.rs::param`  bootstrap/cw-host/src/modules/pg.rs:123-173  pkg:cargo/cw-host  score 0.20  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:123-173, 1.00)  
`fn param(v: &Dynamic, ty: &Type) -> Res<Param>`  
members 1  
callers 1: `bind` (bootstrap/cw-host/src/modules/pg.rs:244)  
calls 21: `param::nullable` (bootstrap/cw-host/src/modules/pg.rs:139), `err` (bootstrap/cw-host/src/modules/pg.rs:136), `param::nullable` (bootstrap/cw-host/src/modules/pg.rs:171) … 18 more  
uses_types 2: `Res` (bootstrap/cw-host/src/modules/pg.rs:123), `Param` (bootstrap/cw-host/src/modules/pg.rs:123)  

### 97. Function batch
`rust:bootstrap/cw-host/src/modules/pg.rs::batch`  bootstrap/cw-host/src/modules/pg.rs:272-275  pkg:cargo/cw-host  score 0.20  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:272-275, 1.00)  
`fn batch(pg: &Pg, sql: &str) -> Res<()>`  
callers 1: `batch` (bootstrap/cw-host/src/modules/pg.rs:292)  
calls 3: `err` (bootstrap/cw-host/src/modules/pg.rs:274), `block_on` (bootstrap/cw-host/src/modules/pg.rs:274), `check` (bootstrap/cw-host/src/modules/pg.rs:273)  
uses_types 2: `Res` (bootstrap/cw-host/src/modules/pg.rs:272), `Pg` (bootstrap/cw-host/src/modules/pg.rs:272)  

### 98. Function execute
`rust:bootstrap/cw-host/src/modules/pg.rs::execute`  bootstrap/cw-host/src/modules/pg.rs:264-270  pkg:cargo/cw-host  score 0.20  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:264-270, 1.00)  
`fn execute(pg: &Pg, sql: &str, params: &Array) -> Res<INT>`  
callers 2: `execute` (bootstrap/cw-host/src/modules/pg.rs:290), `execute` (bootstrap/cw-host/src/modules/pg.rs:287)  
calls 5: `refs` (bootstrap/cw-host/src/modules/pg.rs:267), `check` (bootstrap/cw-host/src/modules/pg.rs:265), `bind` (bootstrap/cw-host/src/modules/pg.rs:266) … 2 more  
uses_types 2: `Pg` (bootstrap/cw-host/src/modules/pg.rs:264), `Res` (bootstrap/cw-host/src/modules/pg.rs:264)  

### 99. Function query
`rust:bootstrap/cw-host/src/modules/pg.rs::query`  bootstrap/cw-host/src/modules/pg.rs:256-262  pkg:cargo/cw-host  score 0.20  via `bootstrap/cw-host/src/modules/pg.rs` -> CONTAINS (bootstrap/cw-host/src/modules/pg.rs:256-262, 1.00)  
`fn query(pg: &Pg, sql: &str, params: &Array) -> Res<Array>`  
callers 2: `query` (bootstrap/cw-host/src/modules/pg.rs:282), `query` (bootstrap/cw-host/src/modules/pg.rs:284)  
calls 6: `block_on` (bootstrap/cw-host/src/modules/pg.rs:259), `bind` (bootstrap/cw-host/src/modules/pg.rs:258), `rows_to_array` (bootstrap/cw-host/src/modules/pg.rs:261) … 3 more  
uses_types 2: `Pg` (bootstrap/cw-host/src/modules/pg.rs:256), `Res` (bootstrap/cw-host/src/modules/pg.rs:256)  

### 100. Method StoreKind::as_str
`rust:crates/catalog-domain/src/binding.rs::StoreKind::as_str`  crates/catalog-domain/src/binding.rs:31-42  pkg:cargo/catalog-domain  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/catalog-domain/src/binding.rs:35, 1.00)  
`pub const fn as_str(self) -> &'static str`  
The storage representation.  
references 9: `StoreKind::UnityCatalog` (crates/catalog-domain/src/binding.rs:33), `StoreKind::Snowflake` (crates/catalog-domain/src/binding.rs:34), `StoreKind::IcebergRest` (crates/catalog-domain/src/binding.rs:37) … 6 more  

### 101. Method StoreKind::label
`rust:crates/catalog-domain/src/binding.rs::StoreKind::label`  crates/catalog-domain/src/binding.rs:46-57  pkg:cargo/catalog-domain  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/catalog-domain/src/binding.rs:50, 1.00)  
`pub const fn label(self) -> &'static str`  
The name a reader sees for the store kind.  
callers 1: `ProposalRow::into_proposal` (component/catalog/core/src/placements.rs:671)  
references 9: `StoreKind::Snowflake` (crates/catalog-domain/src/binding.rs:49), `StoreKind::UnityCatalog` (crates/catalog-domain/src/binding.rs:48), `StoreKind::IcebergRest` (crates/catalog-domain/src/binding.rs:52) … 6 more  

### 102. Method StoreKind::parse
`rust:crates/catalog-domain/src/binding.rs::StoreKind::parse`  crates/catalog-domain/src/binding.rs:64-76  pkg:cargo/catalog-domain  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/catalog-domain/src/binding.rs:68, 1.00)  
`pub fn parse(raw: &str) -> Result<Self, UnknownVocabulary>`  
Parses the storage representation. (doc continues, 103 bytes)  
callers 6: `BindingRow::into_open` (component/catalog/core/src/bindings.rs:1291), `tests::vocabularies_round_trip_and_reject_strangers` (crates/catalog-domain/src/binding.rs:1704), `ProposalRow::into_proposal` (component/catalog/core/src/placements.rs:669) … 3 more  
calls 1: `UnknownVocabulary::new` (crates/catalog-domain/src/binding.rs:74)  
references 9: `StoreKind::HttpApi` (crates/catalog-domain/src/binding.rs:71), `StoreKind::Snowflake` (crates/catalog-domain/src/binding.rs:67), `StoreKind::IcebergRest` (crates/catalog-domain/src/binding.rs:70) … 6 more  
tests 1: `tests::vocabularies_round_trip_and_reject_strangers` (crates/catalog-domain/src/binding.rs:1702)  
uses_types 1: `UnknownVocabulary` (crates/catalog-domain/src/binding.rs:64)  

### 103. Method SourceProfile::locator
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::locator`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:153-169  pkg:cargo/causeway-metadata-ops  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:155, 1.00)  
`fn locator(&self, store_instance: &str, coordinates: &ConnectorTableRef) -> String`  
The locator of a table, as bindings and `file_path` carry it.  
callers 14: `upsert_catalog_asset_for` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1954), `same_asset_clone_source` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1621), `sync_asset` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:962) … 11 more  
calls 6: `MetadataOpsContext::catalog` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:163), `MetadataOperatorRepos::catalog` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:163), `tests::schema` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:164) … 3 more  
references 3: `StoreKind::Postgresql` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:155), `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:158), `StoreKind::IcebergRest` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:161)  
tests 3: `tests::tools_and_resources_bind_to_the_server_they_are_served_from` (crates/causeway-metadata-ops/src/mcp_registry.rs:355), `tests::airflow_openlineage_maps_dag_owners_schedule_and_output` (crates/causeway-metadata-ops/src/source_contracts.rs:1022), `tests::an_operation_binds_to_its_api` (crates/causeway-metadata-ops/src/openapi_import.rs:361)  
uses_types 1: `ConnectorTableRef` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:153)  

### 104. Method SourceProfile::locator_for_name
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::locator_for_name`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:172-188  pkg:cargo/causeway-metadata-ops  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:174, 1.00)  
`fn locator_for_name(&self, store_instance: &str, full_name: &str) -> String`  
The locator of a table named by its full name (a clone source).  
callers 3: `record_clone_binding` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1712), `record_clone` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1810), `same_asset_clone_source` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1617)  
references 3: `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:177), `StoreKind::Postgresql` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:174), `StoreKind::IcebergRest` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:180)  

### 105. Method SourceProfile::schema_container
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::schema_container`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:282-322  pkg:cargo/causeway-metadata-ops  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:308, 1.00)  
`fn schema_container( &self, store_instance: &str, catalog: &str, schema: &str, shared: bool, ) -> catalog_domain::PhysicalContainer`  
The schema container under a catalog-level container.  
callers 4: `same_asset_clone_source` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1629), `run_source_sync` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:697), `sync_asset` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:858) … 1 more  
calls 6: `PhysicalContainer::postgres_schema` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:309), `PhysicalContainer::unity_catalog_schema` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:316), `PhysicalContainer::snowflake_schema` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:306) … 3 more  
references 4: `StoreKind::Postgresql` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:308), `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:298), `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:305) … 1 more  
uses_types 1: `PhysicalContainer` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:288)  

### 106. Method SourceProfile::table_object
`rust:crates/causeway-metadata-ops/src/unity_catalog_sync.rs::SourceProfile::table_object`  crates/causeway-metadata-ops/src/unity_catalog_sync.rs:191-249  pkg:cargo/causeway-metadata-ops  score 0.20  via `StoreKind::Postgresql` <- REFERENCES (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:228, 1.00)  
`fn table_object( &self, store_instance: &str, coordinates: &ConnectorTableRef, shared: bool, ) -> catalog_domain::PhysicalObject`  
The physical object a table is.  
callers 2: `record_primary_binding` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1740), `record_clone_binding` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:1689)  
calls 12: `PhysicalObject::snowflake_table` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:221), `PhysicalObject::unity_catalog_table_in` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:242), `PhysicalObject::iceberg_table` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:205) … 9 more  
references 4: `StoreKind::Snowflake` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:220), `StoreKind::Postgresql` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:228), `StoreKind::IcebergRest` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:203) … 1 more  
uses_types 2: `PhysicalObject` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:196), `ConnectorTableRef` (crates/causeway-metadata-ops/src/unity_catalog_sync.rs:194)  

### 107. Struct Row
`rust:crates/ingest-corpus/tests/conformance.rs::Row`  crates/ingest-corpus/tests/conformance.rs:59-76  pkg:cargo/ingest-corpus  score 0.20  via `send_postgres_batch` -> USES_TYPE (services/flightdeck/src/remote.rs:876, 1.00)  
`struct Row`  
used_by 4: `sniffer_over_the_corpus` (crates/ingest-corpus/tests/conformance.rs:131), `postgres_cell_to_string` (services/flightdeck/src/remote.rs:901), `Table` (crates/ingest-corpus/tests/conformance.rs:54) … 1 more  
uses_types 1: `Expect` (crates/ingest-corpus/tests/conformance.rs:75)  

### 108. Method InfraPlan::start
`rust:crates/causeway-testkit/src/infra.rs::InfraPlan::start`  crates/causeway-testkit/src/infra.rs:177-246  pkg:cargo/causeway-testkit  score 0.20  via `PostgresSecret::LocalInline` <- REFERENCES (crates/causeway-testkit/src/infra.rs:217, 1.00)  
`pub async fn start(self) -> TestkitResult<RunningInfra>`  
Starts the containers concurrently, migrates the workspace into (doc continues, 396 bytes)  
callers 1: `optional` (crates/causeway-testkit/src/infra.rs:255)  
calls 74: `RunIdentity::as_str` (crates/causeway-testkit/src/infra.rs:210), `RunIdentity::as_str` (crates/causeway-testkit/src/infra.rs:207), `report` (crates/causeway-testkit/src/infra.rs:207) … 71 more  
references 9: `Service::Qdrant` (crates/causeway-testkit/src/infra.rs:193), `Service::Valkey` (crates/causeway-testkit/src/infra.rs:191), `EphemeralKafka::start` (crates/causeway-testkit/src/infra.rs:190) … 6 more  
uses_types 3: `RunningInfra` (crates/causeway-testkit/src/infra.rs:177), `Started` (crates/causeway-testkit/src/infra.rs:238), `TestkitResult` (crates/causeway-testkit/src/infra.rs:177)  

### 109. File bootstrap/cw-host/src/modules/env.rs
`file:bootstrap/cw-host/src/modules/env.rs`  bootstrap/cw-host/src/modules/env.rs  pkg:cargo/cw-host  score 0.20  via `Res` <- IMPORTS (bootstrap/cw-host/src/modules/env.rs:16, 1.00)  
members 4  
imports 3: `err_map` (bootstrap/cw-host/src/modules/env.rs:16), `cargo/rhai` (bootstrap/cw-host/src/modules/env.rs:14), `Res` (bootstrap/cw-host/src/modules/env.rs:16)  

### 110. File bootstrap/cw-host/src/modules/wait.rs
`file:bootstrap/cw-host/src/modules/wait.rs`  bootstrap/cw-host/src/modules/wait.rs  pkg:cargo/cw-host  score 0.20  via `Res` <- IMPORTS (bootstrap/cw-host/src/modules/wait.rs:18, 1.00)  
members 6  
imports 5: `cargo/rhai` (bootstrap/cw-host/src/modules/wait.rs:16), `supervise` (bootstrap/cw-host/src/modules/wait.rs:19), `opt_int` (bootstrap/cw-host/src/modules/wait.rs:18) … 2 more  

### 111. Impl StoreKind#impl
`rust:crates/catalog-domain/src/binding.rs::StoreKind#impl`  crates/catalog-domain/src/binding.rs:28-77  pkg:cargo/catalog-domain  score 0.19  via `StoreKind::as_str` <- DEFINES (crates/catalog-domain/src/binding.rs:31-42, 1.00)  
`impl StoreKind`  
members 3  
uses_types 1: `StoreKind` (crates/catalog-domain/src/binding.rs:28-77)  

## Left out

41 included nodes have no source window for want of budget.  
- `rust:services/flightdeck/src/remote.rs::RemoteRuntime::new` Method 0.19 (over budget, 304 tokens)
- `rhai:.mise/rhai/causeway/cluster_preset_test.rhai::test_a_loopback_preset_passes_the_guard_whatever_the_context` Function 0.19 (over budget, 213 tokens)
- `rust:bootstrap/cw-host/src/modules/pg.rs::Param` TypeAlias 0.18 (over budget, 145 tokens)
- and 290 more

## Unresolved references in the seeds' files

- `source` at bootstrap/cw-host/src/modules/pg.rs:109 (ambiguous)
- `push_str` at bootstrap/cw-host/src/modules/pg.rs:113 (no-declaration)
- `push_str` at bootstrap/cw-host/src/modules/pg.rs:114 (no-declaration)
- `source` at bootstrap/cw-host/src/modules/pg.rs:116 (ambiguous)
- `as_int` at bootstrap/cw-host/src/modules/pg.rs:135 (no-declaration)
- `int` at bootstrap/cw-host/src/modules/pg.rs:146 (no-declaration)
- `try_from` at bootstrap/cw-host/src/modules/pg.rs:146 (qualifier-mismatch)
- `int` at bootstrap/cw-host/src/modules/pg.rs:150 (no-declaration)
- `try_from` at bootstrap/cw-host/src/modules/pg.rs:150 (qualifier-mismatch)
- `int` at bootstrap/cw-host/src/modules/pg.rs:152 (no-declaration)

