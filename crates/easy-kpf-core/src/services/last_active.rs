use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Default)]
struct LastActiveState {
  names: BTreeSet<String>,
}

#[derive(Clone)]
pub struct LastActiveSet {
  inner: Arc<Mutex<BTreeSet<String>>>,
  file_path: Option<PathBuf>,
}

impl LastActiveSet {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(BTreeSet::new())),
      file_path: None,
    }
  }

  pub fn with_file(file_path: PathBuf) -> Self {
    let set = Self {
      inner: Arc::new(Mutex::new(BTreeSet::new())),
      file_path: Some(file_path),
    };
    if let Err(e) = set.load() {
      log::warn!("Failed to load last-active state: {}", e);
    }
    set
  }

  fn load(&self) -> Result<()> {
    let Some(ref path) = self.file_path else {
      return Ok(());
    };
    if !path.exists() {
      return Ok(());
    }
    let json = std::fs::read_to_string(path)
      .map_err(|e| AppError::System(format!("Failed to read last-active file: {}", e)))?;
    let state: LastActiveState = serde_json::from_str(&json)
      .map_err(|e| AppError::System(format!("Failed to parse last-active file: {}", e)))?;
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| AppError::System("Failed to acquire last-active lock".to_string()))?;
    *guard = state.names;
    Ok(())
  }

  fn save_locked(&self, names: &BTreeSet<String>) -> Result<()> {
    let Some(ref path) = self.file_path else {
      return Ok(());
    };
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|e| AppError::System(format!("Failed to create state dir: {}", e)))?;
    }
    let state = LastActiveState {
      names: names.clone(),
    };
    let json = serde_json::to_string_pretty(&state)
      .map_err(|e| AppError::System(format!("Failed to serialize last-active state: {}", e)))?;
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, json)
      .map_err(|e| AppError::System(format!("Failed to write temp last-active file: {}", e)))?;
    std::fs::rename(&temp_path, path)
      .map_err(|e| AppError::System(format!("Failed to rename last-active file: {}", e)))?;
    Ok(())
  }

  pub fn add(&self, name: &str) -> Result<()> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| AppError::System("Failed to acquire last-active lock".to_string()))?;
    if guard.insert(name.to_string()) {
      self.save_locked(&guard)?;
    }
    Ok(())
  }

  pub fn remove(&self, name: &str) -> Result<()> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| AppError::System("Failed to acquire last-active lock".to_string()))?;
    if guard.remove(name) {
      self.save_locked(&guard)?;
    }
    Ok(())
  }

  pub fn rename(&self, old: &str, new: &str) -> Result<()> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| AppError::System("Failed to acquire last-active lock".to_string()))?;
    if guard.remove(old) {
      guard.insert(new.to_string());
      self.save_locked(&guard)?;
    }
    Ok(())
  }

  pub fn names(&self) -> Result<Vec<String>> {
    let guard = self
      .inner
      .lock()
      .map_err(|_| AppError::System("Failed to acquire last-active lock".to_string()))?;
    Ok(guard.iter().cloned().collect())
  }
}

impl Default for LastActiveSet {
  fn default() -> Self {
    Self::new()
  }
}
