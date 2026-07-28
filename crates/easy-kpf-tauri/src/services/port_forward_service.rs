use crate::recovery::{HookContext, HookEvent, RecoveryCoordinator, RecoveryTicket};
use easy_kpf_core::error::{AppError, Result};
use easy_kpf_core::services::{
  ConfigCache, ConfigService, InterfaceManager, KubectlCommandBuilder, LastActiveSet,
  ProcessDetector, ProcessManager, SshCommandBuilder, SystemInterfaceManager,
};
use easy_kpf_core::types::{ForwardType, PortForwardConfig};
use serde::Serialize;
use std::time::Duration;
use tauri::{Emitter, Manager};

use super::KubectlOperations;
use tauri_plugin_shell::ShellExt;

#[derive(Clone, Serialize)]
pub struct ServiceErrorEvent {
  pub service_name: String,
  pub error: String,
  pub fatal: bool,
}

#[derive(Clone, Serialize)]
struct ServiceRecoveredEvent {
  service_name: String,
  attempt: u32,
}

pub struct PortForwardService {
  app_handle: tauri::AppHandle,
  config_cache: ConfigCache,
  config_service: ConfigService,
  process_manager: ProcessManager,
  last_active: LastActiveSet,
  interface_manager: SystemInterfaceManager,
  process_detector: ProcessDetector,
  recovery: RecoveryCoordinator,
}

impl PortForwardService {
  pub fn new(
    app_handle: tauri::AppHandle,
    config_service: ConfigService,
    process_manager: ProcessManager,
    last_active: LastActiveSet,
  ) -> Self {
    Self {
      app_handle,
      config_cache: ConfigCache::new(config_service.clone()),
      config_service,
      process_manager,
      last_active,
      interface_manager: SystemInterfaceManager,
      process_detector: ProcessDetector::new(),
      recovery: RecoveryCoordinator::default(),
    }
  }

  pub fn last_active(&self) -> &LastActiveSet {
    &self.last_active
  }

  pub fn get_configs(&self) -> Result<Vec<PortForwardConfig>> {
    self.config_cache.get_configs()
  }

  pub fn add_config(&self, config: PortForwardConfig) -> Result<()> {
    self.config_cache.add_config(config)
  }

  pub fn remove_config(&self, service_key: &str) -> Result<()> {
    self.last_active.remove(service_key)?;
    self.config_cache.remove_config(service_key)
  }

  pub fn update_config(&self, old_service_key: &str, new_config: PortForwardConfig) -> Result<()> {
    // If the service name changed, update the process manager
    if old_service_key != new_config.name {
      self
        .process_manager
        .update_process_name(old_service_key, new_config.name.clone())?;
      self.last_active.rename(old_service_key, &new_config.name)?;
    }

    self.config_cache.update_config(old_service_key, new_config)
  }

  pub fn reorder_config(&self, service_key: &str, new_index: usize) -> Result<()> {
    self.config_cache.reorder_config(service_key, new_index)
  }

  pub fn reorder_group(&self, group_key: &str, new_index: usize) -> Result<()> {
    self.config_cache.reorder_group(group_key, new_index)
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

    self.recovery.cancel(service_key)?;
    self
      .start_port_forward_generic(kubectl_service, config)
      .await
  }

  pub async fn start_port_forward_generic<K: KubectlOperations>(
    &self,
    _kubectl_service: &K,
    config: PortForwardConfig,
  ) -> Result<String> {
    self.verify_port_forwards()?;
    // Check if already running
    if self.process_manager.contains_process(&config.name)? {
      return Err(AppError::PortForward(format!(
        "{} port forwarding is already running",
        config.name
      )));
    }

    // No need to switch contexts - we use --context flag in the kubectl command
    log::info!(
      "Starting port forward for {} in context {}",
      config.name,
      config.context
    );

    self.execute_port_forward(&config).await
  }

  pub async fn restart_port_forward_by_key<K: KubectlOperations>(
    &self,
    kubectl_service: &K,
    service_key: &str,
  ) -> Result<String> {
    self.recovery.cancel(service_key)?;
    self
      .restart_port_forward_after_failure(kubectl_service, service_key)
      .await
  }

