//! Sobriquet - A fuzzy finder for shell aliases
//!
//! Sobriquet (formerly alx) is an interactive command-line tool that helps you find and execute
//! shell aliases quickly using fuzzy search.
//!
//! # Features
//!
//! - **Interactive fuzzy search** - Find aliases quickly with skim-based fuzzy matching
//! - **Multi-shell support** - Works with Zsh, Bash, and Fish
//! - **Smart caching** - Caches alias definitions for fast startup
//! - **Usage tracking** - Tracks frequently used aliases
//! - **Security auditing** - Detects potentially dangerous alias commands
//! - **Preview panel** - Shows detailed command analysis and similar commands
//!
//! # Example
//!
//! ```no_run
//! use sobriquet::run;
//!
//! // Run the interactive alias selector
//! if let Err(e) = run() {
//!     eprintln!("Error: {}", e);
//! }
//! ```

#![allow(clippy::missing_errors_doc)]

pub mod alias;
pub mod audit;
pub mod cli;
pub mod config;
pub mod error;
pub mod preview;
pub mod shell;
pub mod source;
pub mod stats;

pub use alias::Alias;
pub use cli::run;
pub use config::Config;
pub use error::{AlxError, Result, SobriquetError, SobriquetResult};
pub use shell::Shell;
pub use stats::UsageStats;
