# Context pack

Task: search-relay's rpc-access test loads the native @causeway/rpc-access addon from the plain unit-test task, so an affected run that reaches search-relay fails whenever the addon is not built. Move that suite into the addon run the CI ffi job executes after its single addon build, as causeway-relay-ts already does, and make the Rhai test that guards that order cover it.

## Seeds

- `pkg:npm/@causeway/rpc-access` Package 1.00 (identifier `@causeway/rpc-access`)
- `pkg:npm/search-relay` Package 1.00 (identifier `search-relay`)
- `task:mise/ffi:rpc-access:test` Task 0.90 (text)
- `rhai:.mise/rhai/causeway/node_addon_release_test.rhai::test_the_addon_suites_run_after_the_single_build` Function 0.84 (text)
- `task:mise/ffi:rpc-access:fetch-prebuilt` Task 0.66 (text)
- `task:mise/ffi:rpc-access:check` Task 0.66 (text)
- `task:mise/flight-native:test` Task 0.63 (text)
- `typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::loadAddon` Function 0.60 (text)
- `file:torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` File 0.58 (text)
- `typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts::RpcAccess.load without the native addon` Test 0.55 (text)
- `typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts::RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it` Test 0.55 (text)
- `task:mise/ffi:rpc-access:build` Task 0.54 (text)
- `rust:crates/rpc-access-core/src/lib.rs::Repr` Enum 0.40 (identifier `rpc-access`)
- `rust:crates/rpc-access-core/tests/conformance.rs::core` Function 0.40 (identifier `rpc-access`)
- `rust:crates/rpc-access-core/tests/conformance.rs::text` Function 0.40 (identifier `rpc-access`)
- `rust:ffi/node-rpc-access/build.rs::main` Function 0.40 (identifier `rpc-access`)
- `typescript:ffi/node-rpc-access/test/conformance.test.mts::Case` Interface 0.40 (identifier `rpc-access`)

Budget 30000 tokens, used 29924. Seeds 17, candidates 869, included 115, left out 754.

## Context

### 1. Package npm/search-relay
`pkg:npm/search-relay`  services-node/search-relay/package.json:2  pkg:npm/search-relay  score 4.68  
`npm/search-relay`  
members 15  
dependencies 16: `npm/@types/node` (services-node/search-relay/package.json:28), `npm/@causeway/data` (services-node/search-relay/package.json:17), `npm/typescript` (services-node/search-relay/package.json:32) … 13 more  

### 2. Package npm/@causeway/rpc-access
`pkg:npm/@causeway/rpc-access`  ffi/node-rpc-access/package.json:2  pkg:npm/@causeway/rpc-access  score 2.52  
`npm/@causeway/rpc-access`  
dependencies 4: `npm/@napi-rs/cli` (ffi/node-rpc-access/package.json:52), `npm/@types/node` (ffi/node-rpc-access/package.json:53), `npm/typescript` (ffi/node-rpc-access/package.json:54) … 1 more  
importers 1: `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:36)  

### 3. Function test_the_addon_suites_run_after_the_single_build
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::test_the_addon_suites_run_after_the_single_build`  .mise/rhai/causeway/node_addon_release_test.rhai:57-63  pkg:npm/causeway-monorepo  score 2.04  
`fn test_the_addon_suites_run_after_the_single_build()`  
The relay's addon suite makes the binary loadable with ensure-addon.sh, which builds when none loads. In one `moon run` beside the rpc-access build-addon, moon runs the two in parallel and two napi builds race on one target dir and one .node; the task runs them one after the other instead.  
calls 2: `captures_all` (.mise/rhai/causeway/node_addon_release_test.rhai:59), `task_body` (.mise/rhai/causeway/node_addon_release_test.rhai:58)  
runs 1: `mise/ffi:rpc-access:test` (.mise/rhai/causeway/node_addon_release_test.rhai:58)  
.mise/rhai/causeway/node_addon_release_test.rhai:57-63
```rhai
fn test_the_addon_suites_run_after_the_single_build() {
    let body = task_body("ffi:rpc-access:test");
    let runs = re::captures_all(`(?m)^\s*moon run (\S+)\s*$`, body);
    assert(runs.len() == 2, `ffi:rpc-access:test runs two moon commands, not ${runs.len()}`);
    assert(runs[0][1] == "causeway-rpc-access-node:test-addon", "the build and conformance suite first");
    assert(runs[1][1] == "causeway-relay-ts:test-addon", "the relay's addon suites after it");
}
```

### 4. Task ffi:rpc-access:test
`task:mise/ffi:rpc-access:test`  .mise/toml/ffi.toml:101-111  pkg:npm/causeway-monorepo  score 2.00  
`mise run ffi:rpc-access:test`  
Build the @causeway/rpc-access addon, run its decision conformance suite, then the relay's suites that load it. Needs cargo.  
description: Build the @causeway/rpc-access addon, run its decision conformance suite, then the relay's suites that load it. Needs cargo.  
run: set -e
moon run causeway-rpc-access-node:test-addon
moon run causeway-relay-ts:test-addon  
run_by 1: `test_the_addon_suites_run_after_the_single_build` (.mise/rhai/causeway/node_addon_release_test.rhai:58)  
.mise/toml/ffi.toml:101-111
```toml
["ffi:rpc-access:test"]
description = "Build the @causeway/rpc-access addon, run its decision conformance suite, then the relay's suites that load it. Needs cargo."
# Two moon runs, not one: in one run moon starts the relay suite beside
# build-addon, and the relay's ensure-addon.sh, finding no binary yet,
# would start a second napi build on the same target dir and .node.
run = '''
set -e
moon run causeway-rpc-access-node:test-addon
moon run causeway-relay-ts:test-addon
'''

```

### 5. Task ffi:rpc-access:fetch-prebuilt
`task:mise/ffi:rpc-access:fetch-prebuilt`  .mise/toml/ffi.toml:97-100  pkg:npm/causeway-monorepo  score 1.18  
`mise run ffi:rpc-access:fetch-prebuilt`  
Fetch the published prebuilt @causeway/rpc-access binary for this host from npm and put it beside the loader, so the relay's addon suites run without cargo.  
description: Fetch the published prebuilt @causeway/rpc-access binary for this host from npm and put it beside the loader, so the relay's addon suites run without cargo.  
run: sh tools/node-addon/fetch-prebuilt.sh ffi/node-rpc-access  
runs 1: `tools/node-addon/fetch-prebuilt.sh` (.mise/toml/ffi.toml:99)  
.mise/toml/ffi.toml:97-100
```toml
["ffi:rpc-access:fetch-prebuilt"]
description = "Fetch the published prebuilt @causeway/rpc-access binary for this host from npm and put it beside the loader, so the relay's addon suites run without cargo."
run = "sh tools/node-addon/fetch-prebuilt.sh ffi/node-rpc-access"

```

### 6. Task ffi:rpc-access:check
`task:mise/ffi:rpc-access:check`  .mise/toml/ffi.toml:134-142  pkg:npm/causeway-monorepo  score 1.17  
`mise run ffi:rpc-access:check`  
The @causeway/rpc-access release preflight: build, conformance and relay addon suites, generated bindings current, pack smoke. Needs cargo.  
description: The @causeway/rpc-access release preflight: build, conformance and relay addon suites, generated bindings current, pack smoke. Needs cargo.  
run: set -e
mise run ffi:rpc-access:test
mise run ffi:rpc-access:bindings-current
mise run ffi:rpc-access:smoke-pack  
.mise/toml/ffi.toml:134-142
```toml
["ffi:rpc-access:check"]
description = "The @causeway/rpc-access release preflight: build, conformance and relay addon suites, generated bindings current, pack smoke. Needs cargo."
run = '''
set -e
mise run ffi:rpc-access:test
mise run ffi:rpc-access:bindings-current
mise run ffi:rpc-access:smoke-pack
'''

```

### 7. Task flight-native:test
`task:mise/flight-native:test`  .mise/toml/ffi.toml:280-286  pkg:npm/causeway-monorepo  score 1.11  
`mise run flight-native:test`  
Build the @torrentfx/flight-native addon and runtime-flight's recording Flight server, then run the addon suites against it. Needs cargo.  
description: Build the @torrentfx/flight-native addon and runtime-flight's recording Flight server, then run the addon suites against it. Needs cargo.  
run: set -e
moon run flight-native:test-addon  
.mise/toml/ffi.toml:280-286
```toml
["flight-native:test"]
description = "Build the @torrentfx/flight-native addon and runtime-flight's recording Flight server, then run the addon suites against it. Needs cargo."
run = '''
set -e
moon run flight-native:test-addon
'''

```

### 8. Package npm/@causeway/search-core
`pkg:npm/@causeway/search-core`  packages/search-core/package.json:2  pkg:npm/@causeway/search-core  score 1.09  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:20, 1.00)  
`npm/@causeway/search-core`  
members 38  
dependents 2: `npm/search-indexer` (jobs-node/search-indexer/package.json:14), `npm/search-relay` (services-node/search-relay/package.json:20)  
dependencies 13: `npm/@effect/sql` (packages/search-core/package.json:25), `npm/pg` (packages/search-core/package.json:34), `npm/testcontainers` (packages/search-core/package.json:35) … 10 more  
designs 1: `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:304)  
importers 9: `jobs-node/search-indexer/src/main.ts` (jobs-node/search-indexer/src/main.ts:11), `services-node/search-relay/src/runtime.ts` (services-node/search-relay/src/runtime.ts:5), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:9) … 6 more  

### 9. Function loadAddon
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::loadAddon`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:255-263  pkg:npm/@causeway/relay  score 1.09  
`async function loadAddon(): Promise<AddonModule>`  
The addon, imported on first use. Rejects with `ConfigError` when this host has no binary for it: neither the installed prebuilt `@causeway/rpc-access-<platform>` package nor a local build.  
callers 1: `RpcAccess::load` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:133)  
uses_types 1: `ConfigError` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:259)  
torrentfx/packages-node/causeway-relay/src/rpc-access.ts:255-263
```typescript
async function loadAddon(): Promise<AddonModule> {
  try {
    return await import("@causeway/rpc-access");
  } catch (err) {
    throw new ConfigError(
      `the @causeway/rpc-access native addon did not load on ${process.platform}-${process.arch}; install its prebuilt platform package or build it (docs/NODE-ADDONS.md): ${err instanceof Error ? err.message : String(err)}`,
    );
  }
}
```

### 10. Package npm/@causeway/platform
`pkg:npm/@causeway/platform`  torrentfx/packages-node/causeway-platform/package.json:2  pkg:npm/@causeway/platform  score 1.06  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:18, 1.00)  
`npm/@causeway/platform`  
members 33  
dependents 10: `npm/@causeway/data` (torrentfx/packages-node/causeway-data/package.json:15), `npm/@causeway/relay` (torrentfx/packages-node/causeway-relay/package.json:16), `npm/roadmap-relay` (services-node/roadmap-relay/package.json:13) … 7 more  
dependencies 16: `npm/pino` (torrentfx/packages-node/causeway-platform/package.json:24), `npm/@bufbuild/protobuf` (torrentfx/packages-node/causeway-platform/package.json:14), `npm/@opentelemetry/api` (torrentfx/packages-node/causeway-platform/package.json:15) … 13 more  
designs 1: `docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md` (docs/rfd/RFD-s03ee8ga-service-runtime-and-configuration.md:102)  
documents 1: `docs/plans/ASHLAR-REMEDIATION-WORKPLAN.md` (docs/plans/ASHLAR-REMEDIATION-WORKPLAN.md:27)  
importers 46: `services-node/search-relay/tests/config.test.ts` (services-node/search-relay/tests/config.test.ts:9), `apps-web/causeway-web/test/setup-env.ts` (apps-web/causeway-web/test/setup-env.ts:5), `torrentfx/packages-node/causeway-data/test/config.test.ts` (torrentfx/packages-node/causeway-data/test/config.test.ts:5) … 43 more  

