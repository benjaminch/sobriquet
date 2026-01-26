# Code Quality Improvements Based on uutils/coreutils Analysis

This document outlines recommended improvements to the sobriquet codebase based on best practices from the uutils/coreutils project.

## 1. Project Structure Improvements

### Current State
- Single crate structure
- All code in `src/` directory
- Tests mixed with code

### Recommended Changes

#### A. Adopt Cargo Workspace (Optional, for future growth)
If the project grows to multiple binaries or libraries:

```toml
[workspace]
resolver = "3"
members = [
    "sobriquet",      # Main CLI
    "sobriquet-core", # Core library (alias parsing, matching)
    "sobriquet-tui",  # TUI/interactive components
]

[workspace.dependencies]
# Centralize dependency versions
clap = { version = "4.5", features = ["derive"] }
thiserror = "2.0"
```

#### B. Improve Directory Structure
```
sobriquet/
├── src/
│   ├── lib.rs              # Public API
│   ├── main.rs             # CLI entry point
│   ├── error.rs            # Centralized error types
│   ├── alias.rs            # Alias parsing/handling
│   ├── audit.rs            # Security auditing
│   ├── cli.rs              # CLI logic
│   ├── config.rs           # Configuration
│   ├── preview.rs          # Preview rendering
│   ├── shell.rs            # Shell detection/integration
│   ├── source.rs           # File sourcing
│   └── stats.rs            # Usage statistics
├── tests/
│   ├── integration/        # Integration tests
│   │   ├── test_cli.rs
│   │   ├── test_audit.rs
│   │   └── fixtures/       # Test data
│   └── common/             # Test utilities
│       └── mod.rs
└── benches/
    └── benchmarks.rs
```

## 2. Enhanced Error Handling

### Current State
Using basic `Result<T, E>` with different error types

### Recommended Implementation

Create `src/error.rs`:

```rust
use std::fmt;
use std::process::ExitCode;
use thiserror::Error;

/// Exit codes for the application
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const INVALID_USAGE: i32 = 2;
    pub const NO_ALIASES: i32 = 3;
    pub const USER_CANCELLED: i32 = 130; // Ctrl+C
}

/// Trait for errors that know their exit code
pub trait SobriquetError: std::error::Error + Send + Sync {
    fn exit_code(&self) -> i32 {
        exit_codes::GENERAL_ERROR
    }
    
    fn show_usage(&self) -> bool {
        false
    }
}

/// Main result type for the application
pub type SobriquetResult<T> = Result<T, Box<dyn SobriquetError>>;

/// Application-specific errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("No aliases found")]
    NoAliases,
    
    #[error("Invalid shell: {0}")]
    InvalidShell(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("User cancelled")]
    UserCancelled,
}

impl SobriquetError for AppError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::NoAliases => exit_codes::NO_ALIASES,
            Self::InvalidShell(_) | Self::ConfigError(_) => exit_codes::INVALID_USAGE,
            Self::UserCancelled => exit_codes::USER_CANCELLED,
            Self::Io(_) => exit_codes::GENERAL_ERROR,
        }
    }
    
    fn show_usage(&self) -> bool {
        matches!(self, Self::InvalidShell(_) | Self::ConfigError(_))
    }
}

/// Extension trait for adding context to IO errors
pub trait IoErrorContext<T> {
    fn context(self, msg: impl FnOnce() -> String) -> Result<T, Box<dyn SobriquetError>>;
}

impl<T> IoErrorContext<T> for std::io::Result<T> {
    fn context(self, msg: impl FnOnce() -> String) -> Result<T, Box<dyn SobriquetError>> {
        self.map_err(|e| {
            Box::new(AppError::Io(std::io::Error::new(e.kind(), msg()))) as Box<dyn SobriquetError>
        })
    }
}

/// Convert error to exit code
pub fn exit_code_from_error(err: &dyn SobriquetError) -> ExitCode {
    ExitCode::from(err.exit_code() as u8)
}
```

Update `src/main.rs`:

```rust
use sobriquet::error::{exit_code_from_error, SobriquetError};

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            if err.show_usage() {
                eprintln!("\nFor more information, try '--help'.");
            }
            exit_code_from_error(&*err)
        }
    }
}

fn try_main() -> Result<(), Box<dyn SobriquetError>> {
    // Implementation
}
```

