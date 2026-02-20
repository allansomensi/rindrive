use rindrive_core::error::AuditError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] AuditError),

    #[error("Failed to read user input: {0}")]
    Io(#[from] std::io::Error),

    #[error("Operation aborted by user.")]
    UserAbort,
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::UserAbort => 130,
            CliError::Io(_) => 74,
            CliError::Core(_) => 1,
        }
    }
}
