use crate::error::{AppError, Result};
use crate::types::{AppConfig, PortForwardConfig, PortForwardConfigs};
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

    pub fn load_port_forwards(&self) -> Result<Vec<PortForwardConfig>> {
        let config_path = self.config_dir.join("port-forwards.yaml");

        if !config_path.exists() {
            let default_configs = PortForwardConfigs { configs: vec![] };
            self.save_port_forwards(&default_configs.configs)?;
            return Ok(default_configs.configs);
        }

        let config_content = fs::read_to_string(&config_path)?;
        let configs: PortForwardConfigs = serde_yaml::from_str(&config_content)?;
        Ok(configs.configs)
    }

    pub fn save_port_forwards(&self, configs: &[PortForwardConfig]) -> Result<()> {
        let config_path = self.config_dir.join("port-forwards.yaml");
        let configs_wrapper = PortForwardConfigs {
            configs: configs.to_vec(),
        };
        let yaml_content = serde_yaml::to_string(&configs_wrapper)?;
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
