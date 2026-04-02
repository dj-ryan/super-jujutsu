# Architecture

## Overview

`supjj` is a lightweight inline TUI that acts as a flash card helper for the Jujutsu VCS. It occupies ~22 terminal lines without clearing the screen, shows repo context, and provides an autocomplete-enabled command prompt.

## Data Flow

```
1. User runs `supjj`
2. main.rs checks we're in a jj repo (jj root)
3. Fetches jj log + jj status (with ANSI colors)
4. Launches Ratatui with Viewport::Inline(22)
5. Renders log | status side-by-side, suggestions, and input
6. User types → tokenize → query command tree → show completions
7. On Enter: tear down TUI, exec jj <args> with inherited stdio
8. On Esc: tear down TUI, exit cleanly
```

## Modules

### main.rs
Entry point. Parses CLI args via clap (`--help`, `--version`). Checks repo, fetches data, runs TUI, then executes the resulting command.

### jj.rs
Thin wrapper around `std::process::Command` to call `jj`. Functions:
- `in_repo()` — runs `jj root` to detect if we're in a repo
- `log()` — `jj log --limit 10 --color=always`
- `status()` — `jj status --color=always`
- `bookmark_names()` — `jj bookmark list --color=never`, parsed into names

### ansi.rs
Parses ANSI SGR escape sequences into Ratatui `Style`/`Span` objects. Supports:
- Standard colors (30-37 fg, 40-47 bg)
- Bright colors (90-97 fg, 100-107 bg)
- 256-color mode (`38;5;N` / `48;5;N`)
- RGB mode (`38;2;R;G;B` / `48;2;R;G;B`)
- Modifiers: bold, dim, italic, underline, reverse

### commands.rs
Static command tree representing the full `jj` CLI hierarchy. Built at startup via `build_tree()`. Structure:
- `CommandTree` nodes with children (subcommands) and flags
- `completions(&[tokens])` — given the current input tokens, returns matching completions
- `expects_bookmark_arg(&[tokens])` — detects when dynamic bookmark names should be suggested

Covers all ~45 top-level commands, nested subcommands (bookmark, git, config, file, operation, sparse, tag, workspace, util), and 3rd-level nesting (git remote, git colocation).

### tui.rs
The TUI runtime. Key components:
- `App` struct — holds input state, cursor, suggestions, command tree, bookmark cache
- `run()` — sets up Ratatui with `Viewport::Inline`, installs panic hooks, runs event loop
- `event_loop()` — reads crossterm events, dispatches to input handling
- `render()` — two-column layout (log 60% | status 40%), suggestion list, input line with ghost text

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
