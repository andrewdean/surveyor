<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `6b78aed1532a`, dataset at `e5ad13778ccd`: **10 files, 87 changed symbols, 385 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (87)**

| Symbol | Kind | Where |
|---|---|---|
| `SessionBearer` | Struct | `apps/causeway-server/src/browser_auth/middleware.rs:264` |
| `attach_browser_caller` | Function | `apps/causeway-server/src/browser_auth/middleware.rs:169-218` |
| `present_session_token` | Function | `apps/causeway-server/src/browser_auth/middleware.rs:245-258` |
| `BrowserAuth#impl` | Impl | `apps/causeway-server/src/browser_auth/mod.rs:102-219` |
| `BrowserAuth::session_cookie_for_tests` | Method | `apps/causeway-server/src/browser_auth/mod.rs:165-172` |
| `RouteAccess` | Struct | `apps/causeway-server/src/routes/access.rs:77-89` |
| `RouteAccess#impl` | Impl | `apps/causeway-server/src/routes/access.rs:91-279` |
| `RouteAccess::from_config` | Method | `apps/causeway-server/src/routes/access.rs:97-142` |
| `RouteAccess::from_topology` | Method | `apps/causeway-server/src/routes/access.rs:201-232` |
| `RouteAccess::verifying` | Method | `apps/causeway-server/src/routes/access.rs:255-265` |
| `RouteAccess::with_own_origins` | Method | `apps/causeway-server/src/routes/access.rs:162-167` |
| `TestCaller#impl` | Impl | `apps/causeway-server/src/routes/access.rs:296-396` |
| `TestCaller::mokta` | Method | `apps/causeway-server/src/routes/access.rs:323-325` |
| `build_router` | Function | `apps/causeway-server/src/routes/mod.rs:41-127` |
| `own_origins` | Module | `apps/causeway-server/src/routes/mod.rs:11` |
| `OriginVerdict` | Enum | `apps/causeway-server/src/routes/own_origins.rs:31-38` |
| `OriginVerdict::Absent` | Variant | `apps/causeway-server/src/routes/own_origins.rs:33` |
| `OriginVerdict::Foreign` | Variant | `apps/causeway-server/src/routes/own_origins.rs:37` |
| `OriginVerdict::Own` | Variant | `apps/causeway-server/src/routes/own_origins.rs:35` |
| `OwnOrigins` | Struct | `apps/causeway-server/src/routes/own_origins.rs:25-27` |
| `OwnOrigins#impl` | Impl | `apps/causeway-server/src/routes/own_origins.rs:40-83` |
| `OwnOrigins::from_topology` | Method | `apps/causeway-server/src/routes/own_origins.rs:44-52` |
| `OwnOrigins::judge` | Method | `apps/causeway-server/src/routes/own_origins.rs:66-82` |
| `OwnOrigins::new` | Method | `apps/causeway-server/src/routes/own_origins.rs:56-63` |
| `is_same_origin` | Function | `apps/causeway-server/src/routes/own_origins.rs:86-98` |
| `origin_of_url` | Function | `apps/causeway-server/src/routes/own_origins.rs:113-127` |
| `requested_host` | Function | `apps/causeway-server/src/routes/own_origins.rs:102-110` |
| `tests` | Module | `apps/causeway-server/src/routes/own_origins.rs:130-200` |
| `tests::a_forwarded_host_is_compared_without_regard_to_case` | Test | `apps/causeway-server/src/routes/own_origins.rs:172-179` |
| `tests::a_sibling_null_and_garbage_are_foreign_and_no_origin_is_absent` | Test | `apps/causeway-server/src/routes/own_origins.rs:182-199` |
| `tests::headers` | Function | `apps/causeway-server/src/routes/own_origins.rs:143-153` |
| `tests::origins` | Function | `apps/causeway-server/src/routes/own_origins.rs:136-141` |
| `tests::the_requests_own_host_the_public_base_and_cors_are_own` | Test | `apps/causeway-server/src/routes/own_origins.rs:156-169` |
| `routes` | Function | `apps/causeway-server/src/routes/rpc.rs:8-214` |
| `rpc_csrf` | Module | `apps/causeway-server/src/transport.rs:24` |
| `handler` | Function | `apps/causeway-server/src/transport/connect_proxy.rs:69-155` |
| `tests::proxy_forwards_browser_connect_call_to_live_relay` | Test | `apps/causeway-server/src/transport/connect_proxy.rs:211-241` |
| `Admitted` | Enum | `apps/causeway-server/src/transport/rpc_csrf.rs:70-82` |
| `Admitted::ConnectProtocolHeader` | Variant | `apps/causeway-server/src/transport/rpc_csrf.rs:75` |
| `Admitted::NotCookieAuthenticated` | Variant | `apps/causeway-server/src/transport/rpc_csrf.rs:72` |

