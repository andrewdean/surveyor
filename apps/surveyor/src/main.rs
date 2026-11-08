//! Surveyor command line.
//!
//! `surveyor index` walks a checkout, extracts facts, links them and writes the graph
//! to a Lance dataset under `.surveyor/`. The query commands (`explain`, `affected`,
//! `path`, `query`, `hubs`, `communities`, `source`, `stats`) load that dataset and
//! answer with citations. `surveyor mcp` serves the same answers over MCP on stdio.
//! With `--session <id>` (or `SURVEYOR_SESSION`) every answer is also recorded in a
//! session ledger; `surveyor session` and `surveyor hook claude-code` work that
//! ledger for an agent harness, and `surveyor pr` turns a change into the comment
//! and gate a pull request carries.

// Rust guideline compliant 2026-02-21

mod app;
mod eval;
mod harness;
mod pr;
mod render;

use clap::Parser;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = app::Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(if cli.verbose { "info" } else { "warn" })
            }),
        )
        .with_writer(std::io::stderr)
        .init();
    app::run(cli).await
}