### 11. File torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts
`file:torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts`  torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts  pkg:npm/@causeway/relay  score 1.05  
members 2  
imports 8: `bindRecording` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30), `torrentfx/packages-node/causeway-relay/src/index.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:22), `npm/vitest` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:20) … 5 more  

### 12. Package npm/vitest
`pkg:npm/vitest`  pkg:npm/vitest  score 1.04  via `npm/@causeway/rpc-access` -> DEPENDS_ON (ffi/node-rpc-access/package.json:55, 1.00)  
dependents 30: `npm/@causeway/data` (torrentfx/packages-node/causeway-data/package.json:28), `npm/@causeway/ui-vendor` (web/ui-vendor/package.json:20), `npm/@torrentfx/flight-native` (torrentfx/packages-node/flight-native/package.json:33) … 27 more  
importers 133: `jobs-node/search-indexer/tests/setup-env.ts` (jobs-node/search-indexer/tests/setup-env.ts:4), `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:5), `services-node/relay-grpc/tests/config.test.ts` (services-node/relay-grpc/tests/config.test.ts:6) … 130 more  

### 13. File .mise/rhai/causeway/node_addon_release_test.rhai
`file:.mise/rhai/causeway/node_addon_release_test.rhai`  .mise/rhai/causeway/node_addon_release_test.rhai  pkg:npm/causeway-monorepo  score 1.02  via `test_the_addon_suites_run_after_the_single_build` <- CONTAINS (.mise/rhai/causeway/node_addon_release_test.rhai:57-63, 1.00)  
members 5  

### 14. File .mise/toml/ffi.toml
`file:.mise/toml/ffi.toml`  .mise/toml/ffi.toml  pkg:npm/causeway-monorepo  score 1.00  via `mise/ffi:rpc-access:test` <- CONTAINS (.mise/toml/ffi.toml:101-111, 1.00)  
members 30  

### 15. Test RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it
`typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts::RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it`  torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:189-207  pkg:npm/@causeway/relay  score 1.00  
`it("refuses to load with a ConfigError that names the addon and how to get it", async () => { vi.doMock("@causeway/rpc-access", () => { throw new Error("Cannot find native binding"); }); const bundleDir = join(root, "policy/cedar/causeway-rpc"); const load = RpcAccess.load({ bundleDir, classTable: j…`  
calls 2: `parseTokenVerifier` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:197), `RpcAccess::load` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:194)  
tests_for 2: `parseTokenVerifier` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:197), `RpcAccess::load` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:194)  
torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:189-207
```typescript
  it("refuses to load with a ConfigError that names the addon and how to get it", async () => {
    vi.doMock("@causeway/rpc-access", () => {
      throw new Error("Cannot find native binding");
    });
    const bundleDir = join(root, "policy/cedar/causeway-rpc");
    const load = RpcAccess.load({
      bundleDir,
      classTable: join(bundleDir, "manifest.json"),
      verifier: parseTokenVerifier(
        "https://issuer.example.test",
        "causeway",
        "https://issuer.example.test/keys",
        "test",
      ),
      environment: "test",
    });
    await expect(load).rejects.toBeInstanceOf(ConfigError);
    await expect(load).rejects.toThrow(/@causeway\/rpc-access native addon did not load/);
  });
```

### 16. Test RpcAccess.load without the native addon
`typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts::RpcAccess.load without the native addon`  torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:184-208  pkg:npm/@causeway/relay  score 0.97  
`describe("RpcAccess.load without the native addon", () => { afterEach(() => { vi.doUnmock("@causeway/rpc-access"); }); it("refuses to load with a ConfigError that names the addon and how to get it", async () => { vi.doMock("@causeway/rpc-access", () => { throw new Error("Cannot find native binding")…`  
members 1  
torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:184-208
```typescript
describe("RpcAccess.load without the native addon", () => {
  afterEach(() => {
    vi.doUnmock("@causeway/rpc-access");
  });

  it("refuses to load with a ConfigError that names the addon and how to get it", async () => {
    vi.doMock("@causeway/rpc-access", () => {
      throw new Error("Cannot find native binding");
    });
    const bundleDir = join(root, "policy/cedar/causeway-rpc");
    const load = RpcAccess.load({
      bundleDir,
      classTable: join(bundleDir, "manifest.json"),
      verifier: parseTokenVerifier(
        "https://issuer.example.test",
        "causeway",
        "https://issuer.example.test/keys",
        "test",
      ),
      environment: "test",
    });
    await expect(load).rejects.toBeInstanceOf(ConfigError);
    await expect(load).rejects.toThrow(/@causeway\/rpc-access native addon did not load/);
  });
});
```

### 17. Package npm/@causeway/relay
`pkg:npm/@causeway/relay`  torrentfx/packages-node/causeway-relay/package.json:2  pkg:npm/@causeway/relay  score 0.97  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:19, 1.00)  
`npm/@causeway/relay`  
members 20  
dependents 4: `npm/ts-worker` (jobs-node/relay-worker/package.json:13), `npm/roadmap-relay` (services-node/roadmap-relay/package.json:14), `npm/search-relay` (services-node/search-relay/package.json:19) … 1 more  
dependencies 10: `npm/@opentelemetry/api` (torrentfx/packages-node/causeway-relay/package.json:19), `npm/typescript` (torrentfx/packages-node/causeway-relay/package.json:28), `npm/@types/node` (torrentfx/packages-node/causeway-relay/package.json:27) … 7 more  
designs 3: `docs/adrs/ADR-v68na9qx-ts-only-slices-islands-relay-proxy.md` (docs/adrs/ADR-v68na9qx-ts-only-slices-islands-relay-proxy.md:26), `docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md` (docs/rfd/RFD-j1jabh49-environment-scope-and-secret-configuration-contract.md:640), `docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md` (docs/rfd/RFD-krvga4f3-one-directory-of-principals-for-non-production-environments.md:162)  
importers 10: `services-node/search-relay/tests/harness.ts` (services-node/search-relay/tests/harness.ts:14), `services-node/relay-grpc/src/config.ts` (services-node/relay-grpc/src/config.ts:15), `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:7) … 7 more  

### 18. Package npm/@causeway/data
`pkg:npm/@causeway/data`  torrentfx/packages-node/causeway-data/package.json:2  pkg:npm/@causeway/data  score 0.96  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:17, 1.00)  
`npm/@causeway/data`  
members 19  
dependents 2: `npm/search-relay` (services-node/search-relay/package.json:17), `npm/@causeway/search-core` (packages/search-core/package.json:23)  
dependencies 12: `npm/vitest` (torrentfx/packages-node/causeway-data/package.json:28), `npm/@causeway/platform` (torrentfx/packages-node/causeway-data/package.json:15), `npm/@types/node` (torrentfx/packages-node/causeway-data/package.json:24) … 9 more  
designs 1: `docs/adrs/ADR-h84558gs-relay-is-the-typescript-service-road.md` (docs/adrs/ADR-h84558gs-relay-is-the-typescript-service-road.md:376)  
importers 7: `services-node/search-relay/src/service.ts` (services-node/search-relay/src/service.ts:11), `services-node/search-relay/src/runtime.ts` (services-node/search-relay/src/runtime.ts:4), `services-node/search-relay/src/migrate.ts` (services-node/search-relay/src/migrate.ts:7) … 4 more  

### 19. Task ffi:rpc-access:build
`task:mise/ffi:rpc-access:build`  .mise/toml/ffi.toml:90-96  pkg:npm/causeway-monorepo  score 0.91  
`mise run ffi:rpc-access:build`  
Build the @causeway/rpc-access addon with napi build (debug): ffi/node-rpc-access/rpc-access.<platform>.node and the generated index.js and index.d.ts. Needs cargo.  
description: Build the @causeway/rpc-access addon with napi build (debug): ffi/node-rpc-access/rpc-access.<platform>.node and the generated index.js and index.d.ts. Needs cargo.  
run: set -e
npm run build:debug --prefix ffi/node-rpc-access  
.mise/toml/ffi.toml:90-96
```toml
["ffi:rpc-access:build"]
description = "Build the @causeway/rpc-access addon with napi build (debug): ffi/node-rpc-access/rpc-access.<platform>.node and the generated index.js and index.d.ts. Needs cargo."
run = '''
set -e
npm run build:debug --prefix ffi/node-rpc-access
'''

```

### 20. Function core
`rust:crates/rpc-access-core/tests/conformance.rs::core`  crates/rpc-access-core/tests/conformance.rs:43-55  pkg:cargo/rpc-access-core  score 0.90  
`fn core(fixture: &Value) -> RpcDecisionCore<DeclaredClassTable>`  
callers 1: `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:122)  
calls 5: `text` (crates/rpc-access-core/tests/conformance.rs:44), `rpc_bundle_from_sources` (crates/rpc-access-core/tests/conformance.rs:46), `repository_root` (crates/rpc-access-core/tests/conformance.rs:44) … 2 more  
tests 1: `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:122)  
uses_types 2: `DeclaredClassTable` (crates/rpc-access-core/tests/conformance.rs:43), `RpcDecisionCore` (crates/rpc-access-core/tests/conformance.rs:43)  
crates/rpc-access-core/tests/conformance.rs:43-55
```rust
fn core(fixture: &Value) -> RpcDecisionCore<DeclaredClassTable> {
    let bundle = repository_root().join(text(fixture, "bundle"));
    let read = |file: &str| std::fs::read_to_string(bundle.join(file)).expect("bundle file");
    let snapshot = rpc_bundle_from_sources(
        &read("manifest.json"),
        &read("access.cedar"),
        &read("access.cedarschema"),
    )
    .expect("the causeway.rpc bundle validates");
    let table = DeclaredClassTable::parse(&fixture["class_table"].to_string())
        .expect("the fixture's class table parses");
    RpcDecisionCore::from_snapshot(table, snapshot)
}
```

### 21. Function text
`rust:crates/rpc-access-core/tests/conformance.rs::text`  crates/rpc-access-core/tests/conformance.rs:37-41  pkg:cargo/rpc-access-core  score 0.90  
`fn text<'a>(value: &'a Value, field: &str) -> &'a str`  
callers 9: `authentication` (crates/rpc-access-core/tests/conformance.rs:63), `authentication` (crates/rpc-access-core/tests/conformance.rs:61), `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:127) … 6 more  
tests 2: `every_refused_class_table_is_refused_at_load` (crates/rpc-access-core/tests/conformance.rs:151), `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:127)  
crates/rpc-access-core/tests/conformance.rs:37-41
```rust
fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("fixture field {field} is a string"))
}
```

### 22. File services-node/search-relay/src/service.ts
`file:services-node/search-relay/src/service.ts`  services-node/search-relay/src/service.ts  pkg:npm/search-relay  score 0.88  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 9  
importers 4: `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:31), `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:13), `services-node/search-relay/src/runtime.ts` (services-node/search-relay/src/runtime.ts:9) … 1 more  
imports 9: `npm/@bufbuild/protobuf` (services-node/search-relay/src/service.ts:10), `npm/@causeway/platform` (services-node/search-relay/src/service.ts:12), `npm/effect` (services-node/search-relay/src/service.ts:33) … 6 more  

### 23. File services-node/search-relay/tests/harness.ts
`file:services-node/search-relay/tests/harness.ts`  services-node/search-relay/tests/harness.ts  pkg:npm/search-relay  score 0.86  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 8  
importers 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40)  
imports 11: `services-node/search-relay/src/config.ts` (services-node/search-relay/tests/harness.ts:27), `npm/effect` (services-node/search-relay/tests/harness.ts:26), `npm/@causeway/relay` (services-node/search-relay/tests/harness.ts:14) … 8 more  
uses_types 1: `Fixture` (services-node/search-relay/tests/harness.ts:36)  

### 24. Repository causeway
`repo:causeway`  score 0.86  via `npm/@causeway/rpc-access` <- CONTAINS (ffi/node-rpc-access/package.json:2, 1.00)  
members 277  

