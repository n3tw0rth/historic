use std::sync::Arc;

use clap::Parser;
use color_eyre::Result;

use historic::cmd::{Cli, Cmd};
use historic::db::Db;
use historic::terminal::Terminal;
use historic::tui::ui::Tui;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let terminal = Arc::new(Terminal::new()?);
    let db = Arc::new(Db::new().await?);

    let cmd = Cmd::new(&args.command, terminal.clone(), db.clone()).await;
    let mut tui = Tui::new(terminal.clone(), db.clone());

    match args.command {
        None => {
            color_eyre::install()?;
            let ratatui_term = ratatui::init();
            let result = tui.run(ratatui_term);
            ratatui::restore();
            result.await?;
        }
        _ => {
            cmd.run().await?;
        }
    }

    Ok(())
}
