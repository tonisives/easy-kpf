use crate::error::{AppError, Result};
use crate::services::{ConfigService, KubectlOperations, ProcessManager};
use crate::types::PortForwardConfig;
use tauri_plugin_shell::ShellExt;

pub struct PortForwardService {
  app_handle: tauri::AppHandle,
  config_service: ConfigService,
  process_manager: ProcessManager,
}

impl PortForwardService {
  pub fn new(
    app_handle: tauri::AppHandle,
    config_service: ConfigService,
    process_manager: ProcessManager,
  ) -> Self {
    Self {
      app_handle,
      config_service,
      process_manager,
    }
  }

  pub fn get_configs(&self) -> Result<Vec<PortForwardConfig>> {
    self.config_service.load_port_forwards()
  }

  pub fn add_config(&self, config: PortForwardConfig) -> Result<()> {
    let mut configs = self.config_service.load_port_forwards()?;
    configs.push(config);
    self.config_service.save_port_forwards(&configs)
  }

  pub fn remove_config(&self, service_key: &str) -> Result<()> {
    let mut configs = self.config_service.load_port_forwards()?;
    configs.retain(|c| c.name != service_key);
    self.config_service.save_port_forwards(&configs)
  }

  pub fn update_config(&self, old_service_key: &str, new_config: PortForwardConfig) -> Result<()> {
    let mut configs = self.config_service.load_port_forwards()?;

    let current_index = configs
      .iter()
      .position(|c| c.name == old_service_key)
      .ok_or_else(|| {
        AppError::NotFound(format!(
          "Configuration not found for service: {}",
          old_service_key
        ))
      })?;

    // If the service name changed, update the process manager
    if old_service_key != new_config.name {
      self
        .process_manager
        .update_process_name(old_service_key, new_config.name.clone())?;
    }

    // Replace the config at the same position
    configs[current_index] = new_config;

    self.config_service.save_port_forwards(&configs)
  }

  pub fn reorder_config(&self, service_key: &str, new_index: usize) -> Result<()> {
    let mut configs = self.config_service.load_port_forwards()?;

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

    self.config_service.save_port_forwards(&configs)
  }