### 25. File services-node/search-relay/src/config.ts
`file:services-node/search-relay/src/config.ts`  services-node/search-relay/src/config.ts  pkg:npm/search-relay  score 0.86  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 4  
calls 4: `optional` (services-node/search-relay/src/config.ts:44), `optional` (services-node/search-relay/src/config.ts:45), `port` (services-node/search-relay/src/config.ts:33) … 1 more  
importers 7: `services-node/search-relay/tests/config.test.ts` (services-node/search-relay/tests/config.test.ts:11-16), `services-node/search-relay/tests/harness.ts` (services-node/search-relay/tests/harness.ts:27), `services-node/search-relay/src/migrate.ts` (services-node/search-relay/src/migrate.ts:10) … 4 more  
imports 3: `npm/@causeway/search-core` (services-node/search-relay/src/config.ts:29), `npm/@causeway/platform` (services-node/search-relay/src/config.ts:13-27), `npm/@causeway/relay` (services-node/search-relay/src/config.ts:28)  

### 26. Package npm/typescript
`pkg:npm/typescript`  pkg:npm/typescript  score 0.81  via `npm/@causeway/rpc-access` -> DEPENDS_ON (ffi/node-rpc-access/package.json:54, 1.00)  
dependents 48: `npm/vscode-causeway-catalog` (extensions/vscode-causeway-catalog/package.json:83), `npm/@causeway/roadmap-ui` (bindings-ts/causeway-roadmap-ui/package.json:17), `npm/@causeway/framework` (torrentfx/web/framework/app/package.json:25) … 45 more  
importers 1: `apps-web/showcase/tests/stories.ts` (apps-web/showcase/tests/stories.ts:3)  

### 27. Enum Repr
`rust:crates/rpc-access-core/src/lib.rs::Repr`  crates/rpc-access-core/src/lib.rs:95-98  pkg:cargo/rpc-access-core  score 0.80  
`enum Repr`  
members 2  
used_by 2: `ContentMeta::deserialize::Repr` (crates/catalog-domain/src/content.rs:818), `Classification` (crates/rpc-access-core/src/lib.rs:92)  
uses_types 1: `ContentMeta::deserialize::Repr` (crates/rpc-access-core/src/lib.rs:95)  
crates/rpc-access-core/src/lib.rs:95-98
```rust
enum Repr {
    Static(&'static str),
    Shared(Arc<str>),
}
```

### 28. Function main
`rust:ffi/node-rpc-access/build.rs::main`  ffi/node-rpc-access/build.rs:1-3  pkg:cargo/node-rpc-access  score 0.80  
`fn main()`  
ffi/node-rpc-access/build.rs:1-3
```rust
fn main() {
    napi_build::setup();
}
```

### 29. Interface Case
`typescript:ffi/node-rpc-access/test/conformance.test.mts::Case`  ffi/node-rpc-access/test/conformance.test.mts:16-21  pkg:cargo/node-rpc-access  score 0.80  
`interface Case`  
ffi/node-rpc-access/test/conformance.test.mts:16-21
```typescript
interface Case {
  readonly name: string;
  readonly method: string;
  readonly caller: string | null;
  readonly expected: Record<string, string>;
}
```

### 30. File services-node/search-relay/tests/rpc-access.test.ts
`file:services-node/search-relay/tests/rpc-access.test.ts`  services-node/search-relay/tests/rpc-access.test.ts  pkg:npm/search-relay  score 0.78  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 5  
calls 6: `ServedRelay::stop` (services-node/search-relay/tests/rpc-access.test.ts:37), `doublesRuntime` (services-node/search-relay/tests/rpc-access.test.ts:31), `storeDouble` (services-node/search-relay/tests/rpc-access.test.ts:31) … 3 more  
imports 14: `npm/@connectrpc/connect` (services-node/search-relay/tests/rpc-access.test.ts:11), `services-node/search-relay/tests/harness.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24), `as` (services-node/search-relay/tests/rpc-access.test.ts:15-24) … 11 more  
uses_types 1: `ServedRelay` (services-node/search-relay/tests/rpc-access.test.ts:27)  

### 31. File torrentfx/packages-node/causeway-relay/src/rpc-access.ts
`file:torrentfx/packages-node/causeway-relay/src/rpc-access.ts`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts  pkg:npm/@causeway/relay  score 0.77  via `loadAddon` <- CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:255-263, 1.00)  
members 14  
importers 1: `torrentfx/packages-node/causeway-relay/src/app.ts` (torrentfx/packages-node/causeway-relay/src/app.ts:42)  
imports 7: `KeysUnavailable` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:45), `npm/@causeway/rpc-access` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:36), `KeyCache` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:45) … 4 more  

### 32. Package npm/@types/node
`pkg:npm/@types/node`  pkg:npm/@types/node  score 0.77  via `npm/@causeway/rpc-access` -> DEPENDS_ON (ffi/node-rpc-access/package.json:53, 1.00)  
dependents 20: `npm/search-relay` (services-node/search-relay/package.json:28), `npm/vscode-causeway-catalog` (extensions/vscode-causeway-catalog/package.json:78), `npm/@causeway/content-core` (packages/content-core/package.json:35) … 17 more  

### 33. Package npm/effect
`pkg:npm/effect`  pkg:npm/effect  score 0.76  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:24, 1.00)  
dependents 12: `npm/@causeway/content-core` (packages/content-core/package.json:18), `npm/@causeway/web` (apps-web/causeway-web/package.json:22), `npm/@torrentfx/runtime-web` (torrentfx/web/framework/runtime-web/package.json:23) … 9 more  
importers 102: `apps-web/causeway-web/test/harness.ts` (apps-web/causeway-web/test/harness.ts:12), `torrentfx/conformance/session/src/fixture.ts` (torrentfx/conformance/session/src/fixture.ts:13), `torrentfx/web/framework/runtime-web/scripts/bench-codec.ts` (torrentfx/web/framework/runtime-web/scripts/bench-codec.ts:15) … 99 more  

### 34. Package npm/@connectrpc/connect
`pkg:npm/@connectrpc/connect`  pkg:npm/@connectrpc/connect  score 0.71  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:22, 1.00)  
dependents 20: `npm/@causeway/ui-request` (web/ui-request/package.json:20), `npm/@causeway/taxonomy-ui` (bindings-ts/causeway-taxonomy-ui/package.json:14), `npm/@causeway/session` (bindings-ts/causeway-session/package.json:14) … 17 more  
importers 30: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:11), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:23), `torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:32) … 27 more  

### 35. File services-node/search-relay/src/server.ts
`file:services-node/search-relay/src/server.ts`  services-node/search-relay/src/server.ts  pkg:npm/search-relay  score 0.71  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
calls 10: `searchImpl` (services-node/search-relay/src/server.ts:37), `searchRelayConfigFromEnv` (services-node/search-relay/src/server.ts:15), `getLogger` (services-node/search-relay/src/server.ts:16) … 7 more  
imports 11: `searchImpl` (services-node/search-relay/src/server.ts:13), `services-node/search-relay/src/service.ts` (services-node/search-relay/src/server.ts:13), `npm/@causeway/search-proto` (services-node/search-relay/src/server.ts:8) … 8 more  
uses_types 2: `RelayApp` (services-node/search-relay/src/server.ts:32), `SearchRelayConfig` (services-node/search-relay/src/server.ts:15)  

### 36. File services-node/search-relay/tests/integration/attribution.test.ts
`file:services-node/search-relay/tests/integration/attribution.test.ts`  services-node/search-relay/tests/integration/attribution.test.ts  pkg:npm/search-relay  score 0.70  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 3  
calls 14: `sliceDatabaseLayer` (services-node/search-relay/tests/integration/attribution.test.ts:80), `loadRpcAccess` (services-node/search-relay/tests/integration/attribution.test.ts:90), `ServedRelay::stop` (services-node/search-relay/tests/integration/attribution.test.ts:97) … 11 more  
imports 18: `npm/@connectrpc/connect` (services-node/search-relay/tests/integration/attribution.test.ts:23), `withClient` (services-node/search-relay/tests/integration/attribution.test.ts:26-30), `npm/effect` (services-node/search-relay/tests/integration/attribution.test.ts:24) … 15 more  
uses_types 4: `Secret` (services-node/search-relay/tests/integration/attribution.test.ts:63), `SearchRuntime` (services-node/search-relay/tests/integration/attribution.test.ts:47), `EphemeralPostgres` (services-node/search-relay/tests/integration/attribution.test.ts:46) … 1 more  

### 37. File torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts
`file:torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts`  torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts  pkg:npm/@causeway/relay  score 0.68  via `RpcAccess.load without the native addon` <- CONTAINS (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:184-208, 1.00)  
members 2  
imports 7: `npm/vitest` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:14), `torrentfx/packages-node/causeway-relay/test/example-service.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:25), `torrentfx/packages-node/causeway-relay/src/index.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:16-24) … 4 more  
uses_types 1: `RpcGrant` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:27)  

### 38. Function task_body
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::task_body`  .mise/rhai/causeway/node_addon_release_test.rhai:32-39  pkg:npm/causeway-monorepo  score 0.68  via `test_the_addon_suites_run_after_the_single_build` -> CALLS (.mise/rhai/causeway/node_addon_release_test.rhai:58, 1.00)  
`fn task_body(name)`  
The run body of one task in ffi.toml, from its header to the next one.  
callers 1: `test_the_addon_suites_run_after_the_single_build` (.mise/rhai/causeway/node_addon_release_test.rhai:58)  
calls 1: `read` (.mise/rhai/causeway/node_addon_release_test.rhai:33)  
.mise/rhai/causeway/node_addon_release_test.rhai:32-39
```rhai
fn task_body(name) {
    let source = read(global::FFI_TASKS);
    let start = source.index_of(`["${name}"]`);
    assert(start >= 0, `${global::FFI_TASKS} defines ${name}`);
    let rest = source.sub_string(start + 1);
    let end = rest.index_of("\n[\"");
    if end < 0 { rest } else { rest.sub_string(0, end) }
}
```

### 39. File services-node/search-relay/src/runtime.ts
`file:services-node/search-relay/src/runtime.ts`  services-node/search-relay/src/runtime.ts  pkg:npm/search-relay  score 0.67  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 2  
importers 1: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:12)  
imports 7: `services-node/search-relay/src/service.ts` (services-node/search-relay/src/runtime.ts:9), `SearchRelayConfig` (services-node/search-relay/src/runtime.ts:8), `SearchRuntime` (services-node/search-relay/src/runtime.ts:9) … 4 more  

### 40. Method RpcAccess::load
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::RpcAccess::load`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:131-152  pkg:npm/@causeway/relay  score 0.66  via `loadAddon` <- CALLS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:133, 1.00)  
`static async load(options: RpcAccessOptions): Promise<RpcAccess>`  
Loads the authority: reads the bundle and the class table, and parses them in the addon with the verifier. Rejects with `ConfigError` when any of them is refused; a service that cannot load one must not serve.  
callers 4: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:60), `RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:194), `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:35) … 1 more  
calls 2: `loadAddon` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:133), `loadAuthority` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:141)  
tests 2: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:60), `RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:194)  
uses_types 5: `ConfigError` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:147), `RpcAccess` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:149), `AddonDecider` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:150) … 2 more  
torrentfx/packages-node/causeway-relay/src/rpc-access.ts:131-152
```typescript
  static async load(options: RpcAccessOptions): Promise<RpcAccess> {
    const [addon, manifest, policies, schema, classTable] = await Promise.all([
      loadAddon(),
      readFile(join(options.bundleDir, "manifest.json"), "utf-8"),
      readFile(join(options.bundleDir, "access.cedar"), "utf-8"),
      readFile(join(options.bundleDir, "access.cedarschema"), "utf-8"),
      readFile(options.classTable, "utf-8"),
    ]);
    let authority: Addon.RpcAccessAuthority;
    try {
      authority = addon.loadAuthority({ manifest, policies, schema }, classTable, {
        issuer: options.verifier.issuer,
        audience: options.verifier.audience,
        environment: options.environment,
      });
    } catch (err) {
      throw new ConfigError(`rpc access for ${options.classTable}: ${(err as Error).message}`);
    }
    return new RpcAccess(
      new AddonDecider(addon, authority, new KeyCache(options.verifier, options.keys)),
    );
  }
