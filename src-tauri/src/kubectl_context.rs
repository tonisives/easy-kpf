use tauri_plugin_shell::ShellExt;

use crate::config;

pub fn get_kubeconfig_path() -> Option<String> {
    if let Ok(Some(stored_path)) = config::load_kubeconfig_path() {
        eprintln!("DEBUG: Using stored kubeconfig path: {}", stored_path);
        return Some(stored_path);
    }

    let env_result = std::env::var("KUBECONFIG").or_else(|_| {
        let default_config = format!("{}/.kube/config", std::env::var("HOME").unwrap_or_default());
        eprintln!("DEBUG: Checking default config path: {}", default_config);
        if std::path::Path::new(&default_config).exists() {
            Ok(default_config)
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });

    match &env_result {
        Ok(path) => eprintln!("DEBUG: Using environment/default kubeconfig path: {}", path),
        Err(_) => eprintln!("DEBUG: No kubeconfig path found"),
    }

    env_result.ok()
}

pub async fn get_current_context(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = crate::kubectl::get_kubectl_command();

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = get_kubeconfig_path() {
        eprintln!("DEBUG: Setting KUBECONFIG env var: {}", kubeconfig);
        command = command.env("KUBECONFIG", kubeconfig);
    }

    eprintln!("DEBUG: Running kubectl config current-context");
    let output = command
        .args(["config", "current-context"])
        .output()
        .await
        .map_err(|e| {
            eprintln!("DEBUG: kubectl current-context command failed: {}", e);
            e.to_string()
        })?;

    if output.status.success() {
        let context = String::from_utf8_lossy(&output.stdout).trim().to_string();
        eprintln!("DEBUG: Current context: {}", context);
        Ok(context)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("DEBUG: kubectl current-context stderr: {}", error);
        Err(format_kubectl_error(&error))
    }
}

#[tauri::command]
pub async fn get_kubectl_contexts(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = crate::kubectl::get_kubectl_command();

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = get_kubeconfig_path() {
        eprintln!(
            "DEBUG: Setting KUBECONFIG env var for contexts: {}",
            kubeconfig
        );
        command = command.env("KUBECONFIG", kubeconfig);
    } else {
        eprintln!("DEBUG: No KUBECONFIG path available for get_contexts");
    }

    eprintln!("DEBUG: Running kubectl config get-contexts -o name");
    let output = command
        .args(["config", "get-contexts", "-o", "name"])
        .output()
        .await
        .map_err(|e| {
            eprintln!("DEBUG: kubectl get-contexts command failed: {}", e);
            e.to_string()
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("DEBUG: kubectl get-contexts stdout: '{}'", stdout);

        let contexts = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        eprintln!("DEBUG: Parsed contexts: {:?}", contexts);
        Ok(contexts)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("DEBUG: kubectl get-contexts stderr: {}", error);
        Err(format_kubectl_error(&error))
    }
}

#[tauri::command]
pub async fn set_kubectl_context(
    app_handle: tauri::AppHandle,
    context: String,
) -> Result<String, String> {
    let shell = app_handle.shell();
    let kubectl_cmd = crate::kubectl::get_kubectl_command();

    let mut command = shell.command(&kubectl_cmd);
    if let Some(kubeconfig) = get_kubeconfig_path() {
        eprintln!(
            "DEBUG: Setting KUBECONFIG env var for context switch: {}",
            kubeconfig
        );
        command = command.env("KUBECONFIG", kubeconfig);
    }

    eprintln!("DEBUG: Setting kubectl context to: {}", context);
    let output = command
        .args(["config", "use-context", &context])
        .output()
        .await
        .map_err(|e| {
            eprintln!("DEBUG: kubectl use-context command failed: {}", e);
            e.to_string()
        })?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        eprintln!("DEBUG: Context switched successfully: {}", result);
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("DEBUG: kubectl use-context stderr: {}", error);
        Err(format_kubectl_error(&error))
    }
}

pub fn format_kubectl_error(error: &str) -> String {
    let error_lower = error.to_lowercase();

    if error_lower.contains("unable to connect") || error_lower.contains("connection refused") {
        "⚠️  Unable to connect to cluster. Check your internet connection and cluster status."
            .to_string()
    } else if error_lower.contains("unauthorized") || error_lower.contains("forbidden") {
        "🔐 Authentication failed. For GKE clusters, run: gcloud auth application-default login"
            .to_string()
    } else if error_lower.contains("token") && error_lower.contains("expired") {
        "⏰ Authentication token expired. For GKE clusters, run: gcloud auth application-default login".to_string()
    } else if error_lower.contains("no cluster") || error_lower.contains("context") {
        "🚫 No active kubectl context found. Configure kubectl with: kubectl config use-context <context-name>".to_string()
    } else if error_lower.contains("gke_gcloud_auth_plugin") {
        "🔧 GKE auth plugin required. Run: gcloud components install gke-gcloud-auth-plugin"
            .to_string()
    } else {
        format!("❌ kubectl error: {}", error)
    }
}

#[tauri::command]
pub fn get_kubeconfig_env() -> Option<String> {
    get_kubeconfig_path()
}

#[tauri::command]
pub fn set_kubeconfig_env(path: String) -> Result<(), String> {
    eprintln!("DEBUG: Setting kubeconfig path: {}", path);

    if !std::path::Path::new(&path).exists() {
        eprintln!("DEBUG: KUBECONFIG file does not exist: {}", path);
        return Err("KUBECONFIG file does not exist".to_string());
    }

    // Save to config file for persistence
    config::save_kubeconfig_path(path.clone())?;
    eprintln!("DEBUG: Saved kubeconfig path to config file");

    // Also set environment variable for current process
    std::env::set_var("KUBECONFIG", &path);
    eprintln!("DEBUG: Set KUBECONFIG environment variable");
    Ok(())
}
