use std::path::PathBuf;
use std::{env, process::Command};
use strum_macros::Display;

use crate::error::Result;
use crate::utils;

/// Enum representing different types of terminal multiplexers

#[derive(Display, Default, Debug, Clone)]
pub enum TerminalMultiplexerType {
    #[strum(serialize = "tmux")]
    TMUX,
    #[strum(serialize = "zellij")]
    ZELLIJ,
    #[default]
    #[strum(serialize = "none")]
    NONE,
}

#[derive(Default, Debug, Clone)]
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

            Ok(Terminal {
                multiplexer: TerminalMultiplexerType::TMUX,
                session,
                window,
                pane,
                pwd,
            })
        } else {
            Ok(Terminal::default())
        }
    }

    /// Key that commands are saved under: one list per tmux pane, or a single
    /// shared list outside a multiplexer. The working directory is left out so
    /// a pane keeps the same list wherever you `cd`.
    pub fn session_id(&self) -> String {
        // Hashing the Debug output keeps the ids saved by earlier versions
        // valid outside tmux, where `pwd` was always empty.
        let key = Terminal {
            pwd: PathBuf::new(),
            ..self.clone()
        };
        utils::string_to_md5(&format!("{key:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmux_pane(pane: u8, pwd: &str) -> Terminal {
        Terminal {
            multiplexer: TerminalMultiplexerType::TMUX,
            session: "main".to_string(),
            window: 1,
            pane,
            pwd: PathBuf::from(pwd),
        }
    }

    #[test]
    fn test_session_id_ignores_working_directory() {
        assert_eq!(
            tmux_pane(0, "/home/me").session_id(),
            tmux_pane(0, "/home/me/projects").session_id()
        );
    }

    #[test]
    fn test_session_id_differs_per_pane() {
        assert_ne!(
            tmux_pane(0, "/home/me").session_id(),
            tmux_pane(1, "/home/me").session_id()
        );
    }

    #[test]
    fn test_session_id_outside_tmux_is_unchanged() {
        // Earlier versions saved commands under this id; changing it would
        // hide them.
        assert_eq!(
            Terminal::default().session_id(),
            "c1eb7cb0f78910aece76c7629f5e17cd"
        );
    }
}
