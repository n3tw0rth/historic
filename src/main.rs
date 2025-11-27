use clap::Parser;
use color_eyre::Result;
use std::sync::Arc;
use tracing::info;

use historic::{
    cmd::{Args, Cmd},
    db::Db,
    terminal::Terminal,
    tracing::Tracing,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    Tracing::new()?;
    info!("application started");

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
