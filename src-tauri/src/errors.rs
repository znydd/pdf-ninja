#![allow(dead_code)]
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: failed to detect local IP address")]
    NetworkDetection(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Upload error: {0}")]
    Upload(String),

    #[error("Server is already running")]
    AlreadyRunning,

    #[error("Server is not running")]
    NotRunning,
}

// Tauri commands require errors to implement `Into<InvokeError>`,
// which is satisfied by implementing `Serialize`.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
