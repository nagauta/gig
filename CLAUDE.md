# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

gig is a Rust-based terminal autocomplete tool inspired by Fig. It provides intelligent tab completion for CLI tools through TOML-based completion specs.

## Commands

```bash
cargo build                                    # Build
cargo test --all-targets                       # Run all tests
cargo clippy --all-targets -- -D warnings      # Lint
cargo fmt --check                              # Check formatting
cargo fmt                                      # Auto-fix formatting
```

Pre-push hooks (lefthook) run `cargo fmt --check` and `cargo clippy -- -D warnings`.

## Architecture

```
User Shell (zsh/bash)
  → Shell hook (shell.rs, installed via gig install)
    → main.rs (clap CLI: init/install/uninstall/complete/pick)
      → complete.rs (loads TOML specs from ~/.config/gig/specs/)
        → spec.rs (parses specs, generates Completion items, runs generators)
          → tui.rs (ratatui interactive fuzzy picker)
```

**Completion flow:** User presses Tab → shell hook calls `gig pick` with current tokens → spec matching → TUI picker → selected value written to temp file → shell inserts result.

### Key modules

- **spec.rs** - Core logic: TOML spec parsing (`Spec`, `Subcommand`, `Opt`), completion generation, generator execution (runs shell commands for dynamic completions like branch lists), fuzzy matching
- **complete.rs** - Loads specs from `GIG_SPECS_DIR` (default `~/.config/gig/specs/`), formats completions as tab-separated output
- **tui.rs** - Interactive picker with fuzzy filtering, per-kind icons (Subcommand `$`, Branch `ᚠ`, File `□`, Command `▶`), keyboard navigation
- **shell.rs** - Generates zsh/bash hook scripts that integrate with shell's completion system
- **installer.rs** - Manages shell rc file modifications (adds/removes hook), copies bundled specs

### Spec format

TOML files in `specs/` define commands. Subcommands can have a `generator` (shell command for dynamic completions) and `generator_kind` (`branch`/`file`/`command`) which determines the display icon.

## CI

Runs on every push/PR: `cargo test`, `cargo clippy`, `cargo fmt --check`. Releases use cargo-dist targeting macOS (aarch64/x86_64).
