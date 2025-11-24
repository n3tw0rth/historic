use std::sync::Arc;

use self::db::Db;
use self::error::{Error, Result};
use self::terminal::Terminal;
use self::tui::event::{Event, EventHandler};
use self::tui::ui::Tui;

pub mod cmd;
pub mod db;
pub mod error;
pub mod terminal;
pub mod utils;
pub mod tui {
    pub mod event;
    pub mod ui;
}

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
    let ratatui_term = ratatui::init();
    let result = tui.run(ratatui_term, items).await;
    ratatui::restore();
    result?;

    Ok(())
}