  pub async fn start_port_forward_by_key<K: KubectlOperations>(
    &self,
    kubectl_service: &K,
    service_key: &str,
  ) -> Result<String> {
    let configs = self.config_service.load_port_forwards()?;
    let config = configs
      .into_iter()
      .find(|c| c.name == service_key)
      .ok_or_else(|| {
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

    // Get and store current context
    let current_context = kubectl_service.get_current_context().await?;

    // Set the required context
    kubectl_service.set_context(&config.context).await?;

    let result = self.execute_port_forward(&config).await;

    // Restore original context
    let _ = kubectl_service.set_context(&current_context).await;

    result
  }

  async fn execute_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    match config.forward_type {
      crate::types::ForwardType::Kubectl => self.execute_kubectl_port_forward(config).await,
      crate::types::ForwardType::Ssh => self.execute_ssh_port_forward(config).await,
    }
  }

  async fn execute_kubectl_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    let shell = self.app_handle.shell();

    // Create local interface if specified and doesn't exist
    if let Some(ref interface) = config.local_interface {
      self.ensure_interface_exists(interface)?;
    }

    let mut args = vec!["-n", &config.namespace, "port-forward", &config.service];

    // Add --address flag if local interface is specified
    if let Some(ref interface) = config.local_interface {
      args.extend_from_slice(&["--address", interface]);
    }

    let port_refs: Vec<&str> = config.ports.iter().map(|s| s.as_str()).collect();
    args.extend(port_refs);

    let kubectl_cmd = self
      .config_service
      .load_kubectl_path()
      .unwrap_or_else(|_| "kubectl".to_string());

    let mut command = shell.command(&kubectl_cmd);

    // Set kubeconfig if available
    if let Ok(Some(kubeconfig)) = self.config_service.load_kubeconfig_path() {
      command = command.env("KUBECONFIG", kubeconfig);
    }

    let (_rx, child) = command
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
    let shell = self.app_handle.shell();

    // Create local interface if specified and doesn't exist
    if let Some(ref interface) = config.local_interface {
      self.ensure_interface_exists(interface)?;
    }

    // For SSH port forwarding, we need to parse the service as a host:port
    // Format: username@host or host (assuming current user)
    let ssh_target = &config.service;

    let mut ssh_args = vec!["-N"];

    // Collect port forwarding arguments first to avoid borrowing issues
    let mut forward_args = Vec::new();
    for port_mapping in &config.ports {
      let bind_interface = config.local_interface.as_deref().unwrap_or("127.0.0.1");
      let forward_arg = format!("{}:{}", bind_interface, port_mapping);
      forward_args.push(forward_arg);
    }

    // Add port forwarding arguments
    for forward_arg in &forward_args {
      ssh_args.extend_from_slice(&["-L", forward_arg]);
    }

    ssh_args.push(ssh_target);

    let (_rx, child) = shell
      .command("ssh")
      .args(ssh_args)
      .spawn()
      .map_err(|e| AppError::PortForward(e.to_string()))?;

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

  fn ensure_interface_exists(&self, interface: &str) -> Result<()> {
    // Skip for standard interfaces
    if interface == "127.0.0.1" || interface == "0.0.0.0" || interface == "localhost" {
      return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
      use std::process::Command;

      // Check if IP address is assigned to any interface
      let output = Command::new("ifconfig")
        .output()
        .map_err(|e| AppError::System(format!("Failed to check interfaces: {}", e)))?;

      let output_str = String::from_utf8_lossy(&output.stdout);
      let ip_exists = output_str
        .lines()
        .any(|line| line.trim().starts_with("inet") && line.contains(interface));

      if !output.status.success() || !ip_exists {
        // Check if we can create interface without sudo first
        let create_output_nosudo = Command::new("ifconfig")
          .args(["lo0", "alias", interface])
          .output();

        let create_success = match create_output_nosudo {
          Ok(output) => output.status.success(),
          Err(_) => false,
        };

        if !create_success {
          // Try with sudo, but handle password requirement gracefully
          let create_output = Command::new("sudo")
            .args(["-n", "ifconfig", "lo0", "alias", interface]) // -n prevents password prompt
            .output()
            .map_err(|e| AppError::System(format!("Failed to create interface: {}", e)))?;

          if !create_output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&create_output.stderr);
            if stderr_msg.contains("password") || stderr_msg.contains("sudo:") {
              return Err(AppError::System(format!(
                "Interface {} requires admin privileges to create. Please run: 'sudo ifconfig lo0 alias {}'",
                interface, interface
              )));
            } else {
              return Err(AppError::System(format!(
                "Failed to create interface {}: {}",
                interface, stderr_msg
              )));
            }
          }
        }
      }
    }

