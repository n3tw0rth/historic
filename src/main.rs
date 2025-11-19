use clap::Parser;
use historic::cmd::{Cli, Cmd};
use historic::db::Db;
use historic::error::Result;
use historic::terminal::Terminal;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let terminal = Terminal::new()?;
    let db = Db::new().await?;
    let cmd = Cmd::new(&args.command, terminal, db).await;

    match args.command {
        None => {
            println!("Run the tui");
        }
        _ => {
            cmd.run().await?;
            println!("Handle adding records");
        }
    }

    Ok(())
}
