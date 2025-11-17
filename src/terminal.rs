use std::io::{BufReader, Read};
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

#[derive(Default)]
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
            let tmux_result = Command::new("tmux")
                .arg("display-message")
                .arg("-p")
                .arg("-F")
                .arg("#{session_name} #{window_index} #{pane_index}")
                .spawn()
                .expect("failed to run tmux")
                .stdout;

            let output = tmux_result.map_or(String::new(), |r| {
                let mut s = String::new();
                BufReader::new(r)
                    .read_to_string(&mut s)
                    .expect("read failed");
                s
            });

            let mut iter = output.split(' ');

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
        }

        Ok(Terminal::default())
    }
}

#[cfg(test)]
mod tests {}
