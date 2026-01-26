//! Error types and handling for sobriquet
//!
//! This module provides a centralized error handling system with:
//! - Exit code management
//! - Error context
//! - Type-safe error conversion

use std::io;
use std::process::ExitCode;
use thiserror::Error;

/// Exit codes for the application
pub mod exit_codes {
    /// Successful execution
    pub const SUCCESS: i32 = 0;
    /// General error
    pub const GENERAL_ERROR: i32 = 1;
    /// No aliases found or other runtime errors (maintains backward compatibility)
    pub const NO_ALIASES: i32 = 2;
    /// User cancelled (Ctrl+C)
    pub const USER_CANCELLED: i32 = 130;
}

/// Trait for errors that know their exit code and behavior
pub trait SobriquetError: std::error::Error + Send + Sync {
    /// Get the exit code for this error
    fn exit_code(&self) -> i32 {
        exit_codes::GENERAL_ERROR
    }

    /// Whether to show usage hint for this error
    fn show_usage(&self) -> bool {
        false
    }
}

#[derive(Error, Debug)]
pub enum AlxError {
    #[error("no aliases found")]
    NoAliasesFound,

    #[error("failed to execute shell '{shell}': {source}")]
    ShellExecution {
        shell: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to write output: {0}")]
    OutputWrite(#[from] io::Error),

    #[error("user aborted selection")]
    UserAborted,

    #[error("failed to read config file: {0}")]
    ConfigRead(String),

    #[error("failed to parse config file: {0}")]
    ConfigParse(String),

    #[error("failed to serialize output: {0}")]
    Serialization(String),
}

impl SobriquetError for AlxError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::UserAborted => exit_codes::USER_CANCELLED,
            // All other errors use NO_ALIASES exit code (2) for backward compatibility
            _ => exit_codes::NO_ALIASES,
        }
    }

    fn show_usage(&self) -> bool {
        matches!(self, Self::ConfigRead(_) | Self::ConfigParse(_))
    }
}

pub type Result<T> = std::result::Result<T, AlxError>;

/// Main result type for the application
pub type SobriquetResult<T> = std::result::Result<T, Box<dyn SobriquetError>>;

/// Extension trait for adding context to IO errors
pub trait IoErrorContext<T> {
    /// Add context to an IO error
    fn context(self, msg: impl Into<String>) -> Result<T>;
}

impl<T> IoErrorContext<T> for io::Result<T> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            AlxError::OutputWrite(io::Error::new(e.kind(), msg.into()))
        })
    }
}

/// Convert error to exit code
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn exit_code_from_error(err: &dyn SobriquetError) -> ExitCode {
    // Exit codes are guaranteed to be in 0-255 range by convention
    ExitCode::from(err.exit_code() as u8)
}

/// Format an error with optional usage hint
pub fn format_error(err: &dyn SobriquetError) -> String {
    let mut msg = format!("Error: {err}");
    if err.show_usage() {
        msg.push_str("\n\nFor more information, try '--help'.");
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_exit_codes() {
        assert_eq!(
            AlxError::NoAliasesFound.exit_code(),
            exit_codes::NO_ALIASES
        );
        assert_eq!(
            AlxError::ConfigRead("test".into()).exit_code(),
            exit_codes::NO_ALIASES
        );
        assert_eq!(
            AlxError::UserAborted.exit_code(),
            exit_codes::USER_CANCELLED
        );
    }

    #[test]
    fn test_show_usage() {
        assert!(AlxError::ConfigRead("test".into()).show_usage());
        assert!(AlxError::ConfigParse("test".into()).show_usage());
        assert!(!AlxError::NoAliasesFound.show_usage());
    }

    #[test]
    fn test_format_error() {
        let err = AlxError::ConfigRead("bad config".into());
        let formatted = format_error(&err);
        assert!(formatted.contains("failed to read config"));
        assert!(formatted.contains("--help"));
    }
}
