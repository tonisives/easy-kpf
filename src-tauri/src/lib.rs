use tauri::Manager;

mod error;
mod handlers;
mod services;
mod types;
mod utils;

use handlers::*;
use services::{ConfigService, KubectlService, PortForwardService, ProcessManager};
use utils::init_logging;

fn cleanup_all_port_forwards(port_forward_service: &PortForwardService) -> Result<(), String> {
  port_forward_service
    .cleanup_all_port_forwards()
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  init_logging();

  let config_service = ConfigService::new().expect("Failed to initialize config service");
  let process_manager = ProcessManager::new();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      let app_handle = app.handle().clone();
      let kubectl_service = KubectlService::new(app_handle.clone(), config_service.clone());
      let port_forward_service =
        PortForwardService::new(app_handle, config_service.clone(), process_manager.clone());

      app.manage(config_service);
      app.manage(kubectl_service);
      app.manage(port_forward_service);
      app.manage(process_manager);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Port forward handlers
      get_port_forward_configs,
      add_port_forward_config,
      update_port_forward_config,
      remove_port_forward_config,
      reorder_port_forward_config,
      start_port_forward_by_key,
      stop_port_forward,
      get_running_services,
      verify_port_forwards,
      verify_and_update_port_forwards,
      detect_existing_port_forwards,
      sync_with_existing_processes,
      // Kubectl handlers
      get_kubectl_contexts,
      set_kubectl_context,
      get_namespaces,
      get_services,
      get_service_ports,
      get_kubeconfig_env,
      set_kubeconfig_env,
      detect_kubectl_path,
      validate_kubectl_path,
      set_kubectl_path,
      get_kubectl_path
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|app_handle, event| {
      if let tauri::RunEvent::ExitRequested { .. } = event {
        let port_forward_service = app_handle.state::<PortForwardService>();
        let _ = cleanup_all_port_forwards(&port_forward_service);
      }
    });
}