```

### 41. Package npm/@causeway/search-proto
`pkg:npm/@causeway/search-proto`  bindings-ts/causeway-search-proto/package.json:2  pkg:npm/@causeway/search-proto  score 0.60  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:21, 1.00)  
`npm/@causeway/search-proto`  
members 4  
dependents 1: `npm/search-relay` (services-node/search-relay/package.json:21)  
dependencies 2: `npm/typescript` (bindings-ts/causeway-search-proto/package.json:17), `npm/@bufbuild/protobuf` (bindings-ts/causeway-search-proto/package.json:14)  
importers 4: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:8), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:10), `services-node/search-relay/src/service.ts` (services-node/search-relay/src/service.ts:27-31) … 1 more  

### 42. File services-node/search-relay/src/migrate.ts
`file:services-node/search-relay/src/migrate.ts`  services-node/search-relay/src/migrate.ts  pkg:npm/search-relay  score 0.57  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
calls 1: `migrateSlice` (services-node/search-relay/src/migrate.ts:14)  
imports 4: `services-node/search-relay/src/config.ts` (services-node/search-relay/src/migrate.ts:10), `npm/@causeway/data` (services-node/search-relay/src/migrate.ts:7), `npm/@causeway/search-core` (services-node/search-relay/src/migrate.ts:8) … 1 more  

### 43. Package npm/@bufbuild/protobuf
`pkg:npm/@bufbuild/protobuf`  pkg:npm/@bufbuild/protobuf  score 0.55  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:16, 1.00)  
dependents 22: `npm/@causeway/catalog` (bindings-ts/causeway-catalog/package.json:14), `npm/@causeway/ui-catalog-status` (web/ui-catalog-status/package.json:18), `npm/causeway-monorepo` (package.json:8) … 19 more  
importers 109: `services-node/relay-grpc/src/health.ts` (services-node/relay-grpc/src/health.ts:3), `services-node/relay-grpc/gen/causeway/catalog/v2/details_pb.ts` (services-node/relay-grpc/gen/causeway/catalog/v2/details_pb.ts:7), `web/ui-semantic-graph/src/codec.test.ts` (web/ui-semantic-graph/src/codec.test.ts:1) … 106 more  

### 44. Package npm/roadmap-relay
`pkg:npm/roadmap-relay`  services-node/roadmap-relay/package.json:2  pkg:npm/roadmap-relay  score 0.52  via `npm/@causeway/platform` <- DEPENDS_ON (services-node/roadmap-relay/package.json:13, 1.00)  
`npm/roadmap-relay`  
members 7  
dependencies 10: `npm/@bufbuild/protobuf` (services-node/roadmap-relay/package.json:12), `npm/@connectrpc/connect-node` (services-node/roadmap-relay/package.json:17), `npm/tsx` (services-node/roadmap-relay/package.json:22) … 7 more  

### 45. Function loadRpcAccess
`typescript:services-node/search-relay/tests/harness.ts::loadRpcAccess`  services-node/search-relay/tests/harness.ts:48-66  pkg:npm/search-relay  score 0.52  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:48-66, 1.00)  
`function loadRpcAccess(): Promise<RpcAccess>`  
The authority a deployed relay loads, with the fixture issuer's keys.  
callers 3: `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:90), `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:32)  
calls 2: `parseTokenVerifier` (services-node/search-relay/tests/harness.ts:52), `RpcAccess::load` (services-node/search-relay/tests/harness.ts:49)  
importers 2: `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24)  
tests 1: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134)  
services-node/search-relay/tests/harness.ts:48-66
```typescript
export function loadRpcAccess(): Promise<RpcAccess> {
  return RpcAccess.load({
    bundleDir: DEFAULT_RPC_BUNDLE_DIR,
    classTable: DEFAULT_RPC_CLASS_TABLE,
    verifier: parseTokenVerifier(
      FIXTURE.verifier.issuer,
      FIXTURE.verifier.audience,
      JWKS_URI,
      FIXTURE.verifier.environment,
    ),
    environment: FIXTURE.verifier.environment,
    keys: {
      transport: async (url) => {
        if (url !== JWKS_URI) throw new Error(`${url} is unreachable`);
        return FIXTURE.jwks;
      },
    },
  });
}
```

### 46. File services-node/search-relay/tests/config.test.ts
`file:services-node/search-relay/tests/config.test.ts`  services-node/search-relay/tests/config.test.ts  pkg:npm/search-relay  score 0.52  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
members 3  
imports 4: `npm/@causeway/platform` (services-node/search-relay/tests/config.test.ts:9), `services-node/search-relay/src/config.ts` (services-node/search-relay/tests/config.test.ts:11-16), `npm/vitest` (services-node/search-relay/tests/config.test.ts:10) … 1 more  

### 47. Package npm/@napi-rs/cli
`pkg:npm/@napi-rs/cli`  pkg:npm/@napi-rs/cli  score 0.50  via `npm/@causeway/rpc-access` -> DEPENDS_ON (ffi/node-rpc-access/package.json:52, 1.00)  
dependents 3: `npm/@causeway/rpc-access` (ffi/node-rpc-access/package.json:52), `npm/@causeway/drive` (ffi/node-drive/package.json:46), `npm/@torrentfx/flight-native` (torrentfx/packages-node/flight-native/package.json:28)  

### 48. Package npm/ts-grpc
`pkg:npm/ts-grpc`  services-node/relay-grpc/package.json:2  pkg:npm/ts-grpc  score 0.48  via `npm/@causeway/platform` <- DEPENDS_ON (services-node/relay-grpc/package.json:13, 1.00)  
`npm/ts-grpc`  
members 68  
dependencies 12: `npm/@grpc/grpc-js` (services-node/relay-grpc/package.json:17), `npm/@types/node` (services-node/relay-grpc/package.json:23), `npm/@bufbuild/protobuf` (services-node/relay-grpc/package.json:12) … 9 more  
designs 1: `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:1038)  
documents 3: `docs/RELAY.md` (docs/RELAY.md:208), `docs/code/service-matrix.md` (docs/code/service-matrix.md:11), `docs/plans/ASHLAR-REMEDIATION-WORKPLAN.md` (docs/plans/ASHLAR-REMEDIATION-WORKPLAN.md:66)  

### 49. Package npm/search-indexer
`pkg:npm/search-indexer`  jobs-node/search-indexer/package.json:2  pkg:npm/search-indexer  score 0.47  via `npm/@causeway/platform` <- DEPENDS_ON (jobs-node/search-indexer/package.json:13, 1.00)  
`npm/search-indexer`  
members 8  
dependencies 7: `npm/@causeway/search-core` (jobs-node/search-indexer/package.json:14), `npm/vitest` (jobs-node/search-indexer/package.json:21), `npm/typescript` (jobs-node/search-indexer/package.json:20) … 4 more  

### 50. Package npm/@connectrpc/connect-node
`pkg:npm/@connectrpc/connect-node`  pkg:npm/@connectrpc/connect-node  score 0.47  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:23, 1.00)  
dependents 5: `npm/roadmap-relay` (services-node/roadmap-relay/package.json:17), `npm/@causeway/relay` (torrentfx/packages-node/causeway-relay/package.json:18), `npm/ts-grpc` (services-node/relay-grpc/package.json:16) … 2 more  
importers 7: `services-node/relay-grpc/src/flight_client.ts` (services-node/relay-grpc/src/flight_client.ts:15), `torrentfx/packages-node/causeway-relay/src/app.ts` (torrentfx/packages-node/causeway-relay/src/app.ts:38), `torrentfx/packages-node/causeway-relay/test/example-service.ts` (torrentfx/packages-node/causeway-relay/test/example-service.ts:21) … 4 more  

### 51. Package npm/@effect/sql
`pkg:npm/@effect/sql`  pkg:npm/@effect/sql  score 0.47  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:27, 1.00)  
dependents 3: `npm/@causeway/search-core` (packages/search-core/package.json:25), `npm/@causeway/data` (torrentfx/packages-node/causeway-data/package.json:18), `npm/search-relay` (services-node/search-relay/package.json:27)  
importers 6: `torrentfx/packages-node/causeway-data/test/integration/data.test.ts` (torrentfx/packages-node/causeway-data/test/integration/data.test.ts:23), `torrentfx/packages-node/causeway-data/src/rows.ts` (torrentfx/packages-node/causeway-data/src/rows.ts:11), `torrentfx/packages-node/causeway-data/src/pool.ts` (torrentfx/packages-node/causeway-data/src/pool.ts:20) … 3 more  

### 52. Package npm/pg
`pkg:npm/pg`  pkg:npm/pg  score 0.47  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:29, 1.00)  
dependents 3: `npm/@causeway/search-core` (packages/search-core/package.json:34), `npm/search-relay` (services-node/search-relay/package.json:29), `npm/@causeway/data` (torrentfx/packages-node/causeway-data/package.json:21)  
importers 3: `torrentfx/packages-node/causeway-data/src/migrations.ts` (torrentfx/packages-node/causeway-data/src/migrations.ts:38), `torrentfx/packages-node/causeway-data/test/integration/postgres.ts` (torrentfx/packages-node/causeway-data/test/integration/postgres.ts:14), `torrentfx/packages-node/causeway-data/src/pool.ts` (torrentfx/packages-node/causeway-data/src/pool.ts:23)  

### 53. Package npm/testcontainers
`pkg:npm/testcontainers`  pkg:npm/testcontainers  score 0.47  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:30, 1.00)  
dependents 3: `npm/@causeway/search-core` (packages/search-core/package.json:35), `npm/search-relay` (services-node/search-relay/package.json:30), `npm/@causeway/data` (torrentfx/packages-node/causeway-data/package.json:26)  
importers 2: `torrentfx/packages-node/causeway-data/test/integration/postgres.ts` (torrentfx/packages-node/causeway-data/test/integration/postgres.ts:15), `packages/search-core/test/integration/qdrant.ts` (packages/search-core/test/integration/qdrant.ts:9)  

### 54. File services-node/search-relay/tests/setup-env.ts
`file:services-node/search-relay/tests/setup-env.ts`  services-node/search-relay/tests/setup-env.ts  pkg:npm/search-relay  score 0.45  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
imports 1: `npm/vitest` (services-node/search-relay/tests/setup-env.ts:4)  

### 55. Function serve
`typescript:services-node/search-relay/tests/harness.ts::serve`  services-node/search-relay/tests/harness.ts:107-129  pkg:npm/search-relay  score 0.44  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:107-129, 1.00)  
`async function serve(runtime: SearchRuntime, rpcAccess: RpcAccess): Promise<ServedRelay>`  
Serve the search relay over http2, searching in `runtime`.  
callers 4: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:90), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:30) … 1 more  
calls 4: `searchImpl` (services-node/search-relay/tests/harness.ts:113), `address` (services-node/search-relay/tests/harness.ts:118), `RelayApp::asgi` (services-node/search-relay/tests/harness.ts:114) … 1 more  
importers 2: `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24)  
tests 1: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134)  
uses_types 3: `RpcAccess` (services-node/search-relay/tests/harness.ts:107), `SearchRuntime` (services-node/search-relay/tests/harness.ts:107), `RelayApp` (services-node/search-relay/tests/harness.ts:108)  
services-node/search-relay/tests/harness.ts:107-129
```typescript
export async function serve(runtime: SearchRuntime, rpcAccess: RpcAccess): Promise<ServedRelay> {
  const app = new RelayApp<object>({
    serviceName: "search-relay-test",
    protoServices: [SearchService],
    rpcAccess,
  });
  app.bind(SearchService, { search: searchImpl(runtime) });
  const server = http2.createServer(app.asgi({ config: {} }));
  const sessions = new Set<http2.ServerHttp2Session>();
  server.on("session", (session) => sessions.add(session));
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = (server.address() as AddressInfo).port;
  return {
    client: createClient(
      SearchService,
      createConnectTransport({ baseUrl: `http://127.0.0.1:${port}`, httpVersion: "2" }),
    ),
    stop: async () => {
      for (const session of sessions) session.destroy();
      await new Promise<void>((resolve) => server.close(() => resolve()));
    },
  };
}
```

### 56. File crates/rpc-access-core/tests/conformance.rs
`file:crates/rpc-access-core/tests/conformance.rs`  crates/rpc-access-core/tests/conformance.rs  pkg:cargo/rpc-access-core  score 0.44  via `core` <- CONTAINS (crates/rpc-access-core/tests/conformance.rs:43-55, 1.00)  
members 9  
imports 22: `DeclaredClassTable` (crates/rpc-access-core/tests/conformance.rs:22-24), `KeySetAuthenticator` (crates/rpc-access-core/tests/conformance.rs:18), `AuthenticationPoint` (crates/rpc-access-core/tests/conformance.rs:15-17) … 19 more  

### 57. File services-node/search-relay/vitest.config.ts
`file:services-node/search-relay/vitest.config.ts`  services-node/search-relay/vitest.config.ts  pkg:npm/search-relay  score 0.43  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
imports 1: `npm/vitest` (services-node/search-relay/vitest.config.ts:1)  

### 58. File services-node/search-relay/vitest.integration.config.ts
`file:services-node/search-relay/vitest.integration.config.ts`  services-node/search-relay/vitest.integration.config.ts  pkg:npm/search-relay  score 0.43  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  
imports 1: `npm/vitest` (services-node/search-relay/vitest.integration.config.ts:1)  

### 59. Package npm/@torrentfx/session-conformance
`pkg:npm/@torrentfx/session-conformance`  torrentfx/conformance/session/package.json:2  pkg:npm/@torrentfx/session-conformance  score 0.43  via `npm/@connectrpc/connect-node` <- DEPENDS_ON (torrentfx/conformance/session/package.json:16, 1.00)  
`npm/@torrentfx/session-conformance`  
members 19  
dependencies 11: `npm/@causeway/session` (torrentfx/conformance/session/package.json:14), `npm/@bufbuild/protobuf` (torrentfx/conformance/session/package.json:13), `npm/@types/node` (torrentfx/conformance/session/package.json:22) … 8 more  
designs 1: `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:998)  

