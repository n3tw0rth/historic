# Historic

A CLI tool for remembering and quickly accessing terminal commands across different sessions and panes.

## What it does

- Stores frequently used commands for quick recall
- Works with terminal multiplexers like tmux and zellij
- TUI interface built with ratatui for browsing and selecting commands
- Can be used standalone without a multiplexer

## Installation

update `.bashrc`
```bash
function __historic_hook() {
    old_cmd="$(history 1 | sed 's/^[ ]*[0-9]\+[ ]*//')"
    if [[ -n "$old_cmd" && "$old_cmd" != *historic* ]]; then
      \command historic add "${old_cmd}"
    fi
}

# Initialize hook.
if [[ ${PROMPT_COMMAND:=} != *'__historic_hook'* ]]; then
    PROMPT_COMMAND="__historic_hook;${PROMPT_COMMAND#;}"
fi
```

## Usage

```bash
# not yet ready to be used
```

## Features

- Command history storage
- Interactive TUI for command selection
- Terminal multiplexer integration
- Persistent command database

## Requirements

- Rust (for building from source)
- Optional: tmux or zellij for multiplexer features
