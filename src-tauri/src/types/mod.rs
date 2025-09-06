use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardConfig {
  pub name: String,
  pub context: String,
  pub namespace: String,
  pub service: String,
  pub ports: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
  pub kubectl_path: Option<String>,
  pub kubeconfig_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
  pub pid: u32,
  pub config: PortForwardConfig,
  #[allow(dead_code)]
  pub started_at: Instant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardConfigs {
  pub configs: Vec<PortForwardConfig>,
}
