<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `dad123812024`, dataset at `a1d4cb088849`: **8 files, 100 changed symbols, 173 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (100)**

| Symbol | Kind | Where |
|---|---|---|
| `ServerCli` | Struct | `services/flightdeck/src/config.rs:460-672` |
| `postgres_tls` | Module | `services/flightdeck/src/lib.rs:19` |
| `PostgresDsn` | Struct | `services/flightdeck/src/postgres_tls.rs:111-115` |
| `PostgresDsn#impl` | Impl | `services/flightdeck/src/postgres_tls.rs:117-139` |
| `PostgresDsn#impl(FromStr)` | Impl | `services/flightdeck/src/postgres_tls.rs:141-175` |
| `PostgresDsn#impl(TryFrom)` | Impl | `services/flightdeck/src/postgres_tls.rs:177-183` |
| `PostgresDsn::Err` | TypeAlias | `services/flightdeck/src/postgres_tls.rs:142` |
| `PostgresDsn::Error` | TypeAlias | `services/flightdeck/src/postgres_tls.rs:178` |
| `PostgresDsn::config_for` | Method | `services/flightdeck/src/postgres_tls.rs:135-138` |
| `PostgresDsn::from_str` | Method | `services/flightdeck/src/postgres_tls.rs:144-174` |
| `PostgresDsn::root_cert` | Method | `services/flightdeck/src/postgres_tls.rs:126-128` |
| `PostgresDsn::ssl_mode` | Method | `services/flightdeck/src/postgres_tls.rs:120-122` |
| `PostgresDsn::try_from` | Method | `services/flightdeck/src/postgres_tls.rs:180-182` |
| `PostgresDsnError` | Enum | `services/flightdeck/src/postgres_tls.rs:92-103` |
| `PostgresDsnError::Invalid` | Variant | `services/flightdeck/src/postgres_tls.rs:102` |
| `PostgresDsnError::SslMode` | Variant | `services/flightdeck/src/postgres_tls.rs:96` |
| `PostgresDsnError::Syntax` | Variant | `services/flightdeck/src/postgres_tls.rs:98` |
| `PostgresDsnError::SystemRoots` | Variant | `services/flightdeck/src/postgres_tls.rs:100` |
| `PostgresTlsError` | Enum | `services/flightdeck/src/postgres_tls.rs:367-393` |
| `PostgresTlsError::PlaintextOutsideLocal` | Variant | `services/flightdeck/src/postgres_tls.rs:373-376` |
| `PostgresTlsError::RootCertEmpty` | Variant | `services/flightdeck/src/postgres_tls.rs:384` |
| `PostgresTlsError::RootCertRead` | Variant | `services/flightdeck/src/postgres_tls.rs:378-382` |
| `PostgresTlsError::RootCertRejected` | Variant | `services/flightdeck/src/postgres_tls.rs:386-390` |
| `PostgresTlsError::Tls` | Variant | `services/flightdeck/src/postgres_tls.rs:392` |
| `PostgresTransport` | Enum | `services/flightdeck/src/postgres_tls.rs:288-296` |
| `PostgresTransport#impl` | Impl | `services/flightdeck/src/postgres_tls.rs:395-411` |
| `PostgresTransport#impl(Debug)` | Impl | `services/flightdeck/src/postgres_tls.rs:298-308` |
| `PostgresTransport::Plaintext` | Variant | `services/flightdeck/src/postgres_tls.rs:290` |
| `PostgresTransport::Tls` | Variant | `services/flightdeck/src/postgres_tls.rs:292-295` |
| `PostgresTransport::fmt` | Method | `services/flightdeck/src/postgres_tls.rs:299-307` |
| `PostgresTransport::resolve` | Method | `services/flightdeck/src/postgres_tls.rs:402-410` |
| `SSL_KEYS` | Const | `services/flightdeck/src/postgres_tls.rs:186` |
| `SUBJECT_PLACEHOLDER` | Const | `services/flightdeck/src/postgres_tls.rs:40` |
| `SslMode` | Enum | `services/flightdeck/src/postgres_tls.rs:44-51` |
| `SslMode#impl` | Impl | `services/flightdeck/src/postgres_tls.rs:53-66` |
| `SslMode#impl(Display)` | Impl | `services/flightdeck/src/postgres_tls.rs:68-72` |
| `SslMode#impl(FromStr)` | Impl | `services/flightdeck/src/postgres_tls.rs:74-88` |
| `SslMode::Allow` | Variant | `services/flightdeck/src/postgres_tls.rs:46` |
| `SslMode::Disable` | Variant | `services/flightdeck/src/postgres_tls.rs:45` |
| `SslMode::Err` | TypeAlias | `services/flightdeck/src/postgres_tls.rs:75` |

