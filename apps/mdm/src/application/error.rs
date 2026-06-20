use crate::domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("device {0} is not enrolled / reachable")]
    DeviceNotReachable(String),
}
