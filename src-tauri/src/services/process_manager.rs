use crate::error::{AppError, Result};
use crate::types::{PortForwardConfig, ProcessInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct ProcessManager {
  processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
}

impl ProcessManager {
  pub fn new() -> Self {
    Self {
      processes: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn add_process(&self, name: String, pid: u32, config: PortForwardConfig) -> Result<()> {
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
    Ok(())
  }

  pub fn remove_process(&self, name: &str) -> Result<Option<u32>> {
    let mut processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    Ok(processes.remove(name).map(|info| info.pid))
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

  pub fn update_process_name(&self, old_name: &str, new_name: String) -> Result<()> {
    let mut processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    if let Some(mut process_info) = processes.remove(old_name) {
      process_info.config.name = new_name.clone();
      processes.insert(new_name, process_info);
    }

    Ok(())
  }

  pub fn cleanup_all(&self) -> Result<Vec<u32>> {
    let mut processes = self
      .processes
      .lock()
      .map_err(|_| AppError::Process("Failed to acquire lock".to_string()))?;

    let pids: Vec<u32> = processes.values().map(|info| info.pid).collect();
    processes.clear();

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
