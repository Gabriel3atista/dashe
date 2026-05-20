mod cli;
mod prompt;
mod git;
mod theme;
mod config;
mod alias;
mod sync;
mod doctor;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();
    cli.run().await
}
