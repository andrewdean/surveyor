<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `83dd9e101d93`, dataset at `ec266ebaf2c1`: **4 files, 5 changed symbols, 53 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (5)**

| Symbol | Kind | Where |
|---|---|---|
| `Design log` | Document | `docs/DESIGN-LOG.md:1-6154` |
| `USER_AGENT` | Const | `bootstrap/cw-host/src/modules/http.rs:59` |
| `build` | Function | `bootstrap/cw-host/src/modules/http.rs:68-107` |
| `a_request_names_cwsh_as_its_user_agent_unless_it_names_its_own` | Test | `bootstrap/cwsh/tests/cli.rs:674-691` |
| `user_agent_echo` | Function | `bootstrap/cwsh/tests/cli.rs:644-671` |

Changed as whole files (no declaration at the changed lines): `.mise/scripts/check-image-freshness.rhai`.

**Affected (53 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (49)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `open` | Function | `bootstrap/cw-host/src/modules/http.rs:162-196` | 1 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:163` |
| `request` | Function | `bootstrap/cw-host/src/modules/http.rs:132-158` | 1 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:133` |
| `open` | Function | `bootstrap/cw-host/src/modules/http.rs:216` | 2 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:220` |
| `request` | Function | `bootstrap/cw-host/src/modules/http.rs:229` | 2 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:229` |
| `with` | Function | `bootstrap/cw-host/src/modules/http.rs:206-211` | 2 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:210` |
| `sse_open` | Function | `.mise/rhai/causeway/mcp.rhai:561-567` | 3 | 1.00 | CALLS | `.mise/rhai/causeway/mcp.rhai:562` |
| `delete` | Function | `bootstrap/cw-host/src/modules/http.rs:234` | 3 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:235` |
| `get` | Function | `bootstrap/cw-host/src/modules/http.rs:230` | 3 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:230` |
| `post` | Function | `bootstrap/cw-host/src/modules/http.rs:232` | 3 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:232` |
| `put` | Function | `bootstrap/cw-host/src/modules/http.rs:233` | 3 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/http.rs:233` |
| `tests::delete_propagates_in_the_background_unless_told_otherwise` | Test | `bootstrap/cw-host/src/modules/k8s.rs:728-747` | 3 | 1.00 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:746` |
| `send` | Function | `bootstrap/cw-host/src/modules/s3.rs:151-210` | 1 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/s3.rs:191 [syntactic 0.75]` |
| `runtime` | Function | `bootstrap/cw-host/src/runtime.rs:11-23` | 1 | 0.75 | CALLS | `bootstrap/cw-host/src/runtime.rs:20 [syntactic 0.75]` |
| `delete` | Function | `bootstrap/cw-host/src/modules/s3.rs:302-315` | 2 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/s3.rs:307` |
| `list` | Function | `bootstrap/cw-host/src/modules/s3.rs:277-300` | 2 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/s3.rs:287` |
| `block_on` | Function | `bootstrap/cw-host/src/runtime.rs:25-27` | 2 | 0.75 | CALLS | `bootstrap/cw-host/src/runtime.rs:26` |
| `spawn` | Function | `bootstrap/cw-host/src/runtime.rs:29-34` | 2 | 0.75 | CALLS | `bootstrap/cw-host/src/runtime.rs:33` |
| `spawn_abortable` | Function | `bootstrap/cw-host/src/runtime.rs:37-42` | 2 | 0.75 | CALLS | `bootstrap/cw-host/src/runtime.rs:41` |
| `delete` | Function | `bootstrap/cw-host/src/modules/s3.rs:322-324` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/s3.rs:323` |
| `list` | Function | `bootstrap/cw-host/src/modules/s3.rs:319-321` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/s3.rs:320` |
| `on_exit` | Function | `bootstrap/cw-host/src/modules/script.rs:103-108` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/script.rs:106 [syntactic 0.75]` |
| `apply` | Function | `bootstrap/cw-host/src/modules/k8s.rs:331-361` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:333` |
| `connect` | Function | `bootstrap/cw-host/src/modules/k8s.rs:139-172` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:142` |
| `delete` | Function | `bootstrap/cw-host/src/modules/k8s.rs:392-432` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:418` |
| `exec` | Function | `bootstrap/cw-host/src/modules/k8s.rs:455-489` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:462` |
| `get` | Function | `bootstrap/cw-host/src/modules/k8s.rs:270-283` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:273` |
| `list` | Function | `bootstrap/cw-host/src/modules/k8s.rs:285-307` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:296` |
| `logs` | Function | `bootstrap/cw-host/src/modules/k8s.rs:434-443` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:441` |
| `patch` | Function | `bootstrap/cw-host/src/modules/k8s.rs:363-376` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:367` |
| `port_forward` | Function | `bootstrap/cw-host/src/modules/k8s.rs:566-620` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:592` |
| `raw` | Function | `bootstrap/cw-host/src/modules/k8s.rs:516-525` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/k8s.rs:517` |
| `batch` | Function | `bootstrap/cw-host/src/modules/pg.rs:272-275` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/pg.rs:274` |
| `bind` | Function | `bootstrap/cw-host/src/modules/pg.rs:232-247` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/pg.rs:233` |
| `execute` | Function | `bootstrap/cw-host/src/modules/pg.rs:264-270` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/pg.rs:267` |
| `open` | Function | `bootstrap/cw-host/src/modules/pg.rs:87-103` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/pg.rs:90` |
| `query` | Function | `bootstrap/cw-host/src/modules/pg.rs:256-262` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/pg.rs:259` |
| `with_re` | Function | `bootstrap/cw-host/src/modules/re.rs:31-41` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/re.rs:32 [syntactic 0.75]` |
| `take_exit_hooks` | Function | `bootstrap/cw-host/src/modules/script.rs:73-75` | 3 | 0.75 | CALLS | `bootstrap/cw-host/src/modules/script.rs:74 [syntactic 0.75]` |
| `arrow_bench_server` | Test | `apps/causeway-server/src/perf_arrow.rs:139-146` | 3 | 0.38 | CALLS | `apps/causeway-server/src/perf_arrow.rs:145 [syntactic 0.50]` |
| `Authenticator::keys_blocking` | Method | `crates/causeway-oidc/src/auth.rs:215-238` | 3 | 0.38 | CALLS | `crates/causeway-oidc/src/auth.rs:236 [syntactic 0.50]` |

… and 9 more in the JSON.

_Files and packages (4)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `bootstrap/cw-host/src/modules/http.rs` | File | `bootstrap/cw-host/src/modules/http.rs` | 2 | 0.75 | IMPORTS | `bootstrap/cw-host/src/modules/http.rs:53` |
| `bootstrap/cw-host/src/modules/k8s.rs` | File | `bootstrap/cw-host/src/modules/k8s.rs` | 2 | 0.75 | IMPORTS | `bootstrap/cw-host/src/modules/k8s.rs:73` |
| `bootstrap/cw-host/src/modules/pg.rs` | File | `bootstrap/cw-host/src/modules/pg.rs` | 2 | 0.75 | IMPORTS | `bootstrap/cw-host/src/modules/pg.rs:32` |
| `bootstrap/cw-host/src/modules/s3.rs` | File | `bootstrap/cw-host/src/modules/s3.rs` | 2 | 0.75 | IMPORTS | `bootstrap/cw-host/src/modules/s3.rs:23` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 67158 nodes, 288805 edges · 2696 ms</sub>
