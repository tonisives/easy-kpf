use crate::services::{KubectlService, PortForwardService};
use crate::types::PortForwardConfig;
use tauri::State;

#[tauri::command]
pub fn get_port_forward_configs(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<PortForwardConfig>, String> {
  port_forward_service
    .get_configs()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_port_forward_config(
  config: PortForwardConfig,
  port_forward_service: State<'_, PortForwardService>,
) -> Result<(), String> {
  port_forward_service
    .add_config(config)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_port_forward_config(
  service_key: String,
  port_forward_service: State<'_, PortForwardService>,
) -> Result<(), String> {
  port_forward_service
    .remove_config(&service_key)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_port_forward_config(
  old_service_key: String,
  new_config: PortForwardConfig,
  port_forward_service: State<'_, PortForwardService>,
) -> Result<(), String> {
  port_forward_service
    .update_config(&old_service_key, new_config)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_port_forward_config(
  service_key: String,
  new_index: usize,
  port_forward_service: State<'_, PortForwardService>,
) -> Result<(), String> {
  port_forward_service
    .reorder_config(&service_key, new_index)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_port_forward_by_key(
  service_key: String,
  port_forward_service: State<'_, PortForwardService>,
  kubectl_service: State<'_, KubectlService>,
) -> Result<String, String> {
  port_forward_service
    .start_port_forward_by_key(kubectl_service.inner(), &service_key)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_port_forward(
  service_name: String,
  port_forward_service: State<'_, PortForwardService>,
) -> Result<String, String> {
  port_forward_service
    .stop_port_forward(&service_name)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_running_services(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<String>, String> {
  port_forward_service
    .get_running_services()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_port_forwards(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<(String, bool)>, String> {
  port_forward_service
    .verify_port_forwards()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_and_update_port_forwards(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<String>, String> {
  port_forward_service
    .verify_and_update_port_forwards()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_existing_port_forwards(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<String>, String> {
  port_forward_service
    .detect_existing_port_forwards()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_with_existing_processes(
  port_forward_service: State<'_, PortForwardService>,
) -> Result<Vec<String>, String> {
  port_forward_service
    .sync_with_existing_processes()
    .map_err(|e| e.to_string())
}
