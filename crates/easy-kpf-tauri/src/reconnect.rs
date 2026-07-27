use crate::services::{KubectlService, PortForwardService};
use easy_kpf_core::error::{AppError, Result};

/// Restart services the user had previously enabled (the last-active set).
/// A live PID does not guarantee a usable tunnel, so running services are
/// replaced as well. Never starts services the user hasn't explicitly enabled.
pub async fn reconnect_all(
  port_forward_service: &PortForwardService,
  kubectl_service: &KubectlService,
) -> Result<Vec<String>> {
  let last_active = port_forward_service.last_active().names()?;
  let mut reconnected = Vec::new();
  let mut failures = Vec::new();

  for name in last_active {
    match port_forward_service
      .restart_port_forward_by_key(kubectl_service, &name)
      .await
    {
      Ok(_) => reconnected.push(name),
      Err(error) => {
        log::warn!("Failed to reconnect {}: {}", name, error);
        failures.push(format!("{}: {}", name, error));
      }
    }
  }

  if failures.is_empty() {
    Ok(reconnected)
  } else {
    Err(AppError::PortForward(format!(
      "failed to reconnect {}",
      failures.join("; ")
    )))
  }
}
