use crate::error::{AppError, Result};
use crate::services::command_builder::{KubectlCommandBuilder, SshCommandBuilder};
use crate::services::config_cache::ConfigCache;
use crate::services::interface::{InterfaceManager, SystemInterfaceManager};
use crate::services::process_detector::ProcessDetector;
use crate::services::{ConfigService, KubectlOperations, ProcessManager};
use crate::types::PortForwardConfig;
use tauri_plugin_shell::ShellExt;

pub struct PortForwardService {
  app_handle: tauri::AppHandle,
  config_cache: ConfigCache,
  config_service: ConfigService,
  process_manager: ProcessManager,
  interface_manager: SystemInterfaceManager,
  process_detector: ProcessDetector,
}

impl PortForwardService {
  pub fn new(
    app_handle: tauri::AppHandle,
    config_service: ConfigService,
    process_manager: ProcessManager,
  ) -> Self {
    Self {
      app_handle,
      config_cache: ConfigCache::new(config_service.clone()),
      config_service,
      process_manager,
      interface_manager: SystemInterfaceManager,
      process_detector: ProcessDetector::new(),
    }
  }

  pub fn get_configs(&self) -> Result<Vec<PortForwardConfig>> {
    self.config_cache.get_configs()
  }

  pub fn add_config(&self, config: PortForwardConfig) -> Result<()> {
    self.config_cache.add_config(config)
  }

  pub fn remove_config(&self, service_key: &str) -> Result<()> {
    self.config_cache.remove_config(service_key)
  }

  pub fn update_config(&self, old_service_key: &str, new_config: PortForwardConfig) -> Result<()> {
    // If the service name changed, update the process manager
    if old_service_key != new_config.name {
      self
        .process_manager
        .update_process_name(old_service_key, new_config.name.clone())?;
    }

    self.config_cache.update_config(old_service_key, new_config)
  }

  pub fn reorder_config(&self, service_key: &str, new_index: usize) -> Result<()> {
    self.config_cache.reorder_config(service_key, new_index)
  }

  pub async fn start_port_forward_by_key<K: KubectlOperations>(
    &self,
    kubectl_service: &K,
    service_key: &str,
  ) -> Result<String> {
    let config = self.config_cache.find_config(service_key)?.ok_or_else(|| {
      AppError::NotFound(format!(
        "Configuration not found for service: {}",
        service_key
      ))
    })?;

    self
      .start_port_forward_generic(kubectl_service, config)
      .await
  }

  pub async fn start_port_forward_generic<K: KubectlOperations>(
    &self,
    kubectl_service: &K,
    config: PortForwardConfig,
  ) -> Result<String> {
    // Check if already running
    if self.process_manager.contains_process(&config.name)? {
      return Err(AppError::PortForward(format!(
        "{} port forwarding is already running",
        config.name
      )));
    }

    // Only handle kubectl context for kubectl port forwards, not SSH
    let current_context = if config.forward_type == crate::types::ForwardType::Kubectl {
      // Get and store current context
      let context = kubectl_service.get_current_context().await?;
      // Set the required context
      kubectl_service.set_context(&config.context).await?;
      Some(context)
    } else {
      None
    };

    let result = self.execute_port_forward(&config).await;

    // Restore original context if we changed it
    if let Some(original_context) = current_context {
      let _ = kubectl_service.set_context(&original_context).await;
    }

    result
  }