  pub(crate) async fn restart_port_forward_after_failure<K: KubectlOperations>(
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

    if let Some(pid) = self.process_manager.remove_process(service_key)? {
      if let Err(error) = ProcessManager::kill_process(pid) {
        self
          .process_manager
          .add_process(service_key.to_string(), pid, config.clone())?;
        return Err(error);
      }
      self.wait_for_process_exit(service_key, pid).await?;
    }

    self
      .start_port_forward_generic(kubectl_service, config)
      .await
  }

  async fn wait_for_process_exit(&self, service_name: &str, pid: u32) -> Result<()> {
    const PROCESS_EXIT_ATTEMPTS: usize = 50;
    const PROCESS_EXIT_POLL: Duration = Duration::from_millis(100);

    for _ in 0..PROCESS_EXIT_ATTEMPTS {
      if !self.process_detector.is_process_actually_running(pid)? {
        return Ok(());
      }
      tokio::time::sleep(PROCESS_EXIT_POLL).await;
    }

    Err(AppError::Process(format!(
      "{} process {} did not exit after reconnect request",
      service_name, pid
    )))
  }

  async fn execute_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    match config.forward_type {
      ForwardType::Kubectl => self.execute_kubectl_port_forward(config).await,
      ForwardType::Ssh => self.execute_ssh_port_forward(config).await,
    }
  }

  #[allow(clippy::too_many_lines)]
  async fn execute_kubectl_port_forward(&self, config: &PortForwardConfig) -> Result<String> {
    // Create local interface if specified and doesn't exist
    if let Some(ref interface) = config.local_interface {
      self.interface_manager.ensure_interface_exists(interface)?;
    }

    let kubectl_path = match self.config_service.load_kubectl_path() {
      Ok(path) => path,
      Err(e) => {
        log::warn!("Failed to load kubectl path, using default: {}", e);
        "kubectl".to_string()
      }
    };

    let kubeconfig_path = self.config_service.load_kubeconfig_path().ok().flatten();

    let (command, args, env_vars) =
      KubectlCommandBuilder::new(kubectl_path, kubeconfig_path).build_port_forward_command(config);

    let shell = self.app_handle.shell();
    let mut command_builder = shell.command(&command);

    // Set environment variables
    for (key, value) in env_vars {
      command_builder = command_builder.env(key, value);
    }

    let (rx, child) = command_builder
      .args(args)
      .spawn()
      .map_err(|e| AppError::PortForward(e.to_string()))?;

    let pid = child.pid();

    // Add to process manager
    self
      .process_manager
      .add_process(config.name.clone(), pid, config.clone())?;
    self.last_active.add(&config.name)?;

    // Monitor process output in background
    let service_name = config.name.clone();
    let app_handle = self.app_handle.clone();
    let process_manager = self.process_manager.clone();
    let recovery = self.recovery.clone();
    tauri::async_runtime::spawn(async move {
      use tauri_plugin_shell::process::CommandEvent;
      let mut rx = rx;
      let mut unhealthy = false;
      while let Some(event) = rx.recv().await {
        match event {
          CommandEvent::Stdout(line) => {
            log::info!("[{}] {}", service_name, String::from_utf8_lossy(&line));
          }
          CommandEvent::Stderr(line) => {
            let error_text = String::from_utf8_lossy(&line).to_string();
            log::error!("[{}] {}", service_name, error_text);
            let fatal = is_fatal_forward_error(&error_text);
            if !unhealthy && fatal {
              unhealthy = true;
              let was_managed = process_manager
                .remove_process_if_pid(&service_name, pid)
                .unwrap_or(false);
              let _ = ProcessManager::kill_process(pid);
              if was_managed {
                schedule_recovery(
                  app_handle.clone(),
                  recovery.clone(),
                  service_name.clone(),
                  error_text.clone(),
                );
              }
            }
            // Emit error event to frontend
            let _ = app_handle.emit(
              "service-error",
              ServiceErrorEvent {
                service_name: service_name.clone(),
                error: error_text,
                fatal,
              },
            );
          }
          CommandEvent::Error(err) => {
            log::error!("[{}] Process error: {}", service_name, err);
            if !unhealthy {
              unhealthy = true;
              let was_managed = process_manager
                .remove_process_if_pid(&service_name, pid)
                .unwrap_or(false);
              let _ = ProcessManager::kill_process(pid);
              if was_managed {
                schedule_recovery(
                  app_handle.clone(),
                  recovery.clone(),
                  service_name.clone(),
                  format!("Process error: {}", err),
                );
              }
            }
            // Emit error event to frontend
            let _ = app_handle.emit(
              "service-error",
              ServiceErrorEvent {
                service_name: service_name.clone(),
                error: format!("Process error: {}", err),
                fatal: true,
              },
            );
          }
          CommandEvent::Terminated(payload) => {
            log::warn!(
              "[{}] Process terminated with code: {:?}.",
              service_name,
              payload.code
            );
            let was_managed = process_manager
              .remove_process_if_pid(&service_name, pid)
              .unwrap_or(false);
            if was_managed {
              schedule_recovery(
                app_handle.clone(),
                recovery.clone(),
                service_name.clone(),
                "Port forward stopped unexpectedly".to_string(),
              );
              let _ = app_handle.emit(
                "service-error",
                ServiceErrorEvent {
                  service_name: service_name.clone(),
                  error: "Port forward stopped unexpectedly".to_string(),
                  fatal: true,
                },
              );
            }
          }
          _ => {}
        }
      }
    });

    Ok(format!(
      "{} kubectl port forwarding started with PID: {}",
      config.name, pid
    ))
  }

  #[allow(clippy::too_many_lines)]
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
    let (rx, child) = shell
      .command(&command)
      .args(args)
      .spawn()
      .map_err(|e| AppError::PortForward(format!("Failed to start SSH: {}", e)))?;

    let pid = child.pid();

    // Add to process manager
    self
      .process_manager
      .add_process(config.name.clone(), pid, config.clone())?;
    self.last_active.add(&config.name)?;

    // Monitor process output in background
    let service_name = config.name.clone();
    let app_handle = self.app_handle.clone();
    let process_manager = self.process_manager.clone();
    let recovery = self.recovery.clone();
    tauri::async_runtime::spawn(async move {
      use tauri_plugin_shell::process::CommandEvent;
      let mut rx = rx;
      let mut unhealthy = false;
      while let Some(event) = rx.recv().await {
        match event {
          CommandEvent::Stdout(line) => {
            log::info!("[{}] {}", service_name, String::from_utf8_lossy(&line));
          }
          CommandEvent::Stderr(line) => {
            let error_text = String::from_utf8_lossy(&line).to_string();
            log::error!("[{}] {}", service_name, error_text);
            let fatal = is_fatal_forward_error(&error_text);
            if !unhealthy && fatal {
              unhealthy = true;
              let was_managed = process_manager
                .remove_process_if_pid(&service_name, pid)
                .unwrap_or(false);
              let _ = ProcessManager::kill_process(pid);
              if was_managed {
                schedule_recovery(
                  app_handle.clone(),
                  recovery.clone(),
                  service_name.clone(),
                  error_text.clone(),
                );
              }
            }
            // Emit error event to frontend
            let _ = app_handle.emit(
              "service-error",
              ServiceErrorEvent {
                service_name: service_name.clone(),
                error: error_text,
                fatal,
              },
            );
          }
          CommandEvent::Error(err) => {
            log::error!("[{}] Process error: {}", service_name, err);
            if !unhealthy {
              unhealthy = true;
              let was_managed = process_manager
                .remove_process_if_pid(&service_name, pid)
                .unwrap_or(false);
              let _ = ProcessManager::kill_process(pid);
              if was_managed {
                schedule_recovery(
                  app_handle.clone(),
                  recovery.clone(),
                  service_name.clone(),
                  format!("Process error: {}", err),
                );
              }
            }
            // Emit error event to frontend
            let _ = app_handle.emit(
              "service-error",
              ServiceErrorEvent {
                service_name: service_name.clone(),
                error: format!("Process error: {}", err),
                fatal: true,
              },
            );
          }
          CommandEvent::Terminated(payload) => {
            log::warn!(
              "[{}] Process terminated with code: {:?}.",
              service_name,
              payload.code
            );
            let was_managed = process_manager
              .remove_process_if_pid(&service_name, pid)
              .unwrap_or(false);
            if was_managed {
              schedule_recovery(
                app_handle.clone(),
                recovery.clone(),
                service_name.clone(),
                "Port forward stopped unexpectedly".to_string(),
              );
              let _ = app_handle.emit(
                "service-error",
                ServiceErrorEvent {
                  service_name: service_name.clone(),
                  error: "Port forward stopped unexpectedly".to_string(),
                  fatal: true,
                },
              );
            }
          }
          _ => {}
        }
      }
    });

    Ok(format!(
      "{} SSH port forwarding started with PID: {}",
      config.name, pid
    ))
  }

  pub fn stop_port_forward(&self, service_name: &str) -> Result<String> {
    let recovery_was_active = self.recovery.cancel(service_name)?;
    let pid = self.process_manager.remove_process(service_name)?;
    if pid.is_none() && recovery_was_active {
      self.last_active.remove(service_name)?;
      return Ok(format!("Stopped {} pending reconnect", service_name));
    }
    let pid = pid.ok_or_else(|| {
      AppError::NotFound(format!("{} port forwarding is not running", service_name))
    })?;

    self.last_active.remove(service_name)?;

    log::info!("[{}] Stopping port forward (PID: {})", service_name, pid);

    ProcessManager::kill_process(pid)?;

    log::info!("[{}] Port forward stopped successfully", service_name);

    Ok(format!(
      "Stopped {} port forwarding (PID: {})",
      service_name, pid
    ))
  }

  pub fn get_running_services(&self) -> Result<Vec<String>> {
    self.verify_port_forwards()?;
    self.process_manager.get_running_services()
  }

  pub fn cleanup_all_port_forwards(&self) -> Result<()> {
    self.recovery.cancel_all()?;
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
        log::error!(
          "[{}] Port forward process (PID: {}) died unexpectedly. Process is no longer running.",
          service_name,
          pid
        );
        // Clean up dead process
        let _ = self.process_manager.remove_process(&service_name);
        schedule_recovery(
          self.app_handle.clone(),
          self.recovery.clone(),
          service_name.clone(),
          "Port forward process is no longer running".to_string(),
        );
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
    // Single ps aux call to detect all running processes
    let running = self.process_detector.detect_running_processes(&configs)?;
    let mut detected_services = Vec::new();

    for (name, _pid) in &running {
      if !self.process_manager.contains_process(name)? {
        detected_services.push(name.clone());
      }
    }

    Ok(detected_services)
  }

  pub fn sync_with_existing_processes(&self) -> Result<Vec<String>> {
    let configs = self.config_cache.get_configs()?;
    // Single ps aux call to detect all running processes
    let running = self.process_detector.detect_running_processes(&configs)?;
    let mut synced_services = Vec::new();

    for (name, pid) in running {
      if !self.process_manager.contains_process(&name)? {
        if let Some(config) = configs.iter().find(|c| c.name == name) {
          self
            .process_manager
            .add_process(name.clone(), pid, config.clone())?;
          synced_services.push(name);
        }
      }
    }

    Ok(synced_services)
  }
}

