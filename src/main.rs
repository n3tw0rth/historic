use clap::Parser;
use historic::cmd::{Cli, Commands};
use historic::error::Result;
use historic::terminal::Terminal;

pub fn main() -> Result<()> {
    let args = Cli::parse();
    let terminal = Terminal::new()?;

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