    #[cfg(target_os = "linux")]
    {
      use std::process::Command;

      // Check if interface exists
      let output = Command::new("ip")
        .args(&["addr", "show", interface])
        .output()
        .map_err(|e| AppError::System(format!("Failed to check interface: {}", e)))?;

      if !output.status.success() {
        // Check if we can create interface without sudo first
        let create_output_nosudo = Command::new("ip")
          .args(["addr", "add", &format!("{}/32", interface), "dev", "lo"])
          .output();

        let create_success = match create_output_nosudo {
          Ok(output) => output.status.success(),
          Err(_) => false,
        };

        if !create_success {
          // Try with sudo, but handle password requirement gracefully
          let create_output = Command::new("sudo")
            .args([
              "-n", // -n prevents password prompt
              "ip",
              "addr",
              "add",
              &format!("{}/32", interface),
              "dev",
              "lo",
            ])
            .output()
            .map_err(|e| AppError::System(format!("Failed to create interface: {}", e)))?;

          if !create_output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&create_output.stderr);
            if stderr_msg.contains("password") || stderr_msg.contains("sudo:") {
              return Err(AppError::System(format!(
                "Interface {} requires admin privileges to create. Please run: 'sudo ip addr add {}/32 dev lo'",
                interface, interface
              )));
            } else {
              return Err(AppError::System(format!(
                "Failed to create interface {}: {}",
                interface, stderr_msg
              )));
            }
          }
        }
      }
    }

    #[cfg(target_os = "windows")]
    {
      // Windows doesn't support creating virtual interfaces easily
      // Just validate it's a valid IP format
      use std::net::IpAddr;
      interface
        .parse::<IpAddr>()
        .map_err(|_| AppError::InvalidInput(format!("Invalid IP address: {}", interface)))?;
    }

    Ok(())
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
      let is_actually_running = self.is_process_actually_running(pid)?;
      if !is_actually_running {
        // Clean up dead process
        let _ = self.process_manager.remove_process(&service_name);
      }
      results.push((service_name, is_actually_running));
    }

    Ok(results)
  }

  fn is_process_actually_running(&self, pid: u32) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
      use std::process::Command;
      let output = Command::new("ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
      Ok(output.status.success())
    }

    #[cfg(target_os = "linux")]
    {
      use std::process::Command;
      let output = Command::new("ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
      Ok(output.status.success())
    }

    #[cfg(target_os = "windows")]
    {
      use std::process::Command;
      let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid)])
        .output()
        .map_err(|e| AppError::System(format!("Failed to check process: {}", e)))?;
      let output_str = String::from_utf8_lossy(&output.stdout);
      Ok(output_str.contains(&pid.to_string()))
    }
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
    let configs = self.config_service.load_port_forwards()?;
    let mut detected_services = Vec::new();

    for config in &configs {
      if self.is_kubectl_process_running(config)?
        && !self.process_manager.contains_process(&config.name)?
      {
        detected_services.push(config.name.clone());
      }
    }

    Ok(detected_services)
  }

  fn is_kubectl_process_running(&self, config: &PortForwardConfig) -> Result<bool> {
    if config.forward_type != crate::types::ForwardType::Kubectl {
      return Ok(false);
    }

    #[cfg(target_os = "macos")]
    {
      use std::process::Command;

      // Use ps with grep to find kubectl port-forward processes
      let output = Command::new("ps")
        .args(["aux"])
        .output()
        .map_err(|e| AppError::System(format!("Failed to list processes: {}", e)))?;

      let stdout = String::from_utf8_lossy(&output.stdout);

      for line in stdout.lines() {
        if self.matches_kubectl_command(line, config) {
          return Ok(true);
        }
      }
    }

    #[cfg(target_os = "linux")]
    {
      use std::process::Command;

      let output = Command::new("ps")
        .args(["aux"])
        .output()
        .map_err(|e| AppError::System(format!("Failed to list processes: {}", e)))?;

      let stdout = String::from_utf8_lossy(&output.stdout);

      for line in stdout.lines() {
        if self.matches_kubectl_command(line, config) {
          return Ok(true);
        }
      }
    }

    #[cfg(target_os = "windows")]
    {
      // Windows implementation would go here if needed
      return Ok(false);
    }

    Ok(false)
  }

  fn matches_kubectl_command(&self, process_line: &str, config: &PortForwardConfig) -> bool {
    // Check if this is a kubectl port-forward command
    if !process_line.contains("kubectl") || !process_line.contains("port-forward") {
      return false;
    }

    // Check namespace
    if !process_line.contains(&format!("-n {}", config.namespace))
      && !process_line.contains(&format!("--namespace={}", config.namespace))
      && !process_line.contains(&format!("--namespace {}", config.namespace))
    {
      return false;
    }

    // Check service name
    if !process_line.contains(&config.service) {
      return false;
    }

    // Check if at least one port mapping matches
    for port in &config.ports {
      if process_line.contains(port) {
        return true;
      }
    }

    false
  }

  pub fn sync_with_existing_processes(&self) -> Result<Vec<String>> {
    let detected = self.detect_existing_port_forwards()?;
    let mut synced_services = Vec::new();

    for service_name in &detected {
      // Try to extract PID from running process for this service
      if let Some(pid) = self.find_kubectl_process_pid(service_name)? {
        let configs = self.config_service.load_port_forwards()?;
        if let Some(config) = configs.iter().find(|c| c.name == *service_name) {
          // Add to process manager to track it
          self
            .process_manager
            .add_process(service_name.clone(), pid, config.clone())?;
          synced_services.push(service_name.clone());
        }
      }
    }

    Ok(synced_services)
  }

  fn find_kubectl_process_pid(&self, service_name: &str) -> Result<Option<u32>> {
    let configs = self.config_service.load_port_forwards()?;
    let config = configs.iter().find(|c| c.name == *service_name);

    if let Some(config) = config {
      #[cfg(unix)]
      {
        use std::process::Command;

        let output = Command::new("ps")
          .args(["aux"])
          .output()
          .map_err(|e| AppError::System(format!("Failed to list processes: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
          if self.matches_kubectl_command(line, config) {
            // Extract PID from ps output (second column)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              if let Ok(pid) = parts[1].parse::<u32>() {
                return Ok(Some(pid));
              }
            }
          }
        }
      }
    }

    Ok(None)
  }
}
