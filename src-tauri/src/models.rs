use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
  pub role: String,
  pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageData {
  pub mime: String,
  pub base64: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatRequest {
  pub preset_id: Option<String>,
  pub messages: Vec<Message>,
  pub image: Option<ImageData>,
  pub model_override: Option<String>,
  pub stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInfo {
  pub id: String,
  pub label: String,
  pub capability: String,
}

#[derive(Serialize, Deserialize)]
pub struct ModelsResponse {
  pub text_default: String,
  pub vision_default: String,
  pub models: Vec<ModelInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryStoreRequest {
  pub r#type: String,
  pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryStoreResponse {
  pub id: String,
  pub stored_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryQueryRequest {
  pub query: String,
  pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryQueryResponse {
  pub items: Vec<MemoryItem>,
  pub took_ms: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryItem {
  pub r#type: String,
  pub payload: serde_json::Value,
}