  async fn execute_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    match config.forward_type {
      crate::types::ForwardType::Kubectl => self.execute_kubectl_port_forward(config).await,
      crate::types::ForwardType::Ssh => self.execute_ssh_port_forward(config).await,
    }
  }

  async fn execute_kubectl_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    // Create local interface if specified and doesn't exist
    if let Some(ref interface) = config.local_interface {
      self.interface_manager.ensure_interface_exists(interface)?;
    }

    let kubectl_path = self
      .config_service
      .load_kubectl_path()
      .unwrap_or_else(|_| "kubectl".to_string());

    let kubeconfig_path = self.config_service.load_kubeconfig_path().ok().flatten();

    let (command, args, env_vars) = KubectlCommandBuilder::new(kubectl_path, kubeconfig_path)
      .build_port_forward_command(config);

    let shell = self.app_handle.shell();
    let mut command_builder = shell.command(&command);

    // Set environment variables
    for (key, value) in env_vars {
      command_builder = command_builder.env(key, value);
    }

    let (_rx, child) = command_builder
      .args(args)
      .spawn()
      .map_err(|e| AppError::PortForward(e.to_string()))?;

    let pid = child.pid();

    // Add to process manager
    self
      .process_manager
      .add_process(config.name.clone(), pid, config.clone())?;

    Ok(format!(
      "{} kubectl port forwarding started with PID: {}",
      config.name, pid
    ))
  }

  async fn execute_ssh_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    // Create local interface if specified and doesn't exist
    if let Some(ref interface) = config.local_interface {
      self.interface_manager.ensure_interface_exists(interface)?;
    }

    let ssh_builder = SshCommandBuilder::new();
    let (command, args) = ssh_builder.build_port_forward_command(config);

    log::debug!(
      "Starting SSH port forward with command: {} {}",
      command,
      args.join(" ")
    );

    let shell = self.app_handle.shell();
    let (_rx, child) = shell
      .command(&command)
      .args(args)
      .spawn()
      .map_err(|e| AppError::PortForward(format!("Failed to start SSH: {}", e)))?;

    let pid = child.pid();

    // Add to process manager
    self
      .process_manager
      .add_process(config.name.clone(), pid, config.clone())?;

    Ok(format!(
      "{} SSH port forwarding started with PID: {}",
      config.name, pid
    ))
  }

  pub fn stop_port_forward(&self, service_name: &str) -> Result<String> {
    let pid = self
      .process_manager
      .remove_process(service_name)?
      .ok_or_else(|| {
        AppError::NotFound(format!("{} port forwarding is not running", service_name))
      })?;

    ProcessManager::kill_process(pid)?;

    Ok(format!(
      "Stopped {} port forwarding (PID: {})",
      service_name, pid
    ))
  }

  pub fn get_running_services(&self) -> Result<Vec<String>> {
    self.process_manager.get_running_services()
  }

  pub fn cleanup_all_port_forwards(&self) -> Result<()> {
    let pids = self.process_manager.cleanup_all()?;

    for pid in pids {
      let _ = ProcessManager::kill_process(pid);
    }

    Ok(())
  }

  pub fn verify_port_forwards(&self) -> Result<Vec<(String, bool)>> {
    let running_services = self.process_manager.get_running_services_with_pids()?;
    let mut results = Vec::new();

    for (service_name, pid) in running_services {
      let is_actually_running = self.process_detector.is_process_actually_running(pid)?;
      if !is_actually_running {
        // Clean up dead process
        let _ = self.process_manager.remove_process(&service_name);
      }
      results.push((service_name, is_actually_running));
    }

    Ok(results)
  }

  pub fn verify_and_update_port_forwards(&self) -> Result<Vec<String>> {
    let verification_results = self.verify_port_forwards()?;
    let mut stopped_services = Vec::new();

    for (service_name, is_running) in verification_results {
      if !is_running {
        stopped_services.push(service_name);
      }
    }

    Ok(stopped_services)
  }

  pub fn detect_existing_port_forwards(&self) -> Result<Vec<String>> {
    let configs = self.config_cache.get_configs()?;
    let mut detected_services = Vec::new();

    for config in &configs {
      if self.process_detector.is_kubectl_process_running(config)?
        && !self.process_manager.contains_process(&config.name)?
      {
        detected_services.push(config.name.clone());
      }
    }

    Ok(detected_services)
  }

  pub fn sync_with_existing_processes(&self) -> Result<Vec<String>> {
    let detected = self.detect_existing_port_forwards()?;
    let mut synced_services = Vec::new();

    for service_name in &detected {
      let config = self.config_cache.find_config(service_name)?;
      if let Some(config) = config {
        if let Some(pid) = self.process_detector.find_kubectl_process_pid(&config)? {
          // Add to process manager to track it
          self
            .process_manager
            .add_process(service_name.clone(), pid, config)?;
          synced_services.push(service_name.clone());
        }
      }
    }

    Ok(synced_services)
  }
}
