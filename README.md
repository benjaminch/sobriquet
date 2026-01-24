# alxrs

[![CI](https://github.com/benjaminch/alxrs/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminch/alxrs/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/benjaminch/alxrs/branch/main/graph/badge.svg)](https://codecov.io/gh/benjaminch/alx)
[![Crates.io](https://img.shields.io/crates/v/alx.svg)](https://crates.io/crates/alxrs)
[![Downloads](https://img.shields.io/crates/d/alx.svg)](https://crates.io/crates/alxrs)
[![License](https://img.shields.io/crates/l/alx.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue.svg)](https://blog.rust-lang.org/)

A fast, fuzzy finder for your shell aliases written in Rust.

`alx` reads your shell aliases and presents them in an interactive fuzzy finder. Select an alias and its expanded command will be placed on your command line, ready to execute or edit.

## Features

- **Fast**: Written in Rust with minimal dependencies
- **Interactive**: Fuzzy search through all your aliases using [skim](https://github.com/lotabout/skim)
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Multi-shell**: Supports zsh, bash, and fish
- **Smart detection**: Automatically detects your shell and parses aliases
- **Cached**: Aliases are cached for instant startup (~3ms vs ~2s)
- **Smart sorting**: Aliases sorted by frecency (frequency + recency)
- **Usage tracking**: Track which aliases you use most with `alxrs stats`
- **Security audit**: Detect embedded secrets and duplicate aliases with `alxrs audit`
- **Rich preview**: Shows command breakdown, shell compatibility, source location, and warnings
- **Easy setup**: Built-in `init` command for shell integration
- **Shell completions**: Generate completions for zsh, bash, and fish
- **Man page**: Built-in man page generation

## Demo

```
$ alxrs
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
git clone https://github.com/benjaminch/alxrs.git
cd alxrs

# Build and install
cargo build --release
cp target/release/alx ~/.local/bin/

# Or install directly with cargo
cargo install --path .
```

### From Releases

Download the latest binary for your platform from the [Releases](https://github.com/benjaminch/alxrs/releases) page.

#### macOS

```bash
# Intel Mac
curl -LO https://github.com/benjaminch/alxrs/releases/latest/download/alx-x86_64-apple-darwin.tar.gz
tar xzf alx-x86_64-apple-darwin.tar.gz
mv alx-x86_64-apple-darwin/alx ~/.local/bin/

# Apple Silicon
curl -LO https://github.com/benjaminch/alxrs/releases/latest/download/alx-aarch64-apple-darwin.tar.gz
tar xzf alx-aarch64-apple-darwin.tar.gz
mv alx-aarch64-apple-darwin/alx ~/.local/bin/
```

#### Linux

```bash
# x86_64
curl -LO https://github.com/benjaminch/alxrs/releases/latest/download/alx-x86_64-unknown-linux-musl.tar.gz
tar xzf alx-x86_64-unknown-linux-musl.tar.gz
mv alx-x86_64-unknown-linux-musl/alx ~/.local/bin/

# ARM64
curl -LO https://github.com/benjaminch/alxrs/releases/latest/download/alx-aarch64-unknown-linux-gnu.tar.gz
tar xzf alx-aarch64-unknown-linux-gnu.tar.gz
mv alx-aarch64-unknown-linux-gnu/alx ~/.local/bin/
```

#### Windows

Download `alx-x86_64-pc-windows-msvc.zip` from the releases page and extract it to a directory in your `PATH`.

### From Cargo

```bash
cargo install alxrs
```

## Shell Integration

The easiest way to set up shell integration is using the built-in `init` command:

### Zsh

Add this to your `~/.zshrc`:

```zsh
eval "$(alx init zsh)"
```

### Bash

Add this to your `~/.bashrc`:

```bash
eval "$(alx init bash)"
```

### Fish

Add this to your `~/.config/fish/config.fish`:

```fish
alx init fish | source
```

After adding the init command, reload your shell or start a new terminal session.

### Updating Shell Integration

When upgrading `alx` to a new version, you may need to update your shell integration if new subcommands have been added. The easiest way is to re-run the init command:

```bash
# Check what the current init script looks like
alx init zsh   # or bash/fish

# Then update your shell rc file accordingly
```

If you've copied the wrapper function directly into your shell config (instead of using `eval`), make sure the `case` statement includes all subcommands: `init|generate|config|stats|audit|--help|-h|--version|-V`.

## Shell Completions

Generate shell completions for tab-completion support:

```bash
# Zsh - add to your fpath
alx generate complete-zsh > ~/.zsh/completions/_alx

# Bash
alx generate complete-bash > ~/.local/share/bash-completion/completions/alx

# Fish
alx generate complete-fish > ~/.config/fish/completions/alx.fish
```

## Man Page

Generate and install the man page:

```bash
alx generate man | sudo tee /usr/local/share/man/man1/alx.1
sudo mandb  # Update man database (Linux)
```

Then view it with `man alx`.

## Usage

```
alx [OPTIONS] [COMMAND]

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
alx

# Start with a pre-filled query
alx --query git

# List all aliases (non-interactive)
alx --list

# List aliases as JSON
alx --list --format json

# Use a specific shell
alx --shell bash

# Force refresh the alias cache
alx --refresh

# View usage statistics
alx stats

# Clear usage statistics
alx stats clear

# Show config file path
alx config

# Audit for secrets and duplicates
alx audit

# Audit for secrets only
alx audit secrets

# Audit for duplicates only
alx audit duplicates

# Pipe to other commands
alx --list | grep git

# Generate shell init script
alx init zsh

# Generate completions
alx generate complete-zsh > _alx

# Generate man page
alx generate man > alx.1
```

## How It Works

1. `alx` runs your shell in interactive mode to get all defined aliases
2. It parses the output and presents them in a fuzzy finder
3. When you select an alias, it outputs the **expanded command** (not the alias name)
4. The shell wrapper function captures this output and places it on your command line

This approach ensures you see exactly what command will run before executing it, which is especially useful for complex aliases with arguments or environment variables.

## Caching

To improve startup performance, `alx` caches your aliases to `~/.cache/alxrs/aliases.json`. The cache has a default TTL of 5 minutes (300 seconds).

- **First run**: ~2 seconds (reads aliases from shell)
- **Cached run**: ~3 milliseconds

To force a cache refresh:

```bash
alx --refresh
# or
alx -r
```

You can configure the cache TTL in your config file (see Configuration below). Set `cache_ttl = 0` to disable caching.

## Usage Statistics

`alx` tracks which aliases you use to help you understand your workflow:

```bash
# View statistics (all-time and last 7 days)
alx stats

# Clear all statistics
alx stats clear
```

Statistics are stored in `~/.local/share/alxrs/stats.json`.

## Security Audit

Check your aliases for potential security issues and duplicates:

```bash
# Run all checks (secrets + duplicates)
alx audit

# Check for embedded secrets only
alx audit secrets

# Check for duplicate commands only
alx audit duplicates
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
alx audit

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

1. `~/.config/alxrs/config.toml` (recommended, XDG standard)
2. `~/.config/alx.toml`
3. `~/.alx.toml`

Run `alxrs config` to see which config file is being used (or where to create one).

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
git clone https://github.com/benjaminch/alxrs.git
cd alxrs

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
