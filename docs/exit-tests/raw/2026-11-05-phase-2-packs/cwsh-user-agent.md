# Context pack

Task: The daily image-freshness check has printed a bare HTTP 403 for every package since the start of October, and nobody noticed that production images stopped publishing on 2026-10-07. Find out why GitHub refuses the cwsh port of the check and fix it so the check reads package versions again, with a test.

## Seeds

- `task:mise/deploy:images:check` Task 0.90 (text)
- `rhai:.mise/scripts/check-image-freshness.rhai::published_images` Function 0.89 (text)
- `rhai:.mise/rhai/causeway/node_addon_release_test.rhai::test_pre_publish_never_creates_a_github_release` Function 0.84 (text)
- `file:.mise/scripts/check-image-freshness.rhai` File 0.82 (text)
- `rhai:.mise/scripts/check-image-freshness.rhai::parse_utc` Function 0.73 (text)
- `rust:bootstrap/cw-vsa/src/image_modes.rs::image_copy_modes` Function 0.71 (text)
- `rust:bootstrap/cw-vsa/src/checks.rs::Check::ImageCopyModes` Variant 0.71 (text)
- `rhai:.mise/scripts/check-image-freshness.rhai::days_from_civil` Function 0.70 (text)
- `task:mise/rhai:check` Task 0.68 (text)
- `typescript:packages/content-core/test/render.test.ts::GitHub-flavoured Markdown` Test 0.67 (text)
- `file:deploy/overlays/prod/github-dispatch-env-patch.yaml` File 0.40 (identifier `GitHub`)
- `rhai:.mise/rhai/causeway/ci_local_test.rhai::test_github_env_files` Function 0.40 (identifier `GitHub`)
- `rhai:.mise/scripts/check-image-freshness.rhai::format_utc` Function 0.40 (identifier `image-freshness`)
- `rust:apps/causeway-server/src/startup/secrets.rs::GITHUB_DISPATCH_TOKEN` Const 0.40 (identifier `GitHub`)
- `rust:apps/causeway-server/src/startup/secrets.rs::ServerSecrets::github_dispatch_token` Method 0.40 (identifier `GitHub`)

Budget 30000 tokens, used 29982. Seeds 15, candidates 611, included 115, left out 1784.

## Context

### 1. File .mise/scripts/check-image-freshness.rhai
`file:.mise/scripts/check-image-freshness.rhai`  .mise/scripts/check-image-freshness.rhai  pkg:npm/causeway-monorepo  score 4.13  
members 4  
calls 14: `exit` (.mise/scripts/check-image-freshness.rhai:96), `get` (.mise/scripts/check-image-freshness.rhai:99), `exit` (.mise/scripts/check-image-freshness.rhai:104) … 11 more  

### 2. Function published_images
`rhai:.mise/scripts/check-image-freshness.rhai::published_images`  .mise/scripts/check-image-freshness.rhai:25-49  pkg:npm/causeway-monorepo  score 2.75  
`fn published_images()`  
The images the manifest job publishes, in workflow order.  
callers 1: `.mise/scripts/check-image-freshness.rhai` (.mise/scripts/check-image-freshness.rhai:101)  
calls 6: `trimmed` (.mise/scripts/check-image-freshness.rhai:43), `is_match` (.mise/scripts/check-image-freshness.rhai:33), `trimmed` (.mise/scripts/check-image-freshness.rhai:30) … 3 more  
.mise/scripts/check-image-freshness.rhai:25-49
```rhai
fn published_images() {
    let images = [];
    let in_job = false;
    let in_list = false;
    for line in lines(fs::read(global::WORKFLOW)) {
        if line.contains("manifest") && trimmed(line).starts_with("name:") {
            in_job = true;
        }
        if in_job && re::is_match(`^\s+image:\s*$`, line) {
            in_list = true;
            continue;
        }
        if in_list {
            let m = re::captures(`^\s+-\s+([a-z][a-z0-9-]*)\s*$`, line);
            if m != () {
                images.push(m[1]);
                continue;
            }
            if !is_blank(line) && !trimmed(line).starts_with("#") {
                break;
            }
        }
    }
    images
}
```

### 3. Function parse_utc
`rhai:.mise/scripts/check-image-freshness.rhai::parse_utc`  .mise/scripts/check-image-freshness.rhai:63-70  pkg:npm/causeway-monorepo  score 2.50  
`fn parse_utc(stamp)`  
Milliseconds since the epoch of an RFC 3339 UTC stamp (`...Z`), or ().  
callers 1: `.mise/scripts/check-image-freshness.rhai` (.mise/scripts/check-image-freshness.rhai:129)  
calls 2: `captures` (.mise/scripts/check-image-freshness.rhai:64), `days_from_civil` (.mise/scripts/check-image-freshness.rhai:68)  
.mise/scripts/check-image-freshness.rhai:63-70
```rhai
fn parse_utc(stamp) {
    let m = re::captures(`^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?Z$`, stamp);
    if m == () {
        return ();
    }
    let days = days_from_civil(parse_int(m[1]), parse_int(m[2]), parse_int(m[3]));
    (days * 86400 + parse_int(m[4]) * 3600 + parse_int(m[5]) * 60 + parse_int(m[6])) * 1000
}
```

### 4. Function days_from_civil
`rhai:.mise/scripts/check-image-freshness.rhai::days_from_civil`  .mise/scripts/check-image-freshness.rhai:52-60  pkg:npm/causeway-monorepo  score 2.08  
`fn days_from_civil(y, m, d)`  
Days since 1970-01-01 of a civil date (Howard Hinnant's days_from_civil).  
callers 1: `parse_utc` (.mise/scripts/check-image-freshness.rhai:68)  
.mise/scripts/check-image-freshness.rhai:52-60
```rhai
fn days_from_civil(y, m, d) {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
```

### 5. Task deploy:images:check
`task:mise/deploy:images:check`  .mise/toml/deploy.toml:4-7  pkg:npm/causeway-monorepo  score 1.93  
`mise run deploy:images:check`  
Assert every image the build matrix produces also publishes a multi-arch manifest. Offline; no registry access.  
description: Assert every image the build matrix produces also publishes a multi-arch manifest. Offline; no registry access.  
run: .mise/scripts/check-image-publication.rhai  
runs 1: `.mise/scripts/check-image-publication.rhai` (.mise/toml/deploy.toml:6)  
.mise/toml/deploy.toml:4-7
```toml
["deploy:images:check"]
description = "Assert every image the build matrix produces also publishes a multi-arch manifest. Offline; no registry access."
run = ".mise/scripts/check-image-publication.rhai"

```

### 6. Function test_pre_publish_never_creates_a_github_release
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::test_pre_publish_never_creates_a_github_release`  .mise/rhai/causeway/node_addon_release_test.rhai:45-51  pkg:npm/causeway-monorepo  score 1.83  
`fn test_pre_publish_never_creates_a_github_release()`  
`napi pre-publish` creates a GitHub release by default (@napi-rs/cli 3). The pipeline publishes to npm only: with no token the release calls fail noisily, and with a write-capable one they would make a release nobody asked for. The dry run says the same, so it checks what publish runs.  
calls 1: `pre_publish_lines` (.mise/rhai/causeway/node_addon_release_test.rhai:46)  
.mise/rhai/causeway/node_addon_release_test.rhai:45-51
```rhai
fn test_pre_publish_never_creates_a_github_release() {
    let lines = pre_publish_lines();
    assert(lines.len() >= 2, "the package job dry-runs pre-publish and the publish job runs it");
    for line in lines {
        assert(line.contains("--no-gh-release"), `pre-publish without --no-gh-release: ${line}`);
    }
}
```

### 7. Function image_copy_modes
`rust:bootstrap/cw-vsa/src/image_modes.rs::image_copy_modes`  bootstrap/cw-vsa/src/image_modes.rs:63-79  pkg:cargo/cw-vsa  score 1.69  
`pub fn image_copy_modes(ctx: &Ctx<'_>) -> CheckResult`  
Run the check over every Dockerfile in the tree.  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:217)  
calls 3: `findings` (bootstrap/cw-vsa/src/image_modes.rs:71), `read` (bootstrap/cw-vsa/src/image_modes.rs:71), `is_dockerfile` (bootstrap/cw-vsa/src/image_modes.rs:65)  
uses_types 2: `CheckResult` (bootstrap/cw-vsa/src/image_modes.rs:63), `Ctx` (bootstrap/cw-vsa/src/image_modes.rs:63)  
bootstrap/cw-vsa/src/image_modes.rs:63-79
```rust
pub fn image_copy_modes(ctx: &Ctx<'_>) -> CheckResult {
    let mut violations = Vec::new();
    for path in ctx.paths.iter().filter(|p| is_dockerfile(p)) {
        let dir = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
        let is_dir = |src: &str| {
            !src.contains(['*', '?', '$'])
                && (ctx.root.join(src).is_dir() || ctx.root.join(dir).join(src).is_dir())
        };
        for finding in findings(&read(ctx.root, path), &is_dir) {
            violations.push((path.clone(), finding));
        }
    }
    CheckResult {
        violations,
        ..CheckResult::default()
    }
}
```

### 8. Function format_utc
`rhai:.mise/scripts/check-image-freshness.rhai::format_utc`  .mise/scripts/check-image-freshness.rhai:73-87  pkg:npm/causeway-monorepo  score 1.69  
`fn format_utc(ms)`  
`YYYY-MM-DD HH:MM` of an epoch-milliseconds instant.  
callers 1: `.mise/scripts/check-image-freshness.rhai` (.mise/scripts/check-image-freshness.rhai:150)  
.mise/scripts/check-image-freshness.rhai:73-87
```rhai
fn format_utc(ms) {
    let secs = ms / 1000;
    let z = secs / 86400 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    let tod = secs % 86400;
    let two = |n| if n < 10 { `0${n}` } else { `${n}` };
    `${y}-${two.call(m)}-${two.call(d)} ${two.call(tod / 3600)}:${two.call(tod % 3600 / 60)}`
}
```

### 9. Variant Check::ImageCopyModes
`rust:bootstrap/cw-vsa/src/checks.rs::Check::ImageCopyModes`  bootstrap/cw-vsa/src/checks.rs:112  pkg:cargo/cw-vsa  score 1.46  
`ImageCopyModes`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:217), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:164)  

### 10. Task rhai:check
`task:mise/rhai:check`  .mise/toml/bootstrap.toml:21-24  pkg:npm/causeway-monorepo  score 1.33  
`mise run rhai:check`  
Compile every Rhai task script and module under .mise/, resolving imports, so a syntax error or a missing module fails here rather than when its task runs.  
description: Compile every Rhai task script and module under .mise/, resolving imports, so a syntax error or a missing module fails here rather than when its task runs.  
run: cwsh check  
run_by 1: `test_the_tooling_steps_are_ci_yml_s` (.mise/rhai/causeway/ci_local_test.rhai:69)  
runs 1: `main` (.mise/toml/bootstrap.toml:23)  
.mise/toml/bootstrap.toml:21-24
```toml
["rhai:check"]
description = "Compile every Rhai task script and module under .mise/, resolving imports, so a syntax error or a missing module fails here rather than when its task runs."
run = "cwsh check"

```

### 11. Function exit
`rhai:bootstrap/cw-host/src/globals.rs::exit`  bootstrap/cw-host/src/globals.rs:82  pkg:cargo/cw-host  score 1.22  via `.mise/scripts/check-image-freshness.rhai` -> CALLS (.mise/scripts/check-image-freshness.rhai:160, 1.00)  
`engine.register_fn("exit", || exit_with(&Dynamic::UNIT))`  
callers 89: `.mise/scripts/tasks/ci/shell-down.rhai` (.mise/scripts/tasks/ci/shell-down.rhai:69), `.mise/scripts/tasks/ci/web-up.rhai` (.mise/scripts/tasks/ci/web-up.rhai:60), `.mise/scripts/tasks/ops/content-integrity-smoke.rhai` (.mise/scripts/tasks/ops/content-integrity-smoke.rhai:104) … 86 more  
calls 2: `exit_with` (bootstrap/cw-host/src/globals.rs:82), `exit_with` (bootstrap/cw-host/src/globals.rs:83)  
bootstrap/cw-host/src/globals.rs:82-82
```rhai
    engine.register_fn("exit", || exit_with(&Dynamic::UNIT));
```

### 12. Test GitHub-flavoured Markdown
`typescript:packages/content-core/test/render.test.ts::GitHub-flavoured Markdown`  packages/content-core/test/render.test.ts:153-162  pkg:npm/@causeway/content-core  score 1.18  
`describe("GitHub-flavoured Markdown", () => { it("renders tables, strikethrough and footnotes", async () => { const { html } = await renderMarkdown( "| a | b |\n|---|---|\n| 1 | 2 |\n\n~~old~~ text[^1]\n\n[^1]: note", ); expect(html).toContain("<table>"); expect(html).toContain("<del>old</del>"); ex…`  
members 1  
packages/content-core/test/render.test.ts:153-162
```typescript
describe("GitHub-flavoured Markdown", () => {
  it("renders tables, strikethrough and footnotes", async () => {
    const { html } = await renderMarkdown(
      "| a | b |\n|---|---|\n| 1 | 2 |\n\n~~old~~ text[^1]\n\n[^1]: note",
    );
    expect(html).toContain("<table>");
    expect(html).toContain("<del>old</del>");
    expect(html).toContain('id="user-content-fn-1"');
  });
});
```

