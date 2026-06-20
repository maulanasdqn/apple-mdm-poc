use crate::domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("crypto error: {0}")]
    Crypto(#[from] openssl::error::ErrorStack),

    #[error("push error: {0}")]
    Push(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl From<InfraError> for DomainError {
    fn from(e: InfraError) -> Self {
        DomainError::Internal(e.to_string())
    }
}
