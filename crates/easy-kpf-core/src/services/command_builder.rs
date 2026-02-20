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

  /// Build PATH environment variable that includes common locations for credential plugins.
  /// macOS apps launched from Finder don't inherit shell PATH, so we need to construct one.
  fn build_path_env() -> String {
    let mut paths = Vec::new();

    // Start with current PATH if available
    if let Ok(current_path) = std::env::var("PATH") {
      paths.push(current_path);
    }

    // Add common locations for credential plugins (gke-gcloud-auth-plugin, aws-iam-authenticator, etc.)
    let additional_paths = [
      "/usr/local/bin",
      "/opt/homebrew/bin",
      "/opt/homebrew/sbin",
      "/usr/bin",
      "/bin",
      "/usr/sbin",
      "/sbin",
    ];

    for path in additional_paths {
      if std::path::Path::new(path).exists() {
        paths.push(path.to_string());
      }
    }

    // Add user-specific paths
    if let Ok(home) = std::env::var("HOME") {
      let user_paths = [
        format!("{}/.local/bin", home),
        format!("{}/bin", home),
        format!("{}/google-cloud-sdk/bin", home),
        format!("{}/.krew/bin", home),
      ];
      for path in user_paths {
        if std::path::Path::new(&path).exists() {
          paths.push(path);
        }
      }
    }

    paths.join(":")
  }

  /// Get environment variables needed for credential plugins to work.
  fn get_credential_env_vars() -> Vec<(String, String)> {
    let mut env_vars = Vec::new();

    // Pass through PATH so credential plugins can be found
    env_vars.push(("PATH".to_string(), Self::build_path_env()));

    // Pass through HOME - needed by many credential plugins
    if let Ok(home) = std::env::var("HOME") {
      env_vars.push(("HOME".to_string(), home));
    }

    // Pass through cloud provider credentials if set
    let passthrough_vars = [
      // Google Cloud
      "GOOGLE_APPLICATION_CREDENTIALS",
      "CLOUDSDK_CONFIG",
      "CLOUDSDK_ACTIVE_CONFIG_NAME",
      // AWS
      "AWS_PROFILE",
      "AWS_DEFAULT_PROFILE",
      "AWS_CONFIG_FILE",
      "AWS_SHARED_CREDENTIALS_FILE",
      "AWS_ACCESS_KEY_ID",
      "AWS_SECRET_ACCESS_KEY",
      "AWS_SESSION_TOKEN",
      "AWS_REGION",
      "AWS_DEFAULT_REGION",
      // Azure
      "AZURE_CONFIG_DIR",
      // Generic
      "USER",
      "SHELL",
    ];

    for var in passthrough_vars {
      if let Ok(value) = std::env::var(var) {
        env_vars.push((var.to_string(), value));
      }
    }

    env_vars
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
      // Strip port suffix if present (e.g., "127.0.0.2:5335" -> "127.0.0.2")
      let ip = match interface.rsplit_once(':') {
        Some((ip, port)) if port.parse::<u16>().is_ok() => ip.to_string(),
        _ => interface.clone(),
      };
      args.extend_from_slice(&["--address".to_string(), ip]);
    }

    // Add port mappings
    args.extend(config.ports.clone());

    // Get environment variables for credential plugins
    let mut env_vars = Self::get_credential_env_vars();

    // Add KUBECONFIG if specified
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
    // Parse interface - may contain "ip:port" (e.g., "127.0.0.2:5335") or just "ip"
    let (bind_ip, bind_port_override) = match local_interface {
      Some(iface) => match iface.rsplit_once(':') {
        Some((ip, port)) if port.parse::<u16>().is_ok() => (ip, Some(port)),
        _ => (iface, None),
      },
      None => ("127.0.0.1", None),
    };

    ports
      .iter()
      .map(|port_mapping| self.format_port_mapping(port_mapping, bind_ip, bind_port_override))
      .collect()
  }

  fn format_port_mapping(
    &self,
    port_mapping: &str,
    bind_ip: &str,
    bind_port_override: Option<&str>,
  ) -> String {
    let parts: Vec<&str> = port_mapping.split(':').collect();

    match parts.len() {
      1 => {
        // Single port - use override as local port if present, otherwise same port
        let local_port = bind_port_override.unwrap_or(parts[0]);
        format!("{}:{}:localhost:{}", bind_ip, local_port, parts[0])
      }
      2 => {
        // "local:remote" format - use override as local port if present
        let local_port = bind_port_override.unwrap_or(parts[0]);
        format!("{}:{}:localhost:{}", bind_ip, local_port, parts[1])
      }
      _ => {
        // Already in correct format or custom format
        format!("{}:{}", bind_ip, port_mapping)
      }
    }
  }
}

impl Default for SshPortMapper {
  fn default() -> Self {
    Self::new()
  }
}