### 13. Function get
`rhai:bootstrap/cw-host/src/modules/env.rs::get`  bootstrap/cw-host/src/modules/env.rs:48-50  pkg:cargo/cw-host  score 1.07  via `.mise/scripts/check-image-freshness.rhai` -> CALLS (.mise/scripts/check-image-freshness.rhai:99, 1.00)  
`m.set_native_fn("get", |name: &str| { Ok(std::env::var(name).map_or(Dynamic::UNIT, Dynamic::from)) })`  
callers 399: `.mise/scripts/tasks/ops/policy-simulate.rhai` (.mise/scripts/tasks/ops/policy-simulate.rhai:13), `.mise/scripts/tasks/ops/search-smoke.rhai` (.mise/scripts/tasks/ops/search-smoke.rhai:36), `.mise/scripts/tasks/ops/drive-mcp-prompt-smoke.rhai` (.mise/scripts/tasks/ops/drive-mcp-prompt-smoke.rhai:17) … 396 more  
bootstrap/cw-host/src/modules/env.rs:48-50
```rhai
    m.set_native_fn("get", |name: &str| {
        Ok(std::env::var(name).map_or(Dynamic::UNIT, Dynamic::from))
    });
```

### 14. Enum Check
`rust:bootstrap/cw-vsa/src/checks.rs::Check`  bootstrap/cw-vsa/src/checks.rs:91-113  pkg:cargo/cw-vsa  score 1.05  via `Check::ImageCopyModes` <- DEFINES (bootstrap/cw-vsa/src/checks.rs:112, 1.00)  
`pub enum Check`  
A named check and its arguments, parsed when the rules load, so a (doc continues, 176 bytes)  
members 22  
importers 1: `bootstrap/cw-vsa/src/rules.rs` (bootstrap/cw-vsa/src/rules.rs:14)  
used_by 4: `run` (bootstrap/cw-vsa/src/checks.rs:176), `Kind::Named` (bootstrap/cw-vsa/src/rules.rs:93), `Check` (component/platform/coherence/src/finding.rs:58) … 1 more  
uses_types 1: `Check` (bootstrap/cw-vsa/src/checks.rs:91)  
bootstrap/cw-vsa/src/checks.rs:91-113
```rust
pub enum Check {
    GateStaleness(Vec<String>),
    TaskContent(Vec<TaskContent>),
    MskAuthTopology,
    RuntimeEnvResourceCtors,
    RelayRestateStagingOrder,
    ComponentCrossDeps,
    CryptoProviderInstalls,
    CargoFeatureIsolation { krate: String, message: String },
    FacadeSizeBudget,
    ServerRpcDeps,
    ServerServiceImpls,
    ServerOrchestrators,
    IslandCrossRefs,
    CrossSliceSchemaRefs,
    SearchProjectionSchemaRefs,
    CoherenceCheckSchemaRefs,
    DomainModules,
    DomainDependents,
    ConfigurationInputs(ConfigurationInputs),
    PrefixedSettings(PrefixedSettings),
    ImageCopyModes,
}
```

### 15. File bootstrap/cw-vsa/src/image_modes.rs
`file:bootstrap/cw-vsa/src/image_modes.rs`  bootstrap/cw-vsa/src/image_modes.rs  pkg:cargo/cw-vsa  score 1.02  via `image_copy_modes` <- CONTAINS (bootstrap/cw-vsa/src/image_modes.rs:63-79, 1.00)  
members 22  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:5940)  
imports 4: `CheckResult` (bootstrap/cw-vsa/src/image_modes.rs:31), `cargo/regex` (bootstrap/cw-vsa/src/image_modes.rs:29), `Ctx` (bootstrap/cw-vsa/src/image_modes.rs:31) … 1 more  

### 16. File bootstrap/cw-vsa/src/checks.rs
`file:bootstrap/cw-vsa/src/checks.rs`  bootstrap/cw-vsa/src/checks.rs  pkg:cargo/cw-vsa  score 0.99  via `Check` <- CONTAINS (bootstrap/cw-vsa/src/checks.rs:91-113, 1.00)  
members 57  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:318)  
imports 6: `cargo/serde_json` (bootstrap/cw-vsa/src/checks.rs:15), `read` (bootstrap/cw-vsa/src/checks.rs:17), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:17) … 3 more  

### 17. Function trimmed
`rhai:bootstrap/cw-host/src/globals.rs::trimmed`  bootstrap/cw-host/src/globals.rs:95  pkg:cargo/cw-host  score 0.90  via `published_images` -> CALLS (.mise/scripts/check-image-freshness.rhai:43, 1.00)  
`engine.register_fn("trimmed", |s: &str| -> String { s.trim().to_string() })`  
callers 133: `stop_group` (.mise/scripts/tasks/ci/shell-down.rhai:36), `env_file_values` (.mise/rhai/causeway/testkit.rhai:435), `bare_authority` (.mise/scripts/deploy-overlay-conformance.rhai:1576) … 130 more  
bootstrap/cw-host/src/globals.rs:95-95
```rhai
    engine.register_fn("trimmed", |s: &str| -> String { s.trim().to_string() });
```

### 18. File .mise/scripts/check-image-publication.rhai
`file:.mise/scripts/check-image-publication.rhai`  .mise/scripts/check-image-publication.rhai  pkg:npm/causeway-monorepo  score 0.88  via `mise/deploy:images:check` -> RUNS (.mise/toml/deploy.toml:6, 1.00)  
calls 12: `captures` (.mise/scripts/check-image-publication.rhai:48), `trimmed` (.mise/scripts/check-image-publication.rhai:53), `is_blank` (.mise/scripts/check-image-publication.rhai:53) … 9 more  
run_by 1: `mise/deploy:images:check` (.mise/toml/deploy.toml:6)  

### 19. Function captures
`rhai:bootstrap/cw-host/src/modules/re.rs::captures`  bootstrap/cw-host/src/modules/re.rs:71-75  pkg:cargo/cw-host  score 0.87  via `published_images` -> CALLS (.mise/scripts/check-image-freshness.rhai:38, 1.00)  
`m.set_native_fn("captures", |p: &str, s: &str| { with_re(p, |re| { re.captures(s).map_or(Dynamic::UNIT, |c| groups(&c).into()) }) })`  
callers 43: `bootstrap/cwsh/src/self_test.rhai` (bootstrap/cwsh/src/self_test.rhai:42), `test_the_field_manager_is_the_one_k8s_apply_uses` (.mise/rhai/causeway/deploy_test.rhai:397), `.mise/scripts/check-image-publication.rhai` (.mise/scripts/check-image-publication.rhai:48) … 40 more  
calls 2: `groups` (bootstrap/cw-host/src/modules/re.rs:73), `with_re` (bootstrap/cw-host/src/modules/re.rs:72)  
bootstrap/cw-host/src/modules/re.rs:71-75
```rhai
    m.set_native_fn("captures", |p: &str, s: &str| {
        with_re(p, |re| {
            re.captures(s).map_or(Dynamic::UNIT, |c| groups(&c).into())
        })
    });
```

### 20. Function run
`rust:bootstrap/cw-vsa/src/checks.rs::run`  bootstrap/cw-vsa/src/checks.rs:176-229  pkg:cargo/cw-vsa  score 0.86  via `image_copy_modes` <- CALLS (bootstrap/cw-vsa/src/checks.rs:217, 1.00)  
`pub fn run(check: &Check, ctx: &Ctx<'_>) -> CheckResult`  
Run a check.  
callers 12: `run` (bootstrap/cw-vsa/src/scan.rs:456), `a_binary_linking_both_rustls_backends_must_install_the_provider` (bootstrap/cw-vsa/tests/gate.rs:242), `a_top_level_alternation_scans_every_branch` (bootstrap/cw-vsa/tests/gate.rs:137) … 9 more  
calls 28: `burndown` (bootstrap/cw-vsa/src/checks.rs:195), `server_service_impls` (bootstrap/cw-vsa/src/checks.rs:192), `facade_size_budget` (bootstrap/cw-vsa/src/checks.rs:188) … 25 more  
references 23: `Check::ComponentCrossDeps` (bootstrap/cw-vsa/src/checks.rs:183), `Check::ConfigurationInputs` (bootstrap/cw-vsa/src/checks.rs:215), `Check::ServerOrchestrators` (bootstrap/cw-vsa/src/checks.rs:191) … 20 more  
tests 9: `a_ci_job_that_reaches_a_cluster_fails_the_gate` (bootstrap/cw-vsa/tests/gate.rs:534), `a_failed_precondition_exits_2` (bootstrap/cw-vsa/tests/gate.rs:100), `an_exact_burndown_fails_both_ways` (bootstrap/cw-vsa/tests/gate.rs:116) … 6 more  
uses_types 3: `Check` (bootstrap/cw-vsa/src/checks.rs:176), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:176), `Ctx` (bootstrap/cw-vsa/src/checks.rs:176)  
bootstrap/cw-vsa/src/checks.rs:176-229
```rust
pub fn run(check: &Check, ctx: &Ctx<'_>) -> CheckResult {
    match check {
        Check::GateStaleness(paths) => gate_staleness(ctx, paths),
        Check::TaskContent(table) => task_content(ctx, table),
        Check::MskAuthTopology => msk_auth_topology(ctx),
        Check::RuntimeEnvResourceCtors => runtime_env_resource_ctors(ctx),
        Check::RelayRestateStagingOrder => relay_restate_staging_order(ctx),
        Check::ComponentCrossDeps => component_cross_deps(ctx),
        Check::CryptoProviderInstalls => crypto_provider_installs(ctx),
        Check::CargoFeatureIsolation { krate, message } => {
            cargo_feature_isolation(ctx, krate, message)
        }
        Check::FacadeSizeBudget => facade_size_budget(ctx),
        Check::ServerRpcDeps => burndown(server_rpc_deps(ctx).0),
        Check::ServerServiceImpls => burndown(server_service_impls(ctx).0),
        Check::ServerOrchestrators => CheckResult {
            count: server_service_impls(ctx).1,
            ..CheckResult::default()
        },
        Check::IslandCrossRefs => burndown(island_cross_refs(ctx)),
        Check::CrossSliceSchemaRefs => burndown(
            schema_refs(ctx)
                .into_iter()
                .filter(|(seam, ..)| *seam == Seam::Cross)
                .map(|(_, path, own, r)| format!("{path} ({own}) → {r}"))
                .collect(),
        ),
        Check::SearchProjectionSchemaRefs => seam_count(ctx, Seam::Search),
        Check::CoherenceCheckSchemaRefs => seam_count(ctx, Seam::Coherence),
        Check::DomainModules => {
            let module = re(r"^crates/causeway-domain/src/[^/]+\.rs$");
            burndown(
                ctx.paths
                    .iter()
                    .filter(|p| module.is_match(p))
                    .cloned()
                    .collect(),
            )
        }
        Check::ConfigurationInputs(args) => configuration_inputs(ctx, args),
        Check::PrefixedSettings(args) => prefixed_settings(ctx, args),
        Check::ImageCopyModes => crate::image_modes::image_copy_modes(ctx),
        Check::DomainDependents => {
            let dep = re(r"causeway-domain\s*=");
            burndown(
                ctx.paths
                    .iter()
                    .filter(|p| p.ends_with("Cargo.toml") && dep.is_match(&read(ctx.root, p)))
                    .cloned()
                    .collect(),
            )
        }
    }
}
```

