use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Generate code
pub struct Cli {
    // command
    #[command(subcommand)]
    pub command: SubCommands,
    /// module name
    #[arg(long, short)]
    pub module: String,

    /// entity name
    #[arg(long, short)]
    pub entity: String,

    /// table name, default: same to entity name
    #[arg(long, short)]
    pub table: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    Scaffold,
    Api,
    Application,
    Domain,
    Repository,
    // /// CommandHandler
    // Ch,
    // /// QueryHandler
    // Qh,
}
