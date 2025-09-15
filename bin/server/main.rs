use anyhow::Result;
use clap::Parser as _;
use infrastructure::shared::config::Config;
use server::cli::{Cli, Commands};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv_override().ok();
    let cli: Cli = Cli::parse();
    let command = cli.command.clone();
    let config: Config = cli.try_into()?;
    match command {
        Commands::Serve => {
            if let Err(e) = server::serve(config).await {
                eprintln!("❌ Serve error: {:?}", e);
                std::process::exit(1);
            }
        }
        #[cfg(not(feature = "serve_with_sched"))]
        Commands::Sched => {
            if let Err(e) = server::sched(config).await {
                eprintln!("❌ Sched error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
