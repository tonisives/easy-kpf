use crate::services::{ConfigService, KubectlOperations, KubectlService};
use tauri::State;

#[tauri::command]
pub async fn get_namespaces(
    context: String,
    kubectl_service: State<'_, KubectlService>,
) -> Result<Vec<String>, String> {
    kubectl_service
        .get_namespaces(&context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_services(
    context: String,
    namespace: String,
    kubectl_service: State<'_, KubectlService>,
) -> Result<Vec<String>, String> {
    kubectl_service
        .get_services(&context, &namespace)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_service_ports(
    context: String,
    namespace: String,
    service: String,
    kubectl_service: State<'_, KubectlService>,
) -> Result<Vec<String>, String> {
    kubectl_service
        .get_service_ports(&context, &namespace, &service)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_kubectl_contexts(
    kubectl_service: State<'_, KubectlService>,
) -> Result<Vec<String>, String> {
    kubectl_service
        .get_contexts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_kubectl_context(
    context: String,
    kubectl_service: State<'_, KubectlService>,
) -> Result<String, String> {
    kubectl_service
        .set_context(&context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_kubeconfig_env(kubectl_service: State<'_, KubectlService>) -> Option<String> {
    kubectl_service.get_kubeconfig_path()
}

#[tauri::command]
pub fn set_kubeconfig_env(
    path: String,
    config_service: State<'_, ConfigService>,
) -> Result<(), String> {
    log::debug!("Setting kubeconfig path: {}", path);

    if !std::path::Path::new(&path).exists() {
        log::debug!("KUBECONFIG file does not exist: {}", path);
        return Err("KUBECONFIG file does not exist".to_string());
    }

    // Save to config file for persistence
    config_service
        .save_kubeconfig_path(path.clone())
        .map_err(|e| e.to_string())?;
    log::debug!("Saved kubeconfig path to config file");

    // Also set environment variable for current process
    std::env::set_var("KUBECONFIG", &path);
    log::debug!("Set KUBECONFIG environment variable");
    Ok(())
}

#[tauri::command]
pub async fn detect_kubectl_path(
    kubectl_service: State<'_, KubectlService>,
) -> Result<String, String> {
    kubectl_service
        .detect_kubectl_path()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_kubectl_path(
    path: String,
    kubectl_service: State<'_, KubectlService>,
) -> Result<bool, String> {
    kubectl_service
        .validate_kubectl_path(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_kubectl_path(
    path: String,
    config_service: State<'_, ConfigService>,
) -> Result<(), String> {
    config_service
        .save_kubectl_path(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_kubectl_path(
    config_service: State<'_, ConfigService>,
) -> Result<Option<String>, String> {
    config_service.load_kubectl_path().map(Some).or(Ok(None))
}
