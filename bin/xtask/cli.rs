use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Generate code
pub struct Cli {
    // command
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    Scaffold(SubCommandArgs),
    Api(SubCommandArgs),
    Application(SubCommandArgs),
    Domain(SubCommandArgs),
    Repository(SubCommandArgs),
    // /// CommandHandler
    // Ch,
    // /// QueryHandler
    // Qh,
}

#[derive(Debug, Args)]
pub struct SubCommandArgs {
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