### 60. Test every_case_reaches_its_decision
`rust:crates/rpc-access-core/tests/conformance.rs::every_case_reaches_its_decision`  crates/rpc-access-core/tests/conformance.rs:120-145  pkg:cargo/rpc-access-core  score 0.43  via `core` -> TESTED_BY (crates/rpc-access-core/tests/conformance.rs:122, 1.00)  
`fn every_case_reaches_its_decision()`  
calls 15: `outcome` (crates/rpc-access-core/tests/conformance.rs:136), `authenticate` (crates/rpc-access-core/tests/conformance.rs:138), `text` (crates/rpc-access-core/tests/conformance.rs:127) … 12 more  
references 3: `Decision::Public` (crates/rpc-access-core/tests/conformance.rs:137), `AccessClass::Public` (crates/rpc-access-core/tests/conformance.rs:137), `AccessClass::Authorized` (crates/rpc-access-core/tests/conformance.rs:138)  
tests_for 11: `authenticate` (crates/rpc-access-core/tests/conformance.rs:138), `outcome` (crates/rpc-access-core/tests/conformance.rs:136), `authentication` (crates/rpc-access-core/tests/conformance.rs:123) … 8 more  

### 61. TypeAlias SearchRuntime
`typescript:services-node/search-relay/src/service.ts::SearchRuntime`  services-node/search-relay/src/service.ts:43  pkg:npm/search-relay  score 0.43  via `services-node/search-relay/src/runtime.ts` -> IMPORTS (services-node/search-relay/src/runtime.ts:9, 1.00)  
`type SearchRuntime = ManagedRuntime.ManagedRuntime<SearchServices, StartupError>;`  
The runtime the handler runs searches in: live Layers, or doubles.  
importers 3: `services-node/search-relay/tests/harness.ts` (services-node/search-relay/tests/harness.ts:28), `services-node/search-relay/src/runtime.ts` (services-node/search-relay/src/runtime.ts:9), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:31)  
used_by 5: `doublesRuntime` (services-node/search-relay/tests/harness.ts:135), `searchImpl` (services-node/search-relay/src/service.ts:98), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:47) … 2 more  
services-node/search-relay/src/service.ts:43-43
```typescript
export type SearchRuntime = ManagedRuntime.ManagedRuntime<SearchServices, StartupError>;
```

### 62. Function captures_all
`rhai:bootstrap/cw-host/src/modules/re.rs::captures_all`  bootstrap/cw-host/src/modules/re.rs:76-82  pkg:cargo/cw-host  score 0.42  via `test_the_addon_suites_run_after_the_single_build` -> CALLS (.mise/rhai/causeway/node_addon_release_test.rhai:59, 1.00)  
`m.set_native_fn("captures_all", |p: &str, s: &str| { with_re(p, |re| { re.captures_iter(s) .map(|c| Dynamic::from(groups(&c))) .collect::<Array>() }) })`  
callers 24: `test_the_workers_dead_letter_topics_are_ensured` (.mise/rhai/causeway/streams_test.rhai:407), `fn_body` (.mise/rhai/causeway/streams_test.rhai:197), `deploy_paths` (.mise/rhai/causeway/taskchecks.rhai:189) … 21 more  
calls 2: `with_re` (bootstrap/cw-host/src/modules/re.rs:77), `groups` (bootstrap/cw-host/src/modules/re.rs:79)  

### 63. Package npm/@causeway/web
`pkg:npm/@causeway/web`  apps-web/causeway-web/package.json:2  pkg:npm/@causeway/web  score 0.42  via `npm/@causeway/platform` <- DEPENDS_ON (apps-web/causeway-web/package.json:16, 1.00)  
`npm/@causeway/web`  
members 75  
dependencies 21: `npm/jose` (apps-web/causeway-web/package.json:24), `npm/effect` (apps-web/causeway-web/package.json:22), `npm/vitest` (apps-web/causeway-web/package.json:37) … 18 more  
designs 2: `docs/adrs/ADR-3q64wdh0-platform-surface-stack.md` (docs/adrs/ADR-3q64wdh0-platform-surface-stack.md:78), `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:221)  
documents 1: `apps-web/causeway-web/README.md` (apps-web/causeway-web/README.md:1-180)  

### 64. File ffi/node-rpc-access/test/conformance.test.mts
`file:ffi/node-rpc-access/test/conformance.test.mts`  ffi/node-rpc-access/test/conformance.test.mts  pkg:cargo/node-rpc-access  score 0.42  via `Case` <- CONTAINS (ffi/node-rpc-access/test/conformance.test.mts:16-21, 1.00)  
members 5  
imports 2: `npm/vitest` (ffi/node-rpc-access/test/conformance.test.mts:10), `ffi/node-rpc-access/index.js` (ffi/node-rpc-access/test/conformance.test.mts:12)  
uses_types 1: `Fixture` (ffi/node-rpc-access/test/conformance.test.mts:33)  

### 65. Interface Fixture
`typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts::Fixture`  torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:32-44  pkg:npm/@causeway/relay  score 0.41  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:32-44, 1.00)  
`interface Fixture`  
used_by 1: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:47)  

### 66. File services-node/search-relay/moon.yml
`file:services-node/search-relay/moon.yml`  services-node/search-relay/moon.yml  pkg:npm/search-relay  score 0.41  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  

### 67. File services-node/search-relay/package.json
`file:services-node/search-relay/package.json`  services-node/search-relay/package.json  pkg:npm/search-relay  score 0.41  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  

### 68. File services-node/search-relay/tsconfig.json
`file:services-node/search-relay/tsconfig.json`  services-node/search-relay/tsconfig.json  pkg:npm/search-relay  score 0.41  via `npm/search-relay` -> CONTAINS (services-node/search-relay/package.json:2, 1.00)  

### 69. Interface ServedRelay
`typescript:services-node/search-relay/tests/harness.ts::ServedRelay`  services-node/search-relay/tests/harness.ts:101-104  pkg:npm/search-relay  score 0.41  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:101-104, 1.00)  
`interface ServedRelay`  
members 1  
importers 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40)  
used_by 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:27), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:48)  

### 70. File torrentfx/packages-node/causeway-relay/test/example-service.ts
`file:torrentfx/packages-node/causeway-relay/test/example-service.ts`  torrentfx/packages-node/causeway-relay/test/example-service.ts  pkg:npm/@causeway/relay  score 0.41  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> IMPORTS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30, 1.00)  
members 3  
importers 2: `torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:25), `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30)  
imports 4: `npm/@connectrpc/connect` (torrentfx/packages-node/causeway-relay/test/example-service.ts:19), `npm/@bufbuild/protobuf` (torrentfx/packages-node/causeway-relay/test/example-service.ts:12), `npm/@connectrpc/connect-node` (torrentfx/packages-node/causeway-relay/test/example-service.ts:21) … 1 more  

### 71. File ffi/node-rpc-access/build.rs
`file:ffi/node-rpc-access/build.rs`  ffi/node-rpc-access/build.rs  pkg:cargo/node-rpc-access  score 0.40  via `main` <- CONTAINS (ffi/node-rpc-access/build.rs:1-3, 1.00)  
members 1  

### 72. Function searchImpl
`typescript:services-node/search-relay/src/service.ts::searchImpl`  services-node/search-relay/src/service.ts:98-117  pkg:npm/search-relay  score 0.39  via `services-node/search-relay/src/server.ts` -> IMPORTS (services-node/search-relay/src/server.ts:13, 1.00)  
`function searchImpl(runtime: SearchRuntime)`  
The `search` handler, running each search in `runtime`.  
callers 2: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:37), `serve` (services-node/search-relay/tests/harness.ts:113)  
calls 6: `searcherOf` (services-node/search-relay/src/service.ts:101), `program` (services-node/search-relay/src/service.ts:102), `getLogger` (services-node/search-relay/src/service.ts:99) … 3 more  
importers 2: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:13), `services-node/search-relay/tests/harness.ts` (services-node/search-relay/tests/harness.ts:28)  
uses_types 2: `SearchRuntime` (services-node/search-relay/src/service.ts:98), `SearchRequest` (services-node/search-relay/src/service.ts:100)  

### 73. Test rpc-access over the wire, with the addon and the decision fixture
`typescript:torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts::rpc-access over the wire, with the addon and the decision fixture`  torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:46-128  pkg:npm/@causeway/relay  score 0.39  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:46-128, 1.00)  
`describe("rpc-access over the wire, with the addon and the decision fixture", () => { const fixture: Fixture = JSON.parse( readFileSync(join(root, "torrentfx/conformance/rpc-access/fixture.json"), "utf-8"), ); const JWKS_URI = "https://issuer.rpc-access.test/keys"; let directory: string; let relay: …`  
members 1  
calls 4: `bindRecording` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:83), `parseTokenVerifier` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:63), `serve` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:84) … 1 more  
tests_for 4: `parseTokenVerifier` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:63), `serve` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:84), `bindRecording` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:83) … 1 more  
uses_types 2: `RelayApp` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:78), `Fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:47)  

### 74. Function storeDouble
`typescript:services-node/search-relay/tests/harness.ts::storeDouble`  services-node/search-relay/tests/harness.ts:83-88  pkg:npm/search-relay  score 0.39  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:83-88, 1.00)  
`storeDouble = (hits: readonly SearchHit[]) => Layer.succeed(VectorStore, { ensureCollection: Effect.void, upsert: () => Effect.void, queryHybrid: (_dense, _sparse, limit) => Effect.succeed(hits.slice(0, limit)), })`  
A store that answers every query with `hits`.  
callers 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:31), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:88)  
importers 2: `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24)  

### 75. Package npm/@torrentfx/flight-native
`pkg:npm/@torrentfx/flight-native`  torrentfx/packages-node/flight-native/package.json:2  pkg:npm/@torrentfx/flight-native  score 0.39  via `npm/@napi-rs/cli` <- DEPENDS_ON (torrentfx/packages-node/flight-native/package.json:28, 1.00)  
`npm/@torrentfx/flight-native`  
members 12  
dependencies 6: `npm/vitest` (torrentfx/packages-node/flight-native/package.json:33), `npm/apache-arrow` (torrentfx/packages-node/flight-native/package.json:31), `npm/@types/node` (torrentfx/packages-node/flight-native/package.json:30) … 3 more  
designs 1: `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:697)  
documents 1: `torrentfx/packages-node/flight-native/README.md` (torrentfx/packages-node/flight-native/README.md:1-55)  

