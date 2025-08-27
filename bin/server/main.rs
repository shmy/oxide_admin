use anyhow::Result;
use clap::Parser as _;
use infrastructure::shared::{config::Config, pool};
use server::cli::Cli;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv_override().ok();
    let cli: Cli = Cli::parse();
    let config: Config = cli.try_into()?;
    let pool = pool::try_new(&config.database).await?;
    server::bootstrap(pool, config).await
}
