use clap::Parser;
use color_eyre::Result;
use std::sync::Arc;

use historic::cmd::{Args, Cmd};
use historic::db::Db;
use historic::terminal::Terminal;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();
    let terminal = Arc::new(Terminal::new()?);
    let db = Arc::new(Db::new().await?);

    let cmd = Cmd::new(&args.command, terminal.clone(), db.clone()).await;

    match args.command {
        None => {
            historic::start_tui(terminal, db).await?;
        }
        _ => {
            cmd.run().await?;
        }
    }

    Ok(())
}
