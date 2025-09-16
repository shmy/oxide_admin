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
    /// Generate api application domain repository together
    Scaffold(SubCommandArgs),
    /// Generate api only
    Api(SubCommandArgs),
    /// Generate application only
    Application(SubCommandArgs),
    /// Generate domain only
    Domain(SubCommandArgs),
    /// Generate repository only
    Repository(SubCommandArgs),
    /// Generate frontend only
    Frontend(SubCommandArgs),
    /// Generate command and command handler
    Command,
    /// Generate query and query handler
    Query,
    /// Generate background worker
    Worker,
    /// Generate scheduled job
    // Job,
    /// Generate event subscriber
    Event,
}

#[derive(Debug, Args)]
pub struct SubCommandArgs {
    /// module name
    #[arg(long, short)]
    pub module: String,

    /// entity name
    #[arg(long, short)]
    pub entity: String,

    /// table name, default: plural to entity name
    #[arg(long, short)]
    pub table: Option<String>,
}
