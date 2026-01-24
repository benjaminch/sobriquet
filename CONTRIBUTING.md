# Contributing to alxrs

Thank you for your interest in contributing to alx! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We want this to be a welcoming project for everyone.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:

1. A clear, descriptive title
2. Steps to reproduce the issue
3. Expected behavior vs actual behavior
4. Your environment (OS, shell, alx version)
5. Any relevant error messages or logs

### Suggesting Features

Feature suggestions are welcome! Please open an issue with:

1. A clear description of the feature
2. The use case / problem it solves
3. Any ideas for implementation (optional)

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Write tests** for any new functionality
3. **Follow the code style** - run `cargo fmt` before committing
4. **Run lints** - ensure `cargo clippy` passes without warnings
5. **Update documentation** if needed
6. **Write a clear PR description** explaining your changes

## Development Setup

### Prerequisites

- Rust 1.85.0 or later
- A Unix-like shell (zsh, bash, or fish) for testing

### Building

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/alx.git
cd alxrs

# Build
cargo build

# Run tests
cargo test

# Run lints
cargo clippy --all-targets

# Format code
cargo fmt
```

### Running Locally

```bash
# Debug build
cargo run

# Release build
cargo run --release

# With arguments
cargo run -- --list
cargo run -- --shell bash
```

### Running Benchmarks

```bash
cargo bench
```

## Code Style

We follow standard Rust conventions with some additional lints enabled. Key points:

- Use `cargo fmt` for formatting
- All `clippy` warnings should be fixed
- Write doc comments for public items
- Use meaningful variable and function names
- Keep functions focused and reasonably sized

### Clippy Lints

The project has pedantic clippy lints enabled. Run:

```bash
cargo clippy --all-targets
```

And fix any warnings before submitting a PR.

## Testing

- Write unit tests for new functionality
- Place tests in the same file as the code being tested, in a `#[cfg(test)]` module
- Use descriptive test names that explain what's being tested
- Test both success and failure cases

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_alias() {
        let alias = Alias::parse("ls=eza").unwrap();
        assert_eq!(alias.name, "ls");
        assert_eq!(alias.command, "eza");
    }

    #[test]
    fn rejects_invalid_alias_name() {
        assert!(Alias::parse("123=invalid").is_none());
    }
}
```

## Commit Messages

Write clear, concise commit messages:

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Reference issues when relevant ("Fix #123")

Good examples:
- `Add fish shell support`
- `Fix alias parsing for quoted commands`
- `Improve error messages for missing shell`

## Questions?

If you have questions about contributing, feel free to open an issue or start a discussion.

Thank you for contributing!
