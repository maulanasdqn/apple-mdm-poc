#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("command not found: {0}")]
    CommandNotFound(String),

    #[error("invalid MDM message: {0}")]
    InvalidMessage(String),

    #[error("plist serialization error: {0}")]
    Plist(String),

    #[error("internal error: {0}")]
    Internal(String),
}
