use crate::app::{App, LogEntry};
use crate::executor::TokioCommandExecutor;
use easy_kpf_core::{
  services::{KubectlCommandBuilder, SshCommandBuilder},
  traits::ProcessEvent,
  CommandExecutor, ForwardType, PortForwardConfig, Result,
};
use std::process::Command;
use tokio::sync::mpsc;

/// Extract local port from a port mapping string (e.g., "8080:80" -> 8080, "8080" -> 8080)
fn parse_local_port(port_mapping: &str) -> Option<u16> {
  let parts: Vec<&str> = port_mapping.split(':').collect();
  parts.first()?.parse().ok()
}

/// Check if running as root/sudo
#[allow(unsafe_code)]
fn is_running_as_root() -> bool {
  // SAFETY: libc::getuid() is always safe to call - it simply returns
  // the real user ID of the calling process with no side effects.
  unsafe { libc::getuid() == 0 }
}

/// Find any privileged ports (< 1024) in the config
fn find_privileged_ports(config: &PortForwardConfig) -> Vec<u16> {
  config
    .ports
    .iter()
    .filter_map(|p| parse_local_port(p))
    .filter(|&port| port < 1024)
    .collect()
}

/// Check for privileged ports and warn if not running as root
async fn check_privileged_ports(app: &mut App, config: &PortForwardConfig, name: &str) -> bool {
  let privileged_ports = find_privileged_ports(config);
  if privileged_ports.is_empty() || is_running_as_root() {
    return true; // OK to proceed
  }

  let ports_str = privileged_ports
    .iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<_>>()
    .join(", ");
  let warning = format!(
    "Warning: Port(s) {} require root privileges. Run with sudo or use ports >= 1024.",
    ports_str
  );

  app.set_status(warning.clone());
  let _ = app
    .log_sender
    .send((
      name.to_string(),
      LogEntry {
        line: warning,
        is_stderr: true,
      },
    ))
    .await;

  false // Cannot proceed
}

/// Build the command for the port forward based on forward type
fn build_command(
  app: &App,
  config: &PortForwardConfig,
) -> (String, Vec<String>, Vec<(String, String)>) {
  match config.forward_type {
    ForwardType::Ssh => {
      let builder = SshCommandBuilder::new();
      let cmd = builder.build_port_forward_command(config);
      (cmd.0, cmd.1, Vec::new())
    }
    ForwardType::Kubectl => {
      let kubectl_path = app
        .config_service
        .load_kubectl_path()
        .unwrap_or_else(|_| "kubectl".to_string());
      let kubeconfig = app.config_service.load_kubeconfig_path().ok().flatten();

      let builder = KubectlCommandBuilder::new(kubectl_path, kubeconfig);
      builder.build_port_forward_command(config)
    }
  }
}

/// Spawn a background task to read process output and send to log channel
fn spawn_output_reader(
  service_name: String,
  log_sender: mpsc::Sender<(String, LogEntry)>,
  mut rx: mpsc::Receiver<ProcessEvent>,
) {
  tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
      let should_break = handle_process_event(&service_name, &log_sender, event).await;
      if should_break {
        break;
      }
    }
  });
}

/// Handle a single process event, returns true if loop should break
async fn handle_process_event(
  service_name: &str,
  log_sender: &mpsc::Sender<(String, LogEntry)>,
  event: ProcessEvent,
) -> bool {
  match event {
    ProcessEvent::Stdout(data) => {
      let line = String::from_utf8_lossy(&data).to_string();
      let _ = log_sender
        .send((
          service_name.to_string(),
          LogEntry {
            line,
            is_stderr: false,
          },
        ))
        .await;
      false
    }
    ProcessEvent::Stderr(data) => {
      let line = String::from_utf8_lossy(&data).to_string();
      let _ = log_sender
        .send((
          service_name.to_string(),
          LogEntry {
            line,
            is_stderr: true,
          },
        ))
        .await;
      false
    }
    ProcessEvent::Terminated { code } => {
      let msg = format!("Process exited with code {:?}", code);
      let _ = log_sender
        .send((
          service_name.to_string(),
          LogEntry {
            line: msg,
            is_stderr: true,
          },
        ))
        .await;
      true // Break the loop
    }
    ProcessEvent::Error(e) => {
      let _ = log_sender
        .send((
          service_name.to_string(),
          LogEntry {
            line: e,
            is_stderr: true,
          },
        ))
        .await;
      false
    }
  }
}

pub async fn start_port_forward(app: &mut App, config: PortForwardConfig) -> Result<()> {
  let name = config.name.clone();

  // Check for privileged ports when not running as root
  if !check_privileged_ports(app, &config, &name).await {
    return Ok(());
  }

  // Build and spawn the command
  let (program, args, env) = build_command(app, &config);
  let executor = TokioCommandExecutor::new();
  let (handle, rx) = executor.spawn(&program, &args, &env).await?;

  // Register the process
  let pid = handle.pid;
  app.running_services.insert(name.clone(), pid);
  let _ = app.process_manager.add_process(name.clone(), pid, config);
  app.set_status(format!("Started {} (pid {})", name, pid));

  // Spawn output reader
  spawn_output_reader(name, app.log_sender.clone(), rx);

  Ok(())
}

pub fn stop_port_forward(app: &mut App, name: &str) -> Result<()> {
  if let Some(pid) = app.running_services.remove(name) {
    // Kill the process
    let _ = Command::new("kill").arg(pid.to_string()).output();

    // Update process manager state
    let _ = app.process_manager.remove_process(name);
  }

  Ok(())
}