### 21. File deploy/overlays/prod/github-dispatch-env-patch.yaml
`file:deploy/overlays/prod/github-dispatch-env-patch.yaml`  deploy/overlays/prod/github-dispatch-env-patch.yaml  pkg:npm/causeway-monorepo  score 0.80  
documents 2: `deploy/README.md` (deploy/README.md:219), `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1125)  

### 22. Function test_github_env_files
`rhai:.mise/rhai/causeway/ci_local_test.rhai::test_github_env_files`  .mise/rhai/causeway/ci_local_test.rhai:109-113  pkg:npm/causeway-monorepo  score 0.80  
`fn test_github_env_files()`  
.mise/rhai/causeway/ci_local_test.rhai:109-113
```rhai
fn test_github_env_files() {
    let text = "A=1\nB<<EOF\nline one\nline two\nEOF\nA=2\nnot an assignment\n";
    assert_eq(cl::parse_env_file(text), #{ A: "2", B: "line one\nline two" });
    assert_eq(cl::parse_env_file(""), #{});
}
```

### 23. Const GITHUB_DISPATCH_TOKEN
`rust:apps/causeway-server/src/startup/secrets.rs::GITHUB_DISPATCH_TOKEN`  apps/causeway-server/src/startup/secrets.rs:23-24  pkg:cargo/causeway-server  score 0.80  
`pub(crate) const GITHUB_DISPATCH_TOKEN: (&str, &str) = ("github-dispatch-token", "CAUSEWAY_GITHUB_DISPATCH_TOKEN");`  
The token the workbench dispatches a terms draft to GitHub with: (doc continues, 100 bytes)  
apps/causeway-server/src/startup/secrets.rs:23-24
```rust
pub(crate) const GITHUB_DISPATCH_TOKEN: (&str, &str) =
    ("github-dispatch-token", "CAUSEWAY_GITHUB_DISPATCH_TOKEN");
```

### 24. Method ServerSecrets::github_dispatch_token
`rust:apps/causeway-server/src/startup/secrets.rs::ServerSecrets::github_dispatch_token`  apps/causeway-server/src/startup/secrets.rs:57-59  pkg:cargo/causeway-server  score 0.80  
`pub(crate) fn github_dispatch_token(&self) -> Option<&Secret<String>>`  
The GitHub dispatch token, when the deployment mounts one.  
callers 3: `tests::a_deployment_that_mounts_nothing_starts_with_every_feature_off` (apps/causeway-server/src/startup/secrets.rs:157), `local_service_ports` (apps/causeway-server/src/composition/local_services.rs:84), `policy_workbench_emit` (apps/causeway-server/src/routes/pages/governance/workbench.rs:113)  
tests 1: `tests::a_deployment_that_mounts_nothing_starts_with_every_feature_off` (apps/causeway-server/src/startup/secrets.rs:157)  
uses_types 1: `Secret` (apps/causeway-server/src/startup/secrets.rs:57)  
apps/causeway-server/src/startup/secrets.rs:57-59
```rust
    pub(crate) fn github_dispatch_token(&self) -> Option<&Secret<String>> {
        self.github_dispatch_token.as_ref()
    }
```

### 25. Function is_blank
`rhai:bootstrap/cw-host/src/globals.rs::is_blank`  bootstrap/cw-host/src/globals.rs:110-112  pkg:cargo/cw-host  score 0.79  via `published_images` -> CALLS (.mise/scripts/check-image-freshness.rhai:43, 1.00)  
`engine.register_fn("is_blank", |s: Dynamic| -> bool { s.is_unit() || (s.is_string() && s.to_string().trim().is_empty()) })`  
callers 164: `.mise/scripts/tasks/dev/run.rhai` (.mise/scripts/tasks/dev/run.rhai:193), `preset_database` (.mise/rhai/causeway/cluster.rhai:291), `preset_database` (.mise/rhai/causeway/cluster.rhai:281) … 161 more  
calls 1: `is_string` (bootstrap/cw-host/src/globals.rs:111)  
bootstrap/cw-host/src/globals.rs:110-112
```rhai
    engine.register_fn("is_blank", |s: Dynamic| -> bool {
        s.is_unit() || (s.is_string() && s.to_string().trim().is_empty())
    });
```

### 26. Package npm/causeway-monorepo
`pkg:npm/causeway-monorepo`  package.json:2  pkg:npm/causeway-monorepo  score 0.78  via `.mise/scripts/check-image-freshness.rhai` <- CONTAINS (package.json:2, 1.00)  
`npm/causeway-monorepo`  
members 1313  
dependencies 8: `npm/@bufbuild/protoc-gen-es` (package.json:9), `npm/@bufbuild/protobuf` (package.json:8), `npm/@commitlint/config-conventional` (package.json:11) … 5 more  
documents 1: `README.md` (README.md:1-98)  

### 27. Struct CheckResult
`rust:bootstrap/cw-vsa/src/scan.rs::CheckResult`  bootstrap/cw-vsa/src/scan.rs:78-82  pkg:cargo/cw-vsa  score 0.72  via `image_copy_modes` -> USES_TYPE (bootstrap/cw-vsa/src/image_modes.rs:63, 1.00)  
`pub struct CheckResult`  
What a named check reports: violations, or a count for a burndown.  
importers 2: `bootstrap/cw-vsa/src/image_modes.rs` (bootstrap/cw-vsa/src/image_modes.rs:31), `bootstrap/cw-vsa/src/checks.rs` (bootstrap/cw-vsa/src/checks.rs:17)  
used_by 16: `violations` (bootstrap/cw-vsa/src/checks.rs:27), `configuration_inputs` (bootstrap/cw-vsa/src/checks.rs:405), `prefixed_settings` (bootstrap/cw-vsa/src/checks.rs:535) … 13 more  
bootstrap/cw-vsa/src/scan.rs:78-82
```rust
pub struct CheckResult {
    pub violations: Vec<(String, String)>,
    pub count: usize,
    pub items: Vec<String>,
}
```

### 28. File .mise/rhai/causeway/node_addon_release_test.rhai
`file:.mise/rhai/causeway/node_addon_release_test.rhai`  .mise/rhai/causeway/node_addon_release_test.rhai  pkg:npm/causeway-monorepo  score 0.72  via `test_pre_publish_never_creates_a_github_release` <- CONTAINS (.mise/rhai/causeway/node_addon_release_test.rhai:45-51, 1.00)  
members 5  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:3061)  

### 29. File .mise/toml/deploy.toml
`file:.mise/toml/deploy.toml`  .mise/toml/deploy.toml  pkg:npm/causeway-monorepo  score 0.72  via `mise/deploy:images:check` <- CONTAINS (.mise/toml/deploy.toml:4-7, 1.00)  
members 44  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1869)  

### 30. Test tests::checks_are_parsed_when_the_rules_load
`rust:bootstrap/cw-vsa/src/checks.rs::tests::checks_are_parsed_when_the_rules_load`  bootstrap/cw-vsa/src/checks.rs:1393-1426  pkg:cargo/cw-vsa  score 0.71  via `Check::parse` <- CALLS (bootstrap/cw-vsa/src/checks.rs:1401, 1.00)  
`fn checks_are_parsed_when_the_rules_load()`  
calls 9: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:1401), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:1419), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:1402) … 6 more  
references 1: `Json::Null` (bootstrap/cw-vsa/src/checks.rs:1395)  
tests_for 1: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:1394)  
bootstrap/cw-vsa/src/checks.rs:1393-1426
```rust
    fn checks_are_parsed_when_the_rules_load() {
        assert!(Check::parse("domain-modules", None).is_ok());
        assert!(Check::parse("domain-modules", Some(&Json::Null)).is_ok());
        assert!(
            Check::parse("domian-modules", None)
                .unwrap_err()
                .contains("no check named")
        );
        assert!(Check::parse("domain-modules", Some(&serde_json::json!([1]))).is_err());
        assert!(Check::parse("cargo-feature-isolation", None).is_err());
        assert!(
            Check::parse(
                "cargo-feature-isolation",
                Some(&serde_json::json!(["parquet"]))
            )
            .is_err()
        );
        assert!(
            Check::parse(
                "cargo-feature-isolation",
                Some(&serde_json::json!(["parquet", "why"]))
            )
            .is_ok()
        );
        assert!(Check::parse("gate-staleness", Some(&serde_json::json!({"path": ["a"]}))).is_err());
        assert!(
            Check::parse(
                "task-content",
                Some(&serde_json::json!([["t", null, [["(", "m"]], []]]))
            )
            .is_err(),
            "a task-content regex is compiled at load"
        );
    }
```

### 31. Struct Ctx
`rust:bootstrap/cw-vsa/src/scan.rs::Ctx`  bootstrap/cw-vsa/src/scan.rs:71-74  pkg:cargo/cw-vsa  score 0.69  via `image_copy_modes` -> USES_TYPE (bootstrap/cw-vsa/src/image_modes.rs:63, 1.00)  
`pub struct Ctx<'a>`  
What a named check reads.  
importers 2: `bootstrap/cw-vsa/src/checks.rs` (bootstrap/cw-vsa/src/checks.rs:17), `bootstrap/cw-vsa/src/image_modes.rs` (bootstrap/cw-vsa/src/image_modes.rs:31)  
used_by 21: `island_cross_refs` (bootstrap/cw-vsa/src/checks.rs:1256), `msk_auth_topology` (bootstrap/cw-vsa/src/checks.rs:706), `schema_refs` (bootstrap/cw-vsa/src/checks.rs:1324) … 18 more  
bootstrap/cw-vsa/src/scan.rs:71-74
```rust
pub struct Ctx<'a> {
    pub root: &'a Path,
    pub paths: &'a [String],
}
```

### 32. File packages/content-core/test/render.test.ts
`file:packages/content-core/test/render.test.ts`  packages/content-core/test/render.test.ts  pkg:npm/@causeway/content-core  score 0.67  via `GitHub-flavoured Markdown` <- CONTAINS (packages/content-core/test/render.test.ts:153-162, 1.00)  
members 5  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1491)  
imports 7: `packages/content-core/src/render.ts` (packages/content-core/test/render.test.ts:6), `npm/vitest` (packages/content-core/test/render.test.ts:5), `npm/rehype-sanitize` (packages/content-core/test/render.test.ts:2) … 4 more  

### 33. Function read
`rhai:bootstrap/cw-host/src/modules/fs.rs::read`  bootstrap/cw-host/src/modules/fs.rs:213-215  pkg:cargo/cw-host  score 0.59  via `published_images` -> CALLS (.mise/scripts/check-image-freshness.rhai:29, 1.00)  
`m.set_native_fn("read", |p: &str| { io("reading", p, std::fs::read_to_string(p)) })`  
callers 197: `test_every_allowlisted_file_still_needs_its_entry` (.mise/rhai/causeway/tmpdirs_test.rhai:173), `test_the_task_runs_each_jobs_steps_as_a_runner_would` (.mise/rhai/causeway/ci_local_test.rhai:330), `.mise/scripts/check-task-shells.rhai` (.mise/scripts/check-task-shells.rhai:126) … 194 more  
calls 1: `io` (bootstrap/cw-host/src/modules/fs.rs:214)  
bootstrap/cw-host/src/modules/fs.rs:213-215
```rhai
    m.set_native_fn("read", |p: &str| {
        io("reading", p, std::fs::read_to_string(p))
    });
```

### 34. Function is_match
`rhai:bootstrap/cw-host/src/modules/re.rs::is_match`  bootstrap/cw-host/src/modules/re.rs:51-53  pkg:cargo/cw-host  score 0.59  via `published_images` -> CALLS (.mise/scripts/check-image-freshness.rhai:33, 1.00)  
`m.set_native_fn("is_match", |p: &str, s: &str| { with_re(p, |re| re.is_match(s)) })`  
callers 70: `check_workload_identity` (.mise/scripts/deploy-overlay-conformance.rhai:1227), `test_no_workflow_step_removes_containers_itself` (.mise/rhai/causeway/testkit_test.rhai:219), `.mise/scripts/causeway-controller-smoke.rhai` (.mise/scripts/causeway-controller-smoke.rhai:150) … 67 more  
calls 1: `with_re` (bootstrap/cw-host/src/modules/re.rs:52)  
bootstrap/cw-host/src/modules/re.rs:51-53
```rhai
    m.set_native_fn("is_match", |p: &str, s: &str| {
        with_re(p, |re| re.is_match(s))
    });
```

### 35. File .mise/toml/bootstrap.toml
`file:.mise/toml/bootstrap.toml`  .mise/toml/bootstrap.toml  pkg:npm/causeway-monorepo  score 0.54  via `mise/rhai:check` <- CONTAINS (.mise/toml/bootstrap.toml:21-24, 1.00)  
members 4  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:3528)  

### 36. Method Check::parse
`rust:bootstrap/cw-vsa/src/checks.rs::Check::parse`  bootstrap/cw-vsa/src/checks.rs:122-171  pkg:cargo/cw-vsa  score 0.53  via `Check::ImageCopyModes` <- REFERENCES (bootstrap/cw-vsa/src/checks.rs:164, 1.00)  
`pub fn parse(name: &str, args: Option<&Json>) -> Result<Self, String>`  
The check called `name`, with `args` (absent or `()` when it takes (doc continues, 201 bytes)  
callers 10: `tests::checks_are_parsed_when_the_rules_load` (bootstrap/cw-vsa/src/checks.rs:1401), `tests::checks_are_parsed_when_the_rules_load` (bootstrap/cw-vsa/src/checks.rs:1419), `tests::checks_are_parsed_when_the_rules_load` (bootstrap/cw-vsa/src/checks.rs:1402) … 7 more  
calls 8: `prefixed_settings_args` (bootstrap/cw-vsa/src/checks.rs:168), `strs` (bootstrap/cw-vsa/src/checks.rs:140), `strs` (bootstrap/cw-vsa/src/checks.rs:136) … 5 more  
references 16: `Check::CrossSliceSchemaRefs` (bootstrap/cw-vsa/src/checks.rs:159), `Check::CoherenceCheckSchemaRefs` (bootstrap/cw-vsa/src/checks.rs:161), `Check::ServerServiceImpls` (bootstrap/cw-vsa/src/checks.rs:156) … 13 more  
tests 1: `tests::checks_are_parsed_when_the_rules_load` (bootstrap/cw-vsa/src/checks.rs:1394)  
uses_types 2: `Json` (bootstrap/cw-vsa/src/checks.rs:122), `Json` (bootstrap/cw-vsa/src/checks.rs:122)  
bootstrap/cw-vsa/src/checks.rs:122-171
```rust
    pub fn parse(name: &str, args: Option<&Json>) -> Result<Self, String> {
        let args = args.filter(|a| !a.is_null());
        let needs = || args.ok_or_else(|| format!("check {name:?} needs `args`"));
        let bare = |check: Self| match args {
            None => Ok(check),
            Some(_) => Err(format!("check {name:?} takes no `args`")),
        };
        match name {
            "gate-staleness" => {
                let a = needs()?
                    .as_object()
                    .filter(|m| m.len() == 1)
                    .and_then(|m| m.get("paths"))
                    .ok_or("gate-staleness takes #{ paths: [...] }")?;
                Ok(Self::GateStaleness(strs(a)?))
            }
            "task-content" => Ok(Self::TaskContent(task_content_table(needs()?)?)),
            "cargo-feature-isolation" => {
                let a = strs(needs()?)?;
                let [krate, message] = a.as_slice() else {
                    return Err("cargo-feature-isolation takes [crate, message]".into());
                };
                Ok(Self::CargoFeatureIsolation {
                    krate: krate.clone(),
                    message: message.clone(),
                })
            }
            "msk-auth-topology" => bare(Self::MskAuthTopology),
            "runtime-env-resource-ctors" => bare(Self::RuntimeEnvResourceCtors),
            "relay-restate-staging-order" => bare(Self::RelayRestateStagingOrder),
            "component-cross-deps" => bare(Self::ComponentCrossDeps),
            "crypto-provider-installs" => bare(Self::CryptoProviderInstalls),
            "facade-size-budget" => bare(Self::FacadeSizeBudget),
            "server-rpc-deps" => bare(Self::ServerRpcDeps),
            "server-service-impls" => bare(Self::ServerServiceImpls),
            "server-orchestrators" => bare(Self::ServerOrchestrators),
            "island-cross-refs" => bare(Self::IslandCrossRefs),
            "cross-slice-schema-refs" => bare(Self::CrossSliceSchemaRefs),
            "search-projection-schema-refs" => bare(Self::SearchProjectionSchemaRefs),
            "coherence-check-schema-refs" => bare(Self::CoherenceCheckSchemaRefs),
            "domain-modules" => bare(Self::DomainModules),
            "domain-dependents" => bare(Self::DomainDependents),
            "image-copy-modes" => bare(Self::ImageCopyModes),
            "configuration-inputs" => Ok(Self::ConfigurationInputs(configuration_inputs_args(
                needs()?,
            )?)),
            "prefixed-settings" => Ok(Self::PrefixedSettings(prefixed_settings_args(needs()?)?)),
            other => Err(format!("no check named {other:?}")),
        }
    }
```

### 37. Function register
`rust:bootstrap/cw-host/src/globals.rs::register`  bootstrap/cw-host/src/globals.rs:81-153  pkg:cargo/cw-host  score 0.50  via `is_blank` <- DEFINES (bootstrap/cw-host/src/globals.rs:110-112, 1.00)  
`pub fn register(engine: &mut Engine)`  
members 13  
callers 1: `engine` (bootstrap/cw-host/src/lib.rs:95)  
bootstrap/cw-host/src/globals.rs:81-153
```rust
pub fn register(engine: &mut Engine) {
    engine.register_fn("exit", || exit_with(&Dynamic::UNIT));
    engine.register_fn("exit", |code: Dynamic| exit_with(&code));
    engine.register_fn("eprint", |s: Dynamic| {
        let _ = writeln!(std::io::stderr().lock(), "{s}");
    });
    engine.register_fn("out", |s: Dynamic| write_stdout(&s.to_string(), false));
    engine.register_fn("join", |items: Array, sep: &str| -> String {
        items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(sep)
    });
    engine.register_fn("trimmed", |s: &str| -> String { s.trim().to_string() });
    engine.register_fn("replaced", |s: &str, from: &str, to: &str| -> String {
        s.replace(from, to)
    });
    engine.register_fn("lines", |s: &str| -> Array {
        s.lines().map(|l| Dynamic::from(l.to_string())).collect()
    });
    engine.register_fn("repeat", |s: &str, n: rhai::INT| -> String {
        s.repeat(usize::try_from(n).unwrap_or(0))
    });
    engine.register_fn("words", |s: &str| -> Array {
        s.split_whitespace()
            .map(|w| Dynamic::from(w.to_string()))
            .collect()
    });
    engine.register_fn("is_blank", |s: Dynamic| -> bool {
        s.is_unit() || (s.is_string() && s.to_string().trim().is_empty())
    });
    engine.register_fn("assert", |cond: bool| -> Res<()> {
        if cond {
            Ok(())
        } else {
            err("assertion failed")
        }
    });
    engine.register_fn("assert", |cond: bool, msg: ImmutableString| -> Res<()> {
        if cond {
            Ok(())
        } else {
            err(format!("assertion failed: {msg}"))
        }
    });
    engine.register_fn("assert_eq", |a: Dynamic, b: Dynamic| -> Res<()> {
        if same(&a, &b) {
            Ok(())
        } else {
            err(format!(
                "assertion failed: left != right\n  left:  {}\n  right: {}",
                debug(&a),
                debug(&b)
            ))
        }
    });
    engine.register_fn(
        "assert_eq",
        |a: Dynamic, b: Dynamic, msg: ImmutableString| -> Res<()> {
            if same(&a, &b) {
                Ok(())
            } else {
                err(format!(
                    "assertion failed: {msg}\n  left:  {}\n  right: {}",
                    debug(&a),
                    debug(&b)
                ))
            }
        },
    );
    engine.register_fn("to_debug", |x: Dynamic| -> String { debug(&x) });
}
```

### 38. Function burndown
`rust:bootstrap/cw-vsa/src/checks.rs::burndown`  bootstrap/cw-vsa/src/checks.rs:34-40  pkg:cargo/cw-vsa  score 0.49  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:34, 1.00)  
`fn burndown(items: Vec<String>) -> CheckResult`  
callers 6: `run` (bootstrap/cw-vsa/src/checks.rs:195), `run` (bootstrap/cw-vsa/src/checks.rs:189), `run` (bootstrap/cw-vsa/src/checks.rs:196) … 3 more  
uses_types 1: `CheckResult` (bootstrap/cw-vsa/src/checks.rs:34)  
bootstrap/cw-vsa/src/checks.rs:34-40
```rust
fn burndown(items: Vec<String>) -> CheckResult {
    CheckResult {
        count: items.len(),
        items,
        ..CheckResult::default()
    }
}
```

