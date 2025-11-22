use clap::Parser;
use color_eyre::Result;
use std::sync::Arc;

use historic::cmd::{Args, Cmd};
use historic::db::Db;
use historic::terminal::Terminal;
use historic::tui::event::EventHandler;
use historic::tui::ui::Tui;
use historic::utils;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();
    let terminal = Arc::new(Terminal::new()?);
    let db = Arc::new(Db::new().await?);
    let events = EventHandler::new();

    let cmd = Cmd::new(&args.command, terminal.clone(), db.clone()).await;
    let mut tui = Tui::new(events);

    let session_id = utils::string_to_md5(&format!("{:?} ", terminal));
    let mut rows = db.get_commands(&session_id).await?;

    let mut items = Vec::new();
    while let Some(row) = rows.next().await? {
        let r: String = row.get(4)?;
        items.push(r);
    }

    match args.command {
        None => {
            color_eyre::install()?;
            let ratatui_term = ratatui::init();
            let result = tui.run(ratatui_term, items).await;
            ratatui::restore();
            result?;
        }
        _ => {
            cmd.run().await?;
        }
    }

    Ok(())
}
