use crate::error::{AppError, Result};
use crate::services::ConfigService;
use crate::types::{ForwardType, PortForwardConfig};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ConfigCache {
  config_service: ConfigService,
  cache: Arc<Mutex<CacheData>>,
  ttl: Duration,
}

struct CacheData {
  configs: Option<Vec<PortForwardConfig>>,
  last_updated: Option<Instant>,
}

/// Helper to convert PoisonError to AppError
fn lock_error<T>(_: PoisonError<T>) -> AppError {
  AppError::System("Lock poisoned".to_string())
}

impl ConfigCache {
  pub fn new(config_service: ConfigService) -> Self {
    Self {
      config_service,
      cache: Arc::new(Mutex::new(CacheData {
        configs: None,
        last_updated: None,
      })),
      ttl: Duration::from_secs(5), // Cache for 5 seconds
    }
  }

  #[allow(dead_code)]
  pub fn with_ttl(config_service: ConfigService, ttl: Duration) -> Self {
    Self {
      config_service,
      cache: Arc::new(Mutex::new(CacheData {
        configs: None,
        last_updated: None,
      })),
      ttl,
    }
  }

  pub fn get_configs(&self) -> Result<Vec<PortForwardConfig>> {
    let mut cache_data = self.cache.lock().map_err(lock_error)?;

    if let Some(configs) = self.get_valid_cache(&cache_data) {
      return Ok(configs);
    }

    // Cache is invalid, reload from service
    let configs = self.config_service.load_port_forwards()?;
    cache_data.configs = Some(configs.clone());
    cache_data.last_updated = Some(Instant::now());

    Ok(configs)
  }

  #[allow(dead_code)]
  pub fn invalidate(&self) -> Result<()> {
    let mut cache_data = self.cache.lock().map_err(lock_error)?;
    cache_data.configs = None;
    cache_data.last_updated = None;
    Ok(())
  }

  pub fn update_configs(&self, configs: Vec<PortForwardConfig>) -> Result<()> {
    // Save to persistent storage
    self.config_service.save_port_forwards(&configs)?;

    // Update cache
    let mut cache_data = self.cache.lock().map_err(lock_error)?;
    cache_data.configs = Some(configs);
    cache_data.last_updated = Some(Instant::now());

    Ok(())
  }

  pub fn find_config(&self, service_name: &str) -> Result<Option<PortForwardConfig>> {
    let configs = self.get_configs()?;
    Ok(configs.into_iter().find(|c| c.name == service_name))
  }

  pub fn add_config(&self, config: PortForwardConfig) -> Result<()> {
    let mut configs = self.get_configs()?;
    configs.push(config);
    self.update_configs(configs)
  }

  pub fn remove_config(&self, service_key: &str) -> Result<()> {
    let mut configs = self.get_configs()?;
    configs.retain(|c| c.name != service_key);
    self.update_configs(configs)
  }

  pub fn update_config(&self, old_service_key: &str, new_config: PortForwardConfig) -> Result<()> {
    let mut configs = self.get_configs()?;

    if let Some(index) = configs.iter().position(|c| c.name == old_service_key) {
      configs[index] = new_config;
      self.update_configs(configs)
    } else {
      Err(AppError::NotFound(format!(
        "Configuration not found for service: {}",
        old_service_key
      )))
    }
  }

  pub fn reorder_config(&self, service_key: &str, new_index: usize) -> Result<()> {
    let mut configs = self.get_configs()?;

    let current_index = configs
      .iter()
      .position(|c| c.name == service_key)
      .ok_or_else(|| {
        AppError::NotFound(format!(
          "Configuration not found for service: {}",
          service_key
        ))
      })?;

    if new_index >= configs.len() {
      return Err(AppError::InvalidInput("Invalid new index".to_string()));
    }

    let config = configs.remove(current_index);
    configs.insert(new_index, config);

    self.update_configs(configs)
  }

  pub fn reorder_group(&self, group_key: &str, new_index: usize) -> Result<()> {
    let configs = self.get_configs()?;
    let configs = reorder_group_configs(configs, group_key, new_index)?;
    self.update_configs(configs)
  }

  /// Get cached configs if valid, None otherwise
  fn get_valid_cache(&self, cache_data: &CacheData) -> Option<Vec<PortForwardConfig>> {
    let configs = cache_data.configs.as_ref()?;
    let last_updated = cache_data.last_updated?;

    if last_updated.elapsed() < self.ttl {
      Some(configs.clone())
    } else {
      None
    }
  }
}

fn config_group_key(config: &PortForwardConfig) -> &str {
  match &config.forward_type {
    ForwardType::Ssh => "SSH",
    ForwardType::Kubectl => &config.context,
  }
}

fn reorder_group_configs(
  configs: Vec<PortForwardConfig>,
  group_key: &str,
  new_index: usize,
) -> Result<Vec<PortForwardConfig>> {
  let mut groups: Vec<(String, Vec<PortForwardConfig>)> = Vec::new();

  for config in configs {
    let key = config_group_key(&config).to_string();
    if let Some((_, group_configs)) = groups.iter_mut().find(|(group, _)| group == &key) {
      group_configs.push(config);
    } else {
      groups.push((key, vec![config]));
    }
  }

  let current_index = groups
    .iter()
    .position(|(group, _)| group == group_key)
    .ok_or_else(|| AppError::NotFound(format!("Configuration group not found: {}", group_key)))?;

  if new_index >= groups.len() {
    return Err(AppError::InvalidInput("Invalid group index".to_string()));
  }

  let group = groups.remove(current_index);
  groups.insert(new_index, group);

  Ok(
    groups
      .into_iter()
      .flat_map(|(_, group_configs)| group_configs)
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  fn config(name: &str, context: &str, forward_type: ForwardType) -> PortForwardConfig {
    PortForwardConfig {
      name: name.to_string(),
      context: context.to_string(),
      namespace: "default".to_string(),
      service: name.to_string(),
      ports: vec!["8080:80".to_string()],
      local_interface: None,
      forward_type,
    }
  }

  #[test]
  fn reorders_context_groups_without_changing_service_order() -> Result<()> {
    let configs = vec![
      config("a-1", "a", ForwardType::Kubectl),
      config("a-2", "a", ForwardType::Kubectl),
      config("b-1", "b", ForwardType::Kubectl),
      config("c-1", "c", ForwardType::Kubectl),
    ];

    let reordered = reorder_group_configs(configs, "c", 0)?;
    let names: Vec<&str> = reordered
      .iter()
      .map(|config| config.name.as_str())
      .collect();

    assert_eq!(names, vec!["c-1", "a-1", "a-2", "b-1"]);
    Ok(())
  }

  #[test]
  fn treats_ssh_configs_as_one_group() -> Result<()> {
    let configs = vec![
      config("kube", "cluster", ForwardType::Kubectl),
      config("ssh-1", "ignored-1", ForwardType::Ssh),
      config("ssh-2", "ignored-2", ForwardType::Ssh),
    ];

    let reordered = reorder_group_configs(configs, "SSH", 0)?;
    let names: Vec<&str> = reordered
      .iter()
      .map(|config| config.name.as_str())
      .collect();

    assert_eq!(names, vec!["ssh-1", "ssh-2", "kube"]);
    Ok(())
  }
}
