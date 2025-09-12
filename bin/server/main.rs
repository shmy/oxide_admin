use anyhow::Result;
use clap::Parser as _;
use infrastructure::shared::config::Config;
use server::cli::Cli;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv_override().ok();
    let cli: Cli = Cli::parse();
    let config: Config = cli.try_into()?;
    if let Err(e) = server::bootstrap(config).await {
        eprintln!("❌ Bootstrap error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
