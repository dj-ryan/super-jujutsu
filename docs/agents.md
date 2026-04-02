# Agent Guide

How to make changes to the `supjj` repository.

## Project Structure

```
super-jujutsu/
├── Cargo.toml         # Dependencies: ratatui, crossterm, color-eyre, clap
├── README.md
├── docs/
│   ├── architecture.md
│   └── agents.md      # This file
└── src/
    ├── main.rs        # Entry point, CLI, repo check, command execution
    ├── jj.rs          # Shell out to jj binary
    ├── ansi.rs        # ANSI escape code → Ratatui Style parser
    ├── commands.rs    # Static command tree + revset extraction
    └── tui.rs         # Inline viewport, rendering, input, highlighting
```

## Build & Test

Always build and run tests before committing:

```bash
cd ~/workspace/super-jujutsu
cargo build 2>&1 | tail -5       # Check for compile errors
cargo test                        # All tests must pass
```

For release builds:

```bash
cargo build --release
cp target/release/supjj ~/bin/
```

## Making Changes

### Adding a new jj subcommand or flag

Edit `src/commands.rs` → `build_tree()`:

```rust
.child("new-cmd", CommandTree::new(&["--flag1", "--flag2"])
    .child("sub", CommandTree::new(&["--subflag"])))
```

If the new flag takes a revision argument, add it to `REV_FLAGS` in the same file. If the command takes a positional revision, add it to `POSITIONAL_REV_CMDS`.

### Adding dynamic completions

1. Add a fetch function in `src/jj.rs` (like `bookmark_names()`)
2. Add a detection function in `src/commands.rs` (like `expects_bookmark_arg()`)
3. Wire it into `App::update_suggestions()` in `src/tui.rs`

### Changing the TUI layout

Edit `src/tui.rs` → `render()`. The layout uses Ratatui's `Layout::vertical` and `Layout::horizontal` with `Constraint` values. The viewport height is set by `VIEWPORT_HEIGHT` (currently 22 lines).

### Changing ANSI color parsing

Edit `src/ansi.rs`. The `apply_sgr()` function handles SGR parameter codes. Both `parse_ansi_text()` (multi-line) and `parse_ansi_line()` (single line) are public.

## Commit Style

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <subject>

<body>
```

### Types

| Type | When to use |
|---|---|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `refactor` | Code change that doesn't fix a bug or add a feature |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `perf` | Performance improvement |
| `test` | Adding or fixing tests |
| `chore` | Build process, dependencies, tooling |

### Scopes

Use the module name as scope when the change is focused on one module:

- `tui` — rendering, input, layout, highlighting
- `commands` — command tree, completions, revset extraction
- `jj` — jj binary interaction
- `ansi` — ANSI parsing

Omit scope for cross-cutting changes.

### Rules

- Use imperative mood: "Add feature" not "Added feature"
- Don't end the subject with a period
- Keep subject under 50 characters
- Wrap body at 72 characters
- Body explains what and why, not how

### Examples

```
feat(tui): Add revision highlighting in log panel

As the user types revision arguments, the referenced revisions are
resolved via jj and highlighted in the log panel with a yellow marker.
```

```
fix(ansi): Handle missing SGR reset at end of line
```

```
docs: Update README for two-column layout
```

```
refactor(commands): Extract flag parsing into separate function
```

## Testing

Unit tests live in `src/commands.rs` under `#[cfg(test)] mod tests`. Run with:

```bash
cargo test
```

When adding new functionality to `commands.rs` (new extraction logic, new completion behavior), add corresponding tests. The TUI module (`tui.rs`) is tested manually since it requires a terminal.

## Code Style

- Minimal code — only what's needed to solve the problem
- No unnecessary abstractions or indirection
- Use `&str` and slices over owned types where possible
- Static data (command tree, flag lists) uses `&'static [&'static str]`
- Error handling: `color_eyre::Result` for fallible functions, `unwrap_or_default()` for jj command output