## 3. Advanced Testing Infrastructure

### Recommended Test Utilities

Create `tests/common/mod.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestScenario {
    pub tmpdir: TempDir,
    pub fixtures: PathBuf,
}

impl TestScenario {
    pub fn new(name: &str) -> Self {
        let tmpdir = TempDir::new().unwrap();
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        
        Self { tmpdir, fixtures }
    }
    
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("sobriquet").unwrap();
        cmd.current_dir(&self.tmpdir);
        cmd
    }
    
    pub fn fixture_path(&self, name: &str) -> PathBuf {
        self.fixtures.join(name)
    }
    
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.tmpdir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }
}

/// Assertion helpers
pub trait CommandExt {
    fn succeeds(&mut self) -> &mut Self;
    fn fails_with(&mut self, code: i32) -> &mut Self;
    fn stdout_contains(&mut self, expected: &str) -> &mut Self;
    fn stderr_contains(&mut self, expected: &str) -> &mut Self;
}

impl CommandExt for assert_cmd::Command {
    fn succeeds(&mut self) -> &mut Self {
        self.assert().success();
        self
    }
    
    fn fails_with(&mut self, code: i32) -> &mut Self {
        self.assert().code(code);
        self
    }
    
    fn stdout_contains(&mut self, expected: &str) -> &mut Self {
        self.assert().stdout(predicate::str::contains(expected));
        self
    }
    
    fn stderr_contains(&mut self, expected: &str) -> &mut Self {
        self.assert().stderr(predicate::str::contains(expected));
        self
    }
}
```

Example test using the framework:

```rust
use crate::common::{CommandExt, TestScenario};

#[test]
fn test_list_aliases() {
    let scene = TestScenario::new("list");
    scene.create_file(".zshrc", "alias gs='git status'");
    
    scene.cmd()
        .args(&["--list", "--shell", "zsh"])
        .succeeds()
        .stdout_contains("gs")
        .stdout_contains("git status");
}
```

## 4. Enhanced CI/CD Configuration

### A. Add cargo-deny for Dependency Auditing

Create `deny.toml`:

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "ISC", "BSD-2-Clause", "BSD-3-Clause"]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "allow"
```

### B. Add Pre-commit Configuration

Update `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml
      - id: check-added-large-files
      
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-deny
        name: cargo deny
        entry: cargo deny check
        language: system
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test --all-features
        language: system
        types: [rust]
        pass_filenames: false
```

### C. Enhanced GitHub Actions Workflow

Create `.github/workflows/quality.yml`:

```yaml
name: Code Quality

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  format:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2

  test:
    name: Test Suite
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, nightly]
    runs-on: ${{ matrix.os }}
    continue-on-error: ${{ matrix.rust == 'nightly' }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features
      - run: cargo test --no-default-features

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features
      - name: Upload to codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./cobertura.xml
```

## 5. Documentation Improvements

### A. Add Module-Level Documentation

In `src/alias.rs`:

```rust
//! Alias parsing and management
//!
//! This module provides functionality for:
//! - Parsing shell alias definitions from various formats (zsh, bash, fish)
//! - Caching parsed aliases for performance
//! - Managing alias metadata (name, command, source file)
//!
//! # Examples
//!
//! ```
//! use sobriquet::alias::Alias;
//!
//! let alias = Alias::parse("gs='git status'", Shell::Zsh)?;
//! assert_eq!(alias.name, "gs");
//! assert_eq!(alias.command, "git status");
//! ```

#![warn(missing_docs)]

/// Represents a shell alias
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Alias {
    /// The alias name (e.g., "gs")
    pub name: String,
    /// The command the alias expands to (e.g., "git status")
    pub command: String,
    // ... rest of fields
}
```

### B. Add DEVELOPMENT.md

```markdown
# Development Guide

## Building

### Debug Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### With All Features
```bash
cargo build --all-features
```

## Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test
```bash
cargo test test_alias_parsing
```

### With Coverage
```bash
cargo tarpaulin --out Html
open tarpaulin-report.html
```

## Benchmarking

```bash
cargo bench
```

## Code Quality

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy --all-targets --all-features
```

### Check Dependencies
```bash
cargo deny check
```

## Pre-commit Hooks

Install pre-commit hooks to run checks automatically:

```bash
pre-commit install
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run tests: `cargo test --all-features`
4. Create git tag: `git tag v0.x.x`
5. Push: `git push && git push --tags`
```

## 6. Performance Optimizations

### A. Add More Granular Build Profiles

```toml
[profile.release]
opt-level = 3
lto = "thin"
strip = "symbols"
panic = "abort"
codegen-units = 16  # Balance compilation speed and runtime performance

[profile.release-fast]
inherits = "release"
lto = "fat"
codegen-units = 1   # Maximum runtime performance

[profile.release-small]
inherits = "release"
opt-level = "z"     # Optimize for size
lto = "fat"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0       # Fast compilation
debug = true

[profile.dev.package."*"]
opt-level = 2       # Optimize dependencies in dev mode
```

### B. Add Benchmarking Suite

Enhance `benches/benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sobriquet::{Alias, Shell};

fn bench_alias_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("alias_parsing");
    
    let test_cases = vec![
        ("simple", "gs='git status'"),
        ("complex", "gco='git checkout \"$@\" && git pull --rebase'"),
        ("with_quotes", r#"gcm="git commit -m \"$1\"" "#),
    ];
    
    for (name, input) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("zsh", name),
            input,
            |b, input| {
                b.iter(|| Alias::parse(black_box(input), Shell::Zsh))
            }
        );
    }
    
    group.finish();
}

