use tauri::Manager;

mod handlers;
mod ipc_server;
mod reconnect;
mod services;
mod utils;
pub mod window;

pub use easy_kpf_core::error;
pub use easy_kpf_core::types;

use easy_kpf_core::ipc::socket_path::default_socket_path;
use easy_kpf_core::{ConfigService, ProcessManager};
use handlers::*;
use services::{KubectlService, PortForwardService};
use utils::init_logging;

fn cleanup_all_port_forwards(port_forward_service: &PortForwardService) -> Result<(), String> {
  port_forward_service
    .cleanup_all_port_forwards()
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn show_main_window(app_handle: tauri::AppHandle) {
  window::activate_and_show_window(&app_handle);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  init_logging();

  let config_service = ConfigService::new().expect("Failed to initialize config service");

  let process_manager = {
    let state_path = dirs::config_dir()
      .expect("Could not find config directory")
      .join("EasyKpf")
      .join("process-state.json");
    ProcessManager::with_state_file(state_path)
  };

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_shell::init())
    .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
      }
    })
    .setup(|app| {
      let app_handle = app.handle().clone();

      window::activate_and_show_window(&app_handle);

      let kubectl_service = KubectlService::new(app_handle.clone(), config_service.clone());
      let port_forward_service = PortForwardService::new(
        app_handle.clone(),
        config_service.clone(),
        process_manager.clone(),
      );

      app.manage(config_service);
      app.manage(kubectl_service);
      app.manage(port_forward_service);
      app.manage(process_manager.clone());

      tauri::async_runtime::spawn_blocking(move || {
        process_manager.restore_state();
      });

      let ipc_handle = app_handle.clone();
      tauri::async_runtime::spawn(async move {
        ipc_server::spawn(ipc_handle).await;
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
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
      test_ssh_connection,
      reconnect_all_services,
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
      get_kubectl_path,
      show_main_window
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|app_handle, event| match event {
      tauri::RunEvent::ExitRequested { .. } => {
        let port_forward_service = app_handle.state::<PortForwardService>();
        let _ = cleanup_all_port_forwards(&port_forward_service);
        let _ = std::fs::remove_file(default_socket_path());
      }
      #[cfg(target_os = "macos")]
      tauri::RunEvent::Reopen { .. } => {
        if let Some(window) = app_handle.get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
      _ => {}
    });
}
