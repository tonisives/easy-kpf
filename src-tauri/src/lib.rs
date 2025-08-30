use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_shell::ShellExt;

type ProcessMap = Mutex<HashMap<String, u32>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortForwardConfig {
    service_key: String,
    context: String,
    namespace: String,
    service: String,
    ports: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PortForwardConfigs {
    configs: Vec<PortForwardConfig>,
}

fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("easy-kpf");

    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir)
}

fn get_config_file_path() -> Result<PathBuf, String> {
    Ok(get_config_dir()?.join("port-forwards.yaml"))
}

fn load_configs() -> Result<Vec<PortForwardConfig>, String> {
    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        let default_configs = PortForwardConfigs { configs: vec![] };

        save_configs(&default_configs.configs)?;
        return Ok(default_configs.configs);
    }

    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let configs: PortForwardConfigs =
        serde_yaml::from_str(&config_content).map_err(|e| e.to_string())?;
    Ok(configs.configs)
}

fn save_configs(configs: &[PortForwardConfig]) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let configs_wrapper = PortForwardConfigs {
        configs: configs.to_vec(),
    };
    let yaml_content = serde_yaml::to_string(&configs_wrapper).map_err(|e| e.to_string())?;
    fs::write(&config_path, yaml_content).map_err(|e| e.to_string())?;
    Ok(())
}

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
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
    configs.retain(|c| c.service_key != service_key);
    save_configs(&configs)
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
        .find(|c| c.service_key == service_key)
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
        if map.contains_key(&config.service_key) {
            return Err(format!(
                "{} port forwarding is already running",
                config.service_key
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
            map.insert(config.service_key.clone(), pid);
        }

        Ok::<String, String>(format!(
            "{} port forwarding started with PID: {}",
            config.service_key, pid
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
            greet,
            set_kubectl_context,
            start_port_forward_by_key,
            stop_port_forward,
            get_running_services,
            get_port_forward_configs,
            add_port_forward_config,
            remove_port_forward_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
