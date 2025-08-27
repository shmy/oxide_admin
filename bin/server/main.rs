use anyhow::Result;
use clap::Parser as _;
use infrastructure::shared::config::Config;
use infrastructure::shared::pool;
use server::cli::Cli;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv_override().ok();
    let cli: Cli = Cli::parse();
    let config: Config = cli.try_into()?;
    server::bootstrap(config).await
}
