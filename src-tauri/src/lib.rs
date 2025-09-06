use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_shell::ShellExt;

mod config;
mod kubectl;
mod kubectl_context;
mod port_forwards;
use port_forwards::{load_configs, save_configs, PortForwardConfig};

type ProcessMap = Mutex<HashMap<String, u32>>;

#[tauri::command]
fn get_port_forward_configs() -> Result<Vec<PortForwardConfig>, String> {
    load_configs()
}

#[tauri::command]
fn add_port_forward_config(config: PortForwardConfig) -> Result<(), String> {
    let mut configs = load_configs()?;
    configs.push(config);
    save_configs(&configs)
}

#[tauri::command]
fn remove_port_forward_config(service_key: String) -> Result<(), String> {
    let mut configs = load_configs()?;
    configs.retain(|c| c.name != service_key);
    save_configs(&configs)
}

#[tauri::command]
fn update_port_forward_config(
    old_service_key: String,
    new_config: PortForwardConfig,
    process_map: State<'_, ProcessMap>,
) -> Result<(), String> {
    let mut configs = load_configs()?;

    let current_index = configs
        .iter()
        .position(|c| c.name == old_service_key)
        .ok_or_else(|| format!("Configuration not found for service: {}", old_service_key))?;

    // If the service name changed, we need to update the process map
    if old_service_key != new_config.name {
        let mut map = process_map.lock().unwrap();
        if let Some(pid) = map.remove(&old_service_key) {
            map.insert(new_config.name.clone(), pid);
        }
    }

    // Replace the config at the same position
    configs[current_index] = new_config;

    save_configs(&configs)
}

#[tauri::command]
fn reorder_port_forward_config(service_key: String, new_index: usize) -> Result<(), String> {
    let mut configs = load_configs()?;

    let current_index = configs
        .iter()
        .position(|c| c.name == service_key)
        .ok_or_else(|| format!("Configuration not found for service: {}", service_key))?;

    if new_index >= configs.len() {
        return Err("Invalid new index".to_string());
    }

    let config = configs.remove(current_index);
    configs.insert(new_index, config);

    save_configs(&configs)
}

#[tauri::command]
async fn get_namespaces(
    app_handle: tauri::AppHandle,
    context: String,
) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = kubectl::get_kubectl_command();

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = kubectl_context::get_kubeconfig_path() {
        command = command.env("KUBECONFIG", kubeconfig);
    }

    let output = command
        .args([
            "--context",
            &context,
            "get",
            "namespaces",
            "-o",
            "jsonpath={.items[*].metadata.name}",
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let namespaces = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        Ok(namespaces)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(kubectl_context::format_kubectl_error(&error))
    }
}

#[tauri::command]
async fn get_services(
    app_handle: tauri::AppHandle,
    context: String,
    namespace: String,
) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = kubectl::get_kubectl_command();

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = kubectl_context::get_kubeconfig_path() {
        command = command.env("KUBECONFIG", kubeconfig);
    }

    let output = command
        .args([
            "--context",
            &context,
            "-n",
            &namespace,
            "get",
            "services",
            "-o",
            "jsonpath={.items[*].metadata.name}",
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let services = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(|s| format!("svc/{}", s))
            .collect();
        Ok(services)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(kubectl_context::format_kubectl_error(&error))
    }
}

#[tauri::command]
async fn get_service_ports(
    app_handle: tauri::AppHandle,
    context: String,
    namespace: String,
    service: String,
) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = kubectl::get_kubectl_command();

    let service_name = service.strip_prefix("svc/").unwrap_or(&service);

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = kubectl_context::get_kubeconfig_path() {
        command = command.env("KUBECONFIG", kubeconfig);
    }

    let output = command
        .args([
            "--context",
            &context,
            "-n",
            &namespace,
            "get",
            "service",
            service_name,
            "-o",
            "jsonpath={.spec.ports[*].port}",
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let ports = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(|port| format!("{}:{}", port, port)) // Default to same port for local:remote
            .collect();
        Ok(ports)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(kubectl_context::format_kubectl_error(&error))
    }
}

