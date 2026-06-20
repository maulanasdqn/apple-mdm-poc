use config::DbPool;

use crate::domain::{entities::Device, ports::DeviceRepository, DomainError};
use crate::infrastructure::InfraError;

use super::models::DeviceRow;

#[derive(Clone)]
pub struct SqliteDeviceRepository {
    pool: DbPool,
}

impl SqliteDeviceRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DeviceRepository for SqliteDeviceRepository {
    async fn upsert(&self, device: &Device) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO devices
                (udid, push_token, push_magic, topic, unlock_token, enrollment_state, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
            ON CONFLICT(udid) DO UPDATE SET
                push_token       = COALESCE(excluded.push_token, devices.push_token),
                push_magic       = COALESCE(excluded.push_magic, devices.push_magic),
                topic            = COALESCE(excluded.topic, devices.topic),
                unlock_token     = COALESCE(excluded.unlock_token, devices.unlock_token),
                enrollment_state = excluded.enrollment_state,
                updated_at       = datetime('now')
            "#,
        )
        .bind(&device.udid)
        .bind(&device.push_token)
        .bind(&device.push_magic)
        .bind(&device.topic)
        .bind(&device.unlock_token)
        .bind(device.enrollment_state.as_str())
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(())
    }

    async fn get(&self, udid: &str) -> Result<Option<Device>, DomainError> {
        let row: Option<DeviceRow> = sqlx::query_as(
            r#"SELECT udid, push_token, push_magic, topic, unlock_token, enrollment_state
               FROM devices WHERE udid = ?1"#,
        )
        .bind(udid)
        .fetch_optional(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(row.map(Device::from))
    }

    async fn mark_checked_out(&self, udid: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE devices SET enrollment_state = 'checked_out', updated_at = datetime('now')
               WHERE udid = ?1"#,
        )
        .bind(udid)
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Device>, DomainError> {
        let rows: Vec<DeviceRow> = sqlx::query_as(
            r#"SELECT udid, push_token, push_magic, topic, unlock_token, enrollment_state
               FROM devices ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(rows.into_iter().map(Device::from).collect())
    }
}
