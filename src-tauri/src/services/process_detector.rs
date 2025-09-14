use crate::error::{AppError, Result};
use crate::types::{ForwardType, PortForwardConfig};
use std::process::Command;

pub struct ProcessDetector;

impl ProcessDetector {
  pub fn new() -> Self {
    Self
  }

  pub fn is_kubectl_process_running(&self, config: &PortForwardConfig) -> Result<bool> {
    if config.forward_type != ForwardType::Kubectl {
      return Ok(false);
    }

    let process_lines = self.get_process_list()?;

    for line in process_lines.lines() {
      if self.matches_kubectl_command(line, config) {
        return Ok(true);
      }
    }

    Ok(false)
  }

  pub fn find_kubectl_process_pid(&self, config: &PortForwardConfig) -> Result<Option<u32>> {
    if config.forward_type != ForwardType::Kubectl {
      return Ok(None);
    }

    let process_lines = self.get_process_list()?;

    for line in process_lines.lines() {
      if self.matches_kubectl_command(line, config) {
        return Ok(self.extract_pid_from_ps_line(line));
      }
    }

    Ok(None)
  }

  pub fn is_process_actually_running(&self, pid: u32) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
      self.check_process_macos(pid)
    }

    #[cfg(target_os = "linux")]
    {
      self.check_process_linux(pid)
    }

    #[cfg(target_os = "windows")]
    {
      self.check_process_windows(pid)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
      Err(AppError::System(
        "Process verification not supported on this platform".to_string(),
      ))
    }
  }

  fn get_process_list(&self) -> Result<String> {
    #[cfg(unix)]
    {
      let output = Command::new("ps")
        .args(["aux"])
        .output()
        .map_err(|e| AppError::System(format!("Failed to list processes: {}", e)))?;

      Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[cfg(target_os = "windows")]
    {
      // Windows implementation would go here if needed
      Ok(String::new())
    }
  }

  fn matches_kubectl_command(&self, process_line: &str, config: &PortForwardConfig) -> bool {
    // Check if this is a kubectl port-forward command
    if !process_line.contains("kubectl") || !process_line.contains("port-forward") {
      return false;
    }

    // Check namespace
    if !self.matches_namespace(process_line, &config.namespace) {
      return false;
    }

    // Check service name
    if !process_line.contains(&config.service) {
      return false;
    }

    // Check if at least one port mapping matches
    self.matches_any_port(process_line, &config.ports)
  }

  fn matches_namespace(&self, process_line: &str, namespace: &str) -> bool {
    process_line.contains(&format!("-n {}", namespace))
      || process_line.contains(&format!("--namespace={}", namespace))
      || process_line.contains(&format!("--namespace {}", namespace))
  }

  fn matches_any_port(&self, process_line: &str, ports: &[String]) -> bool {
    ports.iter().any(|port| process_line.contains(port))
  }

  fn extract_pid_from_ps_line(&self, line: &str) -> Option<u32> {
    // Extract PID from ps output (second column)
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
      parts[1].parse::<u32>().ok()
    } else {
      None
    }
  }

  #[cfg(target_os = "macos")]
  fn check_process_macos(&self, pid: u32) -> Result<bool> {
    let output = Command::new("ps")
      .args(["-p", &pid.to_string()])
      .output()
      .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
    Ok(output.status.success())
  }

  #[cfg(target_os = "linux")]
  fn check_process_linux(&self, pid: u32) -> Result<bool> {
    let output = Command::new("ps")
      .args(["-p", &pid.to_string()])
      .output()
      .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
    Ok(output.status.success())
  }

  #[cfg(target_os = "windows")]
  fn check_process_windows(&self, pid: u32) -> Result<bool> {
    let output = Command::new("tasklist")
      .args(["/FI", &format!("PID eq {}", pid)])
      .output()
      .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains(&pid.to_string()))
  }
}
