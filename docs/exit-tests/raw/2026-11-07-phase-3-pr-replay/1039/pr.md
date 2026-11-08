<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `9c15a151987e`, dataset at `a3779e8e4d28`: **8 files, 17 changed symbols, 0 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (17)**

| Symbol | Kind | Where |
|---|---|---|
| `test_the_addon_suites_run_after_the_single_build` | Function | `.mise/rhai/causeway/node_addon_release_test.rhai:58-69` |
| `ffi:rpc-access:test` | Task | `.mise/toml/ffi.toml:101-112` |
| `a backend failure` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:126-147` |
| `a backend failure::is Unavailable to the caller, without the backend's reason` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:127-146` |
| `an authorized caller` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:90-124` |
| `an authorized caller::attributes a workload's search to the workload` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:106-111` |
| `an authorized caller::attributes an agent's search to the person it acts for` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:99-104` |
| `an authorized caller::gets no hits for a blank query, and nothing is recorded` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:113-117` |
| `an authorized caller::gets results, and the search is attributed to their subject` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:91-97` |
| `an authorized caller::is told a query over the length cap is invalid` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:119-123` |
| `an unauthorized caller` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:67-88` |
| `an unauthorized caller::is refused when an agent acts for a person without a delegation token` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:80-87` |
| `an unauthorized caller::is refused with an expired token` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:73-78` |
| `an unauthorized caller::is refused without a token, and nothing is searched or recorded` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:68-71` |
| `codeOf` | Function | `services-node/search-relay/tests/rpc-access.addon.test.ts:44-51` |
| `the class table beside the proto` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:53-65` |
| `the class table beside the proto::classifies every SearchService method` | Test | `services-node/search-relay/tests/rpc-access.addon.test.ts:54-64` |

Changed as whole files (no declaration at the changed lines): `services-node/search-relay/moon.yml`, `services-node/search-relay/package.json`, `services-node/search-relay/vitest.config.addon.ts`, `services-node/search-relay/vitest.config.ts`.
Deleted: `services-node/search-relay/tests/rpc-access.test.ts`.
Removed symbols: `codeOf` (Function, `services-node/search-relay/tests/rpc-access.test.ts:44-51` at base), `a backend failure` (Test, `services-node/search-relay/tests/rpc-access.test.ts:126-147` at base), `a backend failure::is Unavailable to the caller, without the backend's reason` (Test, `services-node/search-relay/tests/rpc-access.test.ts:127-146` at base), `an authorized caller` (Test, `services-node/search-relay/tests/rpc-access.test.ts:90-124` at base), `an authorized caller::attributes a workload's search to the workload` (Test, `services-node/search-relay/tests/rpc-access.test.ts:106-111` at base), `an authorized caller::attributes an agent's search to the person it acts for` (Test, `services-node/search-relay/tests/rpc-access.test.ts:99-104` at base), `an authorized caller::gets no hits for a blank query, and nothing is recorded` (Test, `services-node/search-relay/tests/rpc-access.test.ts:113-117` at base), `an authorized caller::gets results, and the search is attributed to their subject` (Test, `services-node/search-relay/tests/rpc-access.test.ts:91-97` at base), `an authorized caller::is told a query over the length cap is invalid` (Test, `services-node/search-relay/tests/rpc-access.test.ts:119-123` at base), `an unauthorized caller` (Test, `services-node/search-relay/tests/rpc-access.test.ts:67-88` at base), `an unauthorized caller::is refused when an agent acts for a person without a delegation token` (Test, `services-node/search-relay/tests/rpc-access.test.ts:80-87` at base), `an unauthorized caller::is refused with an expired token` (Test, `services-node/search-relay/tests/rpc-access.test.ts:73-78` at base), `an unauthorized caller::is refused without a token, and nothing is searched or recorded` (Test, `services-node/search-relay/tests/rpc-access.test.ts:68-71` at base), `the class table beside the proto` (Test, `services-node/search-relay/tests/rpc-access.test.ts:53-65` at base), `the class table beside the proto::classifies every SearchService method` (Test, `services-node/search-relay/tests/rpc-access.test.ts:54-64` at base).

**Affected (0 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

Nothing in the graph depends on the changed nodes.

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 64880 nodes, 274962 edges · 1632 ms</sub>
