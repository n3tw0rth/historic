<div align="center">

# historic

**Remember the commands you run, find them again in a keystroke.**

[![Rust](https://img.shields.io/badge/rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![ratatui](https://img.shields.io/badge/tui-ratatui-blueviolet)](https://ratatui.rs)
[![SQLite](https://img.shields.io/badge/storage-sqlite-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org)
[![tmux](https://img.shields.io/badge/works%20with-tmux-1BB91F?logo=tmux&logoColor=white)](https://github.com/tmux/tmux)
[![Last commit](https://img.shields.io/github/last-commit/n3tw0rth/historic)](https://github.com/n3tw0rth/historic/commits/main)
![Status](https://img.shields.io/badge/status-early%20development-yellow)

</div>

---

`historic` stores your terminal commands in a local SQLite database and gives you a fuzzy-searchable TUI to pull them back into your prompt, across sessions, windows and panes.

## Features

- **Fuzzy search** through everything you've saved
- **Session-aware**: detects tmux session, window and pane
- **Persistent**: one SQLite database, safe to use from many terminals at once
- **Standalone**: works fine without a multiplexer

## Install

Build from source (requires Rust):

```bash
git clone https://github.com/n3tw0rth/historic.git
cd historic
cargo install --path .
```

Or, with [`just`](https://github.com/casey/just): `just install`.

## Usage

```bash
historic                 # open the picker
historic add <command>   # save a command
```

### Shell integration (bash)

Bind the picker to <kbd>Ctrl</kbd>+<kbd>T</kbd> and insert the selection at the cursor:

```bash
__historic__() {
  local selected
  selected="$(historic | sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g')"
  READLINE_LINE="${READLINE_LINE:0:$READLINE_POINT}$selected${READLINE_LINE:$READLINE_POINT}"
  READLINE_POINT=$(( READLINE_POINT + ${#selected} ))
}
bind -x '"\C-t":"__historic__"'
```

Save every command automatically after it runs:

```bash
__historic_add__() { historic add "$(history 1 | sed 's/^ *[0-9]* *//')"; }
PROMPT_COMMAND="__historic_add__${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

### Keys

| Key | Action |
| --- | --- |
| <kbd>i</kbd> | Start searching |
| <kbd>Esc</kbd> | Stop searching |
| <kbd>j</kbd> / <kbd>k</kbd> | Move selection |
| <kbd>Enter</kbd> | Pick command |
| <kbd>q</kbd> | Quit |

## Data

Everything lives in `~/.config/historic/historic.db` (or your platform's config directory). Nothing leaves your machine.