fn bench_alias_matching(c: &mut Criterion) {
    let aliases = vec![
        Alias { name: "gs".into(), command: "git status".into(), .. },
        Alias { name: "gst".into(), command: "git status".into(), .. },
        Alias { name: "gco".into(), command: "git checkout".into(), .. },
        // ... more aliases
    ];
    
    c.bench_function("match_query", |b| {
        b.iter(|| {
            aliases.iter()
                .filter(|a| a.name.contains(black_box("g")))
                .collect::<Vec<_>>()
        })
    });
}

criterion_group!(benches, bench_alias_parsing, bench_alias_matching);
criterion_main!(benches);
```

## 7. Additional Clippy Lints

Add to `Cargo.toml`:

```toml
[lints.clippy]
# Already configured
pedantic = { level = "warn", priority = -1 }

# Additional uutils-inspired lints
all = { level = "warn", priority = -2 }
cargo = { level = "warn", priority = -2 }

# Specific allows (tune these based on your needs)
missing_errors_doc = "allow"          # Can be noisy
missing_panics_doc = "allow"          # Can be noisy
module_name_repetitions = "allow"     # Already allowed

# Additional restriction lints
dbg_macro = "warn"                    # Don't commit debug code
print_stdout = "warn"                 # Use proper logging
print_stderr = "warn"                 # Use proper logging
```

## 8. Security Improvements

### A. Add Security Audit Workflow

`.github/workflows/security.yml`:

```yaml
name: Security Audit

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  push:
    branches: [ main ]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### B. Add Dependabot Configuration

`.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    reviewers:
      - "your-username"
    labels:
      - "dependencies"
```

## Summary of Priority Improvements

### High Priority (Implement First)
1. ✅ Enhanced error handling with `SobriquetError` trait
2. ✅ Add `cargo-deny` for dependency auditing
3. ✅ Improve test infrastructure with test utilities
4. ✅ Add more comprehensive CI checks

### Medium Priority
5. ✅ Add module-level documentation
6. ✅ Create DEVELOPMENT.md guide
7. ✅ Enhance benchmarking suite
8. ✅ Add security audit workflow

### Low Priority (Future)
9. Consider workspace structure if project grows
10. Add internationalization support (like uutils' Fluent integration)
11. Platform-specific optimizations for hot paths

## Implementation Plan

1. **Week 1**: Error handling and CI improvements
2. **Week 2**: Testing infrastructure and documentation
3. **Week 3**: Performance benchmarks and security audits
4. **Week 4**: Polish and additional tooling

These improvements will bring sobriquet closer to production-grade Rust CLI tool standards while maintaining its simplicity and focus.
