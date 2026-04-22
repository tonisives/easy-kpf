use crate::services::{KubectlService, PortForwardService};
use easy_kpf_core::error::Result;

pub async fn reconnect_all(
  port_forward_service: &PortForwardService,
  kubectl_service: &KubectlService,
) -> Result<Vec<String>> {
  let configs = port_forward_service.get_configs()?;
  let running = port_forward_service.get_running_services()?;

  let mut reconnected = Vec::new();

  for config in configs {
    if !running.contains(&config.name) {
      match port_forward_service
        .start_port_forward_by_key(kubectl_service, &config.name)
        .await
      {
        Ok(_) => reconnected.push(config.name),
        Err(e) => log::warn!("Failed to reconnect {}: {}", config.name, e),
      }
    }
  }

  Ok(reconnected)
}