### 39. Function pre_publish_lines
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::pre_publish_lines`  .mise/rhai/causeway/node_addon_release_test.rhai:13-29  pkg:npm/causeway-monorepo  score 0.47  via `test_pre_publish_never_creates_a_github_release` -> CALLS (.mise/rhai/causeway/node_addon_release_test.rhai:46, 1.00)  
`fn pre_publish_lines()`  
Every `napi pre-publish` line in the workflow's steps.  
callers 1: `test_pre_publish_never_creates_a_github_release` (.mise/rhai/causeway/node_addon_release_test.rhai:46)  
calls 1: `read` (.mise/rhai/causeway/node_addon_release_test.rhai:15)  
.mise/rhai/causeway/node_addon_release_test.rhai:13-29
```rhai
fn pre_publish_lines() {
    let lines = [];
    for job in yaml::parse(read(global::WORKFLOW)).jobs.values() {
        for step in job.steps {
            if "run" in step {
                for line in step.run.split("\n") {
                    if line.contains("napi pre-publish") {
                        let command = line;
                        command.trim(); // trims in place
                        lines.push(command);
                    }
                }
            }
        }
    }
    lines
}
```

### 40. Function get
`rhai:bootstrap/cw-host/src/modules/http.rs::get`  bootstrap/cw-host/src/modules/http.rs:222  pkg:cargo/cw-host  score 0.47  via `.mise/scripts/check-image-freshness.rhai` -> CALLS (.mise/scripts/check-image-freshness.rhai:115, 1.00)  
`m.set_native_fn("get", |url: &str| with(url, "GET", &Map::new()))`  
callers 16: `answers_http` (.mise/rhai/causeway/personas.rhai:660), `await_host` (.mise/scripts/tasks/dev/run.rhai:521), `principals` (.mise/rhai/causeway/personas.rhai:425) … 13 more  
calls 2: `with` (bootstrap/cw-host/src/modules/http.rs:222), `with` (bootstrap/cw-host/src/modules/http.rs:223)  
bootstrap/cw-host/src/modules/http.rs:222-222
```rhai
    m.set_native_fn("get", |url: &str| with(url, "GET", &Map::new()));
```

### 41. Function now_ms
`rhai:bootstrap/cw-host/src/modules/time.rs::now_ms`  bootstrap/cw-host/src/modules/time.rs:46  pkg:cargo/cw-host  score 0.47  via `.mise/scripts/check-image-freshness.rhai` -> CALLS (.mise/scripts/check-image-freshness.rhai:107, 1.00)  
`m.set_native_fn("now_ms", || Ok(now_ms()))`  
callers 38: `wait_for_external_drain` (.mise/scripts/metadata-perf-matrix.rhai:161), `.mise/scripts/causeway-controller-api-compat-smoke.rhai` (.mise/scripts/causeway-controller-api-compat-smoke.rhai:14), `.mise/scripts/deploy-converge-smoke.rhai` (.mise/scripts/deploy-converge-smoke.rhai:29) … 35 more  
calls 1: `now_ms` (bootstrap/cw-host/src/modules/time.rs:46)  
bootstrap/cw-host/src/modules/time.rs:46-46
```rhai
    m.set_native_fn("now_ms", || Ok(now_ms()));