### 76. File tools/node-addon/fetch-prebuilt.sh
`file:tools/node-addon/fetch-prebuilt.sh`  tools/node-addon/fetch-prebuilt.sh  pkg:npm/causeway-monorepo  score 0.38  via `mise/ffi:rpc-access:fetch-prebuilt` -> RUNS (.mise/toml/ffi.toml:99, 1.00)  
documents 1: `docs/NODE-ADDONS.md` (docs/NODE-ADDONS.md:32)  
run_by 1: `mise/ffi:rpc-access:fetch-prebuilt` (.mise/toml/ffi.toml:99)  

### 77. Interface SearchRelayConfig
`typescript:services-node/search-relay/src/config.ts::SearchRelayConfig`  services-node/search-relay/src/config.ts:56-61  pkg:npm/search-relay  score 0.38  via `services-node/search-relay/src/config.ts` -> CONTAINS (services-node/search-relay/src/config.ts:56-61, 1.00)  
`interface SearchRelayConfig extends RelayConfig`  
extends 1: `RelayConfig` (services-node/search-relay/src/config.ts:56)  
importers 2: `services-node/search-relay/src/runtime.ts` (services-node/search-relay/src/runtime.ts:8), `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:11)  
used_by 4: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:15), `searchRuntime` (services-node/search-relay/src/runtime.ts:29), `searchLayer` (services-node/search-relay/src/runtime.ts:11) … 1 more  
uses_types 5: `SearchRuntimeConfig` (services-node/search-relay/src/config.ts:59), `ServiceSettings` (services-node/search-relay/src/config.ts:57), `RpcAccessConfig` (services-node/search-relay/src/config.ts:60) … 2 more  

### 78. Function searchRelayConfigFromEnv
`typescript:services-node/search-relay/src/config.ts::searchRelayConfigFromEnv`  services-node/search-relay/src/config.ts:83-102  pkg:npm/search-relay  score 0.38  via `services-node/search-relay/src/config.ts` -> CONTAINS (services-node/search-relay/src/config.ts:83-102, 1.00)  
`function searchRelayConfigFromEnv( env: NodeJS.ProcessEnv = process.env, resolved: ResolvedConfig = resolvedConfigFromEnv(SERVICE_NAME), ): SearchRelayConfig`  
callers 4: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:15), `searchRelayConfigFromEnv::reads its own settings under SEARCH_RELAY_` (services-node/search-relay/tests/config.test.ts:52), `searchRelayConfigFromEnv::refuses to start when the topology names no verifier for its callers` (services-node/search-relay/tests/config.test.ts:69) … 1 more  
calls 5: `resolvedConfigFromEnv` (services-node/search-relay/src/config.ts:85), `rpcVerifierFromResolved` (services-node/search-relay/src/config.ts:98), `searchRuntimeConfigFromEnv` (services-node/search-relay/src/config.ts:94) … 2 more  
importers 2: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:11), `services-node/search-relay/tests/config.test.ts` (services-node/search-relay/tests/config.test.ts:11-16)  
tests 3: `searchRelayConfigFromEnv::takes Qdrant, the workspace and the verifier from the topology` (services-node/search-relay/tests/config.test.ts:32), `searchRelayConfigFromEnv::reads its own settings under SEARCH_RELAY_` (services-node/search-relay/tests/config.test.ts:52), `searchRelayConfigFromEnv::refuses to start when the topology names no verifier for its callers` (services-node/search-relay/tests/config.test.ts:69)  
uses_types 3: `SearchRelayConfig` (services-node/search-relay/src/config.ts:86), `ResolvedConfig` (services-node/search-relay/src/config.ts:85), `ResolvedConfig` (services-node/search-relay/src/config.ts:85)  

### 79. Package npm/ts-worker
`pkg:npm/ts-worker`  jobs-node/relay-worker/package.json:2  pkg:npm/ts-worker  score 0.38  via `npm/@causeway/platform` <- DEPENDS_ON (jobs-node/relay-worker/package.json:12, 1.00)  
`npm/ts-worker`  
members 6  
dependencies 6: `npm/@causeway/relay` (jobs-node/relay-worker/package.json:13), `npm/tsx` (jobs-node/relay-worker/package.json:18), `npm/@types/node` (jobs-node/relay-worker/package.json:17) … 3 more  
documents 1: `docs/code/service-matrix.md` (docs/code/service-matrix.md:16)  

### 80. Package npm/tsx
`pkg:npm/tsx`  pkg:npm/tsx  score 0.38  via `npm/search-relay` -> DEPENDS_ON (services-node/search-relay/package.json:31, 1.00)  
dependents 12: `npm/vscode-causeway-contracts` (extensions/vscode-causeway-contracts/package.json:125), `npm/ts-restate` (services-node/relay-restate/package.json:17), `npm/ts-worker` (jobs-node/relay-worker/package.json:18) … 9 more  
importers 1: `torrentfx/packages-node/arrow-e2e-bench/src/client-worker-boot.mjs` (torrentfx/packages-node/arrow-e2e-bench/src/client-worker-boot.mjs:4)  

### 81. Package npm/@torrentfx/arrow-e2e-bench
`pkg:npm/@torrentfx/arrow-e2e-bench`  torrentfx/packages-node/arrow-e2e-bench/package.json:2  pkg:npm/@torrentfx/arrow-e2e-bench  score 0.37  via `npm/effect` <- DEPENDS_ON (torrentfx/packages-node/arrow-e2e-bench/package.json:17, 1.00)  
`npm/@torrentfx/arrow-e2e-bench`  
members 16  
dependencies 11: `npm/@types/ws` (torrentfx/packages-node/arrow-e2e-bench/package.json:23), `npm/tsx` (torrentfx/packages-node/arrow-e2e-bench/package.json:18), `npm/vitest` (torrentfx/packages-node/arrow-e2e-bench/package.json:26) … 8 more  

### 82. File crates/rpc-access-core/src/lib.rs
`file:crates/rpc-access-core/src/lib.rs`  crates/rpc-access-core/src/lib.rs  pkg:cargo/rpc-access-core  score 0.37  via `Repr` <- CONTAINS (crates/rpc-access-core/src/lib.rs:95-98, 1.00)  
members 26  
imports 22: `AuthorizationError` (crates/rpc-access-core/src/lib.rs:32-35), `PolicySnapshot` (crates/rpc-access-core/src/lib.rs:36), `Authenticated` (crates/rpc-access-core/src/lib.rs:32-35) … 19 more  

### 83. Package npm/causeway-monorepo
`pkg:npm/causeway-monorepo`  package.json:2  pkg:npm/causeway-monorepo  score 0.35  via `.mise/toml/ffi.toml` <- CONTAINS (package.json:2, 1.00)  
`npm/causeway-monorepo`  
members 1280  
dependencies 8: `npm/@bufbuild/protoc-gen-es` (package.json:9), `npm/@bufbuild/protobuf` (package.json:8), `npm/@commitlint/config-conventional` (package.json:11) … 5 more  
documents 1: `README.md` (README.md:1-98)  

### 84. Function authentication
`rust:crates/rpc-access-core/tests/conformance.rs::authentication`  crates/rpc-access-core/tests/conformance.rs:57-71  pkg:cargo/rpc-access-core  score 0.35  via `text` <- CALLS (crates/rpc-access-core/tests/conformance.rs:63, 1.00)  
`fn authentication(fixture: &Value) -> AuthenticationPoint`  
callers 1: `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:123)  
calls 8: `AuthenticationPoint::new` (crates/rpc-access-core/tests/conformance.rs:67), `KeySetAuthenticator::new` (crates/rpc-access-core/tests/conformance.rs:67), `Environment::parse` (crates/rpc-access-core/tests/conformance.rs:63) … 5 more  
tests 1: `every_case_reaches_its_decision` (crates/rpc-access-core/tests/conformance.rs:123)  
uses_types 1: `AuthenticationPoint` (crates/rpc-access-core/tests/conformance.rs:57)  

### 85. Function parseTokenVerifier
`typescript:torrentfx/packages-node/causeway-platform/src/verifier.ts::parseTokenVerifier`  torrentfx/packages-node/causeway-platform/src/verifier.ts:120-133  pkg:npm/@causeway/platform  score 0.34  via `RpcAccess.load without the native addon::refuses to load with a ConfigError that names the addon and how to get it` -> CALLS (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:197, 1.00)  
`function parseTokenVerifier( issuer: string, audience: string, jwksUri: string | undefined, environment: Environment, ): TokenVerifierTopology`  
The verifier three settings state, as a table or a variable states them, in `environment`. Throws `ConfigError` with the first value refused.  
callers 9: `verifier values::no keys URL means discovery, never a conventional path` (torrentfx/packages-node/causeway-platform/test/verifier.test.ts:91), `rpcVerifierFromResolved` (services-node/search-relay/src/config.ts:75), `cache` (torrentfx/packages-node/causeway-relay/test/keys.test.ts:54) … 6 more  
calls 3: `parseAudience` (torrentfx/packages-node/causeway-platform/src/verifier.ts:130), `parseIssuer` (torrentfx/packages-node/causeway-platform/src/verifier.ts:126), `jwksSourceFromSetting` (torrentfx/packages-node/causeway-platform/src/verifier.ts:131)  
importers 2: `torrentfx/packages-node/causeway-platform/src/config.ts` (torrentfx/packages-node/causeway-platform/src/config.ts:54), `torrentfx/packages-node/causeway-platform/test/verifier.test.ts` (torrentfx/packages-node/causeway-platform/test/verifier.test.ts:12-18)  
tests 4: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:63), `relay workload-authentication conformance::refuses a scoped streaming procedure before its handler runs, and serves the token` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:197), `verifier values::no keys URL means discovery, never a conventional path` (torrentfx/packages-node/causeway-platform/test/verifier.test.ts:91) … 1 more  
uses_types 2: `Environment` (torrentfx/packages-node/causeway-platform/src/verifier.ts:124), `TokenVerifierTopology` (torrentfx/packages-node/causeway-platform/src/verifier.ts:125)  

### 86. Function module
`rust:bootstrap/cw-host/src/modules/re.rs::module`  bootstrap/cw-host/src/modules/re.rs:49-98  pkg:cargo/cw-host  score 0.34  via `captures_all` <- DEFINES (bootstrap/cw-host/src/modules/re.rs:76-82, 1.00)  
`pub fn module() -> Module`  
members 10  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:27)  

### 87. Function bindRecording
`typescript:torrentfx/packages-node/causeway-relay/test/example-service.ts::bindRecording`  torrentfx/packages-node/causeway-relay/test/example-service.ts:77-92  pkg:npm/@causeway/relay  score 0.34  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> IMPORTS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30, 1.00)  
`function bindRecording(app: RelayApp<object>, seen: (RpcGrant | "no grant")[]): void`  
Handlers that record the grant each call ran with.  
members 1  
callers 4: `the rpc-access interceptor::classifies before it reads a header, and a public method has no grant` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:138), `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:83), `the rpc-access interceptor::maps every decision to the code the Rust enforcement point returns` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:108) … 1 more  
calls 4: `bindRecording::record` (torrentfx/packages-node/causeway-relay/test/example-service.ts:89), `bindRecording::record` (torrentfx/packages-node/causeway-relay/test/example-service.ts:88), `bindRecording::record` (torrentfx/packages-node/causeway-relay/test/example-service.ts:87) … 1 more  
importers 2: `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30), `torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:25)  
tests 4: `the rpc-access interceptor::classifies before it reads a header, and a public method has no grant` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:138), `the rpc-access interceptor::refuses to start a relay that mounts a method its table does not classify` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:172), `the rpc-access interceptor::maps every decision to the code the Rust enforcement point returns` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:108) … 1 more  
uses_types 1: `RelayApp` (torrentfx/packages-node/causeway-relay/test/example-service.ts:77)  

### 88. Function read
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::read`  .mise/rhai/causeway/node_addon_release_test.rhai:8-10  pkg:npm/causeway-monorepo  score 0.33  via `task_body` -> CALLS (.mise/rhai/causeway/node_addon_release_test.rhai:33, 1.00)  
`fn read(path)`  
callers 2: `task_body` (.mise/rhai/causeway/node_addon_release_test.rhai:33), `pre_publish_lines` (.mise/rhai/causeway/node_addon_release_test.rhai:15)  
calls 2: `repo_root` (.mise/rhai/causeway/node_addon_release_test.rhai:9), `read` (.mise/rhai/causeway/node_addon_release_test.rhai:9)  

