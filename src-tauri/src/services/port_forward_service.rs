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

    pub fn update_config(
        &self,
        old_service_key: &str,
        new_config: PortForwardConfig,
    ) -> Result<()> {
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
            self.process_manager
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

        self.start_port_forward_generic(kubectl_service, config)
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
        let shell = self.app_handle.shell();

        let mut args = vec!["-n", &config.namespace, "port-forward", &config.service];
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
        self.process_manager
            .add_process(config.name.clone(), pid, config.clone())?;

        Ok(format!(
            "{} port forwarding started with PID: {}",
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
}