```

### 42. Function module
`rust:bootstrap/cw-host/src/modules/re.rs::module`  bootstrap/cw-host/src/modules/re.rs:49-98  pkg:cargo/cw-host  score 0.46  via `captures` <- DEFINES (bootstrap/cw-host/src/modules/re.rs:71-75, 1.00)  
`pub fn module() -> Module`  
members 10  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:27)  
bootstrap/cw-host/src/modules/re.rs:49-98
```rust
pub fn module() -> Module {
    let mut m = Module::new();
    m.set_native_fn("is_match", |p: &str, s: &str| {
        with_re(p, |re| re.is_match(s))
    });
    m.set_native_fn("full_match", |p: &str, s: &str| {
        let anchored = format!("^(?:{p})$");
        with_re(&anchored, |re| re.is_match(s))
    });
    m.set_native_fn("find", |p: &str, s: &str| {
        with_re(p, |re| {
            re.find(s)
                .map_or(Dynamic::UNIT, |m| m.as_str().to_string().into())
        })
    });
    m.set_native_fn("find_all", |p: &str, s: &str| {
        with_re(p, |re| {
            re.find_iter(s)
                .map(|m| Dynamic::from(m.as_str().to_string()))
                .collect::<Array>()
        })
    });
    m.set_native_fn("captures", |p: &str, s: &str| {
        with_re(p, |re| {
            re.captures(s).map_or(Dynamic::UNIT, |c| groups(&c).into())
        })
    });
    m.set_native_fn("captures_all", |p: &str, s: &str| {
        with_re(p, |re| {
            re.captures_iter(s)
                .map(|c| Dynamic::from(groups(&c)))
                .collect::<Array>()
        })
    });
    m.set_native_fn("replace", |p: &str, s: &str, with: &str| {
        with_re(p, |re| re.replace(s, with).into_owned())
    });
    m.set_native_fn("replace_all", |p: &str, s: &str, with: &str| {
        with_re(p, |re| re.replace_all(s, with).into_owned())
    });
    m.set_native_fn("split", |p: &str, s: &str| {
        with_re(p, |re| {
            re.split(s)
                .map(|part| Dynamic::from(part.to_string()))
                .collect::<Array>()
        })
    });
    m.set_native_fn("escape", |s: &str| Ok(regex::escape(s)));
    m
}
```

### 43. Function read
`rust:bootstrap/cw-vsa/src/scan.rs::read`  bootstrap/cw-vsa/src/scan.rs:112-122  pkg:cargo/cw-vsa  score 0.43  via `image_copy_modes` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:71, 1.00)  
`pub fn read(root: &Path, path: &str) -> String`  
A file's text, lossily decoded (BOM dropped), or `""` when unreadable.  
callers 19: `schema_refs` (bootstrap/cw-vsa/src/checks.rs:1339), `image_copy_modes` (bootstrap/cw-vsa/src/image_modes.rs:71), `island_cross_refs` (bootstrap/cw-vsa/src/checks.rs:1264) … 16 more  
importers 2: `bootstrap/cw-vsa/src/checks.rs` (bootstrap/cw-vsa/src/checks.rs:17), `bootstrap/cw-vsa/src/image_modes.rs` (bootstrap/cw-vsa/src/image_modes.rs:31)  
bootstrap/cw-vsa/src/scan.rs:112-122
```rust
pub fn read(root: &Path, path: &str) -> String {
    std::fs::read(root.join(path)).map_or_else(
        |_| String::new(),
        |bytes| {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            text.strip_prefix('\u{feff}')
                .map(str::to_string)
                .unwrap_or(text)
        },
    )
}
```

### 44. Test GitHub-flavoured Markdown::renders tables, strikethrough and footnotes
`typescript:packages/content-core/test/render.test.ts::GitHub-flavoured Markdown::renders tables, strikethrough and footnotes`  packages/content-core/test/render.test.ts:154-161  pkg:npm/@causeway/content-core  score 0.43  via `GitHub-flavoured Markdown` -> DEFINES (packages/content-core/test/render.test.ts:154-161, 1.00)  
`it("renders tables, strikethrough and footnotes", async () => { const { html } = await renderMarkdown( "| a | b |\n|---|---|\n| 1 | 2 |\n\n~~old~~ text[^1]\n\n[^1]: note", ); expect(html).toContain("<table>"); expect(html).toContain("<del>old</del>"); expect(html).toContain('id="user-content-fn-1"')…`  
calls 1: `renderMarkdown` (packages/content-core/test/render.test.ts:155)  
tests_for 1: `renderMarkdown` (packages/content-core/test/render.test.ts:155)  
packages/content-core/test/render.test.ts:154-161
```typescript
  it("renders tables, strikethrough and footnotes", async () => {
    const { html } = await renderMarkdown(
      "| a | b |\n|---|---|\n| 1 | 2 |\n\n~~old~~ text[^1]\n\n[^1]: note",
    );
    expect(html).toContain("<table>");
    expect(html).toContain("<del>old</del>");
    expect(html).toContain('id="user-content-fn-1"');
  });
```

### 45. Function findings
`rust:bootstrap/cw-vsa/src/image_modes.rs::findings`  bootstrap/cw-vsa/src/image_modes.rs:431-521  pkg:cargo/cw-vsa  score 0.38  via `image_copy_modes` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:71, 1.00)  
`pub fn findings(text: &str, is_dir: &dyn Fn(&str) -> bool) -> Vec<String>`  
Every unfixed context copy in a Dockerfile's non-root stages. (doc continues, 122 bytes)  
callers 2: `image_copy_modes` (bootstrap/cw-vsa/src/image_modes.rs:71), `tests::check` (bootstrap/cw-vsa/src/image_modes.rs:540)  
calls 8: `context_copy` (bootstrap/cw-vsa/src/image_modes.rs:479), `is_root` (bootstrap/cw-vsa/src/image_modes.rs:476), `same_user` (bootstrap/cw-vsa/src/image_modes.rs:496) … 5 more  
uses_types 1: `Copy` (bootstrap/cw-vsa/src/image_modes.rs:463)  
bootstrap/cw-vsa/src/image_modes.rs:431-510, cut
```rust
pub fn findings(text: &str, is_dir: &dyn Fn(&str) -> bool) -> Vec<String> {
    let (syntax, instructions) = parse(text);
    let symbolic_ok = symbolic_chmod_supported(syntax.as_deref());
    let mut out = Vec::new();
    // Stage name → final user, for `FROM <stage>`.
    let mut finished: Vec<(String, Option<String>, String)> = Vec::new();
    let mut idx = 0;
    while idx < instructions.len() {
        // One stage: from a FROM (or the file head) to the next FROM.
        let from = &instructions[idx];
        let (mut user, mut workdir, name) = if from.keyword == "FROM" {
            let args: Vec<&str> = from
                .rest
                .split_whitespace()
                .filter(|a| !a.starts_with("--"))
                .collect();
            let base = args.first().copied().unwrap_or_default();
            let name = match args.as_slice() {
                [_, kw, name, ..] if kw.eq_ignore_ascii_case("as") => name.to_string(),
                _ => String::new(),
            };
            let parent = finished
                .iter()
                .find(|(n, ..)| !n.is_empty() && n.eq_ignore_ascii_case(base));
            idx += 1;
            match parent {
                Some((_, u, w)) => (u.clone(), w.clone(), name),
                None => (None, "/".to_string(), name),
            }
        } else {
            (None, "/".to_string(), String::new())
        };
        let mut copies: Vec<Copy> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut runs: Vec<(usize, String, String, bool)> = Vec::new();
        while idx < instructions.len() && instructions[idx].keyword != "FROM" {
            let ins = &instructions[idx];
            idx += 1;
            match ins.keyword.as_str() {
                "USER" => user = Some(ins.rest.clone()),
                "WORKDIR" => workdir = absolute(&workdir, &ins.rest),
                "RUN" => runs.push((
                    copies.len(),
                    ins.rest.clone(),
                    workdir.clone(),
                    is_root(user.as_deref()),
                )),
                "COPY" | "ADD" => {
                    if let Some((copy, error)) = context_copy(ins, &workdir, symbolic_ok, is_dir) {
                        errors.extend(error);
                        copies.push(copy);
                    }
                }
                _ => {}
            }
        }
        if !is_root(user.as_deref()) {
            let final_user = user.clone().unwrap_or_default();
            for (i, copy) in copies.iter_mut().enumerate() {
                if copy.fixed {
                    continue;
                }
                let chowned = copy
                    .chown
                    .as_deref()
                    .is_some_and(|c| same_user(c, &final_user));
                let by_run = runs
                    .iter()
                    .filter(|(after, _, _, root)| *after > i && *root)
                    .any(|(_, run, wd, _)| run_fixes(copy, run, wd, &final_user));
                if !(chowned || by_run) {
                    out.push(format!(
                        "line {}: {}: the image runs as {final_user} and this copy keeps the \
                         build host's modes (a 077 umask makes them 0600); add --chmod=0755 for \
                         a binary, or RUN chmod -R u=rwX,go=rX <path> before USER for data",
                        copy.line, copy.what
                    ));
                }
            }
            out.append(&mut errors);
```

### 46. Function is_dockerfile
`rust:bootstrap/cw-vsa/src/image_modes.rs::is_dockerfile`  bootstrap/cw-vsa/src/image_modes.rs:81-90  pkg:cargo/cw-vsa  score 0.38  via `image_copy_modes` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:65, 1.00)  
`fn is_dockerfile(path: &str) -> bool`  
callers 5: `tests::dockerfile_names` (bootstrap/cw-vsa/src/image_modes.rs:675), `tests::dockerfile_names` (bootstrap/cw-vsa/src/image_modes.rs:677), `tests::dockerfile_names` (bootstrap/cw-vsa/src/image_modes.rs:678) … 2 more  
tests 1: `tests::dockerfile_names` (bootstrap/cw-vsa/src/image_modes.rs:675)  
bootstrap/cw-vsa/src/image_modes.rs:81-90
```rust
fn is_dockerfile(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name == "Dockerfile"
        || name == "Containerfile"
        || name.starts_with("Dockerfile.")
        || name.ends_with(".Dockerfile")
}
```

### 47. Impl Check#impl
`rust:bootstrap/cw-vsa/src/checks.rs::Check#impl`  bootstrap/cw-vsa/src/checks.rs:115-172  pkg:cargo/cw-vsa  score 0.37  via `Check::parse` <- DEFINES (bootstrap/cw-vsa/src/checks.rs:122-171, 1.00)  
`impl Check`  
members 1  
uses_types 1: `Check` (bootstrap/cw-vsa/src/checks.rs:115-172)  

### 48. File bootstrap/cw-vsa/src/scan.rs
`file:bootstrap/cw-vsa/src/scan.rs`  bootstrap/cw-vsa/src/scan.rs  pkg:cargo/cw-vsa  score 0.36  via `CheckResult` <- CONTAINS (bootstrap/cw-vsa/src/scan.rs:78-82, 1.00)  
members 26  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:6046)  
imports 3: `Kind` (bootstrap/cw-vsa/src/scan.rs:16), `GateRule` (bootstrap/cw-vsa/src/scan.rs:16), `Severity` (bootstrap/cw-vsa/src/scan.rs:16)  

### 49. Function relay_restate_staging_order
`rust:bootstrap/cw-vsa/src/checks.rs::relay_restate_staging_order`  bootstrap/cw-vsa/src/checks.rs:794-815  pkg:cargo/cw-vsa  score 0.36  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:794, 1.00)  
`fn relay_restate_staging_order(ctx: &Ctx<'_>) -> CheckResult`  
The relay-restate image task regenerates the runtime artifact before the (doc continues, 238 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:182)  
calls 6: `violations` (bootstrap/cw-vsa/src/checks.rs:797), `violation` (bootstrap/cw-vsa/src/checks.rs:808), `re` (bootstrap/cw-vsa/src/checks.rs:795) … 3 more  
uses_types 2: `CheckResult` (bootstrap/cw-vsa/src/checks.rs:794), `Ctx` (bootstrap/cw-vsa/src/checks.rs:794)  

### 50. File .mise/rhai/causeway/ci_local_test.rhai
`file:.mise/rhai/causeway/ci_local_test.rhai`  .mise/rhai/causeway/ci_local_test.rhai  pkg:npm/causeway-monorepo  score 0.36  via `test_github_env_files` <- CONTAINS (.mise/rhai/causeway/ci_local_test.rhai:109-113, 1.00)  
members 18  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:142)  

### 51. Function module
`rust:bootstrap/cw-host/src/modules/fs.rs::module`  bootstrap/cw-host/src/modules/fs.rs:206-297  pkg:cargo/cw-host  score 0.36  via `read` <- DEFINES (bootstrap/cw-host/src/modules/fs.rs:213-215, 1.00)  
`pub fn module() -> Module`  
members 34  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:23)  

### 52. Function prefixed_settings
`rust:bootstrap/cw-vsa/src/checks.rs::prefixed_settings`  bootstrap/cw-vsa/src/checks.rs:535-568  pkg:cargo/cw-vsa  score 0.35  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:535, 1.00)  
`fn prefixed_settings(ctx: &Ctx<'_>, args: &PrefixedSettings) -> CheckResult`  
No Node or Python library or service reads a variable under a bare name (doc continues, 894 bytes)  
callers 2: `run` (bootstrap/cw-vsa/src/checks.rs:216), `tests::the_prefix_search_finds_bare_names_in_node_and_python` (bootstrap/cw-vsa/src/checks.rs:1542)  
calls 5: `violation` (bootstrap/cw-vsa/src/checks.rs:560), `prefixed_settings_findings` (bootstrap/cw-vsa/src/checks.rs:559), `violations` (bootstrap/cw-vsa/src/checks.rs:567) … 2 more  
tests 1: `tests::the_prefix_search_finds_bare_names_in_node_and_python` (bootstrap/cw-vsa/src/checks.rs:1542)  
uses_types 3: `CheckResult` (bootstrap/cw-vsa/src/checks.rs:535), `PrefixedSettings` (bootstrap/cw-vsa/src/checks.rs:535), `Ctx` (bootstrap/cw-vsa/src/checks.rs:535)  

### 53. Function test_the_tooling_steps_are_ci_yml_s
`rhai:.mise/rhai/causeway/ci_local_test.rhai::test_the_tooling_steps_are_ci_yml_s`  .mise/rhai/causeway/ci_local_test.rhai:66-77  pkg:npm/causeway-monorepo  score 0.34  via `mise/rhai:check` <- RUNS (.mise/rhai/causeway/ci_local_test.rhai:69, 1.00)  
`fn test_the_tooling_steps_are_ci_yml_s()`  
calls 3: `ci` (.mise/rhai/causeway/ci_local_test.rhai:67), `ci` (.mise/rhai/causeway/ci_local_test.rhai:72), `ci` (.mise/rhai/causeway/ci_local_test.rhai:73)  
runs 2: `mise/rhai:test` (.mise/rhai/causeway/ci_local_test.rhai:69), `mise/rhai:check` (.mise/rhai/causeway/ci_local_test.rhai:69)  

### 54. Function run
`rust:bootstrap/cw-vsa/src/scan.rs::run`  bootstrap/cw-vsa/src/scan.rs:391-494  pkg:cargo/cw-vsa  score 0.34  via `run` <- CALLS (bootstrap/cw-vsa/src/scan.rs:456, 1.00)  
`pub fn run(root: &Path, rules: &[GateRule]) -> Outcome`  
Run `rules` against the tree at `root`.  
callers 11: `a_path_dependency_outside_the_repository_is_reported` (bootstrap/cw-vsa/tests/gate.rs:284), `an_exact_burndown_fails_both_ways` (bootstrap/cw-vsa/tests/gate.rs:116), `every_rule_kind_reports_like_the_clojure_gate` (bootstrap/cw-vsa/tests/gate.rs:46) … 8 more  
calls 9: `scan_multiline` (bootstrap/cw-vsa/src/scan.rs:449), `run` (bootstrap/cw-vsa/src/scan.rs:456), `path_rule` (bootstrap/cw-vsa/src/scan.rs:454) … 6 more  
importers 1: `bootstrap/cw-vsa/src/lib.rs` (bootstrap/cw-vsa/src/lib.rs:20)  
references 10: `Severity::Burndown` (bootstrap/cw-vsa/src/scan.rs:419), `Kind::Require` (bootstrap/cw-vsa/src/scan.rs:450), `PathRule::Forbid` (bootstrap/cw-vsa/src/scan.rs:454) … 7 more  
tests 9: `an_exact_burndown_fails_both_ways` (bootstrap/cw-vsa/tests/gate.rs:116), `a_failed_precondition_exits_2` (bootstrap/cw-vsa/tests/gate.rs:100), `a_burndown_check_that_cannot_run_is_a_failure` (bootstrap/cw-vsa/tests/gate.rs:167) … 6 more  
uses_types 6: `GateRule` (bootstrap/cw-vsa/src/scan.rs:391), `Violation` (bootstrap/cw-vsa/src/scan.rs:471), `Outcome` (bootstrap/cw-vsa/src/scan.rs:391) … 3 more  

### 55. Struct Check
`rust:component/platform/coherence/src/finding.rs::Check`  component/platform/coherence/src/finding.rs:58-71  pkg:cargo/coherence-component  score 0.33  via `Check` -> USES_TYPE (bootstrap/cw-vsa/src/checks.rs:91, 1.00)  
`pub struct Check`  
One coherence question, with the two queries that answer it.  
importers 3: `component/platform/coherence/src/run.rs` (component/platform/coherence/src/run.rs:7), `component/platform/coherence/src/lib.rs` (component/platform/coherence/src/lib.rs:28), `component/platform/coherence/src/checks.rs` (component/platform/coherence/src/checks.rs:12)  
used_by 4: `CHECKS` (component/platform/coherence/src/checks.rs:15), `Check` (bootstrap/cw-vsa/src/checks.rs:91), `run_check` (component/platform/coherence/src/run.rs:162) … 1 more  
uses_types 2: `Severity` (component/platform/coherence/src/finding.rs:66), `Check` (component/platform/coherence/src/finding.rs:58)  

### 56. Function with_re
`rust:bootstrap/cw-host/src/modules/re.rs::with_re`  bootstrap/cw-host/src/modules/re.rs:31-41  pkg:cargo/cw-host  score 0.32  via `captures` -> CALLS (bootstrap/cw-host/src/modules/re.rs:72, 1.00)  
`fn with_re<T>(pattern: &str, f: impl FnOnce(&Regex) -> T) -> Res<T>`  
callers 9: `replace` (bootstrap/cw-host/src/modules/re.rs:84), `replace_all` (bootstrap/cw-host/src/modules/re.rs:87), `split` (bootstrap/cw-host/src/modules/re.rs:90) … 6 more  
calls 2: `err` (bootstrap/cw-host/src/modules/re.rs:36), `with` (bootstrap/cw-host/src/modules/re.rs:32)  
uses_types 1: `Res` (bootstrap/cw-host/src/modules/re.rs:31)  

### 57. Document Design log
`doc:docs/DESIGN-LOG.md`  docs/DESIGN-LOG.md:1-6121  pkg:npm/causeway-monorepo  score 0.32  via `.mise/toml/deploy.toml` -> DOCUMENTED_BY (docs/DESIGN-LOG.md:1869, 1.00)  
`Design log`  
<!-- Generated by `mise run governance:design-log` from `Design:` commit trailers (RFD-vjzs0qzv). Do not edit; regenerate. -->  
headings: ADR 25frqh42: ADR-0023: Immutable composite gateway policy bundles
RFD 2n2tcfe2: Operational resilience: stuck work, poison records, and the operator's surface
ADR 2y1ekrqz: ADR 0004 — Git workflow
RFD 5fyv311q: Data quality metrics and independent scanning
RFD 9kyb652g: CI build caching and runner isolation
ADR bn90xr5m: Authentication types make illegal callers unrepresentable
RFD c4jbh7qa: Type …  
documents_for 1816: `packages/search-core/src/indexer.ts` (docs/DESIGN-LOG.md:1572), `apps-web/showcase/src/lib/mock-websocket.ts` (docs/DESIGN-LOG.md:6114), `crates/causeway-testkit/src/kafka.rs` (docs/DESIGN-LOG.md:2611) … 1813 more  

