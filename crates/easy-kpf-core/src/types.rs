use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
  #[serde(default)]
  pub recovery: Option<RecoverySettings>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectPolicy {
  #[serde(default = "default_reconnect_enabled")]
  pub enabled: bool,
  #[serde(default = "default_initial_delay_ms")]
  pub initial_delay_ms: u64,
  #[serde(default = "default_max_delay_ms")]
  pub max_delay_ms: u64,
  #[serde(default = "default_stable_after_seconds")]
  pub stable_after_seconds: u64,
  #[serde(default)]
  pub max_attempts: u32,
}

impl Default for ReconnectPolicy {
  fn default() -> Self {
    Self {
      enabled: default_reconnect_enabled(),
      initial_delay_ms: default_initial_delay_ms(),
      max_delay_ms: default_max_delay_ms(),
      stable_after_seconds: default_stable_after_seconds(),
      max_attempts: 0,
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortForwardHooks {
  #[serde(default)]
  pub on_failure: Vec<HookCommand>,
  #[serde(default)]
  pub before_reconnect: Vec<HookCommand>,
  #[serde(default)]
  pub on_recovered: Vec<HookCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCommand {
  #[serde(rename = "type", default)]
  pub kind: HookType,
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
  #[serde(default)]
  pub ssh_host: Option<String>,
  #[serde(default = "default_hook_timeout_seconds")]
  pub timeout_seconds: u64,
  #[serde(default = "default_hook_cooldown_seconds")]
  pub cooldown_seconds: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
  #[default]
  Command,
  Ssh,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoverySettings {
  #[serde(default)]
  pub reconnect: ReconnectPolicy,
  #[serde(default)]
  pub hooks: PortForwardHooks,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryScopes {
  #[serde(default)]
  pub kubernetes_contexts: BTreeMap<String, RecoverySettings>,
  #[serde(default)]
  pub ssh_hosts: BTreeMap<String, RecoverySettings>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryScopeKind {
  KubernetesContext,
  SshHost,
}

impl RecoveryScopes {
  pub fn for_config(&self, config: &PortForwardConfig) -> Option<&RecoverySettings> {
    match config.forward_type {
      ForwardType::Kubectl => self.kubernetes_contexts.get(&config.context),
      ForwardType::Ssh => self.ssh_hosts.get(&config.context),
    }
  }

  pub fn set(&mut self, kind: RecoveryScopeKind, key: String, recovery: Option<RecoverySettings>) {
    let scopes = match kind {
      RecoveryScopeKind::KubernetesContext => &mut self.kubernetes_contexts,
      RecoveryScopeKind::SshHost => &mut self.ssh_hosts,
    };
    if let Some(recovery) = recovery {
      scopes.insert(key, recovery);
    } else {
      scopes.remove(&key);
    }
  }

  pub fn resolve(&self, config: &PortForwardConfig) -> RecoverySettings {
    config
      .recovery
      .clone()
      .or_else(|| self.for_config(config).cloned())
      .unwrap_or_default()
  }
}

const fn default_reconnect_enabled() -> bool {
  true
}

const fn default_initial_delay_ms() -> u64 {
  1_000
}

const fn default_max_delay_ms() -> u64 {
  30_000
}

const fn default_stable_after_seconds() -> u64 {
  30
}

const fn default_hook_timeout_seconds() -> u64 {
  30
}

const fn default_hook_cooldown_seconds() -> u64 {
  30
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortForwardConfigs {
  #[serde(default)]
  pub recovery: RecoveryScopes,
  pub configs: Vec<PortForwardConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProcessManagerState {
  pub processes: std::collections::HashMap<String, SerializableProcessInfo>,
}

#[cfg(test)]
mod tests {
  use super::{HookType, PortForwardConfigs, RecoverySettings};

  #[test]
  fn recovery_settings_round_trip_as_yaml() -> Result<(), serde_yaml::Error> {
    let settings = RecoverySettings {
      reconnect: Default::default(),
      hooks: Default::default(),
    };

    let yaml = serde_yaml::to_string(&settings)?;
    let restored: RecoverySettings = serde_yaml::from_str(&yaml)?;

    assert!(restored.reconnect.enabled);
    assert!(restored.hooks.before_reconnect.is_empty());
    Ok(())
  }

  #[test]
  fn port_forward_can_override_recovery_with_ssh_hook() -> Result<(), Box<dyn std::error::Error>> {
    let configs: PortForwardConfigs = serde_yaml::from_str(
      r#"
configs:
  - name: db gp
    context: tgs
    namespace: infra
    service: postgres
    ports: ["8101:5432"]
    recovery:
      reconnect:
        enabled: true
      hooks:
        before_reconnect:
          - type: ssh
            ssh_host: k3s-host
            command: sudo systemctl restart tunnel
"#,
    )?;
    let recovery = configs.configs[0]
      .recovery
      .as_ref()
      .ok_or_else(|| std::io::Error::other("missing recovery override"))?;
    let hook = &recovery.hooks.before_reconnect[0];

    assert_eq!(hook.kind, HookType::Ssh);
    assert_eq!(hook.ssh_host.as_deref(), Some("k3s-host"));
    assert_eq!(hook.command, "sudo systemctl restart tunnel");
    Ok(())
  }

  #[test]
  fn recovery_resolves_forward_then_scope_then_defaults() -> Result<(), Box<dyn std::error::Error>>
  {
    let document: PortForwardConfigs = serde_yaml::from_str(
      r#"
recovery:
  kubernetes_contexts:
    tgs:
      reconnect:
        initial_delay_ms: 2000
  ssh_hosts:
    job-host:
      reconnect:
        initial_delay_ms: 3000
configs:
  - name: context forward
    context: tgs
    namespace: infra
    service: svc/db
    ports: ["8101:5432"]
  - name: forward override
    context: tgs
    namespace: infra
    service: svc/cache
    ports: ["6380:6379"]
    recovery:
      reconnect:
        initial_delay_ms: 4000
  - name: ssh forward
    context: job-host
    namespace: default
    service: job-host
    ports: ["9000:9000"]
    forward_type: Ssh
  - name: built in
    context: other
    namespace: default
    service: svc/other
    ports: ["8080:80"]
"#,
    )?;

    assert_eq!(
      document
        .recovery
        .resolve(&document.configs[0])
        .reconnect
        .initial_delay_ms,
      2_000
    );
    assert_eq!(
      document
        .recovery
        .resolve(&document.configs[1])
        .reconnect
        .initial_delay_ms,
      4_000
    );
    assert_eq!(
      document
        .recovery
        .resolve(&document.configs[2])
        .reconnect
        .initial_delay_ms,
      3_000
    );
    assert_eq!(
      document
        .recovery
        .resolve(&document.configs[3])
        .reconnect
        .initial_delay_ms,
      1_000
    );
    Ok(())
  }
}
