use config::DbPool;
use uuid::Uuid;

use crate::domain::{
    entities::CommandEnvelope,
    ports::{command_queue::QueuedCommand, CommandQueue},
    DomainError,
};
use crate::infrastructure::InfraError;

#[derive(Clone)]
pub struct SqliteCommandQueue {
    pool: DbPool,
}

impl SqliteCommandQueue {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CommandQueue for SqliteCommandQueue {
    async fn enqueue(&self, udid: &str, command: &CommandEnvelope) -> Result<Uuid, DomainError> {
        let payload = command.to_xml()?;
        let uuid = command.command_uuid;
        sqlx::query(
            r#"INSERT INTO command_queue
                  (command_uuid, udid, request_type, payload_plist, status)
               VALUES (?1, ?2, ?3, ?4, 'pending')"#,
        )
        .bind(uuid.to_string())
        .bind(udid)
        .bind(command.command.request_type())
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(uuid)
    }

    async fn next_for(&self, udid: &str) -> Result<Option<QueuedCommand>, DomainError> {
        let row: Option<(String, String, Vec<u8>)> = sqlx::query_as(
            r#"SELECT command_uuid, request_type, payload_plist
               FROM command_queue
               WHERE udid = ?1 AND status = 'pending'
               ORDER BY created_at ASC
               LIMIT 1"#,
        )
        .bind(udid)
        .fetch_optional(&self.pool)
        .await
        .map_err(InfraError::from)?;

        let Some((uuid_str, request_type, payload)) = row else {
            return Ok(None);
        };

        sqlx::query(
            r#"UPDATE command_queue SET status = 'sent', updated_at = datetime('now')
               WHERE command_uuid = ?1"#,
        )
        .bind(&uuid_str)
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;

        let command_uuid = Uuid::parse_str(&uuid_str)
            .map_err(|e| DomainError::Internal(format!("bad command uuid: {e}")))?;

        Ok(Some(QueuedCommand {
            command_uuid,
            udid: udid.to_string(),
            request_type,
            payload_plist: payload,
        }))
    }

    async fn acknowledge(
        &self,
        command_uuid: &Uuid,
        udid: &str,
        status: &str,
        response_plist: Option<&[u8]>,
    ) -> Result<(), DomainError> {
        let mapped = map_status(status);
        sqlx::query(
            r#"UPDATE command_queue SET status = ?2, updated_at = datetime('now')
               WHERE command_uuid = ?1"#,
        )
        .bind(command_uuid.to_string())
        .bind(mapped)
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;

        sqlx::query(
            r#"INSERT INTO command_results (command_uuid, udid, status, response_plist)
               VALUES (?1, ?2, ?3, ?4)"#,
        )
        .bind(command_uuid.to_string())
        .bind(udid)
        .bind(status)
        .bind(response_plist)
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(())
    }

    async fn requeue(&self, command_uuid: &Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE command_queue SET status = 'pending', updated_at = datetime('now')
               WHERE command_uuid = ?1"#,
        )
        .bind(command_uuid.to_string())
        .execute(&self.pool)
        .await
        .map_err(InfraError::from)?;
        Ok(())
    }
}

fn map_status(status: &str) -> &'static str {
    match status {
        "Acknowledged" => "acknowledged",
        "NotNow" => "not_now",
        _ => "error",
    }
}
