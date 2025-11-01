use crate::error::{AppError, Result};
use crate::types::{PortForwardConfig, ProcessInfo, ProcessManagerState, SerializableProcessInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct ProcessManager {
  processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
  state_file_path: Option<PathBuf>,
}

impl ProcessManager {
  pub fn new() -> Self {
    Self {
      processes: Arc::new(Mutex::new(HashMap::new())),
      state_file_path: None,
    }
  }

  pub fn with_state_file(state_file_path: PathBuf) -> Self {
    let manager = Self {
      processes: Arc::new(Mutex::new(HashMap::new())),
      state_file_path: Some(state_file_path),
    };

    // Try to load existing state
    if let Err(e) = manager.load_state() {
      log::warn!("Failed to load process manager state: {}", e);
    }

    manager
  }

  fn save_state(&self) -> Result<()> {
    if let Some(ref path) = self.state_file_path {
      let processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      let state = ProcessManagerState {
        processes: processes
          .iter()
          .map(|(name, info)| (name.clone(), SerializableProcessInfo::from(info)))
          .collect(),
      };

      let json = serde_json::to_string_pretty(&state)
        .map_err(|e| AppError::System(format!("Failed to serialize state: {}", e)))?;

      // Atomic write: write to temp file then rename
      let temp_path = path.with_extension("tmp");
      std::fs::write(&temp_path, json)
        .map_err(|e| AppError::System(format!("Failed to write temp state file: {}", e)))?;

      std::fs::rename(&temp_path, path)
        .map_err(|e| AppError::System(format!("Failed to rename state file: {}", e)))?;

      log::debug!("Saved process manager state to {:?}", path);
    }
    Ok(())
  }

  fn load_state(&self) -> Result<()> {
    if let Some(ref path) = self.state_file_path {
      if !path.exists() {
        log::debug!("No existing state file found at {:?}", path);
        return Ok(());
      }

      let json = std::fs::read_to_string(path)
        .map_err(|e| AppError::System(format!("Failed to read state file: {}", e)))?;

      let state: ProcessManagerState = serde_json::from_str(&json)
        .map_err(|e| AppError::System(format!("Failed to deserialize state: {}", e)))?;

      let mut processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      let process_detector = crate::services::process_detector::ProcessDetector::new();
      let mut loaded_count = 0;
      let mut skipped_count = 0;

      for (name, serializable_info) in state.processes {
        // Verify process is still running before adding
        if process_detector
          .is_process_actually_running(serializable_info.pid)
          .unwrap_or(false)
        {
          let process_info = ProcessInfo::from(serializable_info);
          processes.insert(name, process_info);
          loaded_count += 1;
        } else {
          log::debug!(
            "Skipping dead process {} (PID: {})",
            name,
            serializable_info.pid
          );
          skipped_count += 1;
        }
      }

      log::info!(
        "Loaded {} active processes from state file ({} dead processes skipped)",
        loaded_count,
        skipped_count
      );
    }
    Ok(())
  }

  pub fn add_process(&self, name: String, pid: u32, config: PortForwardConfig) -> Result<()> {
    {
      let mut processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      let process_info = ProcessInfo {
        pid,
        config,
        started_at: Instant::now(),
      };

      processes.insert(name, process_info);
    }

    self.save_state()?;
    Ok(())
  }

  pub fn remove_process(&self, name: &str) -> Result<Option<u32>> {
    let result = {
      let mut processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      processes.remove(name).map(|info| info.pid)
    };

    self.save_state()?;
    Ok(result)
  }

  #[allow(dead_code)]
  pub fn get_process_pid(&self, name: &str) -> Result<Option<u32>> {
    let processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    Ok(processes.get(name).map(|info| info.pid))
  }

  pub fn contains_process(&self, name: &str) -> Result<bool> {
    let processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    Ok(processes.contains_key(name))
  }

  pub fn get_running_services(&self) -> Result<Vec<String>> {
    let processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    Ok(processes.keys().cloned().collect())
  }

  pub fn get_running_services_with_pids(&self) -> Result<Vec<(String, u32)>> {
    let processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    Ok(
      processes
        .iter()
        .map(|(name, info)| (name.clone(), info.pid))
        .collect(),
    )
  }

  pub fn update_process_name(&self, old_name: &str, new_name: String) -> Result<()> {
    {
      let mut processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      if let Some(mut process_info) = processes.remove(old_name) {
        process_info.config.name = new_name.clone();
        processes.insert(new_name, process_info);
      }
    }

    self.save_state()?;
    Ok(())
  }

  pub fn cleanup_all(&self) -> Result<Vec<u32>> {
    let pids = {
      let mut processes = self
        .processes
        .lock()
        .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

      let pids: Vec<u32> = processes.values().map(|info| info.pid).collect();
      processes.clear();
      pids
    };

    self.save_state()?;
    Ok(pids)
  }

  #[cfg(unix)]
  pub fn kill_process(pid: u32) -> Result<()> {
    use std::process::Command;

    let output = Command::new("kill").arg(pid.to_string()).output()?;

    if output.status.success() {
      Ok(())
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      Err(AppError::Process(format!(
        "Failed to kill process: {}",
        error
      )))
    }
  }

  #[cfg(not(unix))]
  pub fn kill_process(_pid: u32) -> Result<()> {
    Err(AppError::Process(
      "Process termination not supported on this platform".to_string(),
    ))
  }
}