#[tauri::command]
async fn start_port_forward_by_key(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
    service_key: String,
) -> Result<String, String> {
    let configs = load_configs()?;
    let config = configs
        .into_iter()
        .find(|c| c.name == service_key)
        .ok_or_else(|| format!("Configuration not found for service: {}", service_key))?;

    start_port_forward_generic(app_handle, process_map, config).await
}

async fn start_port_forward_generic(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
    config: PortForwardConfig,
) -> Result<String, String> {
    let shell = app_handle.shell();

    {
        let map = process_map.lock().unwrap();
        if map.contains_key(&config.name) {
            return Err(format!(
                "{} port forwarding is already running",
                config.name
            ));
        }
    }

    let current_context = kubectl_context::get_current_context(&app_handle).await?;
    kubectl_context::set_kubectl_context(app_handle.clone(), config.context.clone()).await?;

    let result = async {
        let mut args = vec!["-n", &config.namespace, "port-forward", &config.service];
        let port_refs: Vec<&str> = config.ports.iter().map(|s| s.as_str()).collect();
        args.extend(port_refs);

        let kubectl_cmd = kubectl::get_kubectl_command();
        let mut command = shell.command(&kubectl_cmd);
        if let Some(kubeconfig) = kubectl_context::get_kubeconfig_path() {
            command = command.env("KUBECONFIG", kubeconfig);
        }

        let (_rx, child) = command.args(args).spawn().map_err(|e| e.to_string())?;

        let pid = child.pid();

        {
            let mut map = process_map.lock().unwrap();
            map.insert(config.name.clone(), pid);
        }

        Ok::<String, String>(format!(
            "{} port forwarding started with PID: {}",
            config.name, pid
        ))
    }
    .await;

    kubectl_context::set_kubectl_context(app_handle, current_context)
        .await
        .ok();

    result
}

#[tauri::command]
async fn stop_port_forward(
    service_name: String,
    process_map: State<'_, ProcessMap>,
) -> Result<String, String> {
    let pid = {
        let mut map = process_map.lock().unwrap();
        map.remove(&service_name)
    };

    match pid {
        Some(pid) => {
            // Kill the process
            #[cfg(unix)]
            {
                use std::process::Command;
                let output = Command::new("kill")
                    .arg(pid.to_string())
                    .output()
                    .map_err(|e| e.to_string())?;

                if output.status.success() {
                    Ok(format!(
                        "Stopped {} port forwarding (PID: {})",
                        service_name, pid
                    ))
                } else {
                    Err(format!(
                        "Failed to stop process: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }
            #[cfg(not(unix))]
            {
                Err("Process termination not supported on this platform".to_string())
            }
        }
        None => Err(format!("{} port forwarding is not running", service_name)),
    }
}

#[tauri::command]
fn get_running_services(process_map: State<'_, ProcessMap>) -> Vec<String> {
    let map = process_map.lock().unwrap();
    map.keys().cloned().collect()
}

fn cleanup_all_port_forwards(process_map: &ProcessMap) -> Result<(), String> {
    let mut map = process_map.lock().unwrap();
    let pids: Vec<u32> = map.values().cloned().collect();
    map.clear();

    #[cfg(unix)]
    {
        use std::process::Command;
        for pid in pids {
            let _ = Command::new("kill").arg(pid.to_string()).output();
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let process_map: ProcessMap = Mutex::new(HashMap::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(process_map)
        .invoke_handler(tauri::generate_handler![
            kubectl_context::set_kubectl_context,
            start_port_forward_by_key,
            stop_port_forward,
            get_running_services,
            get_port_forward_configs,
            add_port_forward_config,
            update_port_forward_config,
            remove_port_forward_config,
            reorder_port_forward_config,
            kubectl_context::get_kubectl_contexts,
            get_namespaces,
            get_services,
            get_service_ports,
            kubectl_context::get_kubeconfig_env,
            kubectl_context::set_kubeconfig_env,
            kubectl::detect_kubectl_path,
            kubectl::validate_kubectl_path,
            kubectl::set_kubectl_path,
            kubectl::get_kubectl_path
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let process_map = app_handle.state::<ProcessMap>();
                let _ = cleanup_all_port_forwards(&process_map);
            }
        });
}
