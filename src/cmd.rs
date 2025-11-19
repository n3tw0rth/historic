use crate::db::Db;
use crate::error::Result;
use crate::terminal::Terminal;
use crate::utils;
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
        cmd: Vec<String>,
    },
}

pub struct Cmd<'a> {
    commands: &'a Option<Commands>,
    term: Terminal,
    db: Db,
}

impl<'a> Cmd<'a> {
    pub async fn new(args: &'a Option<Commands>, term: Terminal, db: Db) -> Self {
        Cmd {
            commands: args,
            term,
            db,
        }
    }

    pub async fn run(&self) -> Result<()> {
        match &self.commands {
            Some(Commands::Add { cmd }) => self.handle_add(cmd).await,
            _ => Ok(()),
        }
    }

    async fn handle_add(&self, cmd: &Vec<String>) -> Result<()> {
        let term_debug = format!("{:?} ", self.term);
        let session_id = utils::string_to_md5(&term_debug);
        let joined_cmd = cmd.join(" ");
        self.db.rank_n_save_new(session_id, joined_cmd).await?;
        Ok(())
    }
}
