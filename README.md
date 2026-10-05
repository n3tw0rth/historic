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
__historic_add__() {
  local entry
  entry="$(HISTTIMEFORMAT= builtin history 1)"
  # Skip the shell's first prompt, and any prompt where history didn't change
  # (an empty line, Ctrl+C or a space-prefixed command).
  if [[ ${__historic_last__+set} && $entry != "$__historic_last__" ]]; then
    historic add -- "$(sed '1s/^ *[0-9]*[* ] //' <<<"$entry")"
  fi
  __historic_last__=$entry
}
PROMPT_COMMAND="__historic_add__${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

Put both snippets at the **end** of `~/.bashrc`, so nothing that runs later replaces `PROMPT_COMMAND`. Then open a new shell. Panes and terminals that were already open won't have the hook until you run `source ~/.bashrc` in them.

Inside tmux, each pane has its own list. The picker shows every command saved from that pane, whichever directory you ran it in. Outside tmux, every terminal shares one list. The hook saves what bash adds to its history. With `HISTCONTROL=ignorespace` or `ignoreboth` (the Ubuntu default), commands that start with a space aren't saved.

### Keys

The picker opens ready to search: just type, then press <kbd>Enter</kbd>. Space-separated words can match in any order, and the search is case-sensitive only if you type an uppercase letter.

| Key | Action |
| --- | --- |
| <kbd>Enter</kbd> | Pick command |
| <kbd>↑</kbd> / <kbd>↓</kbd>, <kbd>Ctrl</kbd>+<kbd>P</kbd> / <kbd>Ctrl</kbd>+<kbd>N</kbd> | Move selection |
| <kbd>Ctrl</kbd>+<kbd>U</kbd> / <kbd>Ctrl</kbd>+<kbd>W</kbd> | Clear search / delete last word |
| <kbd>Esc</kbd> | Stop searching and browse |
| <kbd>j</kbd> / <kbd>k</kbd> | Move selection (browsing) |
| <kbd>i</kbd> or <kbd>/</kbd> | Start searching again (browsing) |
| <kbd>q</kbd> or <kbd>Esc</kbd> | Quit (browsing) |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | Quit |

## Data

Everything lives in `~/.config/historic/historic.db` (or your platform's config directory). Nothing leaves your machine.
