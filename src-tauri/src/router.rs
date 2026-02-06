use std::net::TcpListener;
use std::sync::Arc;
use std::time::Instant;

use async_stream::stream;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use tokio::sync::{Mutex, RwLock};
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};

use crate::config::AppConfig;
use crate::models::{ChatRequest, ImageData, MemoryQueryRequest, MemoryStoreRequest, Message, ModelsResponse};
use crate::storage;

pub struct RouterState {
  pub started_at: Instant,
  pub config: Arc<RwLock<AppConfig>>,
  pub db: Arc<Mutex<rusqlite::Connection>>,
}

pub async fn run_router(listener: TcpListener, state: RouterState) -> anyhow::Result<()> {
  let app = Router::new()
    .route("/health", get(health))
    .route("/v1/models", get(models))
    .route("/v1/chat", post(chat))
    .route("/v1/memory/store", post(memory_store))
    .route("/v1/memory/query", post(memory_query))
    .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
    .with_state(Arc::new(state));

  let listener = tokio::net::TcpListener::from_std(listener)?;
  axum::serve(listener, app).await?;
  Ok(())
}

async fn health(State(state): State<Arc<RouterState>>) -> Json<serde_json::Value> {
  let uptime = state.started_at.elapsed().as_millis();
  Json(serde_json::json!({
    "status": "ok",
    "version": "1.0.0",
    "uptime_ms": uptime
  }))
}

async fn models(State(state): State<Arc<RouterState>>) -> Json<ModelsResponse> {
  let config = state.config.read().await.clone();
  Json(ModelsResponse {
    text_default: config.text_default_model,
    vision_default: config.vision_default_model,
    models: config.models,
  })
}

async fn memory_store(
  State(state): State<Arc<RouterState>>,
  Json(req): Json<MemoryStoreRequest>,
) -> impl IntoResponse {
  match storage::memory_store(&state.db, req).await {
    Ok(res) => (StatusCode::OK, Json(res)).into_response(),
    Err(err) => error_response(StatusCode::BAD_REQUEST, "memory_store_failed", &err.to_string()),
  }
}

async fn memory_query(
  State(state): State<Arc<RouterState>>,
  Json(req): Json<MemoryQueryRequest>,
) -> impl IntoResponse {
  match storage::memory_query(&state.db, req).await {
    Ok(res) => (StatusCode::OK, Json(res)).into_response(),
    Err(err) => error_response(StatusCode::BAD_REQUEST, "memory_query_failed", &err.to_string()),
  }
}

async fn chat(
  State(state): State<Arc<RouterState>>,
  Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
  let config = state.config.read().await.clone();
  let model_id = match resolve_model(&req, &config) {
    Ok(m) => m,
    Err(msg) => return error_response(StatusCode::BAD_REQUEST, "model_missing", &msg),
  };

  let (provider, model) = split_provider(&model_id);
  if provider != "openrouter" {
    return error_response(
      StatusCode::BAD_REQUEST,
      "provider_unsupported",
      "Only openrouter is supported in MVP.",
    );
  }

  let key = match get_openrouter_key() {
    Ok(k) => k,
    Err(msg) => return error_response(StatusCode::BAD_REQUEST, "key_missing", &msg),
  };

  let stream = req.stream.unwrap_or(true);
  if stream {
    match stream_openrouter(state, req, &model_id, &model, &key).await {
      Ok(sse) => sse.into_response(),
      Err((status, message)) => error_response(status, "openrouter_error", &message),
    }
  } else {
    match complete_openrouter(state, req, &model_id, &model, &key).await {
      Ok(res) => (StatusCode::OK, Json(res)).into_response(),
      Err((status, message)) => error_response(status, "openrouter_error", &message),
    }
  }
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
  let body = Json(serde_json::json!({ "error": message, "code": code }));
  (status, body).into_response()
}

fn split_provider(model_id: &str) -> (String, String) {
  const PREFIX: &str = "openrouter:";
  if model_id.starts_with(PREFIX) {
    ("openrouter".to_string(), model_id[PREFIX.len()..].to_string())
  } else {
    ("openrouter".to_string(), model_id.to_string())
  }
}

fn resolve_model(req: &ChatRequest, config: &AppConfig) -> Result<String, String> {
  if let Some(override_id) = req.model_override.as_ref() {
    if !override_id.trim().is_empty() {
      return Ok(override_id.trim().to_string());
    }
  }

  if req.image.is_some() {
    if config.vision_default_model.trim().is_empty() {
      return Err("Vision default model not set.".to_string());
    }
    return Ok(config.vision_default_model.clone());
  }

  if config.text_default_model.trim().is_empty() {
    return Err("Text default model not set.".to_string());
  }
  Ok(config.text_default_model.clone())
}

