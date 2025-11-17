use historic::error::HResult;
use historic::terminal::Terminal;

pub fn main() -> HResult<()> {
    let terminal = Terminal::new()?;

    println!(
        "current terminal, {} {}",
        terminal.multiplexer, terminal.session
    );

    Ok(())
}
