use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;

pub struct Logger {
  file: Mutex<std::fs::File>,
}

impl Logger {
  pub fn new(path: &Path) -> anyhow::Result<Self> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(Self {
      file: Mutex::new(file),
    })
  }

  pub fn log(&self, level: &str, message: &str) {
    let ts = Utc::now().to_rfc3339();
    let line = format!("[{ts}] {level}: {message}\n");
    if let Ok(mut file) = self.file.lock() {
      let _ = file.write_all(line.as_bytes());
    }
  }
}