fn get_openrouter_key() -> Result<String, String> {
  let entry = keyring::Entry::new("HaloRouter", "openrouter").map_err(|e| e.to_string())?;
  let key = entry
    .get_password()
    .map_err(|_| "OpenRouter key missing. Set it in Settings.".to_string())?;
  if key.trim().is_empty() {
    Err("OpenRouter key missing. Set it in Settings.".to_string())
  } else {
    Ok(key)
  }
}

#[derive(serde::Serialize)]
struct OpenRouterMessage {
  role: String,
  content: serde_json::Value,
}

#[derive(serde::Serialize)]
struct OpenRouterChatRequest {
  model: String,
  messages: Vec<OpenRouterMessage>,
  stream: bool,
}

fn to_openrouter_messages(messages: &[Message], image: Option<&ImageData>) -> Vec<OpenRouterMessage> {
  let mut result = Vec::new();
  let mut image_attached = false;
  let last_user_index = messages.iter().rposition(|m| m.role == "user");

  for (idx, msg) in messages.iter().enumerate() {
    if Some(idx) == last_user_index && image.is_some() && !image_attached {
      let img = image.unwrap();
      let url = format!("data:{};base64,{}", img.mime, img.base64);
      let content = serde_json::json!([
        { "type": "text", "text": msg.content },
        { "type": "image_url", "image_url": { "url": url } }
      ]);
      result.push(OpenRouterMessage {
        role: msg.role.clone(),
        content,
      });
      image_attached = true;
    } else {
      result.push(OpenRouterMessage {
        role: msg.role.clone(),
        content: serde_json::json!(msg.content),
      });
    }
  }

  if image.is_some() && !image_attached {
    let img = image.unwrap();
    let url = format!("data:{};base64,{}", img.mime, img.base64);
    let content = serde_json::json!([
      { "type": "text", "text": "" },
      { "type": "image_url", "image_url": { "url": url } }
    ]);
    result.push(OpenRouterMessage {
      role: "user".to_string(),
      content,
    });
  }

  result
}

async fn stream_openrouter(
  state: Arc<RouterState>,
  req: ChatRequest,
  model_id: &str,
  model: &str,
  key: &str,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
  let req_clone = req.clone();
  let messages = to_openrouter_messages(&req.messages, req.image.as_ref());

  let client = reqwest::Client::new();
  let mut headers = HeaderMap::new();
  headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {}", key))
      .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?,
  );
  headers.insert("HTTP-Referer", HeaderValue::from_static("http://localhost"));
  headers.insert("X-Title", HeaderValue::from_static("HaloDesk"));

  let payload = OpenRouterChatRequest {
    model: model.to_string(),
    messages,
    stream: true,
  };

  let resp = client
    .post("https://openrouter.ai/api/v1/chat/completions")
    .headers(headers)
    .json(&payload)
    .send()
    .await
    .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;

  if !resp.status().is_success() {
    let upstream_status = resp.status();
    let text = resp
      .text()
      .await
      .unwrap_or_else(|_| "OpenRouter request failed.".to_string());
    let status = StatusCode::BAD_GATEWAY;
    let message = format!("OpenRouter error ({}): {}", upstream_status, text);
    return Err((status, message));
  }

  let mut bytes_stream = resp.bytes_stream();
  let model_id = model_id.to_string();

  let stream = stream! {
    let meta = serde_json::json!({ "model": model_id, "provider": "openrouter" }).to_string();
    yield Ok(Event::default().event("meta").data(meta));

    let mut buffer = String::new();
    let mut full = String::new();
    let mut finish_reason = "stop".to_string();

    while let Some(chunk) = bytes_stream.next().await {
      let chunk = match chunk {
        Ok(c) => c,
        Err(err) => {
          let done = serde_json::json!({ "finish_reason": "error", "error": err.to_string() }).to_string();
          yield Ok(Event::default().event("done").data(done));
          return;
        }
      };

      buffer.push_str(&String::from_utf8_lossy(&chunk));
      loop {
        let boundary = buffer.find("\n\n");
        if boundary.is_none() {
          break;
        }
        let boundary = boundary.unwrap();
        let block = buffer[..boundary].to_string();
        buffer = buffer[boundary + 2..].to_string();

        for line in block.lines() {
          if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim();
            if data == "[DONE]" {
              let _ = storage::store_history(&state.db, &req_clone.messages, &full, &model_id, "openrouter").await;
              let done = serde_json::json!({ "finish_reason": finish_reason }).to_string();
              yield Ok(Event::default().event("done").data(done));
              return;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
              if let Some(reason) = value["choices"][0]["finish_reason"].as_str() {
                finish_reason = reason.to_string();
              }

              if let Some(delta) = value["choices"][0]["delta"]["content"].as_str() {
                if !delta.is_empty() {
                  full.push_str(delta);
                  let payload = serde_json::json!({ "text": delta }).to_string();
                  yield Ok(Event::default().event("delta").data(payload));
                }
              }
            }
          }
        }
      }
    }

    let _ = storage::store_history(&state.db, &req_clone.messages, &full, &model_id, "openrouter").await;
    let done = serde_json::json!({ "finish_reason": finish_reason }).to_string();
    yield Ok(Event::default().event("done").data(done));
  };

  Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15))))
}