### 89. Package npm/ts-restate
`pkg:npm/ts-restate`  services-node/relay-restate/package.json:2  pkg:npm/ts-restate  score 0.33  via `npm/@causeway/platform` <- DEPENDS_ON (services-node/relay-restate/package.json:12, 1.00)  
`npm/ts-restate`  
members 7  
dependencies 5: `npm/tsx` (services-node/relay-restate/package.json:17), `npm/typescript` (services-node/relay-restate/package.json:18), `npm/@restatedev/restate-sdk` (services-node/relay-restate/package.json:13) … 2 more  
documents 1: `docs/code/service-matrix.md` (docs/code/service-matrix.md:14)  

### 90. Package npm/@causeway/content-core
`pkg:npm/@causeway/content-core`  packages/content-core/package.json:2  pkg:npm/@causeway/content-core  score 0.33  via `npm/effect` <- DEPENDS_ON (packages/content-core/package.json:18, 1.00)  
`npm/@causeway/content-core`  
members 27  
dependents 1: `npm/@causeway/web` (apps-web/causeway-web/package.json:15)  
dependencies 18: `npm/rehype-stringify` (packages/content-core/package.json:21), `npm/effect` (packages/content-core/package.json:18), `npm/@types/node` (packages/content-core/package.json:35) … 15 more  
designs 1: `docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md` (docs/rfd/RFD-c4jbh7qa-typescript-and-node-web-tier-over-the-rust-core.md:252)  
importers 12: `apps-web/causeway-web/src/pages/runtime.ts` (apps-web/causeway-web/src/pages/runtime.ts:19-24), `apps-web/causeway-web/src/pages/jsonld.ts` (apps-web/causeway-web/src/pages/jsonld.ts:7), `apps-web/causeway-web/src/pages/routes/Docs.tsx` (apps-web/causeway-web/src/pages/routes/Docs.tsx:3-21) … 9 more  

### 91. Function searchRuntime
`typescript:services-node/search-relay/src/runtime.ts::searchRuntime`  services-node/search-relay/src/runtime.ts:29-31  pkg:npm/search-relay  score 0.32  via `services-node/search-relay/src/runtime.ts` -> CONTAINS (services-node/search-relay/src/runtime.ts:29-31, 1.00)  
`function searchRuntime(config: SearchRelayConfig): SearchRuntime`  
callers 1: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:29)  
calls 1: `searchLayer` (services-node/search-relay/src/runtime.ts:30)  
importers 1: `services-node/search-relay/src/server.ts` (services-node/search-relay/src/server.ts:12)  
uses_types 2: `SearchRelayConfig` (services-node/search-relay/src/runtime.ts:29), `SearchRuntime` (services-node/search-relay/src/runtime.ts:29)  

### 92. Function doublesRuntime
`typescript:services-node/search-relay/tests/harness.ts::doublesRuntime`  services-node/search-relay/tests/harness.ts:132-137  pkg:npm/search-relay  score 0.31  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:132-137, 1.00)  
`function doublesRuntime( store: Layer.Layer<VectorStore>, log: Layer.Layer<SearchLog>, ): SearchRuntime`  
A runtime of doubles.  
callers 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:31), `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134)  
importers 1: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24)  
tests 1: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134)  
uses_types 1: `SearchRuntime` (services-node/search-relay/tests/harness.ts:135)  

### 93. Function logDouble
`typescript:services-node/search-relay/tests/harness.ts::logDouble`  services-node/search-relay/tests/harness.ts:91-94  pkg:npm/search-relay  score 0.31  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:91-94, 1.00)  
`logDouble = (into: SearchPerformed[]) => Layer.succeed(SearchLog, { record: (event) => Effect.sync(() => void into.push(event)), })`  
A log that keeps what it records in `into`.  
callers 2: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134), `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:31)  
importers 1: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24)  
tests 1: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:134)  

### 94. Package npm/@torrentfx/runtime-web
`pkg:npm/@torrentfx/runtime-web`  torrentfx/web/framework/runtime-web/package.json:2  pkg:npm/@torrentfx/runtime-web  score 0.31  via `npm/effect` <- DEPENDS_ON (torrentfx/web/framework/runtime-web/package.json:23, 1.00)  
`npm/@torrentfx/runtime-web`  
members 35  
dependents 17: `npm/@causeway/ui-explorer-codecs` (web/ui-explorer-codecs/package.json:20), `npm/@causeway/showcase` (apps-web/showcase/package.json:27), `npm/@causeway/ui-roadmap` (web/ui-roadmap/package.json:22) … 14 more  
dependencies 15: `npm/@connectrpc/connect-web` (torrentfx/web/framework/runtime-web/package.json:21), `npm/apache-arrow` (torrentfx/web/framework/runtime-web/package.json:22), `npm/@torrentfx/core` (torrentfx/web/framework/runtime-web/package.json:36) … 12 more  
importers 23: `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:18), `web/ui-semantic-mapping/src/index.ts` (web/ui-semantic-mapping/src/index.ts:6), `web/ui-request/src/index.ts` (web/ui-request/src/index.ts:13) … 20 more  

### 95. Method ServedRelay::stop
`typescript:services-node/search-relay/tests/harness.ts::ServedRelay::stop`  services-node/search-relay/tests/harness.ts:103  pkg:npm/search-relay  score 0.31  via `services-node/search-relay/tests/rpc-access.test.ts` -> CALLS (services-node/search-relay/tests/rpc-access.test.ts:37, 1.00)  
`stop(): Promise<void>`  
callers 4: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:37), `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:144), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:97) … 1 more  
tests 1: `a backend failure::is Unavailable to the caller, without the backend's reason` (services-node/search-relay/tests/rpc-access.test.ts:144)  
services-node/search-relay/tests/harness.ts:103-103
```typescript
  stop(): Promise<void>;
```

### 96. File torrentfx/packages-node/causeway-relay/src/app.ts
`file:torrentfx/packages-node/causeway-relay/src/app.ts`  torrentfx/packages-node/causeway-relay/src/app.ts  pkg:npm/@causeway/relay  score 0.30  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` <- IMPORTS (torrentfx/packages-node/causeway-relay/src/app.ts:42, 1.00)  
members 13  
calls 7: `withDefault` (torrentfx/packages-node/causeway-relay/src/app.ts:89), `optional` (torrentfx/packages-node/causeway-relay/src/app.ts:87), `settingsPrefix` (torrentfx/packages-node/causeway-relay/src/app.ts:79) … 4 more  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:2263)  
importers 1: `torrentfx/packages-node/causeway-relay/src/testing.ts` (torrentfx/packages-node/causeway-relay/src/testing.ts:18)  
imports 11: `WorkloadAuthorizer` (torrentfx/packages-node/causeway-relay/src/app.ts:40), `npm/@connectrpc/connect-node` (torrentfx/packages-node/causeway-relay/src/app.ts:38), `npm/@causeway/platform` (torrentfx/packages-node/causeway-relay/src/app.ts:12-36) … 8 more  
uses_types 1: `SettingsPrefix` (torrentfx/packages-node/causeway-relay/src/app.ts:79)  

### 97. Class AddonDecider
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::AddonDecider`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:266-315  pkg:npm/@causeway/relay  score 0.30  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:266-315, 1.00)  
`class AddonDecider implements RpcDecider`  
The decision core in the addon, with the issuer's keys from `KeyCache`.  
members 3  
calls 3: `KeyCache::keysFor` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:298), `unavailable` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:300), `parseKeySet` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:311)  
implements 1: `RpcDecider` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:266)  
used_by 1: `RpcAccess::load` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:150)  
uses_types 2: `AddonModule` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:267), `KeyCache` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:269)  

### 98. Interface RpcDecider
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::RpcDecider`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:70-75  pkg:npm/@causeway/relay  score 0.30  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:70-75, 1.00)  
`interface RpcDecider`  
The decision core a relay asks: `RpcAccess.load` binds it to the addon. A test hands in its own.  
members 2  
implementations 1: `AddonDecider` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:266)  
used_by 3: `RpcAccess::of` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:155), `RpcAccess` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:120), `RpcAccess::constructor` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:122)  

### 99. Class ConfigError
`typescript:torrentfx/packages-node/causeway-platform/src/errors.ts::ConfigError`  torrentfx/packages-node/causeway-platform/src/errors.ts:2-4  pkg:npm/@causeway/platform  score 0.30  via `loadAddon` -> USES_TYPE (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:259, 1.00)  
`class ConfigError extends Error`  
Raised when configuration loading or validation fails.  
extended_by 2: `AliasError` (torrentfx/packages-node/causeway-platform/src/alias.ts:19), `SecretError` (torrentfx/packages-node/causeway-platform/src/secret.ts:37)  
importers 9: `torrentfx/packages-node/causeway-platform/src/browser.ts` (torrentfx/packages-node/causeway-platform/src/browser.ts:14), `torrentfx/packages-node/causeway-platform/src/verifier.ts` (torrentfx/packages-node/causeway-platform/src/verifier.ts:17), `torrentfx/packages-node/causeway-platform/src/scope.ts` (torrentfx/packages-node/causeway-platform/src/scope.ts:16) … 6 more  
used_by 58: `refuse` (torrentfx/packages-node/causeway-platform/src/topology.ts:133), `resolveDirectory` (torrentfx/packages-node/causeway-platform/src/config.ts:403), `parseIssuer` (torrentfx/packages-node/causeway-platform/src/verifier.ts:77) … 55 more  

### 100. TypeAlias AddonModule
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::AddonModule`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:248  pkg:npm/@causeway/relay  score 0.30  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:248, 1.00)  
`type AddonModule = typeof Addon;`  
used_by 2: `AddonDecider::constructor` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:274), `AddonDecider` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:267)  

### 101. File packages/search-core/test/integration/search-log.test.ts
`file:packages/search-core/test/integration/search-log.test.ts`  packages/search-core/test/integration/search-log.test.ts  pkg:npm/@causeway/search-core  score 0.29  via `npm/@causeway/data` <- IMPORTS (packages/search-core/test/integration/search-log.test.ts:11-20, 1.00)  
members 6  
calls 7: `migrate` (packages/search-core/test/integration/search-log.test.ts:101), `componentSchema` (packages/search-core/test/integration/search-log.test.ts:46), `workspaceScope` (packages/search-core/test/integration/search-log.test.ts:45) … 4 more  
imports 18: `SearchLog` (packages/search-core/test/integration/search-log.test.ts:37), `npm/vitest` (packages/search-core/test/integration/search-log.test.ts:30), `packages/search-core/src/search-log.ts` (packages/search-core/test/integration/search-log.test.ts:39) … 15 more  
uses_types 4: `Secret` (packages/search-core/test/integration/search-log.test.ts:102), `EphemeralPostgres` (packages/search-core/test/integration/search-log.test.ts:48), `ResolvedConfig` (packages/search-core/test/integration/search-log.test.ts:49) … 1 more  

