use clap::Parser;
use historic::cmd::{Cli, Commands};
use historic::db::Db;
use historic::error::Result;
use historic::terminal::Terminal;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let terminal = Terminal::new()?;
    let db = Db::new().await?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL
    )",
        ("",),
    )
    .await?;

    db.execute("INSERT INTO users (username) VALUES (?)", ("alice",))
        .await?;
    db.execute("INSERT INTO users (username) VALUES (?)", ("bob",))
        .await?;

    db.query("SELECT * FROM users", ("",)).await?;

    println!(
        "current terminal, {} {}",
        terminal.multiplexer, terminal.session
    );

    match args.command {
        Some(Commands::Add { cmd: _ }) => {
            println!("Handle adding records");
        }
        None => {
            println!("Run the tui");
        }
    }

    Ok(())
}