### 58. File apps/causeway-server/src/startup/secrets.rs
`file:apps/causeway-server/src/startup/secrets.rs`  apps/causeway-server/src/startup/secrets.rs  pkg:cargo/causeway-server  score 0.32  via `GITHUB_DISPATCH_TOKEN` <- CONTAINS (apps/causeway-server/src/startup/secrets.rs:23-24, 1.00)  
members 5  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:1076)  
imports 5: `ResolvedConfig` (apps/causeway-server/src/startup/secrets.rs:17), `Secret` (apps/causeway-server/src/startup/secrets.rs:17), `ConfigError::Secret` (apps/causeway-server/src/startup/secrets.rs:17) … 2 more  

### 59. Struct ServerSecrets
`rust:apps/causeway-server/src/startup/secrets.rs::ServerSecrets`  apps/causeway-server/src/startup/secrets.rs:28-31  pkg:cargo/causeway-server  score 0.32  via `ServerSecrets#impl` -> USES_TYPE (apps/causeway-server/src/startup/secrets.rs:33-60, 1.00)  
`pub(crate) struct ServerSecrets`  
The shell's optional secrets, as startup resolved them.  
members 3  
used_by 9: `AppState::secrets` (apps/causeway-server/src/composition/state.rs:749), `PlatformRuntime` (apps/causeway-server/src/composition/state/platform_runtime.rs:29), `ServerConfig` (apps/causeway-server/src/startup/config.rs:78) … 6 more  
uses_types 1: `Secret` (apps/causeway-server/src/startup/secrets.rs:29)  

### 60. Impl ServerSecrets#impl
`rust:apps/causeway-server/src/startup/secrets.rs::ServerSecrets#impl`  apps/causeway-server/src/startup/secrets.rs:33-60  pkg:cargo/causeway-server  score 0.32  via `ServerSecrets::github_dispatch_token` <- DEFINES (apps/causeway-server/src/startup/secrets.rs:57-59, 1.00)  
`impl ServerSecrets`  
members 3  
uses_types 1: `ServerSecrets` (apps/causeway-server/src/startup/secrets.rs:33-60)  

### 61. Function configuration_inputs
`rust:bootstrap/cw-vsa/src/checks.rs::configuration_inputs`  bootstrap/cw-vsa/src/checks.rs:405-447  pkg:cargo/cw-vsa  score 0.32  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:405, 1.00)  
`fn configuration_inputs(ctx: &Ctx<'_>, args: &ConfigurationInputs) -> CheckResult`  
No code outside the platform libraries selects behaviour from a raw (doc continues, 909 bytes)  
callers 2: `tests::the_configuration_search_reads_runtime_sources_outside_the_platform` (bootstrap/cw-vsa/src/checks.rs:1624), `run` (bootstrap/cw-vsa/src/checks.rs:215)  
calls 5: `configuration_findings` (bootstrap/cw-vsa/src/checks.rs:438), `violation` (bootstrap/cw-vsa/src/checks.rs:439), `violations` (bootstrap/cw-vsa/src/checks.rs:446) … 2 more  
tests 1: `tests::the_configuration_search_reads_runtime_sources_outside_the_platform` (bootstrap/cw-vsa/src/checks.rs:1624)  
uses_types 3: `CheckResult` (bootstrap/cw-vsa/src/checks.rs:405), `Ctx` (bootstrap/cw-vsa/src/checks.rs:405), `ConfigurationInputs` (bootstrap/cw-vsa/src/checks.rs:405)  

### 62. Function seam_count
`rust:bootstrap/cw-vsa/src/checks.rs::seam_count`  bootstrap/cw-vsa/src/checks.rs:1361-1366  pkg:cargo/cw-vsa  score 0.32  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:1361, 1.00)  
`fn seam_count(ctx: &Ctx<'_>, seam: Seam) -> CheckResult`  
callers 2: `run` (bootstrap/cw-vsa/src/checks.rs:203), `run` (bootstrap/cw-vsa/src/checks.rs:204)  
calls 1: `schema_refs` (bootstrap/cw-vsa/src/checks.rs:1363)  
uses_types 3: `Ctx` (bootstrap/cw-vsa/src/checks.rs:1361), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:1361), `Seam` (bootstrap/cw-vsa/src/checks.rs:1361)  

### 63. Function tests::check
`rust:bootstrap/cw-vsa/src/image_modes.rs::tests::check`  bootstrap/cw-vsa/src/image_modes.rs:539-541  pkg:cargo/cw-vsa  score 0.31  via `findings` <- CALLS (bootstrap/cw-vsa/src/image_modes.rs:540, 1.00)  
`fn check(text: &str) -> Vec<String>`  
callers 24: `tests::an_octal_chmod_without_execute_is_refused_where_it_makes_directories` (bootstrap/cw-vsa/src/image_modes.rs:646), `tests::a_chmod_after_user_or_one_denying_others_does_not_count` (bootstrap/cw-vsa/src/image_modes.rs:605), `tests::chown_to_the_final_user_counts_and_to_another_does_not` (bootstrap/cw-vsa/src/image_modes.rs:613) … 21 more  
calls 1: `findings` (bootstrap/cw-vsa/src/image_modes.rs:540)  
tests 10: `tests::an_octal_chmod_without_execute_is_refused_where_it_makes_directories` (bootstrap/cw-vsa/src/image_modes.rs:638), `tests::a_chmod_after_user_or_one_denying_others_does_not_count` (bootstrap/cw-vsa/src/image_modes.rs:599), `tests::chown_to_the_final_user_counts_and_to_another_does_not` (bootstrap/cw-vsa/src/image_modes.rs:613) … 7 more  

### 64. Variant Check::CoherenceCheckSchemaRefs
`rust:bootstrap/cw-vsa/src/checks.rs::Check::CoherenceCheckSchemaRefs`  bootstrap/cw-vsa/src/checks.rs:107  pkg:cargo/cw-vsa  score 0.31  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:161, 1.00)  
`CoherenceCheckSchemaRefs`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:161), `run` (bootstrap/cw-vsa/src/checks.rs:204)  

### 65. Function run_fixes
`rust:bootstrap/cw-vsa/src/image_modes.rs::run_fixes`  bootstrap/cw-vsa/src/image_modes.rs:326-347  pkg:cargo/cw-vsa  score 0.31  via `findings` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:500, 1.00)  
`fn run_fixes(copy: &Copy, run: &str, workdir: &str, user: &str) -> bool`  
What `run` (executed as root, in `workdir`) fixes of `copy`, for an image (doc continues, 95 bytes)  
callers 1: `findings` (bootstrap/cw-vsa/src/image_modes.rs:500)  
calls 6: `grants_others` (bootstrap/cw-vsa/src/image_modes.rs:344), `same_user` (bootstrap/cw-vsa/src/image_modes.rs:342), `absolute` (bootstrap/cw-vsa/src/image_modes.rs:332) … 3 more  
uses_types 1: `Copy` (bootstrap/cw-vsa/src/image_modes.rs:326)  

### 66. Function facade_size_budget
`rust:bootstrap/cw-vsa/src/checks.rs::facade_size_budget`  bootstrap/cw-vsa/src/checks.rs:1186-1198  pkg:cargo/cw-vsa  score 0.30  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:1186, 1.00)  
`fn facade_size_budget(ctx: &Ctx<'_>) -> CheckResult`  
Server slice facades stay under 220 lines (a missing file is fine).  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:188)  
calls 4: `violation` (bootstrap/cw-vsa/src/checks.rs:1194), `read` (bootstrap/cw-vsa/src/checks.rs:1191), `split_lines` (bootstrap/cw-vsa/src/checks.rs:1192) … 1 more  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:1186), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:1186)  

### 67. Function msk_auth_topology
`rust:bootstrap/cw-vsa/src/checks.rs::msk_auth_topology`  bootstrap/cw-vsa/src/checks.rs:706-740  pkg:cargo/cw-vsa  score 0.30  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:706, 1.00)  
`fn msk_auth_topology(ctx: &Ctx<'_>) -> CheckResult`  
For each environment whose topology declares `[kafka.msk]`, the explicit (doc continues, 132 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:180)  
calls 5: `violation` (bootstrap/cw-vsa/src/checks.rs:735), `re` (bootstrap/cw-vsa/src/checks.rs:734), `re` (bootstrap/cw-vsa/src/checks.rs:707) … 2 more  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:706), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:706)  

### 68. Function runtime_env_resource_ctors
`rust:bootstrap/cw-vsa/src/checks.rs::runtime_env_resource_ctors`  bootstrap/cw-vsa/src/checks.rs:764-788  pkg:cargo/cw-vsa  score 0.30  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:764, 1.00)  
`fn runtime_env_resource_ctors(ctx: &Ctx<'_>) -> CheckResult`  
causeway-runtime resource types expose typed topology constructors, not (doc continues, 141 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:181)  
calls 5: `impl_block` (bootstrap/cw-vsa/src/checks.rs:779), `violation` (bootstrap/cw-vsa/src/checks.rs:781), `read` (bootstrap/cw-vsa/src/checks.rs:766) … 2 more  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:764), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:764)  

### 69. Function task_content
`rust:bootstrap/cw-vsa/src/checks.rs::task_content`  bootstrap/cw-vsa/src/checks.rs:676-700  pkg:cargo/cw-vsa  score 0.30  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:676, 1.00)  
`fn task_content(ctx: &Ctx<'_>, table: &[TaskContent]) -> CheckResult`  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:179)  
calls 6: `violation` (bootstrap/cw-vsa/src/checks.rs:692), `violations` (bootstrap/cw-vsa/src/checks.rs:699), `task_body` (bootstrap/cw-vsa/src/checks.rs:680) … 3 more  
uses_types 3: `Ctx` (bootstrap/cw-vsa/src/checks.rs:676), `TaskContent` (bootstrap/cw-vsa/src/checks.rs:676), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:676)  

### 70. Method RawRule::compile
`rust:bootstrap/cw-vsa/src/rules.rs::RawRule::compile`  bootstrap/cw-vsa/src/rules.rs:128-179  pkg:cargo/cw-vsa  score 0.30  via `Check::parse` <- CALLS (bootstrap/cw-vsa/src/rules.rs:152, 1.00)  
`pub fn compile(self) -> Result<GateRule, String>`  
Validate and compile. (doc continues, 190 bytes)  
callers 2: `compile_all` (bootstrap/cw-vsa/src/rules.rs:193), `opt_compile` (bootstrap/cw-vsa/src/rules.rs:118)  
calls 8: `opt_compile` (bootstrap/cw-vsa/src/rules.rs:169), `opt_compile` (bootstrap/cw-vsa/src/rules.rs:167), `compile` (bootstrap/cw-vsa/src/rules.rs:133) … 5 more  
references 1: `Severity::Fail` (bootstrap/cw-vsa/src/rules.rs:166)  
uses_types 2: `tests::Named` (bootstrap/cw-vsa/src/rules.rs:150), `GateRule` (bootstrap/cw-vsa/src/rules.rs:128)  

### 71. Function island_cross_refs
`rust:bootstrap/cw-vsa/src/checks.rs::island_cross_refs`  bootstrap/cw-vsa/src/checks.rs:1256-1283  pkg:cargo/cw-vsa  score 0.30  via `read` <- CALLS (bootstrap/cw-vsa/src/checks.rs:1275, 1.00)  
`fn island_cross_refs(ctx: &Ctx<'_>) -> Vec<String>`  
Each `web/ui-*` package's own name joined against every (doc continues, 189 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:195)  
calls 6: `read` (bootstrap/cw-vsa/src/checks.rs:1264), `re` (bootstrap/cw-vsa/src/checks.rs:1260), `read` (bootstrap/cw-vsa/src/checks.rs:1275) … 3 more  
uses_types 1: `Ctx` (bootstrap/cw-vsa/src/checks.rs:1256)  

### 72. Variant Check::ComponentCrossDeps
`rust:bootstrap/cw-vsa/src/checks.rs::Check::ComponentCrossDeps`  bootstrap/cw-vsa/src/checks.rs:97  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:152, 1.00)  
`ComponentCrossDeps`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:183), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:152)  

### 73. Variant Check::CrossSliceSchemaRefs
`rust:bootstrap/cw-vsa/src/checks.rs::Check::CrossSliceSchemaRefs`  bootstrap/cw-vsa/src/checks.rs:105  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:159, 1.00)  
`CrossSliceSchemaRefs`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:159), `run` (bootstrap/cw-vsa/src/checks.rs:196)  

### 74. Variant Check::CryptoProviderInstalls
`rust:bootstrap/cw-vsa/src/checks.rs::Check::CryptoProviderInstalls`  bootstrap/cw-vsa/src/checks.rs:98  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:153, 1.00)  
`CryptoProviderInstalls`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:153), `run` (bootstrap/cw-vsa/src/checks.rs:184)  

### 75. Variant Check::DomainDependents
`rust:bootstrap/cw-vsa/src/checks.rs::Check::DomainDependents`  bootstrap/cw-vsa/src/checks.rs:109  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:163, 1.00)  
`DomainDependents`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:163), `run` (bootstrap/cw-vsa/src/checks.rs:218)  

### 76. Variant Check::DomainModules
`rust:bootstrap/cw-vsa/src/checks.rs::Check::DomainModules`  bootstrap/cw-vsa/src/checks.rs:108  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:162, 1.00)  
`DomainModules`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:162), `run` (bootstrap/cw-vsa/src/checks.rs:205)  

### 77. Variant Check::FacadeSizeBudget
`rust:bootstrap/cw-vsa/src/checks.rs::Check::FacadeSizeBudget`  bootstrap/cw-vsa/src/checks.rs:100  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:154, 1.00)  
`FacadeSizeBudget`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:188), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:154)  

### 78. Variant Check::IslandCrossRefs
`rust:bootstrap/cw-vsa/src/checks.rs::Check::IslandCrossRefs`  bootstrap/cw-vsa/src/checks.rs:104  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:158, 1.00)  
`IslandCrossRefs`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:195), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:158)  

