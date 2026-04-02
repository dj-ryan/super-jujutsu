# Architecture

## Overview

`supjj` is a lightweight inline TUI that acts as a flash card helper for the Jujutsu VCS. It occupies ~22 terminal lines without clearing the screen, shows repo context with revision highlighting, and provides an autocomplete-enabled command prompt.

## Data Flow

```
1. User runs `supjj`
2. main.rs checks we're in a jj repo (jj root)
3. Fetches jj log + jj status (with ANSI colors)
4. Launches Ratatui with Viewport::Inline(22)
5. Renders log | status side-by-side, suggestions, and input
6. User types → tokenize → query command tree → show completions
7. Input changes → extract revset args → debounce 250ms → resolve → highlight log
8. On Enter: tear down TUI, exec jj <args> with inherited stdio
9. On Esc: tear down TUI, exit cleanly
```

## Modules

### main.rs
Entry point. Parses CLI args via clap (`--help`, `--version`). Checks repo, fetches data, runs TUI, then executes the resulting command.

### jj.rs
Thin wrapper around `std::process::Command` to call `jj`. Functions:
- `in_repo()` — runs `jj root` to detect if we're in a repo
- `log()` — `jj log --limit 10 --color=always`
- `status()` — `jj status --color=always`
- `resolve_revset(revset)` — resolves a revset expression to short change IDs via `jj log -r <revset> --template 'change_id.shortest(8)'`
- `bookmark_names()` — `jj bookmark list --color=never`, parsed into names

### ansi.rs
Parses ANSI SGR escape sequences into Ratatui `Style`/`Span` objects. Supports:
- Standard colors (30-37 fg, 40-47 bg)
- Bright colors (90-97 fg, 100-107 bg)
- 256-color mode (`38;5;N` / `48;5;N`)
- RGB mode (`38;2;R;G;B` / `48;2;R;G;B`)
- Modifiers: bold, dim, italic, underline, reverse

Exposes both `parse_ansi_text()` (full string → Vec<Line>) and `parse_ansi_line()` (single line) for use by the highlighting system.

### commands.rs
Static command tree representing the full `jj` CLI hierarchy. Key components:
- `CommandTree` nodes with children (subcommands) and flags
- `completions(&[tokens])` — given the current input tokens, returns matching completions
- `expects_bookmark_arg(&[tokens])` — detects when dynamic bookmark names should be suggested
- `extract_revsets(&[tokens])` — parses revision arguments from the input (flags like `-r`, `--from`, `--to`, `--onto`, `--source`, and positional args for commands like `show`, `edit`, `abandon`)

Covers all ~45 top-level commands, nested subcommands (bookmark, git, config, file, operation, sparse, tag, workspace, util), and 3rd-level nesting (git remote, git colocation).

### tui.rs
The TUI runtime. Key components:
- `App` struct — holds input state, cursor, suggestions, command tree, bookmark cache, highlight state (highlighted_ids, last_revset, debounce timer)
- `run()` — sets up Ratatui with `Viewport::Inline`, installs panic hooks, runs event loop
- `event_loop()` — polls with timeout for debounce, reads crossterm events, dispatches to input handling, triggers revset resolution when debounce expires
- `render()` — two-column layout (log 60% | status 40%), suggestion list, input line with ghost text
- `render_log()` — parses ANSI per-line, checks each line against highlighted change IDs, applies yellow `▌` marker and dark background to matching lines
- `line_contains_id()` — strips ANSI from a raw line and checks if any highlighted change ID is present

## Revision Highlighting Pipeline

```
Input: "rebase -r abc --onto main"
  ↓
extract_revsets() → ["abc", "main"]
  ↓ (debounce 250ms)
resolve_revset("abc | main") → ["abc1234", "mai5678"]
  ↓
render_log() checks each line for these IDs
  ↓
Matching lines get: ▌ marker + bg(236) on all spans
```

The event loop uses `event::poll(timeout)` with a short timeout (50ms) when a debounce is pending, and a longer timeout (500ms) otherwise. This keeps CPU usage minimal while ensuring responsive highlighting.

## Adding New Commands

When `jj` adds new subcommands, update `build_tree()` in `commands.rs`:

```rust
.child("new-command", CommandTree::new(&["--flag1", "--flag2"])
    .child("subcommand", CommandTree::new(&["--subflag"])))
```

## Adding Dynamic Completions

To add dynamic completions for a new context (e.g. tag names):

1. Add a fetch function in `jj.rs`
2. Add a detection function in `commands.rs` (like `expects_bookmark_arg`)
3. Wire it into `App::update_suggestions()` in `tui.rs`

## Future: Level 3 — Auto-Expanding Log

The current architecture is designed to support auto-expanding the log to include referenced revisions that aren't in the default `--limit 10` view. The implementation path:

1. In `try_resolve_revsets()`, after resolving change IDs, check if any are missing from the current log output
2. If missing, re-fetch the log with an expanded revset: `jj log -r 'ancestors(visible_heads(), 10) | <target_revs>' --color=always`
3. Replace the static `log_output` with the new dynamic output
4. The rest of the highlighting pipeline works unchanged

This would make the log panel fully contextual — always showing what's relevant to the command being composed.
