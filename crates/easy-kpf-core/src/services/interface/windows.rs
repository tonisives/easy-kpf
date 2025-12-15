use super::InterfaceManager;
use crate::error::{AppError, Result};
use std::net::IpAddr;

pub struct WindowsInterfaceManager;

impl InterfaceManager for WindowsInterfaceManager {
  fn ensure_interface_exists(&self, interface: &str) -> Result<()> {
    // Windows doesn't support creating virtual interfaces easily
    // Just validate it's a valid IP format
    interface
      .parse::<IpAddr>()
      .map_err(|_| AppError::InvalidInput(format!("Invalid IP address: {}", interface)))?;

    Ok(())
  }
}