### 102. Function as
`typescript:services-node/search-relay/tests/harness.ts::as`  services-node/search-relay/tests/harness.ts:140-144  pkg:npm/search-relay  score 0.29  via `services-node/search-relay/tests/harness.ts` -> CONTAINS (services-node/search-relay/tests/harness.ts:140-144, 1.00)  
`function as(caller: string): { headers: Record<string, string> }`  
The headers a caller presenting the fixture's `caller` token sends.  
callers 10: `an unauthorized caller::is refused with an expired token` (services-node/search-relay/tests/rpc-access.test.ts:74), `the search relay over Postgres::records nothing for a caller Cedar refuses` (services-node/search-relay/tests/integration/attribution.test.ts:124), `an unauthorized caller::is refused when an agent acts for a person without a delegation token` (services-node/search-relay/tests/rpc-access.test.ts:83) … 7 more  
importers 2: `services-node/search-relay/tests/rpc-access.test.ts` (services-node/search-relay/tests/rpc-access.test.ts:15-24), `services-node/search-relay/tests/integration/attribution.test.ts` (services-node/search-relay/tests/integration/attribution.test.ts:32-40)  
tests 10: `an authorized caller::gets results, and the search is attributed to their subject` (services-node/search-relay/tests/rpc-access.test.ts:92), `an authorized caller::gets no hits for a blank query, and nothing is recorded` (services-node/search-relay/tests/rpc-access.test.ts:114), `an authorized caller::attributes a workload's search to the workload` (services-node/search-relay/tests/rpc-access.test.ts:107) … 7 more  

### 103. Package npm/@causeway/ui-roadmap
`pkg:npm/@causeway/ui-roadmap`  web/ui-roadmap/package.json:2  pkg:npm/@causeway/ui-roadmap  score 0.29  via `npm/effect` <- DEPENDS_ON (web/ui-roadmap/package.json:23, 1.00)  
`npm/@causeway/ui-roadmap`  
members 9  
dependencies 10: `npm/typescript` (web/ui-roadmap/package.json:27), `npm/solid-js` (web/ui-roadmap/package.json:24), `npm/@torrentfx/runtime-web` (web/ui-roadmap/package.json:22) … 7 more  
designs 1: `docs/adrs/ADR-v68na9qx-ts-only-slices-islands-relay-proxy.md` (docs/adrs/ADR-v68na9qx-ts-only-slices-islands-relay-proxy.md:93)  

### 104. TypeAlias Method
`typescript:torrentfx/packages-node/causeway-relay/test/example-service.ts::Method`  torrentfx/packages-node/causeway-relay/test/example-service.ts:49  pkg:npm/@causeway/relay  score 0.29  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> IMPORTS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30, 1.00)  
`type Method = "ping" | "getThing" | "putThing" | "exportThing";`  
importers 1: `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30)  
used_by 1: `serve` (torrentfx/packages-node/causeway-relay/test/example-service.ts:53)  

### 105. Function serve
`typescript:torrentfx/packages-node/causeway-relay/test/example-service.ts::serve`  torrentfx/packages-node/causeway-relay/test/example-service.ts:52-74  pkg:npm/@causeway/relay  score 0.29  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> IMPORTS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30, 1.00)  
`async function serve(app: RelayApp<object>): Promise<{ call: (method: Method, headers?: Record<string, string>) => Promise<void>; stop: () => Promise<void>; }>`  
Serves `app` over http2 until the returned stop is called.  
callers 4: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:84), `the rpc-access interceptor::classifies before it reads a header, and a public method has no grant` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:139), `the rpc-access interceptor::maps every decision to the code the Rust enforcement point returns` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:109) … 1 more  
calls 3: `RelayApp::asgi` (torrentfx/packages-node/causeway-relay/test/example-service.ts:56), `address` (torrentfx/packages-node/causeway-relay/test/example-service.ts:60), `listen` (torrentfx/packages-node/causeway-relay/test/example-service.ts:59)  
importers 2: `torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:25), `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:23-30)  
tests 3: `rpc-access over the wire, with the addon and the decision fixture` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:84), `the rpc-access interceptor::maps every decision to the code the Rust enforcement point returns` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:109), `the rpc-access interceptor::classifies before it reads a header, and a public method has no grant` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:139)  
uses_types 2: `RelayApp` (torrentfx/packages-node/causeway-relay/test/example-service.ts:52), `Method` (torrentfx/packages-node/causeway-relay/test/example-service.ts:53)  

### 106. File torrentfx/packages-node/causeway-relay/src/index.ts
`file:torrentfx/packages-node/causeway-relay/src/index.ts`  torrentfx/packages-node/causeway-relay/src/index.ts  pkg:npm/@causeway/relay  score 0.29  via `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` -> IMPORTS (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:22, 1.00)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:854)  
importers 6: `torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:36-42), `torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:16-24), `torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts` (torrentfx/packages-node/causeway-relay/test/rpc-access.addon.test.ts:22) … 3 more  

### 107. Package npm/vscode-causeway-catalog
`pkg:npm/vscode-causeway-catalog`  extensions/vscode-causeway-catalog/package.json:2  pkg:npm/vscode-causeway-catalog  score 0.28  via `npm/tsx` <- DEPENDS_ON (extensions/vscode-causeway-catalog/package.json:82, 1.00)  
`npm/vscode-causeway-catalog`  
members 19  
dependencies 6: `npm/typescript` (extensions/vscode-causeway-catalog/package.json:83), `npm/@types/node` (extensions/vscode-causeway-catalog/package.json:78), `npm/esbuild` (extensions/vscode-causeway-catalog/package.json:81) … 3 more  
documents 1: `extensions/vscode-causeway-catalog/README.md` (extensions/vscode-causeway-catalog/README.md:1-67)  

### 108. Package npm/vscode-causeway-contracts
`pkg:npm/vscode-causeway-contracts`  extensions/vscode-causeway-contracts/package.json:2  pkg:npm/vscode-causeway-contracts  score 0.28  via `npm/tsx` <- DEPENDS_ON (extensions/vscode-causeway-contracts/package.json:125, 1.00)  
`npm/vscode-causeway-contracts`  
members 9  
dependencies 6: `npm/tsx` (extensions/vscode-causeway-contracts/package.json:125), `npm/@types/node` (extensions/vscode-causeway-contracts/package.json:121), `npm/@vscode/vsce` (extensions/vscode-causeway-contracts/package.json:123) … 3 more  
documents 1: `extensions/vscode-causeway-contracts/README.md` (extensions/vscode-causeway-contracts/README.md:1-45)  

### 109. Package npm/vscode-causeway-pure
`pkg:npm/vscode-causeway-pure`  extensions/vscode-causeway-pure/package.json:2  pkg:npm/vscode-causeway-pure  score 0.28  via `npm/tsx` <- DEPENDS_ON (extensions/vscode-causeway-pure/package.json:90, 1.00)  
`npm/vscode-causeway-pure`  
members 15  
dependencies 12: `npm/@types/react` (extensions/vscode-causeway-pure/package.json:85), `npm/esbuild` (extensions/vscode-causeway-pure/package.json:89), `npm/tsx` (extensions/vscode-causeway-pure/package.json:90) … 9 more  
documents 1: `extensions/vscode-causeway-pure/README.md` (extensions/vscode-causeway-pure/README.md:1-46)  

### 110. File torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts
`file:torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts`  torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts  pkg:npm/@causeway/relay  score 0.28  via `npm/@connectrpc/connect-node` <- IMPORTS (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:33, 1.00)  
members 14  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:857)  
imports 6: `torrentfx/packages-node/causeway-relay/src/index.ts` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:36-42), `npm/@connectrpc/connect` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:32), `npm/@bufbuild/protobuf` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:24) … 3 more  
uses_types 1: `Fixture` (torrentfx/packages-node/causeway-relay/test/relay-conformance.test.ts:110)  

### 111. Test every_refused_class_table_is_refused_at_load
`rust:crates/rpc-access-core/tests/conformance.rs::every_refused_class_table_is_refused_at_load`  crates/rpc-access-core/tests/conformance.rs:148-158  pkg:cargo/rpc-access-core  score 0.28  via `text` -> TESTED_BY (crates/rpc-access-core/tests/conformance.rs:151, 1.00)  
`fn every_refused_class_table_is_refused_at_load()`  
calls 4: `text` (crates/rpc-access-core/tests/conformance.rs:155), `fixture` (crates/rpc-access-core/tests/conformance.rs:149), `DeclaredClassTable::parse` (crates/rpc-access-core/tests/conformance.rs:152) … 1 more  
tests_for 3: `DeclaredClassTable::parse` (crates/rpc-access-core/tests/conformance.rs:152), `fixture` (crates/rpc-access-core/tests/conformance.rs:149), `text` (crates/rpc-access-core/tests/conformance.rs:151)  

### 112. TypeAlias RpcMethodClass
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::RpcMethodClass`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:54  pkg:npm/@causeway/relay  score 0.27  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:54, 1.00)  
`type RpcMethodClass = Addon.MethodClass;`  
How a service's class table classifies a method.  
used_by 3: `the rpc-access interceptor::scripted` (torrentfx/packages-node/causeway-relay/test/rpc-access.test.ts:48), `AddonDecider::classify` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:281), `RpcDecider::classify` (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:72)  

### 113. Function test_pre_publish_never_creates_a_github_release
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::test_pre_publish_never_creates_a_github_release`  .mise/rhai/causeway/node_addon_release_test.rhai:45-51  pkg:npm/causeway-monorepo  score 0.27  via `.mise/rhai/causeway/node_addon_release_test.rhai` -> CONTAINS (.mise/rhai/causeway/node_addon_release_test.rhai:45-51, 1.00)  
`fn test_pre_publish_never_creates_a_github_release()`  
`napi pre-publish` creates a GitHub release by default (@napi-rs/cli 3). The pipeline publishes to npm only: with no token the release calls fail noisily, and with a write-capable one they would make a release nobody asked for. The dry run says the same, so it checks what publish runs.  
calls 1: `pre_publish_lines` (.mise/rhai/causeway/node_addon_release_test.rhai:46)  

### 114. File apps-web/showcase/tests/stories.ts
`file:apps-web/showcase/tests/stories.ts`  apps-web/showcase/tests/stories.ts  pkg:npm/@causeway/showcase  score 0.27  via `npm/typescript` <- IMPORTS (apps-web/showcase/tests/stories.ts:3, 1.00)  
members 9  
importers 1: `apps-web/showcase/tests/visual.spec.ts` (apps-web/showcase/tests/visual.spec.ts:6)  
imports 1: `npm/typescript` (apps-web/showcase/tests/stories.ts:3)  

### 115. TypeAlias RpcDecision
`typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::RpcDecision`  torrentfx/packages-node/causeway-relay/src/rpc-access.ts:51  pkg:npm/@causeway/relay  score 0.27  via `torrentfx/packages-node/causeway-relay/src/rpc-access.ts` -> CONTAINS (torrentfx/packages-node/causeway-relay/src/rpc-access.ts:51, 1.00)  
`type RpcDecision = Addon.Decision;`  
The decision for one call, as the decision core returns it.  

## Left out

28 included nodes have no source window for want of budget.  
- `typescript:torrentfx/packages-node/causeway-relay/src/rpc-access.ts::RpcGrant` TypeAlias 0.27 (over budget, 225 tokens)
- `file:torrentfx/packages-node/causeway-data/test/integration/postgres.ts` File 0.26 (over budget, 254 tokens)
- `file:torrentfx/packages-node/causeway-relay/test/streaming-span.test.ts` File 0.26 (over budget, 234 tokens)
- and 751 more

## Unresolved references in the seeds' files

- `Static` at crates/rpc-access-core/src/lib.rs:108 (no-declaration)
- `is_ascii_digit` at crates/rpc-access-core/src/lib.rs:122 (no-declaration)
- `is_ascii_lowercase` at crates/rpc-access-core/src/lib.rs:122 (no-declaration)
- `Shared` at crates/rpc-access-core/src/lib.rs:125 (no-declaration)
- `debug_tuple` at crates/rpc-access-core/src/lib.rs:146 (no-declaration)
- `field` at crates/rpc-access-core/src/lib.rs:147 (ambiguous)
- `finish` at crates/rpc-access-core/src/lib.rs:148 (ambiguous)
- `debug_struct` at crates/rpc-access-core/src/lib.rs:399 (no-declaration)
- `finish_non_exhaustive` at crates/rpc-access-core/src/lib.rs:400 (no-declaration)
- `Granted` at crates/rpc-access-core/src/lib.rs:473 (no-declaration)

