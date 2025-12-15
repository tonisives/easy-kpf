use easy_kpf_core::services::ConfigService;
use std::process::Command;

#[derive(Clone)]
pub struct KubectlService {
  config_service: ConfigService,
}

impl KubectlService {
  pub fn new(config_service: ConfigService) -> Self {
    Self { config_service }
  }

  fn get_kubectl_command(&self) -> String {
    self
      .config_service
      .load_kubectl_path()
      .unwrap_or_else(|_| "kubectl".to_string())
  }

  fn get_kubeconfig_path(&self) -> Option<String> {
    if let Ok(Some(stored_path)) = self.config_service.load_kubeconfig_path() {
      return Some(stored_path);
    }

    std::env::var("KUBECONFIG")
      .or_else(|_| {
        let default_config = format!("{}/.kube/config", std::env::var("HOME").unwrap_or_default());
        if std::path::Path::new(&default_config).exists() {
          Ok(default_config)
        } else {
          Err(std::env::VarError::NotPresent)
        }
      })
      .ok()
  }

  fn create_command(&self) -> Command {
    let kubectl_cmd = self.get_kubectl_command();
    let mut command = Command::new(&kubectl_cmd);

    if let Some(kubeconfig) = self.get_kubeconfig_path() {
      command.env("KUBECONFIG", kubeconfig);
    }

    command
  }

  pub fn get_contexts(&self) -> Vec<String> {
    let output = self
      .create_command()
      .args(["config", "get-contexts", "-o", "name"])
      .output();

    match output {
      Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect(),
      _ => vec![],
    }
  }

  pub fn get_namespaces(&self, context: &str) -> Vec<String> {
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
      .output();

    match output {
      Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect(),
      _ => vec![],
    }
  }

  pub fn get_services(&self, context: &str, namespace: &str) -> Vec<String> {
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
      .output();

    match output {
      Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|s| format!("svc/{}", s))
        .collect(),
      _ => vec![],
    }
  }

  pub fn get_service_ports(&self, context: &str, namespace: &str, service: &str) -> Vec<String> {
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
      .output();

    match output {
      Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|port| format!("{}:{}", port, port))
        .collect(),
      _ => vec![],
    }
  }
}
