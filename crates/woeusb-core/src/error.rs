use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WoeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(PathBuf),

    #[error("Device is busy: {0}")]
    DeviceBusy(String),

    #[error("Invalid source media: {0}")]
    InvalidSource(String),

    #[error("Command failed: '{command}' exited with code {code:?}. Details:\n{stderr}")]
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("Missing system dependency: '{0}'. Please install the required package.")]
    MissingDependency(String),

    #[error("Insufficient disk space on target: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },

    #[error("Operation was cancelled by user")]
    Cancelled,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, WoeError>;
