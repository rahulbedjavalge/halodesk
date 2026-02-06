use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;

use crate::models::{MemoryItem, MemoryQueryRequest, MemoryQueryResponse, MemoryStoreRequest, MemoryStoreResponse, Message};

pub fn init_db(path: &Path) -> anyhow::Result<Connection> {
  let conn = Connection::open(path)?;
  conn.execute_batch(
    "
    CREATE TABLE IF NOT EXISTS history (
      id TEXT PRIMARY KEY,
      created_at TEXT NOT NULL,
      messages_json TEXT NOT NULL,
      model TEXT,
      provider TEXT
    );
    CREATE TABLE IF NOT EXISTS pinned (
      id TEXT PRIMARY KEY,
      created_at TEXT NOT NULL,
      text TEXT NOT NULL,
      tags_json TEXT
    );
    CREATE TABLE IF NOT EXISTS presets (
      id TEXT PRIMARY KEY,
      created_at TEXT NOT NULL,
      name TEXT NOT NULL,
      system_prompt TEXT,
      constraints_json TEXT,
      routing_policy_json TEXT
    );
    CREATE TABLE IF NOT EXISTS settings (
      id TEXT PRIMARY KEY,
      created_at TEXT NOT NULL,
      key TEXT NOT NULL,
      value_json TEXT NOT NULL
    );
    ",
  )?;
  Ok(conn)
}

pub async fn store_history(
  db: &Mutex<Connection>,
  messages: &[Message],
  assistant: &str,
  model: &str,
  provider: &str,
) -> anyhow::Result<String> {
  let mut all = messages.to_vec();
  if !assistant.trim().is_empty() {
    all.push(Message {
      role: "assistant".to_string(),
      content: assistant.to_string(),
    });
  }

  let messages_json = serde_json::to_string(&all)?;
  let id = uuid::Uuid::new_v4().to_string();
  let created_at = Utc::now().to_rfc3339();
  let conn = db.lock().await;
  conn.execute(
    "INSERT INTO history (id, created_at, messages_json, model, provider) VALUES (?1, ?2, ?3, ?4, ?5)",
    params![id, created_at, messages_json, model, provider],
  )?;
  Ok(id)
}

pub async fn memory_store(
  db: &Mutex<Connection>,
  req: MemoryStoreRequest,
) -> anyhow::Result<MemoryStoreResponse> {
  let id = uuid::Uuid::new_v4().to_string();
  let created_at = Utc::now().to_rfc3339();
  let conn = db.lock().await;

  match req.r#type.as_str() {
    "history" => {
      let messages_json = req.payload.to_string();
      conn.execute(
        "INSERT INTO history (id, created_at, messages_json, model, provider) VALUES (?1, ?2, ?3, NULL, NULL)",
        params![id, created_at, messages_json],
      )?;
    }
    "pinned" => {
      let text = req
        .payload
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
      let tags = req
        .payload
        .get("tags")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());
      conn.execute(
        "INSERT INTO pinned (id, created_at, text, tags_json) VALUES (?1, ?2, ?3, ?4)",
        params![id, created_at, text, tags],
      )?;
    }
    "preset" => {
      let name = req
        .payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
      let system_prompt = req
        .payload
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");
      let constraints = req
        .payload
        .get("constraints")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());
      let routing = req
        .payload
        .get("routing_policy")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());
      conn.execute(
        "INSERT INTO presets (id, created_at, name, system_prompt, constraints_json, routing_policy_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, created_at, name, system_prompt, constraints, routing],
      )?;
    }
    "settings" => {
      let key = req
        .payload
        .get("key")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
      let value = req
        .payload
        .get("value")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string());
      conn.execute(
        "INSERT INTO settings (id, created_at, key, value_json) VALUES (?1, ?2, ?3, ?4)",
        params![id, created_at, key, value],
      )?;
    }
    _ => return Err(anyhow::anyhow!("Unsupported memory type.")),
  }

  Ok(MemoryStoreResponse { id, stored_at: created_at })
}

pub async fn memory_query(
  db: &Mutex<Connection>,
  req: MemoryQueryRequest,
) -> anyhow::Result<MemoryQueryResponse> {
  let start = Instant::now();
  let limit = req.limit.unwrap_or(20);
  let like = format!("%{}%", req.query);
  let conn = db.lock().await;

  let mut items: Vec<MemoryItem> = Vec::new();

  let mut stmt = conn.prepare(
    "SELECT id, created_at, messages_json, model, provider FROM history WHERE messages_json LIKE ?1 ORDER BY created_at DESC LIMIT ?2",
  )?;
  let rows = stmt.query_map(params![like, limit], |row| {
    Ok((
      row.get::<_, String>(0)?,
      row.get::<_, String>(1)?,
      row.get::<_, String>(2)?,
      row.get::<_, Option<String>>(3)?,
      row.get::<_, Option<String>>(4)?,
    ))
  })?;

  for row in rows {
    let (id, created_at, messages_json, model, provider) = row?;
    let payload: serde_json::Value = serde_json::from_str(&messages_json)
      .unwrap_or(serde_json::Value::String(messages_json));
    items.push(MemoryItem {
      r#type: "history".to_string(),
      payload: serde_json::json!({
        "id": id,
        "created_at": created_at,
        "messages": payload,
        "model": model,
        "provider": provider
      }),
    });
  }

  let mut stmt = conn.prepare(
    "SELECT id, created_at, text, tags_json FROM pinned WHERE text LIKE ?1 ORDER BY created_at DESC LIMIT ?2",
  )?;
  let rows = stmt.query_map(params![like, limit], |row| {
    Ok((
      row.get::<_, String>(0)?,
      row.get::<_, String>(1)?,
      row.get::<_, String>(2)?,
      row.get::<_, Option<String>>(3)?,
    ))
  })?;

  for row in rows {
    let (id, created_at, text, tags_json) = row?;
    let tags: serde_json::Value = tags_json
      .and_then(|t| serde_json::from_str(&t).ok())
      .unwrap_or(serde_json::Value::Array(vec![]));
    items.push(MemoryItem {
      r#type: "pinned".to_string(),
      payload: serde_json::json!({
        "id": id,
        "created_at": created_at,
        "text": text,
        "tags": tags
      }),
    });
  }

  let mut stmt = conn.prepare(
    "SELECT id, created_at, name, system_prompt, constraints_json, routing_policy_json FROM presets WHERE name LIKE ?1 ORDER BY created_at DESC LIMIT ?2",
  )?;
  let rows = stmt.query_map(params![like, limit], |row| {
    Ok((
      row.get::<_, String>(0)?,
      row.get::<_, String>(1)?,
      row.get::<_, String>(2)?,
      row.get::<_, Option<String>>(3)?,
      row.get::<_, Option<String>>(4)?,
      row.get::<_, Option<String>>(5)?,
    ))
  })?;

  for row in rows {
    let (id, created_at, name, system_prompt, constraints_json, routing_json) = row?;
    let constraints: serde_json::Value = constraints_json
      .and_then(|c| serde_json::from_str(&c).ok())
      .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let routing: serde_json::Value = routing_json
      .and_then(|c| serde_json::from_str(&c).ok())
      .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    items.push(MemoryItem {
      r#type: "preset".to_string(),
      payload: serde_json::json!({
        "id": id,
        "created_at": created_at,
        "name": name,
        "system_prompt": system_prompt,
        "constraints": constraints,
        "routing_policy": routing
      }),
    });
  }

  Ok(MemoryQueryResponse {
    items,
    took_ms: start.elapsed().as_millis() as i64,
  })
}
