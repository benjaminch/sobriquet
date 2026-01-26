# sobriquet

[![CI](https://github.com/benjaminch/sobriquet/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminch/sobriquet/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/benjaminch/sobriquet/branch/main/graph/badge.svg)](https://codecov.io/gh/benjaminch/sobriquet)
[![Crates.io](https://img.shields.io/crates/v/sobriquet.svg)](https://crates.io/crates/sobriquet)
[![Downloads](https://img.shields.io/crates/d/sobriquet.svg)](https://crates.io/crates/sobriquet)
[![License](https://img.shields.io/crates/l/sobriquet.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue.svg)](https://blog.rust-lang.org/)

## The Problem

I have quite a few shell aliases, especially for my daily work dealing with a lot of Kubernetes clusters and configurations. Unfortunately, I don't always remember all of them. 

I know they're there but always have a typo in them, and it's created a lot of frustration. I'd spend more time trying to recall or correct an alias than actually running the command.

**sobriquet** solves this.

## What is sobriquet?

`sobriquet` is a blazingly fast fuzzy finder for your shell aliases. It reads your aliases and presents them in a beautiful, full-screen interactive search interface. Just type to search, preview the exact command before executing it, and press Enter to run it.

Think of it like [Atuin](https://github.com/atuinsh/atuin) but for your aliases instead of shell history.

## Features

- 🚀 **Blazingly Fast** - Written in Rust with minimal dependencies, startup time is ~3ms
- 🔍 **Full-Screen Search** - Beautiful, distraction-free fuzzy finder
- 📊 **Frecency Sorting** - Smartly sorts aliases by frequency and recency
- 🖥️ **Cross-Platform** - Works on macOS, Linux, and Windows
- 🐚 **Multi-Shell** - Supports zsh, bash, and fish
- 🚨 **Security Audit** - Detect embedded secrets and duplicate aliases
- 🔒 **Safe Preview** - Commands shown without expansion by default (no accidental execution)
- 💾 **Smart Caching** - Instant startup with automatic cache invalidation
- 📈 **Usage Tracking** - See your most-used aliases with built-in stats
- 👁️ **Rich Preview** - Command breakdown, warnings, source location, and more
- 🔄 **Dynamic Toggle** - Expand variables and command substitutions on-demand with Ctrl+X
- ⚡ **Zero Config** - Works out of the box, but customizable if you want
- 📚 **Shell Integration** - Built-in `init` command for easy setup
- 🔄 **Shell Completions** - Generate completions for zsh, bash, and fish

## Quick Demo

```
$ sq
┌─ sobriquet ────────────────────────────────────────┐
│ k8s▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  3/150           │
├──────────────────────────────────────────────────────┤
│ > k8s_prod                                           │
│ > k8s_staging                                        │
│ > k8s_logs                                           │
│                                                      │
│ ────────────────────────────────────────────────────│
│ KUBECONFIG=~/.kube/prod.yaml kubectl               │
│ ────────────────────────────────────────────────────│
│ Command: kubectl                                     │
│ Args: (none)                                         │
│ Source: ~/.zshrc:42                                  │
│ Usage: 157 times (last used 2 hours ago)            │
└──────────────────────────────────────────────────────┘
```

Start typing to search, press Enter to select.

## Installation

### From Homebrew

```bash
brew tap benjaminch/tap
brew install sobriquet
```

### From AUR (Arch Linux)

```bash
yay -S sobriquet
# or
paru -S sobriquet
```

Or manually:

```bash
git clone https://aur.archlinux.org/sobriquet.git
cd sobriquet
makepkg -si
```

### From Cargo (Recommended for other systems)

```bash
cargo install sobriquet
```

This installs `sobriquet` globally and works on any system with Rust installed.

### From Source

```bash
git clone https://github.com/benjaminch/sobriquet.git
cd sobriquet
cargo install --path .
```

### From Releases

Download the latest binary for your platform from the [Releases](https://github.com/benjaminch/sobriquet/releases) page.

#### macOS

```bash
# Apple Silicon
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-aarch64-apple-darwin.tar.gz
tar xzf sobriquet-aarch64-apple-darwin.tar.gz
mv sobriquet-aarch64-apple-darwin/sobriquet ~/.local/bin/

# Intel
curl -LO https://github.com/benjaminch/sobriquet/releases/latest/download/sobriquet-x86_64-apple-darwin.tar.gz
tar xzf sobriquet-x86_64-apple-darwin.tar.gz
mv sobriquet-x86_64-apple-darwin/sobriquet ~/.local/bin/
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

## Setup

After installing, set up shell integration. The easiest way is to use the built-in `init` command:

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

Then restart your shell and you're ready to go!

```bash
# Run with Ctrl+S or just type:
sq
```

## Usage

### Interactive Search

Just run `sobriquet` (or `sq` if you set up the alias):

```bash
sq
```

The full-screen search interface will open. Start typing to search, use arrow keys to navigate, and press Enter to execute.

**Keybindings:**
- `Enter` - Execute the selected alias
- `Ctrl+X` - Toggle secret masking (show/hide sensitive values like tokens, API keys)
- `Ctrl+C` / `Esc` - Exit without executing

**Security Note:** By default, sensitive values (tokens, API keys, secrets) are automatically masked in both the list and preview. Press `Ctrl+X` to reveal the full command.

### List All Aliases

```bash
sobriquet --list
```

Or with formatting:

```bash
sobriquet --list --format json
sobriquet --list --format json-pretty
```

### Search with a Query

```bash
sobriquet --query "git"
```

This opens the search interface with "git" pre-filled.

### View Statistics

```bash
sobriquet stats
```

See your most-used aliases and usage patterns:

```bash
# Clear statistics
sobriquet stats clear
```

### Security Audit

```bash
sobriquet audit
```

Scan for potential security issues:

```bash
# Check for secrets only
sobriquet audit secrets

# Check for duplicate commands
sobriquet audit duplicates
```

### Generate Completions

```bash
# Zsh
sobriquet generate complete-zsh > ~/.zsh/completions/_sobriquet

# Bash
sobriquet generate complete-bash > ~/.local/share/bash-completion/completions/sobriquet

# Fish
sobriquet generate complete-fish > ~/.config/fish/completions/sobriquet.fish
```

## Configuration

sobriquet looks for a config file in these locations (in order of priority):

1. `~/.config/sobriquet/config.toml` (recommended, XDG standard)
2. `~/.config/sobriquet.toml`
3. `~/.sobriquet.toml`

Run `sobriquet config` to see which config file is being used.

### Example Config

```toml
[ui]
height = "100%"                # Full screen by default
prompt = "Select alias > "     # Search prompt
preview = true                 # Show rich preview
preview_position = "right"     # Preview position: right, up, down
preview_expand_details = false # Expand variables and show command details (default: false)
                               # When false: Shows raw commands (secure, no expansion)
                               # When true: Expands $HOME, $(commands), etc.
                               # Tip: Toggle on-the-fly with Ctrl+X

[shell]
prefer = "zsh"                 # Preferred shell
cache_ttl = 300                # Cache validity in seconds (0 to disable)

[output]
color = "auto"                 # Color mode: auto, always, never
```

## How It Works

1. **Collection** - sobriquet runs your shell in interactive mode to collect all defined aliases
2. **Caching** - Aliases are cached for instant startup (~3ms vs ~2s without cache)
3. **Search** - When you run `sobriquet`, it launches a full-screen fuzzy finder using [skim](https://github.com/lotabout/skim)
4. **Preview** - As you search, sobriquet shows a rich preview with command breakdown, warnings, and source location
5. **Execute** - Select an alias and the expanded command is placed on your command line, ready to execute or edit

The key difference from just typing an alias: you see the **expanded command** before execution, making it impossible to have typos frustrate you.

## Performance

| Operation | Time |
|-----------|------|
| Cold start (no cache) | ~2s |
| Warm start (cached) | ~3ms |
| Search/Filter | <10ms |
| Shell startup overhead | <5ms |

sobriquet is carefully optimized to be invisible in your workflow.

## Supported Shells

- ✅ zsh
- ✅ bash
- ✅ fish

## Security

sobriquet can audit your aliases for:

- **Embedded Secrets** - API keys, tokens, passwords, AWS credentials, etc.
- **Duplicate Commands** - Multiple aliases pointing to the same command
- **Dangerous Patterns** - Risky commands like `rm -rf`, `sudo rm`, etc.

All analysis happens locally on your machine. Nothing is sent anywhere.

## Why "sobriquet"?

A sobriquet is a nickname or epithet that describes someone or something. Your shell aliases are exactly that—custom nicknames for your commands. The tool helps you remember and use them effectively.

## Building from Source

```bash
git clone https://github.com/benjaminch/sobriquet.git
cd sobriquet

# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets

# Install
cargo install --path .
```

### Minimum Supported Rust Version

The minimum supported Rust version is **1.92.0**.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Install git hooks: `./scripts/install-hooks.sh`
4. Make your changes
5. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Setup

After cloning the repository, install the git hooks to ensure code quality:

```bash
./scripts/install-hooks.sh
```

This installs a pre-commit hook that automatically runs:
- `cargo fmt --check` - Ensures code is properly formatted
- `cargo clippy` - Catches common mistakes and enforces best practices

If you need to bypass the hooks temporarily (not recommended), use:
```bash
git commit --no-verify
```

### Code Quality

Please make sure to:
- Run `cargo fmt` before committing (automated by pre-commit hook)
- Run `cargo clippy --all-targets` and fix any warnings (automated by pre-commit hook)
- Add tests for new functionality
- Update documentation if needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Atuin](https://github.com/atuinsh/atuin) - Inspiration for the full-screen search UI
- [skim](https://github.com/lotabout/skim) - Fuzzy finder library
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- The open-source community for amazing tools and libraries
