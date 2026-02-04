use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptBankError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    #[error("Invalid prompt category: {0}")]
    InvalidCategory(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, PromptBankError>;
