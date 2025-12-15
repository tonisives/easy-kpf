use easy_kpf_core::error::{AppError, Result};
use easy_kpf_core::services::ConfigService;
use async_trait::async_trait;
use std::path::Path;
use tauri_plugin_shell::ShellExt;

const KUBECTL_DETECTION_PATHS: &[&str] = &[
  "/opt/homebrew/bin/kubectl",
  "/usr/local/bin/kubectl",
  "/usr/bin/kubectl",
  "/snap/bin/kubectl",
  "/usr/local/google-cloud-sdk/bin/kubectl",
];

#[async_trait]
pub trait KubectlOperations {
  async fn get_namespaces(&self, context: &str) -> Result<Vec<String>>;
  async fn get_services(&self, context: &str, namespace: &str) -> Result<Vec<String>>;
  async fn get_service_ports(
    &self,
    context: &str,
    namespace: &str,
    service: &str,
  ) -> Result<Vec<String>>;
  async fn get_contexts(&self) -> Result<Vec<String>>;
  async fn get_current_context(&self) -> Result<String>;
  async fn set_context(&self, context: &str) -> Result<String>;
}

pub struct KubectlService {
  app_handle: tauri::AppHandle,
  config_service: ConfigService,
}

impl KubectlService {
  pub fn new(app_handle: tauri::AppHandle, config_service: ConfigService) -> Self {
    Self {
      app_handle,
      config_service,
    }
  }

  pub fn get_kubectl_command(&self) -> String {
    self
      .config_service
      .load_kubectl_path()
      .unwrap_or_else(|_| "kubectl".to_string())
  }

  pub fn get_kubeconfig_path(&self) -> Option<String> {
    if let Ok(Some(stored_path)) = self.config_service.load_kubeconfig_path() {
      log::debug!("Using stored kubeconfig path: {}", stored_path);
      return Some(stored_path);
    }

    let env_result = std::env::var("KUBECONFIG").or_else(|_| {
      let default_config = format!("{}/.kube/config", std::env::var("HOME").unwrap_or_default());
      log::debug!("Checking default config path: {}", default_config);
      if std::path::Path::new(&default_config).exists() {
        Ok(default_config)
      } else {
        Err(std::env::VarError::NotPresent)
      }
    });

    match &env_result {
      Ok(path) => log::debug!("Using environment/default kubeconfig path: {}", path),
      Err(_) => log::debug!("No kubeconfig path found"),
    }

    env_result.ok()
  }

  pub async fn detect_kubectl_path(&self) -> Result<String> {
    let shell = self.app_handle.shell();

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

    Err(AppError::Kubectl(
      "kubectl not found in common locations".to_string(),
    ))
  }

  pub async fn validate_kubectl_path(&self, path: &str) -> Result<bool> {
    if !Path::new(path).exists() {
      return Ok(false);
    }

    let shell = self.app_handle.shell();
    let output = shell
      .command(path)
      .args(["version", "--client"])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    Ok(output.status.success())
  }

  pub fn format_kubectl_error(error: &str) -> String {
    let error_lower = error.to_lowercase();

    if error_lower.contains("unable to connect") || error_lower.contains("connection refused") {
      "Unable to connect to cluster. Check your internet connection and cluster status."
        .to_string()
    } else if error_lower.contains("unauthorized") || error_lower.contains("forbidden") {
      "Authentication failed. For GKE clusters, run: gcloud auth application-default login"
        .to_string()
    } else if error_lower.contains("token") && error_lower.contains("expired") {
      "Authentication token expired. For GKE clusters, run: gcloud auth application-default login".to_string()
    } else if error_lower.contains("no cluster") || error_lower.contains("context") {
      "No active kubectl context found. Configure kubectl with: kubectl config use-context <context-name>".to_string()
    } else if error_lower.contains("gke_gcloud_auth_plugin") {
      "GKE auth plugin required. Run: gcloud components install gke-gcloud-auth-plugin"
        .to_string()
    } else {
      format!("kubectl error: {}", error)
    }
  }

  fn create_command(&self) -> tauri_plugin_shell::process::Command {
    let shell = self.app_handle.shell();
    let kubectl_cmd = self.get_kubectl_command();
    let mut command = shell.command(&kubectl_cmd);

    if let Some(kubeconfig) = self.get_kubeconfig_path() {
      log::debug!("Setting KUBECONFIG env var: {}", kubeconfig);
      command = command.env("KUBECONFIG", kubeconfig);
    }

    command
  }
}

#[async_trait]
impl KubectlOperations for KubectlService {
  async fn get_namespaces(&self, context: &str) -> Result<Vec<String>> {
    log::debug!("Getting namespaces for context: {}", context);

    let output = self
      .create_command()
      .args([
        "--context",
        context,
        "get",
        "namespaces",
        "-o",
        "jsonpath={.items[*].metadata.name}",
      ])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let namespaces = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
      Ok(namespaces)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }

  async fn get_services(&self, context: &str, namespace: &str) -> Result<Vec<String>> {
    log::debug!(
      "Getting services for context: {}, namespace: {}",
      context,
      namespace
    );

    let output = self
      .create_command()
      .args([
        "--context",
        context,
        "-n",
        namespace,
        "get",
        "services",
        "-o",
        "jsonpath={.items[*].metadata.name}",
      ])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let services = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|s| format!("svc/{}", s))
        .collect();
      Ok(services)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }

  async fn get_service_ports(
    &self,
    context: &str,
    namespace: &str,
    service: &str,
  ) -> Result<Vec<String>> {
    log::debug!(
      "Getting service ports for context: {}, namespace: {}, service: {}",
      context,
      namespace,
      service
    );

    let service_name = service.strip_prefix("svc/").unwrap_or(service);

    let output = self
      .create_command()
      .args([
        "--context",
        context,
        "-n",
        namespace,
        "get",
        "service",
        service_name,
        "-o",
        "jsonpath={.spec.ports[*].port}",
      ])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let ports = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|port| format!("{}:{}", port, port)) // Default to same port for local:remote
        .collect();
      Ok(ports)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }

  async fn get_contexts(&self) -> Result<Vec<String>> {
    log::debug!("Getting kubectl contexts");

    let output = self
      .create_command()
      .args(["config", "get-contexts", "-o", "name"])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      log::debug!("kubectl get-contexts stdout: '{}'", stdout);

      let contexts = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

      log::debug!("Parsed contexts: {:?}", contexts);
      Ok(contexts)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      log::debug!("kubectl get-contexts stderr: {}", error);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }

  async fn get_current_context(&self) -> Result<String> {
    log::debug!("Getting current kubectl context");

    let output = self
      .create_command()
      .args(["config", "current-context"])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let context = String::from_utf8_lossy(&output.stdout).trim().to_string();
      log::debug!("Current context: {}", context);
      Ok(context)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      log::debug!("kubectl current-context stderr: {}", error);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }

  async fn set_context(&self, context: &str) -> Result<String> {
    log::debug!("Setting kubectl context to: {}", context);

    let output = self
      .create_command()
      .args(["config", "use-context", context])
      .output()
      .await
      .map_err(|e| AppError::Kubectl(e.to_string()))?;

    if output.status.success() {
      let result = String::from_utf8_lossy(&output.stdout).to_string();
      log::debug!("Context switched successfully: {}", result);
      Ok(result)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      log::debug!("kubectl use-context stderr: {}", error);
      Err(AppError::Kubectl(Self::format_kubectl_error(&error)))
    }
  }
}