… and 47 more in the JSON.


**Affected (385 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (312)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `AtlasError::code` | Method | `apps/causeway-server/src/atlas.rs:46-59` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/atlas.rs:49` |
| `AtlasError::fmt` | Method | `apps/causeway-server/src/atlas.rs:63-87` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/atlas.rs:79` |
| `DecidedAtlasCall::decide` | Method | `apps/causeway-server/src/atlas.rs:136-166` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/atlas.rs:151` |
| `canonical_semantic_artifacts_are_mounted_in_the_application_router` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:412-475` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:415` |
| `mounted_rights_service_saves_drafts_with_optimistic_concurrency` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:482-676` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:502` |
| `mounted_semantic_neighborhood_enforces_authorization_and_limits` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:699-861` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:813` |
| `remote_service_ports_smoke_against_rpc_router` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:350-409` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:359` |
| `AppState::mock_for_tests_with_access` | Method | `apps/causeway-server/src/composition/state.rs:355-366` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state.rs:357` |
| `ObjectRefusal::fmt` | Method | `apps/causeway-server/src/explorer/entry_object.rs:255-281` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/explorer/entry_object.rs:262` |
| `EntryRefusal::for_dataset` | Method | `apps/causeway-server/src/explorer/governed.rs:323-332` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/explorer/governed.rs:326` |
| `EntryRefusal::from_decision` | Method | `apps/causeway-server/src/explorer/governed.rs:306-314` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/explorer/governed.rs:311` |
| `PageRefusal::entry` | Method | `apps/causeway-server/src/explorer/page.rs:226-236` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/explorer/page.rs:229` |
| `RouteAccess::for_tests` | Method | `apps/causeway-server/src/routes/access.rs:239-250` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/access.rs:241` |
| `TestCaller` | Struct | `apps/causeway-server/src/routes/access.rs:289-293` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/access.rs:291` |
| `TestCaller::access` | Method | `apps/causeway-server/src/routes/access.rs:328-330` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/access.rs:328` |
| `TestCaller::start` | Method | `apps/causeway-server/src/routes/access.rs:307-320` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/access.rs:314` |
| `access_from_the_topology_alone` | Function | `apps/causeway-server/src/routes/access/tests.rs:65-77` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/access/tests.rs:68` |
| `shell` | Function | `apps/causeway-server/src/routes/access/tests.rs:81-84` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/access/tests.rs:81` |
| `refused` | Function | `apps/causeway-server/src/routes/api.rs:94-131` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/routes/api.rs:101` |
| `Shell::anonymous_router` | Method | `apps/causeway-server/src/routes/api/tests.rs:195-197` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/api/tests.rs:196` |
| `shell` | Function | `apps/causeway-server/src/routes/cors/tests.rs:71-75` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/cors/tests.rs:74` |
| `api_metadata_trace` | Function | `apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:207-273` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/pages/metadata_ops/trace.rs:243` |
| `Shell::anonymous_router` | Method | `apps/causeway-server/src/routes/pages/test_shell.rs:126-128` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/pages/test_shell.rs:127` |
| `shell` | Function | `apps/causeway-server/src/routes/security_headers/tests.rs:57-68` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/security_headers/tests.rs:67` |
| `SocketGrant::decide` | Method | `apps/causeway-server/src/socket_grant.rs:50-74` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/socket_grant.rs:61` |
| `SocketRefusal::code` | Method | `apps/causeway-server/src/socket_grant.rs:111-120` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/socket_grant.rs:114` |
| `SocketRefusal::fmt` | Method | `apps/causeway-server/src/socket_grant.rs:124-133` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/socket_grant.rs:129` |
| `Binding` | Struct | `apps/causeway-server/src/startup.rs:37-40` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/startup.rs:39` |
| `Configuring` | Struct | `apps/causeway-server/src/startup.rs:32-35` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/startup.rs:34` |
| `Server::configure` | Method | `apps/causeway-server/src/startup.rs:71-110` | 1 | 1.00 | CALLS | `apps/causeway-server/src/startup.rs:89` |
| `Server::run` | Method | `apps/causeway-server/src/startup.rs:189-271` | 1 | 1.00 | CALLS | `apps/causeway-server/src/startup.rs:243` |
| `build_state` | Function | `apps/causeway-server/src/startup/runtime.rs:20-80` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/startup/runtime.rs:22` |
| `catalog_command_serving` | Function | `apps/causeway-server/src/startup/runtime.rs:136-168` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/startup/runtime.rs:139` |
| `tests::the_router_forwards_the_catalog_path_when_the_topology_says_so` | Test | `apps/causeway-server/src/transport/rpc_forward.rs:473-502` | 1 | 1.00 | CALLS | `apps/causeway-server/src/transport/rpc_forward.rs:479` |
| `Refused` | Enum | `crates/causeway-access-control/src/lib.rs:293-298` | 1 | 1.00 | USES_TYPE | `crates/causeway-access-control/src/lib.rs:293` |
| `tests::the_converted_methods_need_rpc_delegate_from_every_actor::Admitted` | Enum | `crates/connect-rpc-access/src/lib.rs:2091-2095` | 1 | 1.00 | USES_TYPE | `crates/connect-rpc-access/src/lib.rs:2091` |
| `ExplorerRejection::fmt` | Method | `apps/causeway-server-web/src/protocol/explorer.rs:190-233` | 2 | 1.00 | USES_TYPE | `apps/causeway-server-web/src/protocol/explorer.rs:196` |
| `a_metadata_admin_service_mounted_without_the_enforcement_point_refuses` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:1233-1269` | 2 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:1234` |
| `a_remote_entry_the_host_refuses_the_reader_is_answered_as_an_absent_one` | Test | `apps/causeway-server/src/composition/remote_services/tests.rs:239-274` | 2 | 1.00 | CALLS | `apps/causeway-server/src/composition/remote_services/tests.rs:240` |
| `catalog_rpc_app` | Function | `apps/causeway-server/src/composition/remote_services/tests.rs:157-165` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/remote_services/tests.rs:157` |

… and 272 more in the JSON.

_Files and packages (73)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `apps/causeway-server/src/browser_auth/mod.rs` | File | `apps/causeway-server/src/browser_auth/mod.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/browser_auth/mod.rs:28-32` |
| `apps/causeway-server/src/browser_auth/tests.rs` | File | `apps/causeway-server/src/browser_auth/tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/browser_auth/tests.rs:18-20` |
| `apps/causeway-server/src/routes/access/tests.rs` | File | `apps/causeway-server/src/routes/access/tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/access/tests.rs:22` |
| `apps/causeway-server/src/routes/cors/tests.rs` | File | `apps/causeway-server/src/routes/cors/tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/cors/tests.rs:13` |
| `apps/causeway-server/src/routes/mod.rs` | File | `apps/causeway-server/src/routes/mod.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/mod.rs:36` |
| `apps/causeway-server/src/routes/security_headers/tests.rs` | File | `apps/causeway-server/src/routes/security_headers/tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/security_headers/tests.rs:13` |
| `apps/causeway-server/src/transport/rpc_csrf.rs` | File | `apps/causeway-server/src/transport/rpc_csrf.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/transport/rpc_csrf.rs:42` |
| `apps/causeway-server/src/transport/rpc_csrf/tests.rs` | File | `apps/causeway-server/src/transport/rpc_csrf/tests.rs` | 1 | 1.00 | IMPORTS | `apps/causeway-server/src/transport/rpc_csrf/tests.rs:14` |
| `apps/causeway-server/src/composition/remote_services/tests.rs` | File | `apps/causeway-server/src/composition/remote_services/tests.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/composition/remote_services/tests.rs:23` |
| `apps/causeway-server/src/routes/pages.rs` | File | `apps/causeway-server/src/routes/pages.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages.rs:53-58` |
| `apps/causeway-server/src/routes/pages/metadata_ops.rs` | File | `apps/causeway-server/src/routes/pages/metadata_ops.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages/metadata_ops.rs:12` |
| `apps/causeway-server/src/startup/bind.rs` | File | `apps/causeway-server/src/startup/bind.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/startup/bind.rs:6` |
| `crates/flightdeck-auth/src/cedar.rs` | File | `crates/flightdeck-auth/src/cedar.rs` | 2 | 1.00 | IMPORTS | `crates/flightdeck-auth/src/cedar.rs:10-13` |
| `apps/causeway-server-web/src/access/tests.rs` | File | `apps/causeway-server-web/src/access/tests.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server-web/src/access/tests.rs:8` |
| `apps/causeway-server/src/browser_auth/handlers.rs` | File | `apps/causeway-server/src/browser_auth/handlers.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/browser_auth/handlers.rs:9-12` |
| `apps/causeway-server/src/routes/api/tests.rs` | File | `apps/causeway-server/src/routes/api/tests.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/api/tests.rs:17` |
| `apps/causeway-server/src/routes/pages/test_shell.rs` | File | `apps/causeway-server/src/routes/pages/test_shell.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages/test_shell.rs:14` |
| `apps/causeway-server/src/ws/tests.rs` | File | `apps/causeway-server/src/ws/tests.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/ws/tests.rs:10` |
| `apps/causeway-server/src/ws/tests/explorer.rs` | File | `apps/causeway-server/src/ws/tests/explorer.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/ws/tests/explorer.rs:17` |
| `crates/causeway-policy-bundle/tests/repository_bundles.rs` | File | `crates/causeway-policy-bundle/tests/repository_bundles.rs` | 3 | 1.00 | IMPORTS | `crates/causeway-policy-bundle/tests/repository_bundles.rs:13` |
| `crates/platform-flags/tests/group_rules.rs` | File | `crates/platform-flags/tests/group_rules.rs` | 3 | 1.00 | IMPORTS | `crates/platform-flags/tests/group_rules.rs:16` |
| `services/flightdeck/tests/auth_test.rs` | File | `services/flightdeck/tests/auth_test.rs` | 3 | 1.00 | IMPORTS | `services/flightdeck/tests/auth_test.rs:21` |
| `services/flightdeck/tests/common/mod.rs` | File | `services/flightdeck/tests/common/mod.rs` | 3 | 1.00 | IMPORTS | `services/flightdeck/tests/common/mod.rs:25` |
| `services/flightdeck/tests/content_test.rs` | File | `services/flightdeck/tests/content_test.rs` | 3 | 1.00 | IMPORTS | `services/flightdeck/tests/content_test.rs:23` |
| `services/pure-flight/tests/e2e.rs` | File | `services/pure-flight/tests/e2e.rs` | 3 | 1.00 | IMPORTS | `services/pure-flight/tests/e2e.rs:22` |
| `tests` | Module | `apps/causeway-catalog-host/src/search.rs:963-1421` | 3 | 1.00 | IMPORTS | `apps/causeway-catalog-host/src/search.rs:972` |
| `tests` | Module | `apps/causeway-server/src/query_flight_auth.rs:464-978` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/query_flight_auth.rs:468` |
| `tests` | Module | `apps/causeway-server/src/ws/session/subscription.rs:186-242` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/ws/session/subscription.rs:187` |
| `tests` | Module | `component/catalog/core/src/approval.rs:588-669` | 3 | 1.00 | IMPORTS | `component/catalog/core/src/approval.rs:595` |
| `database_tests` | Module | `component/catalog/core/src/placements.rs:777-1158` | 3 | 1.00 | IMPORTS | `component/catalog/core/src/placements.rs:782` |
| `tests` | Module | `component/catalog/core/src/review.rs:1377-1622` | 3 | 1.00 | IMPORTS | `component/catalog/core/src/review.rs:1384` |
| `tests` | Module | `component/governance/contracts/src/unity_catalog.rs:559-736` | 3 | 1.00 | IMPORTS | `component/governance/contracts/src/unity_catalog.rs:572` |
| `database_tests` | Module | `component/governance/policy/src/repo.rs:3624-3814` | 3 | 1.00 | IMPORTS | `component/governance/policy/src/repo.rs:3626` |
| `tests` | Module | `crates/causeway-access-control/src/audit.rs:828-941` | 3 | 1.00 | IMPORTS | `crates/causeway-access-control/src/audit.rs:831` |
| `tests` | Module | `crates/causeway-access-control/src/lib.rs:1106-1349` | 3 | 1.00 | IMPORTS | `crates/causeway-access-control/src/lib.rs:1108` |
| `tests` | Module | `crates/causeway-cedar/src/lib.rs:548-846` | 3 | 1.00 | IMPORTS | `crates/causeway-cedar/src/lib.rs:554` |
| `database_tests` | Module | `crates/causeway-metadata-ops/src/rights_drafts.rs:292-424` | 3 | 1.00 | IMPORTS | `crates/causeway-metadata-ops/src/rights_drafts.rs:295` |
| `tests` | Module | `crates/causeway-principal/src/lib.rs:331-491` | 3 | 1.00 | IMPORTS | `crates/causeway-principal/src/lib.rs:333` |
| `tests` | Module | `crates/causeway-principal/src/recorded.rs:451-563` | 3 | 1.00 | IMPORTS | `crates/causeway-principal/src/recorded.rs:453` |
| `tests` | Module | `crates/connect-rpc-access/src/lib.rs:1343-2510` | 3 | 1.00 | IMPORTS | `crates/connect-rpc-access/src/lib.rs:1347` |

… and 33 more in the JSON.

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 62059 nodes, 263426 edges · 1656 ms</sub>
