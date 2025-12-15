use super::InterfaceManager;
use crate::error::{AppError, Result};
use std::process::Command;

pub struct LinuxInterfaceManager;

impl InterfaceManager for LinuxInterfaceManager {
  fn ensure_interface_exists(&self, interface: &str) -> Result<()> {
    if self.interface_exists(interface)? {
      return Ok(());
    }

    self.create_interface(interface)
  }
}

impl LinuxInterfaceManager {
  fn interface_exists(&self, interface: &str) -> Result<bool> {
    let output = Command::new("ip")
      .args(["addr", "show", interface])
      .output()
      .map_err(|e| AppError::System(format!("Failed to check interface: {}", e)))?;

    Ok(output.status.success())
  }

  fn create_interface(&self, interface: &str) -> Result<()> {
    if self.try_create_without_sudo(interface)? {
      return Ok(());
    }

    self.try_create_with_sudo(interface)
  }

  fn try_create_without_sudo(&self, interface: &str) -> Result<bool> {
    let output = Command::new("ip")
      .args(["addr", "add", &format!("{}/32", interface), "dev", "lo"])
      .output();

    match output {
      Ok(result) => Ok(result.status.success()),
      Err(_) => Ok(false),
    }
  }

  fn try_create_with_sudo(&self, interface: &str) -> Result<()> {
    let output = Command::new("sudo")
      .args([
        "-n",
        "ip",
        "addr",
        "add",
        &format!("{}/32", interface),
        "dev",
        "lo",
      ])
      .output()
      .map_err(|e| AppError::System(format!("Failed to create interface: {}", e)))?;

    if !output.status.success() {
      let stderr_msg = String::from_utf8_lossy(&output.stderr);
      if stderr_msg.contains("password") || stderr_msg.contains("sudo:") {
        return Err(AppError::System(format!(
          "Interface {} requires admin privileges to create. Please run: 'sudo ip addr add {}/32 dev lo'",
          interface, interface
        )));
      } else {
        return Err(AppError::System(format!(
          "Failed to create interface {}: {}",
          interface, stderr_msg
        )));
      }
    }

    Ok(())
  }
}
