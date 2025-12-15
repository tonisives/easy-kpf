use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardConfig {
  pub name: String,
  pub context: String,
  pub namespace: String,
  pub service: String,
  pub ports: Vec<String>,
  #[serde(default)]
  pub local_interface: Option<String>,
  #[serde(default)]
  pub forward_type: ForwardType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ForwardType {
  #[default]
  Kubectl,
  Ssh,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProcessInfo {
  pub pid: u32,
  pub config: PortForwardConfig,
}

impl From<&ProcessInfo> for SerializableProcessInfo {
  fn from(info: &ProcessInfo) -> Self {
    Self {
      pid: info.pid,
      config: info.config.clone(),
    }
  }
}

impl From<SerializableProcessInfo> for ProcessInfo {
  fn from(info: SerializableProcessInfo) -> Self {
    Self {
      pid: info.pid,
      config: info.config,
      started_at: Instant::now(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardConfigs {
  pub configs: Vec<PortForwardConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProcessManagerState {
  pub processes: std::collections::HashMap<String, SerializableProcessInfo>,
}
