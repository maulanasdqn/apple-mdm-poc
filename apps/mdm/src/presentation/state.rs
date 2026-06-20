use std::sync::Arc;

use config::{Config, DbPool};

use crate::domain::ports::{CertProvider, CommandQueue, DeviceRepository, PushNotifier};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: DbPool,
    pub devices: Arc<dyn DeviceRepository>,
    pub queue: Arc<dyn CommandQueue>,
    pub push: Arc<dyn PushNotifier>,
    pub cert: Arc<dyn CertProvider>,
}
