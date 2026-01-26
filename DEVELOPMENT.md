# Development Guide

This guide covers everything you need to know to contribute to sobriquet.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Building](#building)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Documentation](#documentation)
- [Pre-commit Hooks](#pre-commit-hooks)
- [Release Process](#release-process)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- **Rust 1.92 or later** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For version control
- **cargo-deny** (optional) - For dependency auditing: `cargo install cargo-deny`
- **cargo-audit** (optional) - For security auditing: `cargo install cargo-audit`

## Getting Started

1. **Clone the repository**

```bash
git clone https://github.com/benjaminch/sobriquet.git
cd sobriquet
```

2. **Build the project**

```bash
cargo build
```

3. **Run tests**

```bash
cargo test
```

4. **Run sobriquet locally**

```bash
cargo run
# or with specific arguments
cargo run -- --list
```

## Project Structure

```
sobriquet/
├── src/
│   ├── lib.rs              # Library entry point and public API
│   ├── main.rs             # Binary entry point
│   ├── alias.rs            # Alias parsing, caching, and management
│   ├── audit.rs            # Security auditing functionality
│   ├── cli.rs              # CLI argument parsing and interactive UI
│   ├── config.rs           # Configuration file handling
│   ├── error.rs            # Error types and handling
│   ├── preview.rs          # Preview panel analysis
│   ├── shell.rs            # Shell detection and integration
│   ├── source.rs           # Alias source file location
│   └── stats.rs            # Usage statistics tracking
├── tests/
│   ├── integration.rs      # Integration test entry point
│   ├── common/             # Test utilities and helpers
│   │   └── mod.rs          # TestScenario, CommandExt traits
│   └── cli/                # CLI integration tests
│       ├── audit.rs        # Audit command tests
│       ├── config.rs       # Config command tests
│       ├── error_handling.rs # Error handling tests
│       ├── generate.rs     # Completion generation tests
│       ├── help.rs         # Help command tests
│       ├── init.rs         # Init command tests
│       ├── list.rs         # List command tests
│       ├── refresh.rs      # Refresh command tests
│       ├── stats.rs        # Stats command tests
│       └── version.rs      # Version command tests
├── .github/
│   └── workflows/
│       ├── ci.yml          # Continuous integration
│       ├── coverage.yml    # Code coverage tracking
│       └── release.yml     # Release automation
├── deny.toml               # cargo-deny configuration
├── IMPROVEMENTS.md         # Best practices and improvement guide
└── DEVELOPMENT.md          # This file
```

## Building

### Development Build

```bash
cargo build
```

The binary will be at `target/debug/sobriquet`.

### Release Build

```bash
cargo build --release
```

The optimized binary will be at `target/release/sobriquet`.

### Platform-Specific Builds

**Windows (disable interactive features):**
```bash
cargo build --release --no-default-features
```

**Cross-compilation:**
```bash
# Install cross
cargo install cross

# Build for specific target
cross build --release --target x86_64-unknown-linux-musl
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Specific test module
cargo test --test integration cli::list

# Single test
cargo test --test integration list_outputs_aliases
```

### Run Tests with Output

```bash
# Show stdout/stderr from passing tests
cargo test -- --nocapture

# Show test names as they run
cargo test -- --show-output
```

### Test Coverage

```bash
# Generate coverage report (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

Coverage reports are automatically generated in CI and uploaded to Codecov.

## Code Quality

### Format Code

```bash
cargo fmt
```

Check formatting without modifying files:

```bash
cargo fmt --check
```

### Lint with Clippy

```bash
cargo clippy --all-targets --all-features
```

Fix auto-fixable issues:

```bash
cargo clippy --fix --all-targets --all-features
```

### Security Auditing

**Check for known vulnerabilities:**
```bash
cargo audit
```

**Check dependencies, licenses, and sources:**
```bash
cargo deny check
```

The `deny.toml` configuration enforces:
- Approved licenses (MIT, Apache-2.0, BSD-*)
- No known security vulnerabilities
- Warnings on duplicate dependencies

### All Quality Checks

Run all checks before submitting a PR:

```bash
# Format
cargo fmt --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test

# Documentation
cargo doc --no-deps --document-private-items

# Security
cargo deny check
cargo audit
```

## Documentation

### Build Documentation

```bash
# Build docs for public API
cargo doc --no-deps

# Build docs including private items
cargo doc --no-deps --document-private-items

# Build and open in browser
cargo doc --no-deps --open
```

### Documentation Standards

All public items must have documentation comments:

```rust
/// Brief one-line description
///
/// More detailed explanation if needed.
///
/// # Examples
///
/// ```
/// use sobriquet::Alias;
/// let alias = Alias::new("gs", "git status");
/// ```
///
/// # Errors
///
/// Returns an error if...
pub fn example() -> Result<()> {
    // ...
}
```

Module-level documentation should include:

```rust
//! Brief module description
//!
//! More detailed explanation of what the module does
//! and how it fits into the overall architecture.
```

## Pre-commit Hooks

The project uses pre-commit hooks to ensure code quality:

```bash
# Hooks are automatically installed via .git/hooks/pre-commit
# They run on every commit

# To run hooks manually:
git commit --no-verify  # Skip hooks (not recommended)
```

Current pre-commit checks:
- `cargo fmt --check` - Ensures code is formatted
- `cargo clippy` - Ensures no lint warnings

## Release Process

Releases are automated via GitHub Actions when a version tag is pushed.

### Steps to Release

1. **Update version in `Cargo.toml`**

```toml
[package]
version = "0.1.5"  # Bump version
```

2. **Update CHANGELOG.md**

Document all changes since the last release.

3. **Commit changes**

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.1.5"
```

4. **Create and push tag**

```bash
git tag v0.1.5
git push origin main --tags
```

5. **GitHub Actions will automatically:**
   - Build binaries for all platforms
   - Create a GitHub release
   - Upload release artifacts
   - Publish to crates.io (if configured)

## Troubleshooting

### Build Issues

**Issue: `error: linking with 'cc' failed`**

Solution: Install build essentials
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install
```

**Issue: `feature 'interactive' not found`**

Solution: The interactive feature is enabled by default. On Windows, disable it:
```bash
cargo build --no-default-features
```

### Test Issues

**Issue: Integration tests fail with "binary not found"**

Solution: Build the binary first
```bash
cargo build
cargo test
```

**Issue: Tests timeout or hang**

Solution: Some tests require TTY. Run with:
```bash
cargo test -- --test-threads=1
```

### Documentation Issues

**Issue: `cargo doc` fails with warnings**

Solution: Fix all documentation warnings
```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items
```

### Pre-commit Hook Issues

**Issue: Hooks prevent commit**

Solution: Fix the issues reported, or temporarily bypass (not recommended)
```bash
git commit --no-verify
```

## Additional Resources

- [Contributing Guidelines](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)
- [Best Practices Guide](IMPROVEMENTS.md)
- [Packaging Guide](PACKAGING.md)

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/benjaminch/sobriquet/issues)
- **Discussions**: [GitHub Discussions](https://github.com/benjaminch/sobriquet/discussions)
- **Documentation**: [docs.rs](https://docs.rs/sobriquet)

## Code Style Guidelines

### General Principles

1. **Clarity over cleverness** - Write code that is easy to understand
2. **Fail fast** - Use early returns and the `?` operator
3. **Document public APIs** - All public items must have doc comments
4. **Write tests** - Add tests for new functionality and bug fixes
5. **Handle errors** - Don't use `.unwrap()` except in tests

### Error Handling

Use the custom error types defined in `src/error.rs`:

```rust
use crate::error::{AlxError, Result};

pub fn example() -> Result<()> {
    let file = fs::read_to_string("config.toml")
        .map_err(|e| AlxError::ConfigRead(e))?;
    Ok(())
}
```

### Testing Patterns

Use the test utilities in `tests/common/mod.rs`:

```rust
use crate::common::{TestScenario, CommandExt};

#[test]
fn test_example() {
    let ts = TestScenario::new("test_example");
    ts.create_file("config.toml", "key = value");
    
    ts.cmd()
        .arg("--config")
        .arg(ts.tmpdir_path().join("config.toml"))
        .succeeds_with_stdout_containing("success");
}
```

## Performance Considerations

- **Minimize allocations** - Use references where possible
- **Lazy evaluation** - Don't compute values that might not be used
- **Cache expensive operations** - Like alias parsing and file reads
- **Profile before optimizing** - Use `cargo flamegraph` or similar tools

## Commit Message Format

Follow conventional commits:

```
type(scope): subject

body

footer
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(audit): add detection for embedded API keys

fix(cache): prevent stale cache after shell config changes

docs(readme): update installation instructions
```
