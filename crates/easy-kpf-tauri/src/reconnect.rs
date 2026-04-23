use crate::services::{KubectlService, PortForwardService};
use easy_kpf_core::error::Result;

/// Reconnect services the user had previously enabled (the last-active set).
/// Skips services that are currently running. Never starts services the user
/// hasn't explicitly enabled.
pub async fn reconnect_all(
  port_forward_service: &PortForwardService,
  kubectl_service: &KubectlService,
) -> Result<Vec<String>> {
  let last_active = port_forward_service.last_active().names()?;
  let running = port_forward_service.get_running_services()?;

  let mut reconnected = Vec::new();

  for name in last_active {
    if running.contains(&name) {
      continue;
    }
    match port_forward_service
      .start_port_forward_by_key(kubectl_service, &name)
      .await
    {
      Ok(_) => reconnected.push(name),
      Err(e) => log::warn!("Failed to reconnect {}: {}", name, e),
    }
  }

  Ok(reconnected)
}
