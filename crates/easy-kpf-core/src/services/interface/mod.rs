use crate::error::Result;

pub trait InterfaceManager {
  fn ensure_interface_exists(&self, interface: &str) -> Result<()>;
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacosInterfaceManager;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxInterfaceManager;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsInterfaceManager;

pub struct SystemInterfaceManager;

impl InterfaceManager for SystemInterfaceManager {
  fn ensure_interface_exists(&self, interface: &str) -> Result<()> {
    // Strip port suffix if present (e.g., "127.0.0.2:5335" -> "127.0.0.2")
    let ip = match interface.rsplit_once(':') {
      Some((ip, port)) if port.parse::<u16>().is_ok() => ip,
      _ => interface,
    };

    // Skip for standard interfaces
    if ip == "127.0.0.1" || ip == "0.0.0.0" || ip == "localhost" {
      return Ok(());
    }

    let interface = ip;

    #[cfg(target_os = "macos")]
    {
      let manager = MacosInterfaceManager;
      manager.ensure_interface_exists(interface)
    }

    #[cfg(target_os = "linux")]
    {
      let manager = LinuxInterfaceManager;
      manager.ensure_interface_exists(interface)
    }

    #[cfg(target_os = "windows")]
    {
      let manager = WindowsInterfaceManager;
      manager.ensure_interface_exists(interface)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
      use crate::error::AppError;
      Err(AppError::System(
        "Interface management not supported on this platform".to_string(),
      ))
    }
  }
}
