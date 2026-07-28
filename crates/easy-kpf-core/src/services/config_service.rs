use crate::error::{AppError, Result};
use crate::types::{AppConfig, PortForwardConfig, PortForwardConfigs, RecoveryScopes};
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ConfigService {
  config_dir: PathBuf,
}

impl ConfigService {
  pub fn new() -> Result<Self> {
    let config_dir = dirs::config_dir()
      .ok_or_else(|| AppError::Config("Could not find config directory".to_string()))?
      .join("EasyKpf");

    fs::create_dir_all(&config_dir)?;

    Ok(Self { config_dir })
  }

  pub fn config_dir(&self) -> &PathBuf {
    &self.config_dir
  }

  pub fn load_port_forwards(&self) -> Result<Vec<PortForwardConfig>> {
    Ok(self.load_port_forward_config()?.configs)
  }

  pub fn load_recovery_scopes(&self) -> Result<RecoveryScopes> {
    Ok(self.load_port_forward_config()?.recovery)
  }

  fn load_port_forward_config(&self) -> Result<PortForwardConfigs> {
    let config_path = self.config_dir.join("port-forwards.yaml");

    if !config_path.exists() {
      let default_config = PortForwardConfigs::default();
      self.save_port_forward_config(&default_config)?;
      return Ok(default_config);
    }

    let config_content = fs::read_to_string(&config_path)?;
    let mut config: PortForwardConfigs = serde_yaml::from_str(&config_content)?;

    // Migrate old configs to include new fields with defaults
    for port_forward in &mut config.configs {
      if port_forward.local_interface.is_none() {
        port_forward.local_interface = None; // Keep as None for existing configs
      }
    }

    Ok(config)
  }

  pub fn save_port_forwards(&self, configs: &[PortForwardConfig]) -> Result<()> {
    let mut config = self.load_port_forward_config()?;
    config.configs = configs.to_vec();
    self.save_port_forward_config(&config)
  }

  pub fn save_recovery_scopes(&self, recovery: RecoveryScopes) -> Result<()> {
    let mut config = self.load_port_forward_config()?;
    config.recovery = recovery;
    self.save_port_forward_config(&config)
  }

  fn save_port_forward_config(&self, config: &PortForwardConfigs) -> Result<()> {
    let config_path = self.config_dir.join("port-forwards.yaml");
    let yaml_content = serde_yaml::to_string(config)?;
    fs::write(&config_path, yaml_content)?;
    Ok(())
  }

  pub fn load_app_config(&self) -> Result<AppConfig> {
    let config_path = self.config_dir.join("app-config.yaml");

    if !config_path.exists() {
      let default_config = AppConfig {
        kubectl_path: None,
        kubeconfig_path: None,
      };
      self.save_app_config(&default_config)?;
      return Ok(default_config);
    }

    let config_content = fs::read_to_string(&config_path)?;
    let config: AppConfig = serde_yaml::from_str(&config_content)?;
    Ok(config)
  }

  pub fn save_app_config(&self, config: &AppConfig) -> Result<()> {
    let config_path = self.config_dir.join("app-config.yaml");
    let yaml_content = serde_yaml::to_string(config)?;
    fs::write(&config_path, yaml_content)?;
    Ok(())
  }

  pub fn load_kubectl_path(&self) -> Result<String> {
    let config = self.load_app_config()?;
    config
      .kubectl_path
      .ok_or_else(|| AppError::Config("kubectl path not configured".to_string()))
  }

  pub fn save_kubectl_path(&self, path: String) -> Result<()> {
    let mut config = self.load_app_config()?;
    config.kubectl_path = Some(path);
    self.save_app_config(&config)
  }

  pub fn load_kubeconfig_path(&self) -> Result<Option<String>> {
    let config = self.load_app_config()?;
    Ok(config.kubeconfig_path)
  }

  pub fn save_kubeconfig_path(&self, path: String) -> Result<()> {
    let mut config = self.load_app_config()?;
    config.kubeconfig_path = Some(path);
    self.save_app_config(&config)
  }
}

#[cfg(test)]
mod tests {
  use super::ConfigService;
  use crate::types::PortForwardConfigs;
  use std::fs;
  use std::time::{SystemTime, UNIX_EPOCH};

  #[test]
  fn saving_forwards_preserves_scoped_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let config_dir = std::env::temp_dir().join(format!(
      "easy-kpf-config-service-{}-{}",
      std::process::id(),
      unique
    ));
    fs::create_dir_all(&config_dir)?;
    let service = ConfigService {
      config_dir: config_dir.clone(),
    };
    fs::write(
      config_dir.join("port-forwards.yaml"),
      r#"
recovery:
  kubernetes_contexts:
    tgs:
      reconnect:
        initial_delay_ms: 2500
configs:
  - name: db
    context: tgs
    namespace: infra
    service: svc/db
    ports: ["8101:5432"]
"#,
    )?;

    let mut configs = service.load_port_forwards()?;
    configs[0].ports = vec!["8102:5432".to_string()];
    service.save_port_forwards(&configs)?;

    let saved: PortForwardConfigs =
      serde_yaml::from_str(&fs::read_to_string(config_dir.join("port-forwards.yaml"))?)?;
    assert_eq!(
      saved.recovery.kubernetes_contexts["tgs"]
        .reconnect
        .initial_delay_ms,
      2_500
    );
    assert_eq!(saved.configs[0].ports, vec!["8102:5432"]);

    fs::remove_dir_all(config_dir)?;
    Ok(())
  }
}
