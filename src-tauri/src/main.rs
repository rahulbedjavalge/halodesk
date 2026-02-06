#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod logger;
mod models;
mod router;
mod storage;

use std::{path::PathBuf, sync::Arc, time::Instant};

use anyhow::Context;
use tauri::{GlobalShortcutManager, Manager, State};
use tokio::sync::RwLock;

use config::{load_or_init, save_config, AppConfig};
use router::{run_router, RouterState};
use storage::init_db;

struct AppState {
  router_port: u16,
  config_path: PathBuf,
  config: Arc<RwLock<AppConfig>>,
  log_path: PathBuf,
}

#[tauri::command]
fn router_port(state: State<'_, AppState>) -> u16 {
  state.router_port
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
  Ok(state.config.read().await.clone())
}

#[tauri::command]
async fn set_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
  save_config(&state.config_path, &config).map_err(|e| e.to_string())?;
  *state.config.write().await = config;
  Ok(())
}

#[tauri::command]
fn set_openrouter_key(key: String) -> Result<(), String> {
  let entry = keyring::Entry::new("HaloRouter", "openrouter").map_err(|e| e.to_string())?;
  entry.set_password(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn has_openrouter_key() -> bool {
  keyring::Entry::new("HaloRouter", "openrouter")
    .and_then(|e| e.get_password())
    .map(|p| !p.is_empty())
    .unwrap_or(false)
}

#[tauri::command]
fn capture_primary_display() -> Result<models::ImageData, String> {
  capture::capture_primary_display().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_log_path(state: State<'_, AppState>) -> String {
  state.log_path.display().to_string()
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      (|| -> anyhow::Result<()> {
        let data_dir = app
          .path_resolver()
          .app_data_dir()
          .context("missing app data dir")?;
        std::fs::create_dir_all(&data_dir)?;

        let config_path = data_dir.join("config.json");
        let db_path = data_dir.join("halodesk.sqlite3");
        let log_path = data_dir.join("halodesk.log");

        let config = load_or_init(&config_path)?;
        let config = Arc::new(RwLock::new(config));

        let db = init_db(&db_path)?;
        let db = Arc::new(tokio::sync::Mutex::new(db));

        let logger = Arc::new(logger::Logger::new(&log_path)?);
        logger.log("INFO", "HaloDesk starting up");

        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();

        let router_state = RouterState {
          started_at: Instant::now(),
          config: config.clone(),
          db,
          logger: logger.clone(),
          port,
        };

        tauri::async_runtime::spawn(async move {
          if let Err(err) = run_router(listener, router_state).await {
            eprintln!("router error: {err}");
          }
        });

        app.manage(AppState {
          router_port: port,
          config_path,
          config,
          log_path,
        });

        if let Some(window) = app.get_window("main") {
          let _ = window.set_content_protected(true);
        }

        let handle = app.handle();
        let mut gsm = handle.global_shortcut_manager();
        let _ = gsm.register("CmdOrCtrl+Shift+Space", move || {
          if let Some(window) = handle.get_window("main") {
            let visible = window.is_visible().unwrap_or(true);
            if visible {
              let _ = window.hide();
            } else {
              let _ = window.show();
              let _ = window.set_focus();
            }
          }
        });

        Ok(())
      })()
      .map_err(|e| e.into())
    })
    .invoke_handler(tauri::generate_handler![
      router_port,
      get_config,
      set_config,
      set_openrouter_key,
      has_openrouter_key,
      capture_primary_display,
      get_log_path
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