### 79. Variant Check::MskAuthTopology
`rust:bootstrap/cw-vsa/src/checks.rs::Check::MskAuthTopology`  bootstrap/cw-vsa/src/checks.rs:94  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:149, 1.00)  
`MskAuthTopology`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:149), `run` (bootstrap/cw-vsa/src/checks.rs:180)  

### 80. Variant Check::RelayRestateStagingOrder
`rust:bootstrap/cw-vsa/src/checks.rs::Check::RelayRestateStagingOrder`  bootstrap/cw-vsa/src/checks.rs:96  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:151, 1.00)  
`RelayRestateStagingOrder`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:151), `run` (bootstrap/cw-vsa/src/checks.rs:182)  

### 81. Variant Check::RuntimeEnvResourceCtors
`rust:bootstrap/cw-vsa/src/checks.rs::Check::RuntimeEnvResourceCtors`  bootstrap/cw-vsa/src/checks.rs:95  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:150, 1.00)  
`RuntimeEnvResourceCtors`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:150), `run` (bootstrap/cw-vsa/src/checks.rs:181)  

### 82. Variant Check::SearchProjectionSchemaRefs
`rust:bootstrap/cw-vsa/src/checks.rs::Check::SearchProjectionSchemaRefs`  bootstrap/cw-vsa/src/checks.rs:106  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:160, 1.00)  
`SearchProjectionSchemaRefs`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:203), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:160)  

### 83. Variant Check::ServerOrchestrators
`rust:bootstrap/cw-vsa/src/checks.rs::Check::ServerOrchestrators`  bootstrap/cw-vsa/src/checks.rs:103  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:157, 1.00)  
`ServerOrchestrators`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:191), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:157)  

### 84. Variant Check::ServerRpcDeps
`rust:bootstrap/cw-vsa/src/checks.rs::Check::ServerRpcDeps`  bootstrap/cw-vsa/src/checks.rs:101  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:155, 1.00)  
`ServerRpcDeps`  
referenced_by 2: `run` (bootstrap/cw-vsa/src/checks.rs:189), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:155)  

### 85. Variant Check::ServerServiceImpls
`rust:bootstrap/cw-vsa/src/checks.rs::Check::ServerServiceImpls`  bootstrap/cw-vsa/src/checks.rs:102  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> REFERENCES (bootstrap/cw-vsa/src/checks.rs:156, 1.00)  
`ServerServiceImpls`  
referenced_by 2: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:156), `run` (bootstrap/cw-vsa/src/checks.rs:190)  

### 86. Function strs
`rust:bootstrap/cw-vsa/src/checks.rs::strs`  bootstrap/cw-vsa/src/checks.rs:42-52  pkg:cargo/cw-vsa  score 0.29  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:140, 1.00)  
`fn strs(v: &Json) -> Result<Vec<String>, String>`  
callers 6: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:140), `Check::parse` (bootstrap/cw-vsa/src/checks.rs:136), `configuration_inputs_args` (bootstrap/cw-vsa/src/checks.rs:314) … 3 more  
uses_types 2: `Json` (bootstrap/cw-vsa/src/checks.rs:42), `Json` (bootstrap/cw-vsa/src/checks.rs:42)  

### 87. Test tests::dockerfile_names
`rust:bootstrap/cw-vsa/src/image_modes.rs::tests::dockerfile_names`  bootstrap/cw-vsa/src/image_modes.rs:674-679  pkg:cargo/cw-vsa  score 0.28  via `is_dockerfile` -> TESTED_BY (bootstrap/cw-vsa/src/image_modes.rs:675, 1.00)  
`fn dockerfile_names()`  
calls 4: `is_dockerfile` (bootstrap/cw-vsa/src/image_modes.rs:675), `is_dockerfile` (bootstrap/cw-vsa/src/image_modes.rs:677), `is_dockerfile` (bootstrap/cw-vsa/src/image_modes.rs:678) … 1 more  
tests_for 1: `is_dockerfile` (bootstrap/cw-vsa/src/image_modes.rs:675)  

### 88. Function crypto_provider_installs
`rust:bootstrap/cw-vsa/src/checks.rs::crypto_provider_installs`  bootstrap/cw-vsa/src/checks.rs:995-1021  pkg:cargo/cw-vsa  score 0.28  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:995, 1.00)  
`fn crypto_provider_installs(ctx: &Ctx<'_>) -> CheckResult`  
Every binary whose workspace links rustls 0.23 with both the `ring` and (doc continues, 733 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:184)  
calls 2: `violations` (bootstrap/cw-vsa/src/checks.rs:1020), `crypto_provider_zone` (bootstrap/cw-vsa/src/checks.rs:1011)  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:995), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:995)  

### 89. Function module
`rust:bootstrap/cw-host/src/modules/env.rs::module`  bootstrap/cw-host/src/modules/env.rs:46-66  pkg:cargo/cw-host  score 0.28  via `get` <- DEFINES (bootstrap/cw-host/src/modules/env.rs:48-50, 1.00)  
`pub fn module() -> Module`  
members 5  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:29)  

### 90. Function module
`rust:bootstrap/cw-host/src/modules/http.rs::module`  bootstrap/cw-host/src/modules/http.rs:205-229  pkg:cargo/cw-host  score 0.28  via `get` <- DEFINES (bootstrap/cw-host/src/modules/http.rs:222, 1.00)  
`pub fn module(engine: &mut Engine) -> Module`  
members 8  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:32)  
uses_types 1: `Stream` (bootstrap/cw-host/src/modules/http.rs:206)  

### 91. Function module
`rust:bootstrap/cw-host/src/modules/time.rs::module`  bootstrap/cw-host/src/modules/time.rs:44-52  pkg:cargo/cw-host  score 0.28  via `now_ms` <- DEFINES (bootstrap/cw-host/src/modules/time.rs:46, 1.00)  
`pub fn module() -> Module`  
members 3  
callers 1: `register` (bootstrap/cw-host/src/modules/mod.rs:30)  

### 92. Struct Copy
`rust:bootstrap/cw-vsa/src/image_modes.rs::Copy`  bootstrap/cw-vsa/src/image_modes.rs:295-306  pkg:cargo/cw-vsa  score 0.28  via `findings` -> USES_TYPE (bootstrap/cw-vsa/src/image_modes.rs:463, 1.00)  
`struct Copy`  
A context copy waiting for its modes to be fixed.  
used_by 11: `context_copy` (bootstrap/cw-vsa/src/image_modes.rs:357), `metadata_health_content` (apps/causeway-server-web/src/views/platform/operations/metadata_health.rs:12), `TuningInt` (torrentfx/crates/causeway-platform/src/tuning.rs:109) … 8 more  

### 93. Function server_service_impls
`rust:bootstrap/cw-vsa/src/checks.rs::server_service_impls`  bootstrap/cw-vsa/src/checks.rs:1226-1249  pkg:cargo/cw-vsa  score 0.28  via `read` <- CALLS (bootstrap/cw-vsa/src/checks.rs:1237, 1.00)  
`fn server_service_impls(ctx: &Ctx<'_>) -> (Vec<String>, usize)`  
(files implementing gRPC services outside the orchestrators, orchestrator (doc continues, 319 bytes)  
callers 2: `run` (bootstrap/cw-vsa/src/checks.rs:192), `run` (bootstrap/cw-vsa/src/checks.rs:190)  
calls 3: `re` (bootstrap/cw-vsa/src/checks.rs:1228), `read` (bootstrap/cw-vsa/src/checks.rs:1237), `re` (bootstrap/cw-vsa/src/checks.rs:1227)  
uses_types 1: `Ctx` (bootstrap/cw-vsa/src/checks.rs:1226)  

### 94. Task rhai:test
`task:mise/rhai:test`  .mise/toml/bootstrap.toml:25-28  pkg:npm/causeway-monorepo  score 0.27  via `test_the_tooling_steps_are_ci_yml_s` -> RUNS (.mise/rhai/causeway/ci_local_test.rhai:69, 1.00)  
`mise run rhai:test`  
Run the Rhai unit tests: every zero-argument test_* function in every *_test.rhai file under .mise/.  
description: Run the Rhai unit tests: every zero-argument test_* function in every *_test.rhai file under .mise/.  
run: cwsh test  
run_by 1: `test_the_tooling_steps_are_ci_yml_s` (.mise/rhai/causeway/ci_local_test.rhai:69)  
runs 1: `main` (.mise/toml/bootstrap.toml:27)  

### 95. Function groups
`rust:bootstrap/cw-host/src/modules/re.rs::groups`  bootstrap/cw-host/src/modules/re.rs:43-47  pkg:cargo/cw-host  score 0.27  via `captures` -> CALLS (bootstrap/cw-host/src/modules/re.rs:73, 1.00)  
`fn groups(caps: &regex::Captures<'_>) -> Array`  
callers 4: `captures` (bootstrap/cw-host/src/modules/re.rs:73), `captures_all` (bootstrap/cw-host/src/modules/re.rs:79), `K8s::resolve` (bootstrap/cw-host/src/modules/k8s.rs:179) … 1 more  

### 96. Function component_cross_deps
`rust:bootstrap/cw-vsa/src/checks.rs::component_cross_deps`  bootstrap/cw-vsa/src/checks.rs:823-918  pkg:cargo/cw-vsa  score 0.26  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:823, 1.00)  
`fn component_cross_deps(ctx: &Ctx<'_>) -> CheckResult`  
A component workspace member's path dependencies may resolve only within (doc continues, 229 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:183)  
calls 8: `violation` (bootstrap/cw-vsa/src/checks.rs:854), `violations` (bootstrap/cw-vsa/src/checks.rs:854), `violation` (bootstrap/cw-vsa/src/checks.rs:847) … 5 more  
uses_types 4: `CheckResult` (bootstrap/cw-vsa/src/checks.rs:823), `Ctx` (bootstrap/cw-vsa/src/checks.rs:823), `Json` (bootstrap/cw-vsa/src/checks.rs:843) … 1 more  

### 97. Function gate_staleness
`rust:bootstrap/cw-vsa/src/checks.rs::gate_staleness`  bootstrap/cw-vsa/src/checks.rs:574-582  pkg:cargo/cw-vsa  score 0.26  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:574, 1.00)  
`fn gate_staleness(ctx: &Ctx<'_>, paths: &[String]) -> CheckResult`  
Every path the rules reference must exist, or nothing after it can be (doc continues, 138 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:178)  
calls 2: `violations` (bootstrap/cw-vsa/src/checks.rs:575), `violation` (bootstrap/cw-vsa/src/checks.rs:579)  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:574), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:574)  

### 98. Function cargo_feature_isolation
`rust:bootstrap/cw-vsa/src/checks.rs::cargo_feature_isolation`  bootstrap/cw-vsa/src/checks.rs:926-966  pkg:cargo/cw-vsa  score 0.26  via `CheckResult` <- USES_TYPE (bootstrap/cw-vsa/src/checks.rs:926, 1.00)  
`fn cargo_feature_isolation(ctx: &Ctx<'_>, krate: &str, message: &str) -> CheckResult`  
`cargo tree -i CRATE` over causeway-server's default-free feature set: (doc continues, 366 bytes)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:186)  
calls 4: `violations` (bootstrap/cw-vsa/src/checks.rs:965), `unreachable_package` (bootstrap/cw-vsa/src/checks.rs:955), `violation` (bootstrap/cw-vsa/src/checks.rs:965) … 1 more  
uses_types 2: `Ctx` (bootstrap/cw-vsa/src/checks.rs:926), `CheckResult` (bootstrap/cw-vsa/src/checks.rs:926)  

### 99. Function read
`rhai:.mise/rhai/causeway/node_addon_release_test.rhai::read`  .mise/rhai/causeway/node_addon_release_test.rhai:8-10  pkg:npm/causeway-monorepo  score 0.26  via `pre_publish_lines` -> CALLS (.mise/rhai/causeway/node_addon_release_test.rhai:15, 1.00)  
`fn read(path)`  
callers 2: `task_body` (.mise/rhai/causeway/node_addon_release_test.rhai:33), `pre_publish_lines` (.mise/rhai/causeway/node_addon_release_test.rhai:15)  
calls 2: `repo_root` (.mise/rhai/causeway/node_addon_release_test.rhai:9), `read` (.mise/rhai/causeway/node_addon_release_test.rhai:9)  

### 100. Function context_copy
`rust:bootstrap/cw-vsa/src/image_modes.rs::context_copy`  bootstrap/cw-vsa/src/image_modes.rs:352-426  pkg:cargo/cw-vsa  score 0.26  via `findings` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:479, 1.00)  
`fn context_copy( ins: &Instruction, workdir: &str, symbolic_ok: bool, is_dir: &dyn Fn(&str) -> bool, ) -> Option<(Copy, Option<String>)>`  
A `COPY` or `ADD` from the build context, run in `workdir`, and the fault (doc continues, 175 bytes)  
callers 1: `findings` (bootstrap/cw-vsa/src/image_modes.rs:479)  
calls 10: `split_flags` (bootstrap/cw-vsa/src/image_modes.rs:358), `absolute` (bootstrap/cw-vsa/src/image_modes.rs:371), `parent` (bootstrap/cw-vsa/src/image_modes.rs:405) … 7 more  
uses_types 2: `Copy` (bootstrap/cw-vsa/src/image_modes.rs:357), `Instruction` (bootstrap/cw-vsa/src/image_modes.rs:353)  

### 101. Function ci
`rhai:.mise/rhai/causeway/ci_local_test.rhai::ci`  .mise/rhai/causeway/ci_local_test.rhai:19-21  pkg:npm/causeway-monorepo  score 0.25  via `test_the_tooling_steps_are_ci_yml_s` -> CALLS (.mise/rhai/causeway/ci_local_test.rhai:73, 1.00)  
`fn ci()`  
callers 7: `test_it_classifies_with_what_changes_hands_its_scripts` (.mise/rhai/causeway/ci_local_test.rhai:40), `test_every_step_of_a_local_job_runs_here` (.mise/rhai/causeway/ci_local_test.rhai:55), `test_the_tooling_steps_are_ci_yml_s` (.mise/rhai/causeway/ci_local_test.rhai:67) … 4 more  
calls 1: `source` (.mise/rhai/causeway/ci_local_test.rhai:20)  

