//! CLI integration tests organized by functional scope

mod audit;
mod config;
mod error_handling;
mod generate;
mod help;
mod init;
mod list;
mod refresh;
mod stats;
mod version;

use assert_cmd::{Command, cargo::cargo_bin_cmd};

pub fn alx() -> Command {
    cargo_bin_cmd!("alxrs")
}
