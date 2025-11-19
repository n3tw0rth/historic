use std::path::PathBuf;
use std::{env, process::Command};
use strum_macros::Display;

use crate::error::Result;

/// Enum representing different types of terminal multiplexers

#[derive(Display, Default, Debug)]
pub enum TerminalMultiplexerType {
    #[strum(serialize = "tmux")]
    TMUX,
    #[strum(serialize = "zellij")]
    ZELLIJ,
    #[default]
    #[strum(serialize = "none")]
    NONE,
}

#[derive(Default, Debug)]
pub struct Terminal {
    pub multiplexer: TerminalMultiplexerType,
    pub session: String,
    pub window: u8,
    pub pane: u8,
    /// session current working directory
    pub pwd: PathBuf,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let is_tmux = env::var("TMUX").is_ok();
        let pwd: PathBuf = env::current_dir()?;

        if is_tmux {
            let output = Command::new("tmux")
                .arg("display-message")
                .arg("-p")
                .arg("-F")
                .arg("#{session_name} #{window_index} #{pane_index}")
                .output()?;

            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut iter = result.split(' ');

            let session = iter.next().unwrap_or("").to_string();
            let window = iter.next().unwrap_or("").parse::<u8>().unwrap_or_default();
            let pane = iter.next().unwrap_or("").parse::<u8>().unwrap_or_default();

            return Ok(Terminal {
                multiplexer: TerminalMultiplexerType::TMUX,
                session,
                window,
                pane,
                pwd,
            });
        } else {
            return Ok(Terminal::default());
        }
    }
}

#[cfg(test)]
mod tests {}