### 102. Function main
`rust:bootstrap/cwsh/src/main.rs::main`  bootstrap/cwsh/src/main.rs:33-44  pkg:cargo/cwsh  score 0.25  via `mise/rhai:check` -> RUNS (.mise/toml/bootstrap.toml:23, 0.75)  
`fn main() -> ExitCode`  
calls 3: `install_panic_hook` (bootstrap/cwsh/src/main.rs:34), `process_exit_code` (bootstrap/cwsh/src/main.rs:43), `run` (bootstrap/cwsh/src/main.rs:36)  
run_by 4: `mise/devdata:test` (.mise/toml/devdata.toml:12), `mise/rhai:defs` (.mise/toml/bootstrap.toml:31), `mise/rhai:check` (.mise/toml/bootstrap.toml:23) … 1 more  

### 103. Function is_root
`rust:bootstrap/cw-vsa/src/image_modes.rs::is_root`  bootstrap/cw-vsa/src/image_modes.rs:190-195  pkg:cargo/cw-vsa  score 0.24  via `findings` -> CALLS (bootstrap/cw-vsa/src/image_modes.rs:487, 1.00)  
`fn is_root(user: Option<&str>) -> bool`  
callers 2: `findings` (bootstrap/cw-vsa/src/image_modes.rs:476), `findings` (bootstrap/cw-vsa/src/image_modes.rs:487)  

### 104. Struct ConfigurationInputs
`rust:bootstrap/cw-vsa/src/checks.rs::ConfigurationInputs`  bootstrap/cw-vsa/src/checks.rs:67-72  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:165, 1.00)  
`pub struct ConfigurationInputs`  
The arguments of `configuration-inputs`: where the configuration inputs (doc continues, 137 bytes)  
callers 1: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:165)  
used_by 3: `configuration_inputs_args` (bootstrap/cw-vsa/src/checks.rs:303), `configuration_inputs` (bootstrap/cw-vsa/src/checks.rs:405), `Check::ConfigurationInputs` (bootstrap/cw-vsa/src/checks.rs:110)  

### 105. Struct PrefixedSettings
`rust:bootstrap/cw-vsa/src/checks.rs::PrefixedSettings`  bootstrap/cw-vsa/src/checks.rs:77-85  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:168, 1.00)  
`pub struct PrefixedSettings`  
The arguments of `prefixed-settings`: where the Node and Python sources (doc continues, 117 bytes)  
callers 1: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:168)  
used_by 4: `prefixed_settings` (bootstrap/cw-vsa/src/checks.rs:535), `prefixed_settings_findings` (bootstrap/cw-vsa/src/checks.rs:493), `prefixed_settings_args` (bootstrap/cw-vsa/src/checks.rs:454) … 1 more  

### 106. Struct TaskContent
`rust:bootstrap/cw-vsa/src/checks.rs::TaskContent`  bootstrap/cw-vsa/src/checks.rs:57-62  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:138, 1.00)  
`pub struct TaskContent`  
One `task-content` row: a task, an extra file read with its body, and (doc continues, 135 bytes)  
callers 1: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:138)  
used_by 3: `task_content` (bootstrap/cw-vsa/src/checks.rs:676), `task_content_table` (bootstrap/cw-vsa/src/checks.rs:631), `Check::TaskContent` (bootstrap/cw-vsa/src/checks.rs:93)  

### 107. Function configuration_inputs_args
`rust:bootstrap/cw-vsa/src/checks.rs::configuration_inputs_args`  bootstrap/cw-vsa/src/checks.rs:303-330  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:165, 1.00)  
`fn configuration_inputs_args(args: &Json) -> Result<ConfigurationInputs, String>`  
callers 3: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:165), `tests::the_configuration_search_reads_runtime_sources_outside_the_platform` (bootstrap/cw-vsa/src/checks.rs:1642), `tests::the_configuration_search_reads_runtime_sources_outside_the_platform` (bootstrap/cw-vsa/src/checks.rs:1615)  
calls 2: `strs` (bootstrap/cw-vsa/src/checks.rs:314), `strs` (bootstrap/cw-vsa/src/checks.rs:307)  
tests 1: `tests::the_configuration_search_reads_runtime_sources_outside_the_platform` (bootstrap/cw-vsa/src/checks.rs:1615)  
uses_types 3: `ConfigurationInputs` (bootstrap/cw-vsa/src/checks.rs:303), `Json` (bootstrap/cw-vsa/src/checks.rs:303), `Json` (bootstrap/cw-vsa/src/checks.rs:303)  

### 108. Function prefixed_settings_args
`rust:bootstrap/cw-vsa/src/checks.rs::prefixed_settings_args`  bootstrap/cw-vsa/src/checks.rs:454-473  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:168, 1.00)  
`fn prefixed_settings_args(args: &Json) -> Result<PrefixedSettings, String>`  
callers 3: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:168), `tests::the_prefix_search_finds_bare_names_in_node_and_python` (bootstrap/cw-vsa/src/checks.rs:1536), `tests::the_prefix_search_finds_bare_names_in_node_and_python` (bootstrap/cw-vsa/src/checks.rs:1570)  
calls 10: `strs` (bootstrap/cw-vsa/src/checks.rs:458), `list` (bootstrap/cw-vsa/src/checks.rs:459), `list` (bootstrap/cw-vsa/src/checks.rs:459) … 7 more  
tests 1: `tests::the_prefix_search_finds_bare_names_in_node_and_python` (bootstrap/cw-vsa/src/checks.rs:1536)  
uses_types 3: `PrefixedSettings` (bootstrap/cw-vsa/src/checks.rs:454), `Json` (bootstrap/cw-vsa/src/checks.rs:454), `Json` (bootstrap/cw-vsa/src/checks.rs:454)  

### 109. Function task_content_table
`rust:bootstrap/cw-vsa/src/checks.rs::task_content_table`  bootstrap/cw-vsa/src/checks.rs:631-674  pkg:cargo/cw-vsa  score 0.23  via `Check::parse` -> CALLS (bootstrap/cw-vsa/src/checks.rs:138, 1.00)  
`fn task_content_table(table: &Json) -> Result<Vec<TaskContent>, String>`  
A table of task-body content rules: `[task, extra_script, requires, (doc continues, 138 bytes)  
callers 1: `Check::parse` (bootstrap/cw-vsa/src/checks.rs:138)  
calls 3: `pairs` (bootstrap/cw-vsa/src/checks.rs:669), `pairs` (bootstrap/cw-vsa/src/checks.rs:670), `strs` (bootstrap/cw-vsa/src/checks.rs:637)  
references 1: `Json::Null` (bootstrap/cw-vsa/src/checks.rs:665)  
uses_types 3: `TaskContent` (bootstrap/cw-vsa/src/checks.rs:631), `Json` (bootstrap/cw-vsa/src/checks.rs:631), `Json` (bootstrap/cw-vsa/src/checks.rs:631)  

### 110. Function preflight
`rhai:.mise/rhai/causeway/prod.rhai::preflight`  .mise/rhai/causeway/prod.rhai:205-462  pkg:npm/causeway-monorepo  score 0.23  via `is_match` <- CALLS (.mise/rhai/causeway/prod.rhai:262, 1.00)  
`fn preflight(io, cfg)`  
The full production preflight: render checks, live-cluster checks, and a server-side dry run. Changes nothing. Throws on the first violated gate.  
callers 7: `.mise/scripts/tasks/deploy/prod-preflight.rhai` (.mise/scripts/tasks/deploy/prod-preflight.rhai:16), `rollout` (.mise/rhai/causeway/prod.rhai:652), `.mise/scripts/prod-preflight-smoke.rhai` (.mise/scripts/prod-preflight-smoke.rhai:270) … 4 more  
calls 74: `fail` (.mise/rhai/causeway/prod.rhai:327), `on_exit` (.mise/rhai/causeway/prod.rhai:451), `fail` (.mise/rhai/causeway/prod.rhai:377) … 71 more  

### 111. Function schema_refs
`rust:bootstrap/cw-vsa/src/checks.rs::schema_refs`  bootstrap/cw-vsa/src/checks.rs:1324-1359  pkg:cargo/cw-vsa  score 0.23  via `read` <- CALLS (bootstrap/cw-vsa/src/checks.rs:1339, 1.00)  
`fn schema_refs(ctx: &Ctx<'_>) -> BTreeSet<(Seam, String, String, String)>`  
Distinct `(seam, path, own slice, referenced slice)` for every (doc continues, 165 bytes)  
callers 2: `run` (bootstrap/cw-vsa/src/checks.rs:197), `seam_count` (bootstrap/cw-vsa/src/checks.rs:1363)  
calls 2: `read` (bootstrap/cw-vsa/src/checks.rs:1339), `re` (bootstrap/cw-vsa/src/checks.rs:1325)  
references 3: `Seam::Cross` (bootstrap/cw-vsa/src/checks.rs:1353), `Seam::Search` (bootstrap/cw-vsa/src/checks.rs:1346), `Seam::Coherence` (bootstrap/cw-vsa/src/checks.rs:1351)  
uses_types 2: `Seam` (bootstrap/cw-vsa/src/checks.rs:1324), `Ctx` (bootstrap/cw-vsa/src/checks.rs:1324)  

### 112. Function server_rpc_deps
`rust:bootstrap/cw-vsa/src/checks.rs::server_rpc_deps`  bootstrap/cw-vsa/src/checks.rs:1203-1219  pkg:cargo/cw-vsa  score 0.23  via `read` <- CALLS (bootstrap/cw-vsa/src/checks.rs:1204, 1.00)  
`fn server_rpc_deps(ctx: &Ctx<'_>) -> (Vec<String>, usize)`  
(non-orchestrator `*-rpc` dependency lines, orchestrator count)  
callers 1: `run` (bootstrap/cw-vsa/src/checks.rs:189)  
calls 4: `re` (bootstrap/cw-vsa/src/checks.rs:1205), `read` (bootstrap/cw-vsa/src/checks.rs:1204), `split_lines` (bootstrap/cw-vsa/src/checks.rs:1207) … 1 more  
uses_types 1: `Ctx` (bootstrap/cw-vsa/src/checks.rs:1203)  

### 113. Function io
`rust:bootstrap/cw-host/src/modules/fs.rs::io`  bootstrap/cw-host/src/modules/fs.rs:49-51  pkg:cargo/cw-host  score 0.22  via `read` -> CALLS (bootstrap/cw-host/src/modules/fs.rs:214, 1.00)  
`fn io<T>(what: &str, path: &str, r: std::io::Result<T>) -> Res<T>`  
callers 20: `write_blob` (bootstrap/cw-host/src/modules/fs.rs:223), `read` (bootstrap/cw-host/src/modules/fs.rs:214), `size` (bootstrap/cw-host/src/modules/fs.rs:263) … 17 more  
calls 1: `err` (bootstrap/cw-host/src/modules/fs.rs:50)  
uses_types 1: `Res` (bootstrap/cw-host/src/modules/fs.rs:49)  

### 114. Test a_burndown_check_that_cannot_run_is_a_failure
`rust:bootstrap/cw-vsa/tests/gate.rs::a_burndown_check_that_cannot_run_is_a_failure`  bootstrap/cw-vsa/tests/gate.rs:159-173  pkg:cargo/cw-vsa  score 0.22  via `run` <- CALLS (bootstrap/cw-vsa/tests/gate.rs:167, 0.50)  
`fn a_burndown_check_that_cannot_run_is_a_failure()`  
calls 4: `report` (bootstrap/cw-vsa/tests/gate.rs:167), `rules` (bootstrap/cw-vsa/tests/gate.rs:163), `run` (bootstrap/cw-vsa/tests/gate.rs:167) … 1 more  
tests_for 4: `report` (bootstrap/cw-vsa/tests/gate.rs:167), `rules` (bootstrap/cw-vsa/tests/gate.rs:163), `run` (bootstrap/cw-vsa/tests/gate.rs:167) … 1 more  

### 115. Repository causeway
`repo:causeway`  score 0.21  via `npm/causeway-monorepo` <- CONTAINS (package.json:2, 1.00)  
members 279  

## Left out

48 included nodes have no source window for want of budget.  
- `rust:bootstrap/cw-host/src/globals.rs::exit_with` Function 0.22 (over budget, 194 tokens)
- `rust:bootstrap/cw-host/src/modules/http.rs::with` Function 0.22 (over budget, 228 tokens)
- `rust:bootstrap/cw-vsa/src/checks.rs::Check::ConfigurationInputs` Variant 0.21 (over budget, 132 tokens)
- and 1781 more

## Unresolved references in the seeds' files

- `expressions` at .mise/rhai/causeway/ci_local_test.rhai:102 (qualifier-mismatch)
- `expand` at .mise/rhai/causeway/ci_local_test.rhai:103 (qualifier-mismatch)
- `expand` at .mise/rhai/causeway/ci_local_test.rhai:104 (qualifier-mismatch)
- `expand` at .mise/rhai/causeway/ci_local_test.rhai:105 (qualifier-mismatch)
- `expand` at .mise/rhai/causeway/ci_local_test.rhai:106 (qualifier-mismatch)
- `parse_env_file` at .mise/rhai/causeway/ci_local_test.rhai:111 (qualifier-mismatch)
- `parse_env_file` at .mise/rhai/causeway/ci_local_test.rhai:112 (qualifier-mismatch)
- `plan` at .mise/rhai/causeway/ci_local_test.rhai:128 (qualifier-mismatch)
- `plan` at .mise/rhai/causeway/ci_local_test.rhai:135 (qualifier-mismatch)
- `plan` at .mise/rhai/causeway/ci_local_test.rhai:144 (qualifier-mismatch)

