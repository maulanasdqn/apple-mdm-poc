use sqlx::SqlitePool;

pub async fn run(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("running database migrations");
    sqlx::migrate!("./migrations").run(pool).await
}
