use std::io;
use thiserror::Error;

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

pub type Result<T> = std::result::Result<T, AlxError>;
