// Ports `crypto_provider_installs` (bootstrap/cw-vsa/src/checks.rs): in a
// cargo workspace that links rustls 0.23 with both the `ring` and `aws-lc-rs`
// backends, every binary whose package reaches rustls calls
// `install_crypto_provider()` in its `main`, or its first TLS connection panics.
//
// Graph facts: Function nodes named `main` in `src/main.rs` or `src/bin/*.rs`,
// the package that owns each (`package_of`), what a default build of that package
// pulls in (`cargo_build`, from the section, optional, default-features and feature
// facts on DEPENDS_ON edges), and CALLS edges from `main` to the platform's
// `install_crypto_provider` (the causeway_runtime re-export resolves to the
// same function).
//
// Gaps, stated rather than guessed:
// - Which workspaces link both backends is decided by registry crates' features
//   (tonic brings aws-lc-rs, reqwest brings ring), and the graph holds first-party
//   manifests only, so it cannot tell. WORKSPACES below is what `cargo metadata`
//   said for Causeway 40f993d7; cw-vsa computes it on every run. The nested
//   workspaces services/relay-restate and tools/xtask link ring alone, as do
//   bootstrap/ and torrentfx/, so their binaries are outside the rule.
// - "Reaches rustls" means a default build of the binary's package pulls in a
//   first-party crate that names rustls (causeway-platform, cw-host) or names it
//   directly; dev-only and disabled optional paths no longer count. A binary that
//   reaches rustls through a registry crate alone (reqwest, tonic) is not seen;
//   cw-vsa walks the full resolve graph.
// - A `[[bin]]` with a custom `path` is found by cw-vsa through cargo targets
//   and not by the file-name convention used here.
const ID = "crypto-provider-installs";
const DESC = "a binary that links rustls with both ring and aws-lc-rs installs the process crypto provider first in main, or its first TLS connection panics";

const WORKSPACES = ["apps/", "component/", "crates/", "ffi/node-drive/", "ffi/py-drive/", "jobs/", "services/", "tools/"];
const RING_ONLY_NESTED = ["services/relay-restate/", "tools/xtask/"];
const RUSTLS = "pkg:cargo/rustls";

// A script function sees no top-level constant; `check` does, so it passes them.
fn is_binary_main(f) {
    f.name == "main" && f.file != ()
        && (f.file.ends_with("/src/main.rs") || f.file.contains("/src/bin/"))
}

fn in_scope(file, workspaces, ring_only) {
    workspaces.some(|w| file.starts_with(w)) && !ring_only.some(|w| file.starts_with(w))
}

fn check(g) {
    let out = [];
    let links_rustls = #{}; // package id -> whether its default build pulls rustls in
    for f in g.nodes("Function") {
        if !is_binary_main(f) || !in_scope(f.file, WORKSPACES, RING_ONLY_NESTED) { continue; }
        let pkg = g.package_of(f.id);
        if pkg == () { continue; }
        if !links_rustls.contains(pkg) {
            links_rustls[pkg] = g.cargo_build(pkg, true).packages.contains(RUSTLS);
        }
        if !links_rustls[pkg] { continue; }
        let installs = g.out(f.id, "CALLS").some(|c| c.name == "install_crypto_provider");
        if !installs {
            out.push(violation(f, "links rustls with both ring and aws-lc-rs; call causeway_platform::install_crypto_provider() first in main (components: causeway_runtime::install_crypto_provider())"));
        }
    }
    out
}
