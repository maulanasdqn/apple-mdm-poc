use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

pub type DbPool = SqlitePool;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("failed to connect to database: {0}")]
    Connect(#[from] sqlx::Error),
}

pub async fn connect(database_url: &str) -> Result<DbPool, DatabaseError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
    Ok(pool)
}
