use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_shell::ShellExt;

type ProcessMap = Mutex<HashMap<String, u32>>;

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
async fn start_db_port_forward(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
) -> Result<String, String> {
    let shell = app_handle.shell();

    // Check if already running
    {
        let map = process_map.lock().unwrap();
        if map.contains_key("db-s") {
            return Err("DB port forwarding is already running".to_string());
        }
    }

    let (_rx, child) = shell
        .command("kubectl")
        .args([
            "-n",
            "monitoring",
            "port-forward",
            "svc/postgres-eth-job-proxy",
            "5332:25060",
            "5333:25061",
            "5334:25062",
            "5335:25063",
            "5336:25064",
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let pid = child.pid();

    {
        let mut map = process_map.lock().unwrap();
        map.insert("db-s".to_string(), pid);
    }

    Ok(format!("DB port forwarding started with PID: {}", pid))
}

#[tauri::command]
async fn start_grafana_port_forward(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
) -> Result<String, String> {
    let shell = app_handle.shell();

    // Check if already running
    {
        let map = process_map.lock().unwrap();
        if map.contains_key("grafana-docn") {
            return Err("Grafana port forwarding is already running".to_string());
        }
    }

    let (_rx, child) = shell
        .command("kubectl")
        .args([
            "-n",
            "monitoring",
            "port-forward",
            "svc/prometheus-grafana",
            "2999:80",
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let pid = child.pid();

    {
        let mut map = process_map.lock().unwrap();
        map.insert("grafana-docn".to_string(), pid);
    }

    Ok(format!("Grafana port forwarding started with PID: {}", pid))
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
async fn start_postgres_cluster_port_forward(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
) -> Result<String, String> {
    let shell = app_handle.shell();

    // Check if already running
    {
        let map = process_map.lock().unwrap();
        if map.contains_key("postgres-cluster-rw") {
            return Err("Postgres cluster port forwarding is already running".to_string());
        }
    }

    let (_rx, child) = shell
        .command("kubectl")
        .args([
            "-n",
            "infra",
            "port-forward",
            "svc/postgres-cluster-rw",
            "8100:5432",
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let pid = child.pid();

    {
        let mut map = process_map.lock().unwrap();
        map.insert("postgres-cluster-rw".to_string(), pid);
    }

    Ok(format!(
        "Postgres cluster port forwarding started with PID: {}",
        pid
    ))
}

#[tauri::command]
async fn start_vmks_grafana_port_forward(
    app_handle: tauri::AppHandle,
    process_map: State<'_, ProcessMap>,
) -> Result<String, String> {
    let shell = app_handle.shell();

    // Check if already running
    {
        let map = process_map.lock().unwrap();
        if map.contains_key("vmks-grafana") {
            return Err("VMKS Grafana port forwarding is already running".to_string());
        }
    }

    let (_rx, child) = shell
        .command("kubectl")
        .args(["-n", "infra", "port-forward", "svc/vmks-grafana", "2998:80"])
        .spawn()
        .map_err(|e| e.to_string())?;

    let pid = child.pid();

    {
        let mut map = process_map.lock().unwrap();
        map.insert("vmks-grafana".to_string(), pid);
    }

    Ok(format!(
        "VMKS Grafana port forwarding started with PID: {}",
        pid
    ))
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
            start_db_port_forward,
            start_grafana_port_forward,
            start_postgres_cluster_port_forward,
            start_vmks_grafana_port_forward,
            stop_port_forward,
            get_running_services
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