… and 60 more in the JSON.

Changed as whole files (no declaration at the changed lines): `services/flightdeck/Cargo.toml`.
Not in the dataset: `services/Cargo.lock`.

**Affected (173 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (162)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `CatalogRemoteRouting` | Struct | `services/flightdeck/src/catalog_refresh.rs:69-75` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/catalog_refresh.rs:70` |
| `ServerConfig::from_cli` | Method | `services/flightdeck/src/config.rs:702-804` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/config.rs:702` |
| `ServerConfigError::Cli` | Variant | `services/flightdeck/src/config.rs:28` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/config.rs:28` |
| `catalog_credentials` | Function | `services/flightdeck/src/config.rs:306-394` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/config.rs:307` |
| `tests::catalog_cli` | Function | `services/flightdeck/src/config.rs:1346-1363` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/config.rs:1346` |
| `tests::expect_cli_error` | Function | `services/flightdeck/src/config.rs:1252-1259` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/config.rs:1252` |
| `RemoteSource::PostgreSql` | Variant | `services/flightdeck/src/remote.rs:104` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/remote.rs:104` |
| `tests::a_statement_with_effects_is_never_sent_to_a_remote_source` | Test | `services/flightdeck/src/remote.rs:1683-1700` | 1 | 1.00 | CALLS | `services/flightdeck/src/remote.rs:1691` |
| `tests::local_query_stays_local` | Test | `services/flightdeck/src/remote.rs:1654-1660` | 1 | 1.00 | CALLS | `services/flightdeck/src/remote.rs:1655` |
| `tests::mixed_local_and_remote_query_is_rejected` | Test | `services/flightdeck/src/remote.rs:1703-1717` | 1 | 1.00 | CALLS | `services/flightdeck/src/remote.rs:1704` |
| `tests::postgres_query_rewrites_source_prefix` | Test | `services/flightdeck/src/remote.rs:1663-1680` | 1 | 1.00 | CALLS | `services/flightdeck/src/remote.rs:1664` |
| `tests::unity_query_rewrites_to_aliases` | Test | `services/flightdeck/src/remote.rs:1720-1738` | 1 | 1.00 | CALLS | `services/flightdeck/src/remote.rs:1721` |
| `QueryService` | Struct | `services/flightdeck/src/service.rs:197-211` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:208` |
| `QueryService::with_remote_runtime` | Method | `services/flightdeck/src/service.rs:330-333` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:330` |
| `Server#impl` | Impl | `services/flightdeck/src/startup.rs:240-315` | 1 | 1.00 | USES_TYPE | `services/flightdeck/src/startup.rs:240` |
| `TestRemoteRuntime` | TypeAlias | `services/flightdeck/tests/flight_test.rs:34` | 1 | 1.00 | USES_TYPE | `services/flightdeck/tests/flight_test.rs:34` |
| `bootstrap_catalog_refresh` | Function | `services/flightdeck/src/catalog_refresh.rs:150-181` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/catalog_refresh.rs:154` |
| `refresh_catalog_tables` | Function | `services/flightdeck/src/catalog_refresh.rs:219-325` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/catalog_refresh.rs:223` |
| `register_catalog_table` | Function | `services/flightdeck/src/catalog_refresh.rs:628-750` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/catalog_refresh.rs:631` |
| `spawn_catalog_refresh_task` | Function | `services/flightdeck/src/catalog_refresh.rs:183-217` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/catalog_refresh.rs:187` |
| `ServerConfig::from_env` | Method | `services/flightdeck/src/config.rs:815-817` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:816` |
| `ServerConfig::try_parse_from` | Method | `services/flightdeck/src/config.rs:693-700` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:699` |
| `tests::outside_local_the_catalog_client_comes_from_the_topology` | Test | `services/flightdeck/src/config.rs:1366-1428` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:1415` |
| `tests::outside_local_the_catalog_client_secret_is_never_inline` | Test | `services/flightdeck/src/config.rs:1431-1487` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:1474` |
| `tests::rejects_invalid_pool_mib` | Test | `services/flightdeck/src/config.rs:1546-1550` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:1548` |
| `tests::rejects_unknown_argument` | Test | `services/flightdeck/src/config.rs:1539-1543` | 2 | 1.00 | CALLS | `services/flightdeck/src/config.rs:1541` |
| `RemoteRuntime::plan_query` | Method | `services/flightdeck/src/remote.rs:377-472` | 2 | 1.00 | REFERENCES | `services/flightdeck/src/remote.rs:434` |
| `RemoteSource::allows_mode` | Method | `services/flightdeck/src/remote.rs:129-135` | 2 | 1.00 | REFERENCES | `services/flightdeck/src/remote.rs:131` |
| `RemoteSource::backend_kind` | Method | `services/flightdeck/src/remote.rs:138-144` | 2 | 1.00 | REFERENCES | `services/flightdeck/src/remote.rs:140` |
| `RemoteSource::default_mode` | Method | `services/flightdeck/src/remote.rs:120-126` | 2 | 1.00 | REFERENCES | `services/flightdeck/src/remote.rs:122` |
| `RemoteSource::name` | Method | `services/flightdeck/src/remote.rs:111-117` | 2 | 1.00 | REFERENCES | `services/flightdeck/src/remote.rs:113` |
| `QueryService#impl` | Impl | `services/flightdeck/src/service.rs:274-564` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:274-564` |
| `QueryService#impl(Debug)` | Impl | `services/flightdeck/src/service.rs:250-272` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:250-272` |
| `QueryService#impl(FlightSqlService)` | Impl | `services/flightdeck/src/service.rs:979-1529` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:979-1529` |
| `QueryService::FlightService` | TypeAlias | `services/flightdeck/src/service.rs:980` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/service.rs:980` |
| `Bound` | Struct | `services/flightdeck/src/startup.rs:164-176` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/startup.rs:170` |
| `Server::run` | Method | `services/flightdeck/src/startup.rs:331-405` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/startup.rs:350` |
| `build_flight_service` | Function | `services/flightdeck/src/startup.rs:884-892` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/startup.rs:885` |
| `build_policy_aware_query_service` | Function | `services/flightdeck/src/startup.rs:857-882` | 2 | 1.00 | USES_TYPE | `services/flightdeck/src/startup.rs:862` |
| `TesterService` | TypeAlias | `services/flightdeck/tests/common/mod.rs:102-105` | 2 | 1.00 | USES_TYPE | `services/flightdeck/tests/common/mod.rs:103` |

… and 122 more in the JSON.

_Files and packages (11)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `services/flightdeck/src/remote.rs` | File | `services/flightdeck/src/remote.rs` | 1 | 1.00 | IMPORTS | `services/flightdeck/src/remote.rs:8` |
| `services/flightdeck/src/service.rs` | File | `services/flightdeck/src/service.rs` | 1 | 1.00 | IMPORTS | `services/flightdeck/src/service.rs:43` |
| `services/flightdeck/src/startup.rs` | File | `services/flightdeck/src/startup.rs` | 1 | 1.00 | IMPORTS | `services/flightdeck/src/startup.rs:22` |
| `services/flightdeck/tests/common/mod.rs` | File | `services/flightdeck/tests/common/mod.rs` | 2 | 1.00 | IMPORTS | `services/flightdeck/tests/common/mod.rs:28` |
| `services/pure-flight/src/oql_service.rs` | File | `services/pure-flight/src/oql_service.rs` | 2 | 1.00 | IMPORTS | `services/pure-flight/src/oql_service.rs:60-62` |
| `services/pure-flight/src/main.rs` | File | `services/pure-flight/src/main.rs` | 3 | 1.00 | IMPORTS | `services/pure-flight/src/main.rs:45` |
| `services/pure-flight/tests/e2e.rs` | File | `services/pure-flight/tests/e2e.rs` | 3 | 1.00 | IMPORTS | `services/pure-flight/tests/e2e.rs:35` |
| `services/pure-flight/tests/sample_policy.rs` | File | `services/pure-flight/tests/sample_policy.rs` | 3 | 1.00 | IMPORTS | `services/pure-flight/tests/sample_policy.rs:52` |
| `crates/causeway-runtime/src/lib.rs` | File | `crates/causeway-runtime/src/lib.rs` | 2 | 0.50 | IMPORTS | `crates/causeway-runtime/src/lib.rs:38-43` |
| `crates/causeway-runtime/src/restate_host.rs` | File | `crates/causeway-runtime/src/restate_host.rs` | 2 | 0.50 | IMPORTS | `crates/causeway-runtime/src/restate_host.rs:11-14` |
| `jobs/causeway-job-runtime/src/lib.rs` | File | `jobs/causeway-job-runtime/src/lib.rs` | 2 | 0.50 | IMPORTS | `jobs/causeway-job-runtime/src/lib.rs:12-15` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 61908 nodes, 262556 edges · 1776 ms</sub>
