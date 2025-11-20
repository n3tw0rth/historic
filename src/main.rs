use clap::Parser;
use color_eyre::Result;

use historic::cmd::{Cli, Cmd};
use historic::db::Db;
use historic::terminal::Terminal;
use historic::tui::ui::Tui;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let terminal = Terminal::new()?;
    let db = Db::new().await?;
    let cmd = Cmd::new(&args.command, terminal, db).await;

    match args.command {
        None => {
            color_eyre::install()?;
            let terminal = ratatui::init();
            let result = Tui::new().run(terminal);
            ratatui::restore();
            result?;
        }
        _ => {
            cmd.run().await?;
        }
    }

    Ok(())
}
