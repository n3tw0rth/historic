# Historic

A CLI tool for remembering and quickly accessing terminal commands across different sessions and panes.

## What it does

- Stores frequently used commands for quick recall
- Works with terminal multiplexers like tmux and zellij
- TUI interface built with ratatui for browsing and selecting commands
- Can be used standalone without a multiplexer

## Installation

### Prebuilt binaries

Download the archive for your platform from the
[Releases](https://github.com/n3tw0rth/historic/releases) page. Builds are
available for Linux (x86_64, aarch64) and macOS (Intel, Apple Silicon).

```bash
tar -xzf historic-<version>-<target>.tar.gz
sudo mv historic /usr/local/bin/
```

Each archive ships with a `.sha256` checksum file.

### From source

```bash
just install
```


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
