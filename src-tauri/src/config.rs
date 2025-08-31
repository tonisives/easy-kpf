use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    kubectl_path: Option<String>,
}

fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("EasyKpf");

    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir)
}

fn get_config_file_path() -> Result<PathBuf, String> {
    Ok(get_config_dir()?.join("app-config.yaml"))
}

fn load_app_config() -> Result<AppConfig, String> {
    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        let default_config = AppConfig { kubectl_path: None };
        save_app_config(&default_config)?;
        return Ok(default_config);
    }

    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_yaml::from_str(&config_content).map_err(|e| e.to_string())?;
    Ok(config)
}

fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let yaml_content = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, yaml_content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_kubectl_path() -> Result<String, String> {
    let config = load_app_config()?;
    config
        .kubectl_path
        .ok_or_else(|| "kubectl path not configured".to_string())
}

pub fn save_kubectl_path(path: String) -> Result<(), String> {
    let mut config = load_app_config()?;
    config.kubectl_path = Some(path);
    save_app_config(&config)
}
