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
pub use error::{AlxError, Result};
pub use shell::Shell;
pub use stats::UsageStats;
