use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Platform specific error: {0}")]
    Platform(String),
    #[error("Drive not found or access denied")]
    DriveAccessDenied,
}
