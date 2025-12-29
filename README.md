# Historic

A CLI tool for remembering and quickly accessing terminal commands across different sessions and panes.

## What it does

- Stores frequently used commands for quick recall
- Works with terminal multiplexers like tmux and zellij
- TUI interface built with ratatui for browsing and selecting commands
- Can be used standalone without a multiplexer

## Installation

Please install from the source for now


## Usage

```bash
__historic__() {
  export SELECTED="$(historic | sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g')"
  READLINE_LINE="${READLINE_LINE:0:$READLINE_POINT}$SELECTED${READLINE_LINE:$READLINE_POINT}"
  READLINE_POINT=$(( READLINE_POINT + ${#SELECTED} ))
}
bind -x '"\C-t":"__historic__"'
```

## Features

- Command history storage
- Interactive TUI for command selection
- Terminal multiplexer integration
- Persistent command database

## Requirements

- Rust (for building from source)
- Optional: tmux or zellij for multiplexer features
