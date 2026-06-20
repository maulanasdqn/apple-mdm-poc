use std::sync::Arc;

use config::{ApnsMode, Config};
use mdm::build_router;
use mdm::domain::ports::{CommandQueue, DeviceRepository, PushNotifier};
use mdm::infrastructure::apns::a2_client::RealApnsPush;
use mdm::infrastructure::apns::NoopPush;
use mdm::infrastructure::cert::CertManager;
use mdm::infrastructure::sqlite::{SqliteCommandQueue, SqliteDeviceRepository};
use mdm::presentation::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::init_tracing();

    let cfg = Config::from_env()?;
    tracing::info!(addr = %cfg.server_addr, base_url = %cfg.base_url, "starting rust-apple-mdm");

    let pool = config::connect(&cfg.database_url).await?;
    migrations::run(&pool).await?;

    let devices: Arc<dyn DeviceRepository> = Arc::new(SqliteDeviceRepository::new(pool.clone()));
    let queue: Arc<dyn CommandQueue> = Arc::new(SqliteCommandQueue::new(pool.clone()));
    let cert = Arc::new(CertManager::load_or_generate(&cfg)?);

    let push: Arc<dyn PushNotifier> = match cfg.apns_mode {
        ApnsMode::Real => {
            let cert_path = cfg
                .apns_cert_path
                .clone()
                .ok_or_else(|| anyhow::anyhow!("APNS_MODE=real requires APNS_CERT_PATH"))?;
            let password = cfg.apns_cert_password.clone().unwrap_or_default();
            tracing::info!("APNs: real client (cert auth)");
            Arc::new(RealApnsPush::new(cert_path, password))
        }
        ApnsMode::Noop => {
            tracing::info!("APNs: no-op (set APNS_MODE=real to deliver pushes)");
            Arc::new(NoopPush)
        }
    };

    let state = Arc::new(AppState {
        config: Arc::new(cfg.clone()),
        pool,
        devices,
        queue,
        push,
        cert,
    });

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&cfg.server_addr).await?;
    tracing::info!(addr = %cfg.server_addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
