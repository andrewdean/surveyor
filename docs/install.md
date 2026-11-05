# Installing and releasing Surveyor

Surveyor is distributed as prebuilt binaries attached to GitHub releases of
[andrewdean/surveyor](https://github.com/andrewdean/surveyor), under Apache-2.0
(RFD section 17, item 3, decided 2026-11-04). The crates are not published to
crates.io. This note records how the distribution was set up on 2026-11-04, what an
adopter runs, and what a maintainer does to cut the next release.

## For adopters

Each release carries one archive per target and a `SHA256SUMS` file:

| Archive | Built on |
|---|---|
| `surveyor-<version>-aarch64-apple-darwin.tar.gz` | macOS 15 runner |
| `surveyor-<version>-x86_64-apple-darwin.tar.gz` | macOS 15 runner, cross-compiled |
| `surveyor-<version>-x86_64-unknown-linux-gnu.tar.gz` | Ubuntu 22.04 runner (glibc 2.35 or newer at run time) |

Each archive unpacks to a directory of the same name holding `surveyor`, `LICENSE` and
`README.md`.

### Homebrew (macOS and Linux)

```bash
brew install andrewdean/tap/surveyor
```

The formula lives in [andrewdean/homebrew-tap](https://github.com/andrewdean/homebrew-tap)
as `Formula/surveyor.rb`. `brew upgrade surveyor` follows later releases once the tap is
updated.

### cargo-binstall

```bash
cargo binstall surveyor --git https://github.com/andrewdean/surveyor
```

`[package.metadata.binstall]` in `apps/surveyor/Cargo.toml` names the archive layout, so
binstall downloads the archive for the host target instead of compiling. Add
`--install-path <dir>` to put the binary somewhere other than `~/.cargo/bin`.

### cargo install from source

```bash
cargo install --git https://github.com/andrewdean/surveyor --tag v0.1.0 surveyor
```

Building needs the toolchain `rust-toolchain.toml` names (rustup installs it on first
use) and `protoc` on the path: the Lance crates compile their protobuf schemas in build
scripts. Expect a cold build to take some minutes; the dependency tree includes Lance and
DataFusion.

### Checking an install

```bash
surveyor --version
```

prints `surveyor <version>`. The MCP server is the same binary (`surveyor mcp`), so an
`.mcp.json` that names `surveyor` as its command works with any of the three installs.

### Verifying a download by hand

```bash
curl -fsSLO https://github.com/andrewdean/surveyor/releases/download/v0.1.0/SHA256SUMS
curl -fsSLO https://github.com/andrewdean/surveyor/releases/download/v0.1.0/surveyor-0.1.0-aarch64-apple-darwin.tar.gz
shasum -a 256 -c --ignore-missing SHA256SUMS
```

## For maintainers

### What runs where

- `.github/workflows/ci.yml` runs the gate (`cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
  `cargo deny check`, `cargo machete`) on every pull request and push to `main`. It also
  answers `workflow_call`, which is how the release workflow reuses it.
- `.github/workflows/release.yml` runs on a `v*` tag: the gate, a check that the tag
  matches `workspace.package.version` in `Cargo.toml`, one build job per target
  (`cargo build --release --locked --package surveyor --target <target>`), a smoke test of
  the packaged binary on the two targets the runner can execute, then a GitHub release
  with the three archives and `SHA256SUMS`. A manual run (`workflow_dispatch`) builds the
  same archives from any ref and keeps them as workflow artifacts without publishing a
  release; use it to rehearse the pipeline before tagging.
- `packaging/homebrew/surveyor.rb.in` is the formula template;
  `packaging/homebrew/render-formula.sh <version>` fills it from the release's
  `SHA256SUMS`.

### Cutting a release

1. Bump `version` under `[workspace.package]` in `Cargo.toml`, run `cargo build` so
   `Cargo.lock` follows, and commit on `main`. The release workflow refuses a tag whose
   version differs from the manifest.
2. Rehearse if anything in the build changed:
   `gh workflow run release.yml --ref main`, then watch it with `gh run watch`.
3. Tag and push:

   ```bash
   git tag -a v0.2.0 -m "surveyor 0.2.0"
   git push origin v0.2.0
   ```

4. Watch the run (`gh run list --workflow release.yml`, `gh run watch <id>`) and confirm
   the assets: `gh release view v0.2.0` lists three `.tar.gz` files and `SHA256SUMS`.
5. Update the tap:

   ```bash
   packaging/homebrew/render-formula.sh 0.2.0 > ../homebrew-tap/Formula/surveyor.rb
   git -C ../homebrew-tap commit -am "surveyor 0.2.0" && git -C ../homebrew-tap push
   ```

6. Check an install from a clean prefix (see below).

### Verifying a release from a clean environment

cargo-binstall into a throwaway `CARGO_HOME`:

```bash
tmp="$(mktemp -d)"
CARGO_HOME="$tmp" cargo binstall --no-confirm surveyor --git https://github.com/andrewdean/surveyor
"$tmp/bin/surveyor" --version
```

Homebrew into a throwaway prefix (a second, untapped Homebrew; it downloads the archive,
nothing is compiled):

```bash
tmp="$(mktemp -d)"
git clone --depth 1 https://github.com/Homebrew/brew "$tmp/brew"
"$tmp/brew/bin/brew" install andrewdean/tap/surveyor
"$tmp/brew/bin/surveyor" --version
```

## Record of the first release (2026-11-04 to 2026-11-05)

What happened, in order, so the next release and the Causeway wiring can cite it.

1. `7d5378a` added `ci.yml`, `release.yml`, the binstall metadata, the formula
   template and this note; the gate was green locally first.
2. Andrew created the empty public repository `andrewdean/surveyor`; `main` was pushed to
   it. The first CI run (41963024498) passed.
3. A manual run of `release.yml` (41963042417) rehearsed the pipeline: the cold gate took
   55 minutes on the Ubuntu runner and each target build about 30 minutes after it. The
   `aarch64-apple-darwin` artifact was downloaded, its checksum verified, and its binary
   run on an Apple silicon Mac.
4. `10ebf08` (a README path fix) was pushed and tagged `v0.1.0`. Run 42043171310
   published <https://github.com/andrewdean/surveyor/releases/tag/v0.1.0> with:

   | Asset | Size |
   |---|---|
   | `surveyor-0.1.0-aarch64-apple-darwin.tar.gz` | 76 MB |
   | `surveyor-0.1.0-x86_64-apple-darwin.tar.gz` | 81 MB |
   | `surveyor-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | 85 MB |
   | `SHA256SUMS` | 330 bytes |

5. `cargo binstall --no-confirm surveyor --git https://github.com/andrewdean/surveyor`
   into an empty `CARGO_HOME`, with an empty environment, downloaded the arm64 archive and
   installed a binary that prints `surveyor 0.1.0`.
6. `packaging/homebrew/render-formula.sh 0.1.0` produced `Formula/surveyor.rb`;
   `andrewdean/homebrew-tap` was created from it (commit `29a6896`).
   `brew install andrewdean/tap/surveyor` in a fresh Homebrew clone under a throwaway
   prefix tapped the repository from GitHub, installed the archive, printed
   `surveyor 0.1.0`, and passed `brew test surveyor`. The prefix was deleted afterwards.

Things learned on the way:

- Homebrew refuses a prefix under its temporary directory (`/private/tmp` on macOS) and
  its build sandbox fails a self-check there too; put a throwaway prefix under `$HOME`.
- `arduino/setup-protoc` downloads `protoc` from GitHub releases on every run; one run
  failed on a 504 and passed on the next push. Rerun the job when that happens.
- The release binary is 218 MB (Lance, DataFusion and the Tree-sitter grammars);
  `strip` would take it to 173 MB at the cost of symbol names in backtraces. The release
  profile was left as it was.
- The Linux archive is built on Ubuntu 22.04 and links glibc 2.35, so it runs on Debian
  12 and Ubuntu 22.04 or newer.
