# supjj

An inline TUI flash card for [Jujutsu](https://github.com/jj-vcs/jj) version control.

Unlike full-screen TUIs, `supjj` renders a compact panel directly in your terminal without clearing the screen — like a dropdown cheat sheet. It shows your repo state, lets you compose a `jj` command with autocomplete, then executes it and gets out of the way.

```
┌─────────── Log ──────────────┬──── Status ─────┐
│ ○  abc123 Add feature        │ Working copy    │
│▌◆  def456 Fix typo           │ M src/main.rs   │
│ ○  ghi789 Initial commit     │ A src/new.rs    │
└──────────────────────────────┴─────────────────┘
  bookmark
  commit
  describe
> jj show def456█
```

## Features

- **Inline rendering** — occupies ~22 lines at your cursor, no alternate screen takeover
- **Two-column layout** — log graph (60%) and status (40%) side by side
- **ANSI color passthrough** — log and status render with the same colors as your terminal
- **Deep autocomplete** — full `jj` command tree: subcommands, sub-subcommands, and flags
- **Dynamic completions** — bookmark names fetched live from the repo
- **Revision highlighting** — as you type `-r`, `--from`, `--to`, etc., the referenced revision is highlighted in the log panel with a yellow `▌` marker
- **Debounced resolution** — revset expressions are resolved after 250ms of inactivity, keeping the UI responsive

## Install

Requires Rust 1.70+ and `jj` on your PATH.

```bash
git clone <repo-url> && cd super-jujutsu
cargo install --path .
```

Or build manually:

```bash
cargo build --release
cp target/release/supjj ~/bin/
```

## Usage

Run `supjj` from inside any Jujutsu repository:

```bash
supjj
```

The flash card appears inline showing:
- **Log panel** — recent commit graph with full colors and branch structure
- **Status panel** — working copy changes
- **Command input** — type your `jj` command with autocomplete

### Controls

| Key | Action |
|---|---|
| Type | Compose your `jj` command |
| Tab | Accept the top autocomplete suggestion |
| ↑ / ↓ | Cycle through suggestions |
| Enter | Dismiss the card and execute `jj <command>` |
| Esc / Ctrl-C | Dismiss without executing |
| ← / → | Move cursor in input |
| Backspace | Delete character before cursor |

### Autocomplete

`supjj` provides deep autocomplete for the full `jj` command tree:

- **Top-level commands** — `log`, `bookmark`, `git`, `describe`, etc.
- **Nested subcommands** — `bookmark advance`, `git remote add`, `operation log`, etc.
- **Flags** — `--limit`, `--revision`, `--no-graph`, etc. (context-aware per command)
- **Dynamic values** — bookmark names are fetched live from the repo

### Revision Highlighting

When your command references a revision, the log panel highlights it in real time:

```
> jj rebase -r abc --onto main
```

Both `abc` and `main` will be highlighted in the log with a yellow `▌` marker and dark background. This works with:
- Flags: `-r`, `--revision`, `--from`, `--to`, `--onto`, `--into`, `--source`
- Positional args: `show <rev>`, `edit <rev>`, `abandon <rev>`, etc.
- Revset expressions: `@-`, `heads(mutable())`, `abc | def`

Resolution is debounced at 250ms so the UI stays snappy while you type.

## Architecture

```
src/
├── main.rs      # Entry point, clap CLI, repo check, command execution
├── jj.rs        # Shell out to jj for log, status, bookmark list, revset resolution
├── ansi.rs      # ANSI SGR escape code → Ratatui Style/Span parser
├── commands.rs  # Static command tree + revset extraction from input
└── tui.rs       # Ratatui inline viewport, rendering, input, highlighting
```

### Key design decisions

- **Viewport::Inline** — Ratatui renders ~22 lines at the cursor position without entering alternate screen. Your terminal history stays intact.
- **Static command tree** — The full `jj` CLI hierarchy is embedded at compile time for instant autocomplete with zero startup cost.
- **Execute and dismiss** — After Enter, the TUI tears down and `jj` runs with inherited stdio, so output appears naturally in your terminal.
- **Debounced revset resolution** — Revision expressions are extracted from the input, debounced at 250ms, then resolved via `jj log -r <revset>` to get change IDs for highlighting.

## License

MIT
