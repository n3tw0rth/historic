use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "A CLI to remember commands you run")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add a command to the database
    Add {
        #[clap(required = true)]
        cmd: String,
    },
}
