//! Surveyor command line.
//!
//! `surveyor index` walks a checkout, extracts facts, links them and writes the graph
//! to a Lance dataset under `.surveyor/`. The query commands (`explain`, `affected`,
//! `path`, `query`, `hubs`, `communities`, `source`, `stats`) load that dataset and
//! answer with citations. `surveyor mcp` serves the same answers over MCP on stdio.

// Rust guideline compliant 2026-02-21

mod app;
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
