# sobriquet

[![CI](https://github.com/benjaminch/sobriquet/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminch/sobriquet/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/benjaminch/sobriquet/branch/main/graph/badge.svg)](https://codecov.io/gh/benjaminch/sobriquet)
[![Crates.io](https://img.shields.io/crates/v/sobriquet.svg)](https://crates.io/crates/sobriquet)
[![Downloads](https://img.shields.io/crates/d/sobriquet.svg)](https://crates.io/crates/sobriquet)
[![License](https://img.shields.io/crates/l/sobriquet.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue.svg)](https://blog.rust-lang.org/)

A fast, fuzzy finder for your shell aliases written in Rust.

`sobriquet` reads your shell aliases and presents them in an interactive fuzzy finder. Select an alias and its expanded command will be placed on your command line, ready to execute or edit.

## Features

- **Fast**: Written in Rust with minimal dependencies
- **Interactive**: Fuzzy search through all your aliases using [skim](https://github.com/lotabout/skim)
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Multi-shell**: Supports zsh, bash, and fish
- **Smart detection**: Automatically detects your shell and parses aliases
- **Cached**: Aliases are cached for instant startup (~3ms vs ~2s)
- **Smart sorting**: Aliases sorted by frecency (frequency + recency)
- **Usage tracking**: Track which aliases you use most with `sobriquet stats`
- **Security audit**: Detect embedded secrets and duplicate aliases with `sobriquet audit`
- **Rich preview**: Shows command breakdown, shell compatibility, source location, and warnings
- **Easy setup**: Built-in `init` command for shell integration
- **Shell completions**: Generate completions for zsh, bash, and fish
- **Man page**: Built-in man page generation

## Demo

```
$ sobriquet
> k8s                                 # Type to filter
  k8s_prod -> KUBECONFIG=~/.kube/prod.yaml kubectl
  k8s_staging -> KUBECONFIG=~/.kube/staging.yaml kubectl
  k8s_logs -> kubectl logs -f --tail=100
  3/150

# Press Enter to select, and the command appears on your command line
$ KUBECONFIG=~/.kube/prod.yaml kubectl█
```

## Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/benjaminch/sobriquet.git
cd sobriquet

# Build and install
cargo build --release
cp target/release/sobriquet ~/.local/bin/

# Or install directly with cargo
cargo install --path .
```

### From Releases

Download the latest binary for your platform from the [Releases](https://github.com/benjaminch/sobriquet/releases) page.

#### macOS

```bash
# Intel Mac
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-x86_64-apple-darwin.tar.gz
tar xzf sobriquet-x86_64-apple-darwin.tar.gz
mv sobriquet-x86_64-apple-darwin/sobriquet ~/.local/bin/

# Apple Silicon
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-aarch64-apple-darwin.tar.gz
tar xzf sobriquet-aarch64-apple-darwin.tar.gz
mv sobriquet-aarch64-apple-darwin/sobriquet ~/.local/bin/
```

#### Linux

```bash
# x86_64
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-x86_64-unknown-linux-musl.tar.gz
tar xzf sobriquet-x86_64-unknown-linux-musl.tar.gz
mv sobriquet-x86_64-unknown-linux-musl/sobriquet ~/.local/bin/

# ARM64
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-aarch64-unknown-linux-gnu.tar.gz
tar xzf sobriquet-aarch64-unknown-linux-gnu.tar.gz
mv sobriquet-aarch64-unknown-linux-gnu/sobriquet ~/.local/bin/
```

#### Windows

Download `sobriquet-x86_64-pc-windows-msvc.zip` from the releases page and extract it to a directory in your `PATH`.

### From Cargo

```bash
cargo install sobriquet
```

## Shell Integration

The easiest way to set up shell integration is using the built-in `init` command:

### Zsh

Add this to your `~/.zshrc`:

```zsh
eval "$(sobriquet init zsh)"
```

### Bash

Add this to your `~/.bashrc`:

```bash
eval "$(sobriquet init bash)"
```

### Fish

Add this to your `~/.config/fish/config.fish`:

```fish
sobriquet init fish | source
```

After adding the init command, reload your shell or start a new terminal session.

### Updating Shell Integration

When upgrading `sobriquet` to a new version, you may need to update your shell integration if new subcommands have been added. The easiest way is to re-run the init command:

```bash
# Check what the current init script looks like
sobriquet init zsh   # or bash/fish

# Then update your shell rc file accordingly
```

If you've copied the wrapper function directly into your shell config (instead of using `eval`), make sure the `case` statement includes all subcommands: `init|generate|config|stats|audit|--help|-h|--version|-V`.

## Shell Completions

Generate shell completions for tab-completion support:

```bash
# Zsh - add to your fpath
sobriquet generate complete-zsh > ~/.zsh/completions/_sobriquet

# Bash
sobriquet generate complete-bash > ~/.local/share/bash-completion/completions/sobriquet

# Fish
sobriquet generate complete-fish > ~/.config/fish/completions/sobriquet.fish
```

## Man Page

Generate and install the man page:

```bash
sobriquet generate man | sudo tee /usr/local/share/man/man1/sobriquet.1
sudo mandb  # Update man database (Linux)
```

Then view it with `man sobriquet`.

## Usage

```
sobriquet [OPTIONS] [COMMAND]

Commands:
  init      Initialize shell integration (add to your shell's rc file)
  generate  Generate shell completions or man page
  config    Show configuration file path
  stats     Show usage statistics (use `stats clear` to reset)
  audit     Check for embedded secrets and duplicate aliases

Options:
  -l, --list             List all aliases without interactive selection
  -s, --shell <SHELL>    Specify which shell to use [possible values: zsh, bash, fish]
  -q, --query <QUERY>    Start with a pre-filled query
  -f, --format <FORMAT>  Output format for --list [default: plain] [possible values: plain, json, json-pretty]
  -r, --refresh          Force refresh of the alias cache
  --color <WHEN>         Color mode [default: auto] [possible values: auto, always, never]
  --print-query          Print the query if no match is selected
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

```bash
# Open interactive fuzzy finder
sobriquet

# Start with a pre-filled query
sobriquet --query git

# List all aliases (non-interactive)
sobriquet --list

# List aliases as JSON
sobriquet --list --format json

# Use a specific shell
sobriquet --shell bash

# Force refresh the alias cache
sobriquet --refresh

# View usage statistics
sobriquet stats

# Clear usage statistics
sobriquet stats clear

# Show config file path
sobriquet config

# Audit for secrets and duplicates
sobriquet audit

# Audit for secrets only
sobriquet audit secrets

# Audit for duplicates only
sobriquet audit duplicates

# Pipe to other commands
sobriquet --list | grep git

# Generate shell init script
sobriquet init zsh

# Generate completions
sobriquet generate complete-zsh > _sobriquet

# Generate man page
sobriquet generate man > sobriquet.1
```

## How It Works

1. `sobriquet` runs your shell in interactive mode to get all defined aliases
2. It parses the output and presents them in a fuzzy finder
3. When you select an alias, it outputs the **expanded command** (not the alias name)
4. The shell wrapper function captures this output and places it on your command line

This approach ensures you see exactly what command will run before executing it, which is especially useful for complex aliases with arguments or environment variables.

## Caching

To improve startup performance, `sobriquet` caches your aliases to `~/.cache/sobriquet/aliases.json`. The cache has a default TTL of 5 minutes (300 seconds).

- **First run**: ~2 seconds (reads aliases from shell)
- **Cached run**: ~3 milliseconds

To force a cache refresh:

```bash
sobriquet --refresh
# or
sobriquet -r
```

You can configure the cache TTL in your config file (see Configuration below). Set `cache_ttl = 0` to disable caching.

## Usage Statistics

`sobriquet` tracks which aliases you use to help you understand your workflow:

```bash
# View statistics (all-time and last 7 days)
sobriquet stats

# Clear all statistics
sobriquet stats clear
```

Statistics are stored in `~/.local/share/sobriquet/stats.json`.

## Security Audit

Check your aliases for potential security issues and duplicates:

```bash
# Run all checks (secrets + duplicates)
sobriquet audit

# Check for embedded secrets only
sobriquet audit secrets

# Check for duplicate commands only
sobriquet audit duplicates
```

### What it detects

**Secrets:**
- API keys (`API_KEY=`, `APIKEY=`)
- Tokens (`TOKEN=`, `ACCESS_TOKEN=`, `AUTH_TOKEN=`)
- Passwords (`PASSWORD=`, `SECRET=`)
- Cloud credentials (`AWS_SECRET_ACCESS_KEY=`, `AKIA...`)
- Known key formats (`sk-ant-`, `ghp_`, `xoxb-`, etc.)

**Duplicates:**
- Multiple aliases pointing to the same command

The audit also shows the **file location** where each problematic alias is defined, making it easy to fix issues.

Example output:
```
sobriquet audit

Scanned 150 aliases

⚠ 2 alias(es) contain secrets:
  • claude         API key          ~/.zshrc:87
    ANTHROPIC_API_KEY=sk-ant-...
  • aws_prod       AWS credentials  ~/.config/zsh/aws.zsh:12
    AWS_SECRET_ACCESS_KEY=...

⚠ 3 duplicate command(s) found:
  • "git status"
    → gs (~/.zshrc:42)
    → gst (~/.zshrc:43)

✓ 145 aliases passed all checks
```

## Keyboard Shortcuts

In the interactive fuzzy finder:

| Key | Action |
|-----|--------|
| `Enter` | Select the highlighted alias |
| `Esc` / `Ctrl+C` | Cancel selection |
| `Ctrl+J` / `Ctrl+N` / `Down` | Move to next item |
| `Ctrl+K` / `Ctrl+P` / `Up` | Move to previous item |
| `Ctrl+U` | Clear the search query |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success (alias selected or list printed) |
| `1` | No selection made (user cancelled) |
| `2` | Error occurred |

## Configuration

`alx` looks for configuration in the following locations (in order of priority):

1. `~/.config/sobriquet/config.toml` (recommended, XDG standard)
2. `~/.config/alx.toml`
3. `~/.alx.toml`

Run `sobriquet config` to see which config file is being used (or where to create one).

```toml
[ui]
height = "50%"      # Height of the fuzzy finder
prompt = "> "       # Prompt string
preview = true      # Show command preview
preview_position = "right"  # Preview position: right, up, down

[shell]
prefer = "zsh"      # Preferred shell: zsh, bash, fish
cache_ttl = 300     # Cache TTL in seconds (0 to disable)

[output]
color = "auto"      # Color mode: auto, always, never
```

## Requirements

- A Unix-like shell (zsh, bash, or fish)
- A terminal that supports ANSI escape codes

## Building from Source

```bash
# Clone
git clone https://github.com/benjaminch/sobriquet.git
cd sobriquet

# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets

# Run with optimizations
cargo run --release
```

### Minimum Supported Rust Version

The minimum supported Rust version is **1.92.0**.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/) format
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Message Format

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Examples:
- `feat: add fish shell support`
- `fix(parser): handle quoted aliases correctly`
- `docs: update installation instructions`

### Code Quality

Please make sure to:
- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets` and fix any warnings
- Add tests for new functionality
- Update documentation if needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [skim](https://github.com/lotabout/skim) - Fuzzy finder library for Rust
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Inspiration for project structure and CI setup
