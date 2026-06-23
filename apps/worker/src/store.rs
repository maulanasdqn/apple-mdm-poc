use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

fn s(v: &str) -> JsValue {
    JsValue::from(v)
}

fn opt(v: Option<&str>) -> JsValue {
    match v {
        Some(x) => JsValue::from(x),
        None => JsValue::NULL,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRow {
    pub udid: String,
    pub push_token: Option<String>,
    pub push_magic: Option<String>,
    pub topic: Option<String>,
    pub enrollment_state: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandRow {
    pub command_uuid: String,
    pub command_json: String,
}

pub async fn upsert_authenticate(db: &D1Database, udid: &str, topic: Option<&str>) -> Result<()> {
    db.prepare(
        "INSERT INTO devices (udid, topic, enrollment_state, updated_at) \
         VALUES (?1, ?2, 'authenticated', datetime('now')) \
         ON CONFLICT(udid) DO UPDATE SET topic=COALESCE(?2, topic), updated_at=datetime('now')",
    )
    .bind(&[s(udid), opt(topic)])?
    .run()
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_token(
    db: &D1Database,
    udid: &str,
    push_token: &str,
    push_magic: &str,
    topic: &str,
    unlock_token: Option<&str>,
) -> Result<()> {
    db.prepare(
        "INSERT INTO devices (udid, push_token, push_magic, topic, unlock_token, enrollment_state, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'enrolled', datetime('now')) \
         ON CONFLICT(udid) DO UPDATE SET \
           push_token=?2, push_magic=?3, topic=?4, \
           unlock_token=COALESCE(?5, unlock_token), \
           enrollment_state='enrolled', updated_at=datetime('now')",
    )
    .bind(&[s(udid), s(push_token), s(push_magic), s(topic), opt(unlock_token)])?
    .run()
    .await?;
    Ok(())
}

pub async fn checkout(db: &D1Database, udid: &str) -> Result<()> {
    db.prepare(
        "UPDATE devices SET enrollment_state='checked_out', updated_at=datetime('now') WHERE udid=?1",
    )
    .bind(&[s(udid)])?
    .run()
    .await?;
    Ok(())
}

pub async fn list_devices(db: &D1Database) -> Result<Vec<DeviceRow>> {
    db.prepare(
        "SELECT udid, push_token, push_magic, topic, enrollment_state FROM devices ORDER BY updated_at DESC",
    )
    .all()
    .await?
    .results::<DeviceRow>()
}

pub async fn device_exists(db: &D1Database, udid: &str) -> Result<bool> {
    let row: Option<DeviceRow> = db
        .prepare("SELECT udid, push_token, push_magic, topic, enrollment_state FROM devices WHERE udid=?1")
        .bind(&[s(udid)])?
        .first(None)
        .await?;
    Ok(row.is_some())
}

pub async fn enqueue(
    db: &D1Database,
    command_uuid: &str,
    udid: &str,
    request_type: &str,
    command_json: &str,
) -> Result<()> {
    db.prepare(
        "INSERT INTO command_queue (command_uuid, udid, request_type, command_json, status, created_at) \
         VALUES (?1, ?2, ?3, ?4, 'pending', datetime('now'))",
    )
    .bind(&[s(command_uuid), s(udid), s(request_type), s(command_json)])?
    .run()
    .await?;
    Ok(())
}

pub async fn next_pending(db: &D1Database, udid: &str) -> Result<Option<CommandRow>> {
    db.prepare(
        "SELECT command_uuid, command_json FROM command_queue \
         WHERE udid=?1 AND status='pending' ORDER BY created_at ASC LIMIT 1",
    )
    .bind(&[s(udid)])?
    .first(None)
    .await
}

pub async fn mark_sent(db: &D1Database, command_uuid: &str) -> Result<()> {
    db.prepare("UPDATE command_queue SET status='sent', updated_at=datetime('now') WHERE command_uuid=?1")
        .bind(&[s(command_uuid)])?
        .run()
        .await?;
    Ok(())
}

pub async fn mark_acknowledged(db: &D1Database, command_uuid: &str) -> Result<()> {
    db.prepare("UPDATE command_queue SET status='acknowledged', updated_at=datetime('now') WHERE command_uuid=?1")
        .bind(&[s(command_uuid)])?
        .run()
        .await?;
    Ok(())
}
