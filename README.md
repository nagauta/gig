# gig

Terminal autocomplete for any CLI tool.

gig adds intelligent tab completion to your terminal by loading simple TOML-based completion specs. Inspired by [Fig](https://fig.io) (YC S20).

## Features

- Tab completion for any CLI tool via TOML specs
- Built-in specs for **git**, **docker**, **npm**
- Descriptions shown alongside completions
- zsh and bash support
- Easy to add your own specs

## Install

```bash
# Build from source
cargo install --path .

# Set up shell integration
gig install
```

Or manually:

```bash
# Add to your .zshrc or .bashrc
eval "$(gig init zsh)"
```

## Usage

After installation, just use Tab as usual:

```
$ git <Tab>
clone   -- Clone a repository into a new directory
commit  -- Record changes to the repository
push    -- Update remote refs along with associated objects

$ docker run -<Tab>
--detach  -- Run container in background
--name    -- Assign a name to the container
--publish -- Publish a container's port(s) to the host
```

## Writing a completion spec

Specs are TOML files in `~/.config/gig/specs/`. Example:

```toml
name = "mycli"
description = "My awesome CLI tool"

[[options]]
name = "--verbose"
description = "Enable verbose output"
short = "-v"

[[subcommands]]
name = "deploy"
description = "Deploy the application"

[[subcommands.options]]
name = "--env"
description = "Target environment"
arg = "environment"
short = "-e"
```

## Commands

| Command | Description |
|---------|-------------|
| `gig init <shell>` | Print shell hook script (zsh/bash) |
| `gig install` | Add hook to shell config + install specs |
| `gig uninstall` | Remove hook from shell config |
| `gig complete <cmd> <args...>` | Generate completions (used by shell hook) |

## Configuration

| Env var | Default | Description |
|---------|---------|-------------|
| `GIG_SPECS_DIR` | `~/.config/gig/specs` | Directory to load specs from |

## License

MIT
