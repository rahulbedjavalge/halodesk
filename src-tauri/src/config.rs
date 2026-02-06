use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::models::ModelInfo;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub text_default_model: String,
  pub vision_default_model: String,
  pub fallback_model: String,
  pub models: Vec<ModelInfo>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      text_default_model: "openrouter:openai/gpt-4o-mini".to_string(),
      vision_default_model: "openrouter:openai/gpt-4o-mini-vision".to_string(),
      fallback_model: "openrouter:openai/gpt-4o-mini".to_string(),
      models: vec![
        ModelInfo {
          id: "openrouter:openai/gpt-4o-mini".to_string(),
          label: "GPT-4o mini".to_string(),
          capability: "text".to_string(),
        },
        ModelInfo {
          id: "openrouter:openai/gpt-4o-mini-vision".to_string(),
          label: "GPT-4o mini (vision)".to_string(),
          capability: "vision".to_string(),
        }
      ],
    }
  }
}

pub fn load_or_init(path: &Path) -> anyhow::Result<AppConfig> {
  if path.exists() {
    let data = std::fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&data)?;
    Ok(config)
  } else {
    let config = AppConfig::default();
    save_config(path, &config)?;
    Ok(config)
  }
}

pub fn save_config(path: &Path, config: &AppConfig) -> anyhow::Result<()> {
  let json = serde_json::to_string_pretty(config)?;
  std::fs::write(path, json)?;
  Ok(())
}
