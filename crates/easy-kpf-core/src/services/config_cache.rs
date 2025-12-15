use crate::error::{AppError, Result};
use crate::services::ConfigService;
use crate::types::PortForwardConfig;
use std::sync::{Arc, Mutex};
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
    let mut cache_data = self.cache.lock().unwrap();

    if self.is_cache_valid(&cache_data) {
      return Ok(cache_data.configs.as_ref().unwrap().clone());
    }

    // Cache is invalid, reload from service
    let configs = self.config_service.load_port_forwards()?;
    cache_data.configs = Some(configs.clone());
    cache_data.last_updated = Some(Instant::now());

    Ok(configs)
  }

  #[allow(dead_code)]
  pub fn invalidate(&self) {
    let mut cache_data = self.cache.lock().unwrap();
    cache_data.configs = None;
    cache_data.last_updated = None;
  }

  pub fn update_configs(&self, configs: Vec<PortForwardConfig>) -> Result<()> {
    // Save to persistent storage
    self.config_service.save_port_forwards(&configs)?;

    // Update cache
    let mut cache_data = self.cache.lock().unwrap();
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

  fn is_cache_valid(&self, cache_data: &CacheData) -> bool {
    if let (Some(_), Some(last_updated)) = (&cache_data.configs, cache_data.last_updated) {
      last_updated.elapsed() < self.ttl
    } else {
      false
    }
  }
}
