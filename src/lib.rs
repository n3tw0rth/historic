use std::io;
use std::sync::Arc;

use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::prelude::{CrosstermBackend, Terminal as RatTerminal};

use self::db::Db;
use self::error::{Error, Result};
use self::terminal::Terminal;
use self::tui::event::{Event, EventHandler};
use self::tui::ui::Tui;

pub mod cmd;
pub mod db;
pub mod error;
pub mod terminal;
pub mod tracing;
pub mod tui;
pub mod utils;

pub async fn start_tui(term: Arc<Terminal>, db: Arc<Db>) -> Result<()> {
    let mut tui = Tui::new();

    let session_id = utils::string_to_md5(&format!("{:?} ", term));
    let mut rows = db.get_commands(&session_id).await?;

    let mut items = Vec::new();
    while let Some(row) = rows.next().await? {
        let r: String = row.get(4)?;
        items.push(r);
    }

    color_eyre::install().map_err(|e| Error::Unknown { msg: e.to_string() })?;

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let rat_terminal = RatTerminal::new(backend)?;

    let result = tui.run(rat_terminal, items).await;
    ratatui::restore();

    let selection = result?.unwrap_or_default();
    prefill_buff(selection)?;

    Ok(())
}

fn prefill_buff(selected: String) -> Result<()> {
    println!("{}", selected);

    Ok(())
}