async fn complete_openrouter(
  state: Arc<RouterState>,
  req: ChatRequest,
  model_id: &str,
  model: &str,
  key: &str,
) -> Result<serde_json::Value, (StatusCode, String)> {
  let messages = to_openrouter_messages(&req.messages, req.image.as_ref());

  let client = reqwest::Client::new();
  let mut headers = HeaderMap::new();
  headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {}", key))
      .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?,
  );
  headers.insert("HTTP-Referer", HeaderValue::from_static("http://localhost"));
  headers.insert("X-Title", HeaderValue::from_static("HaloDesk"));

  let payload = OpenRouterChatRequest {
    model: model.to_string(),
    messages,
    stream: false,
  };

  let resp = client
    .post("https://openrouter.ai/api/v1/chat/completions")
    .headers(headers)
    .json(&payload)
    .send()
    .await
    .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;

  if !resp.status().is_success() {
    let upstream_status = resp.status();
    let text = resp
      .text()
      .await
      .unwrap_or_else(|_| "OpenRouter request failed.".to_string());
    let status = StatusCode::BAD_GATEWAY;
    let message = format!("OpenRouter error ({}): {}", upstream_status, text);
    return Err((status, message));
  }

  let json_body = resp
    .json::<serde_json::Value>()
    .await
    .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;
  let content = json_body["choices"][0]["message"]["content"]
    .as_str()
    .unwrap_or("")
    .to_string();

  storage::store_history(&state.db, &req.messages, &content, model_id, "openrouter")
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

  Ok(serde_json::json!({
    "text": content,
    "model": model_id,
    "provider": "openrouter"
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::AppConfig;

  fn base_config() -> AppConfig {
    AppConfig {
      text_default_model: "openrouter:text-default".to_string(),
      vision_default_model: "openrouter:vision-default".to_string(),
      fallback_model: "openrouter:fallback".to_string(),
      models: vec![],
    }
  }

  #[test]
  fn split_provider_with_prefix() {
    let (provider, model) = split_provider("openrouter:openai/gpt-4o-mini");
    assert_eq!(provider, "openrouter");
    assert_eq!(model, "openai/gpt-4o-mini");
  }

  #[test]
  fn split_provider_without_prefix() {
    let (provider, model) = split_provider("openai/gpt-4o-mini");
    assert_eq!(provider, "openrouter");
    assert_eq!(model, "openai/gpt-4o-mini");
  }

  #[test]
  fn split_provider_handles_colon_in_model() {
    let (provider, model) = split_provider("nvidia/nemotron-3-nano-30b-a3b:free");
    assert_eq!(provider, "openrouter");
    assert_eq!(model, "nvidia/nemotron-3-nano-30b-a3b:free");
  }

  #[test]
  fn resolve_model_uses_override() {
    let config = base_config();
    let req = ChatRequest {
      preset_id: None,
      messages: vec![],
      image: None,
      model_override: Some("openrouter:override".to_string()),
      stream: Some(true),
    };

    let resolved = resolve_model(&req, &config).expect("override should resolve");
    assert_eq!(resolved, "openrouter:override");
  }

  #[test]
  fn resolve_model_uses_vision_default_when_image_present() {
    let config = base_config();
    let req = ChatRequest {
      preset_id: None,
      messages: vec![],
      image: Some(ImageData {
        mime: "image/png".to_string(),
        base64: "abc".to_string(),
      }),
      model_override: None,
      stream: Some(true),
    };

    let resolved = resolve_model(&req, &config).expect("vision default should resolve");
    assert_eq!(resolved, "openrouter:vision-default");
  }

  #[test]
  fn resolve_model_uses_text_default_without_image() {
    let config = base_config();
    let req = ChatRequest {
      preset_id: None,
      messages: vec![],
      image: None,
      model_override: None,
      stream: Some(true),
    };

    let resolved = resolve_model(&req, &config).expect("text default should resolve");
    assert_eq!(resolved, "openrouter:text-default");
  }

  #[test]
  fn to_openrouter_messages_attaches_image_to_last_user() {
    let messages = vec![
      Message {
        role: "user".to_string(),
        content: "First".to_string(),
      },
      Message {
        role: "assistant".to_string(),
        content: "Ack".to_string(),
      },
      Message {
        role: "user".to_string(),
        content: "Second".to_string(),
      },
    ];
    let image = ImageData {
      mime: "image/png".to_string(),
      base64: "abc".to_string(),
    };
    let result = to_openrouter_messages(&messages, Some(&image));
    assert_eq!(result.len(), 3);
    let last = &result[2];
    assert_eq!(last.role, "user");
    assert!(last.content.is_array());
  }
}
