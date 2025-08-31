use crate::config;
use std::path::Path;
use tauri_plugin_shell::ShellExt;

const KUBECTL_DETECTION_PATHS: &[&str] = &[
    "/opt/homebrew/bin/kubectl",
    "/usr/local/bin/kubectl",
    "/usr/bin/kubectl",
    "/snap/bin/kubectl",
    "/usr/local/google-cloud-sdk/bin/kubectl",
];

#[tauri::command]
pub async fn detect_kubectl_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let shell = app_handle.shell();

    // First try to use `which` command to find kubectl in PATH
    let output = shell.command("which").args(["kubectl"]).output().await;

    if let Ok(output) = output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && Path::new(&path).exists() {
                return Ok(path);
            }
        }
    }

    // If `which` fails, try common installation paths
    for path in KUBECTL_DETECTION_PATHS {
        if Path::new(*path).exists() {
            return Ok(path.to_string());
        }
    }

    Err("kubectl not found in common locations".to_string())
}

#[tauri::command]
pub async fn validate_kubectl_path(
    app_handle: tauri::AppHandle,
    path: String,
) -> Result<bool, String> {
    if !Path::new(&path).exists() {
        return Ok(false);
    }

    let shell = app_handle.shell();
    let output = shell
        .command(&path)
        .args(["version", "--client"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    Ok(output.status.success())
}

#[tauri::command]
pub fn set_kubectl_path(path: String) -> Result<(), String> {
    config::save_kubectl_path(path)
}

#[tauri::command]
pub fn get_kubectl_path() -> Option<String> {
    config::load_kubectl_path().ok()
}

pub fn get_kubectl_command() -> String {
    config::load_kubectl_path().unwrap_or_else(|_| "kubectl".to_string())
}