fn schedule_recovery(
  app_handle: tauri::AppHandle,
  recovery: RecoveryCoordinator,
  service_name: String,
  error: String,
) {
  tauri::async_runtime::spawn(async move {
    if let Err(recovery_error) =
      recover_port_forward(app_handle, recovery, service_name.clone(), error).await
    {
      log::error!(
        "[{}] Automatic reconnect failed: {}",
        service_name,
        recovery_error
      );
    }
  });
}

#[allow(clippy::too_many_lines)]
async fn recover_port_forward(
  app_handle: tauri::AppHandle,
  recovery: RecoveryCoordinator,
  service_name: String,
  initial_error: String,
) -> Result<()> {
  let port_forward_service = app_handle.state::<PortForwardService>();
  let recovery_settings = port_forward_service
    .get_configs()?
    .into_iter()
    .find(|config| config.name == service_name)
    .and_then(|config| config.recovery)
    .unwrap_or_default();
  if !recovery_settings.reconnect.enabled {
    log::info!("[{}] Automatic reconnect is disabled", service_name);
    return Ok(());
  }

  let Some(ticket) = recovery.begin(&service_name)? else {
    log::debug!("[{}] Recovery is already in progress", service_name);
    return Ok(());
  };

  let mut error = initial_error;
  let mut first_attempt = true;
  loop {
    let Some(attempt) = recovery.next_attempt(&ticket, &recovery_settings.reconnect)? else {
      recovery.finish(&ticket)?;
      let max_attempts = recovery_settings.reconnect.max_attempts;
      if max_attempts > 0 {
        let message = format!(
          "Automatic reconnect stopped after {} attempts",
          max_attempts
        );
        log::error!("[{}] {}", service_name, message);
        let _ = app_handle.emit(
          "service-error",
          ServiceErrorEvent {
            service_name,
            error: message,
            fatal: true,
          },
        );
      }
      return Ok(());
    };

    let context = HookContext {
      service_name: &service_name,
      error: &error,
      attempt: attempt.number,
    };
    if first_attempt {
      recovery
        .run_hooks(
          HookEvent::Failure,
          &recovery_settings.hooks.on_failure,
          &context,
        )
        .await;
      first_attempt = false;
    }

    log::warn!(
      "[{}] Reconnecting in {:?} (attempt {})",
      service_name,
      attempt.delay,
      attempt.number
    );
    tokio::time::sleep(attempt.delay).await;
    if !recovery.is_active(&ticket)? {
      return Ok(());
    }

    recovery
      .run_hooks(
        HookEvent::BeforeReconnect,
        &recovery_settings.hooks.before_reconnect,
        &context,
      )
      .await;
    if !recovery.is_active(&ticket)? {
      return Ok(());
    }

    let kubectl_service = app_handle.state::<super::KubectlService>();
    match port_forward_service
      .restart_port_forward_after_failure(kubectl_service.inner(), &service_name)
      .await
    {
      Ok(_) => {
        recovery.finish(&ticket)?;
        schedule_recovered_hook(
          app_handle,
          recovery,
          ticket,
          service_name,
          error,
          attempt.number,
          recovery_settings.reconnect.stable_after_seconds,
          recovery_settings.hooks.on_recovered,
        );
        return Ok(());
      }
      Err(reconnect_error) => {
        error = reconnect_error.to_string();
        log::warn!(
          "[{}] Reconnect attempt {} failed: {}",
          service_name,
          attempt.number,
          error
        );
      }
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn schedule_recovered_hook(
  app_handle: tauri::AppHandle,
  recovery: RecoveryCoordinator,
  ticket: RecoveryTicket,
  service_name: String,
  error: String,
  attempt: u32,
  stable_after_seconds: u64,
  hooks: Vec<easy_kpf_core::types::HookCommand>,
) {
  tauri::async_runtime::spawn(async move {
    tokio::time::sleep(Duration::from_secs(stable_after_seconds)).await;
    match recovery.mark_stable(&ticket) {
      Ok(true) => {
        log::info!(
          "[{}] Port forward remained stable for {} seconds",
          service_name,
          stable_after_seconds
        );
        recovery
          .run_hooks(
            HookEvent::Recovered,
            &hooks,
            &HookContext {
              service_name: &service_name,
              error: &error,
              attempt,
            },
          )
          .await;
        let _ = app_handle.emit(
          "service-recovered",
          ServiceRecoveredEvent {
            service_name,
            attempt,
          },
        );
      }
      Ok(false) => {}
      Err(recovery_error) => {
        log::warn!(
          "[{}] Could not mark port forward recovered: {}",
          service_name,
          recovery_error
        );
      }
    }
  });
}

fn is_fatal_forward_error(error: &str) -> bool {
  let error = error.to_ascii_lowercase();
  error.contains("error forwarding port")
    || error.contains("error creating forwarding stream")
    || error.contains("lost connection to pod")
    || error.contains("administratively prohibited")
}

#[cfg(test)]
mod tests {
  use super::is_fatal_forward_error;

  #[test]
  fn identifies_broken_forward_errors() {
    assert!(is_fatal_forward_error(
      "an error occurred forwarding 8101 -> 5432: error forwarding port 5432 to pod"
    ));
    assert!(is_fatal_forward_error("lost connection to pod"));
    assert!(is_fatal_forward_error(
      "error creating forwarding stream for port 8101 -> 5432"
    ));
  }

  #[test]
  fn ignores_non_fatal_process_output() {
    assert!(!is_fatal_forward_error("Handling connection for 8101"));
    assert!(!is_fatal_forward_error("Forwarding from 127.0.0.1:8101"));
  }
}
