use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_shell::ShellExt;

mod config;
use config::{load_configs, save_configs, PortForwardConfig};

type ProcessMap = Mutex<HashMap<String, u32>>;

async fn get_current_context(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let shell = app_handle.shell();

    let output = shell
        .command("kubectl")
        .args(["config", "current-context"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn set_kubectl_context(
    app_handle: tauri::AppHandle,
    context: String,
) -> Result<String, String> {
    let shell = app_handle.shell();

    let output = shell
        .command("kubectl")
        .args(["config", "use-context", &context])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

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
async fn get_kubectl_contexts(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();
    let output = shell
        .command("kubectl")
        .args(["config", "get-contexts", "-o", "name"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let contexts = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(contexts)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn get_namespaces(
    app_handle: tauri::AppHandle,
    context: String,
) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();

    let output = shell
        .command("kubectl")
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
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn get_services(
    app_handle: tauri::AppHandle,
    context: String,
    namespace: String,
) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();

    let output = shell
        .command("kubectl")
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
        Err(String::from_utf8_lossy(&output.stderr).to_string())
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

    let service_name = service.strip_prefix("svc/").unwrap_or(&service);

    let output = shell
        .command("kubectl")
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
        Err(String::from_utf8_lossy(&output.stderr).to_string())
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

    let current_context = get_current_context(&app_handle).await?;
    set_kubectl_context(app_handle.clone(), config.context.clone()).await?;

    let result = async {
        let mut args = vec!["-n", &config.namespace, "port-forward", &config.service];
        let port_refs: Vec<&str> = config.ports.iter().map(|s| s.as_str()).collect();
        args.extend(port_refs);

        let (_rx, child) = shell
            .command("kubectl")
            .args(args)
            .spawn()
            .map_err(|e| e.to_string())?;

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

    set_kubectl_context(app_handle, current_context).await.ok();

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let process_map: ProcessMap = Mutex::new(HashMap::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(process_map)
        .invoke_handler(tauri::generate_handler![
            set_kubectl_context,
            start_port_forward_by_key,
            stop_port_forward,
            get_running_services,
            get_port_forward_configs,
            add_port_forward_config,
            remove_port_forward_config,
            get_kubectl_contexts,
            get_namespaces,
            get_services,
            get_service_ports
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
