use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardConfig {
    pub name: String,
    pub context: String,
    pub namespace: String,
    pub service: String,
    pub ports: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PortForwardConfigs {
    configs: Vec<PortForwardConfig>,
}

fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("EasyKpf");

    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir)
}

fn get_config_file_path() -> Result<PathBuf, String> {
    Ok(get_config_dir()?.join("port-forwards.yaml"))
}

pub fn load_configs() -> Result<Vec<PortForwardConfig>, String> {
    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        let default_configs = PortForwardConfigs { configs: vec![] };

        save_configs(&default_configs.configs)?;
        return Ok(default_configs.configs);
    }

    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let configs: PortForwardConfigs =
        serde_yaml::from_str(&config_content).map_err(|e| e.to_string())?;
    Ok(configs.configs)
}

pub fn save_configs(configs: &[PortForwardConfig]) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let configs_wrapper = PortForwardConfigs {
        configs: configs.to_vec(),
    };
    let yaml_content = serde_yaml::to_string(&configs_wrapper).map_err(|e| e.to_string())?;
    fs::write(&config_path, yaml_content).map_err(|e| e.to_string())?;
    Ok(())
}
