use crate::types::PortForwardConfig;

pub struct KubectlCommandBuilder {
  kubectl_path: String,
  kubeconfig_path: Option<String>,
}

impl KubectlCommandBuilder {
  pub fn new(kubectl_path: String, kubeconfig_path: Option<String>) -> Self {
    Self {
      kubectl_path,
      kubeconfig_path,
    }
  }

  pub fn build_port_forward_command(
    &self,
    config: &PortForwardConfig,
  ) -> (String, Vec<String>, Vec<(String, String)>) {
    let mut args = vec![];

    // Add context flag only if context is not empty
    if !config.context.is_empty() {
      args.extend_from_slice(&["--context".to_string(), config.context.clone()]);
    }

    args.extend_from_slice(&[
      "-n".to_string(),
      config.namespace.clone(),
      "port-forward".to_string(),
      config.service.clone(),
    ]);

    // Add --address flag if local interface is specified
    if let Some(ref interface) = config.local_interface {
      args.extend_from_slice(&["--address".to_string(), interface.clone()]);
    }

    // Add port mappings
    args.extend(config.ports.clone());

    let mut env_vars = Vec::new();
    if let Some(ref kubeconfig) = self.kubeconfig_path {
      env_vars.push(("KUBECONFIG".to_string(), kubeconfig.clone()));
    }

    (self.kubectl_path.clone(), args, env_vars)
  }
}

pub struct SshCommandBuilder;

impl SshCommandBuilder {
  pub fn new() -> Self {
    Self
  }

  pub fn build_port_forward_command(&self, config: &PortForwardConfig) -> (String, Vec<String>) {
    let mut args = vec![
      "-N".to_string(), // Don't execute remote command
      "-o".to_string(),
      "BatchMode=yes".to_string(), // Don't prompt for passwords
      "-o".to_string(),
      "StrictHostKeyChecking=no".to_string(), // Don't prompt for host key verification
      "-o".to_string(),
      "ConnectTimeout=10".to_string(), // 10 second connection timeout
      "-o".to_string(),
      "ServerAliveInterval=60".to_string(), // Keep connection alive
      "-o".to_string(),
      "ServerAliveCountMax=3".to_string(), // Max keep-alive attempts
    ];

    // Add port forwarding arguments
    let port_mapper = SshPortMapper::new();
    let forward_args =
      port_mapper.build_port_mappings(&config.ports, config.local_interface.as_deref());

    for forward_arg in forward_args {
      args.extend_from_slice(&["-L".to_string(), forward_arg]);
    }

    args.push(config.service.clone());

    ("ssh".to_string(), args)
  }
}

impl Default for SshCommandBuilder {
  fn default() -> Self {
    Self::new()
  }
}

pub struct SshPortMapper;

impl SshPortMapper {
  pub fn new() -> Self {
    Self
  }

  pub fn build_port_mappings(
    &self,
    ports: &[String],
    local_interface: Option<&str>,
  ) -> Vec<String> {
    let bind_interface = local_interface.unwrap_or("127.0.0.1");

    ports
      .iter()
      .map(|port_mapping| self.format_port_mapping(port_mapping, bind_interface))
      .collect()
  }

  fn format_port_mapping(&self, port_mapping: &str, bind_interface: &str) -> String {
    let parts: Vec<&str> = port_mapping.split(':').collect();

    match parts.len() {
      1 => {
        // Single port means same port for local and remote
        format!("{}:{}:localhost:{}", bind_interface, parts[0], parts[0])
      }
      2 => {
        // "local:remote" format
        format!("{}:{}:localhost:{}", bind_interface, parts[0], parts[1])
      }
      _ => {
        // Already in correct format or custom format
        format!("{}:{}", bind_interface, port_mapping)
      }
    }
  }
}

impl Default for SshPortMapper {
  fn default() -> Self {
    Self::new()
  }
}
