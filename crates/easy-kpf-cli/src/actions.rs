use crate::app::{App, ConfirmAction, LogEntry, Mode, Panel};
use crate::executor::TokioCommandExecutor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use easy_kpf_core::{
  services::{KubectlCommandBuilder, SshCommandBuilder},
  traits::ProcessEvent,
  CommandExecutor, ForwardType, PortForwardConfig, Result,
};
use std::process::Command;

pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
  match app.mode {
    Mode::Normal => handle_normal_mode(app, key).await,
    Mode::Search => handle_search_mode(app, key),
    Mode::Help => handle_help_mode(app, key),
    Mode::Edit | Mode::Create => handle_edit_mode(app, key),
    Mode::Confirm => handle_confirm_mode(app, key).await,
  }
}

async fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Quit
    KeyCode::Char('q') => app.should_quit = true,
    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.should_quit = true;
    }

    // Navigation
    KeyCode::Char('j') | KeyCode::Down => {
      if app.active_panel == Panel::ServiceList {
        app.move_selection(1);
      }
    }
    KeyCode::Char('k') | KeyCode::Up => {
      if app.active_panel == Panel::ServiceList {
        app.move_selection(-1);
      }
    }
    KeyCode::Char('h') | KeyCode::Left => {
      app.active_panel = Panel::ServiceList;
    }
    KeyCode::Char('l') | KeyCode::Right => {
      app.active_panel = Panel::Logs;
    }

    // Toggle port forward
    KeyCode::Char(' ') | KeyCode::Enter => {
      if app.active_panel == Panel::ServiceList {
        toggle_selected(app).await?;
      }
    }

    // Toggle all
    KeyCode::Char('a') => {
      toggle_all(app).await?;
    }

    // New config
    KeyCode::Char('n') => {
      app.enter_create_mode();
    }

    // Edit config
    KeyCode::Char('e') => {
      app.enter_edit_mode();
    }

    // Delete config
    KeyCode::Char('d') | KeyCode::Delete => {
      if let Some(name) = app.selected_name() {
        app.confirm_action = Some(ConfirmAction::Delete(name));
        app.mode = Mode::Confirm;
      }
    }

    // Search
    KeyCode::Char('/') => {
      app.mode = Mode::Search;
      app.search_query.clear();
    }

    // Help
    KeyCode::Char('?') => {
      app.mode = Mode::Help;
    }

    // Refresh
    KeyCode::Char('r') => {
      app.sync_running_services();
      app.set_status("Refreshed process state");
    }

    // Open in $EDITOR
    KeyCode::Char('E') => {
      open_in_editor(app)?;
    }

    _ => {}
  }

  Ok(())
}

fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc => {
      app.mode = Mode::Normal;
      app.search_query.clear();
      app.update_visual_order();
    }
    KeyCode::Enter => {
      app.mode = Mode::Normal;
    }
    KeyCode::Backspace => {
      app.search_query.pop();
      app.update_visual_order();
    }
    KeyCode::Char(c) => {
      app.search_query.push(c);
      app.update_visual_order();
    }
    _ => {}
  }
  Ok(())
}

fn handle_help_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
      app.mode = Mode::Normal;
    }
    _ => {}
  }
  Ok(())
}

fn handle_edit_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc => {
      app.cancel_edit();
    }
    KeyCode::Enter => {
      app.save_edit()?;
      app.set_status("Configuration saved");
    }
    KeyCode::Tab => {
      app.next_edit_field();
    }
    KeyCode::BackTab => {
      app.prev_edit_field();
    }
    KeyCode::Backspace => {
      app.edit_field_value.pop();
    }
    KeyCode::Char(c) => {
      app.edit_field_value.push(c);
    }
    _ => {}
  }
  Ok(())
}

async fn handle_confirm_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc | KeyCode::Char('n') => {
      app.confirm_action = None;
      app.mode = Mode::Normal;
    }
    KeyCode::Char('y') => {
      if let Some(action) = app.confirm_action.take() {
        match action {
          ConfirmAction::Delete(name) => {
            // Stop if running
            if app.running_services.contains_key(&name) {
              stop_port_forward(app, &name)?;
            }
            // Remove from configs
            app.configs.retain(|c| c.name != name);
            app.save_configs()?;
            app.update_visual_order();
            app.set_status(format!("Deleted {}", name));
          }
        }
      }
      app.mode = Mode::Normal;
    }
    _ => {}
  }
  Ok(())
}

async fn toggle_selected(app: &mut App) -> Result<()> {
  let Some(config) = app.selected_config().cloned() else {
    return Ok(());
  };

  let name = config.name.clone();

  if app.running_services.contains_key(&name) {
    stop_port_forward(app, &name)?;
    app.set_status(format!("Stopped {}", name));
  } else {
    start_port_forward(app, config).await?;
  }

  Ok(())
}

async fn toggle_all(app: &mut App) -> Result<()> {
  // If any are running, stop all. Otherwise start all.
  let any_running = !app.running_services.is_empty();

  if any_running {
    let names: Vec<String> = app.running_services.keys().cloned().collect();
    for name in names {
      stop_port_forward(app, &name)?;
    }
    app.set_status("Stopped all port forwards");
  } else {
    let configs: Vec<_> = app.configs.clone();
    for config in configs {
      start_port_forward(app, config).await?;
    }
    app.set_status("Started all port forwards");
  }

  Ok(())
}

async fn start_port_forward(app: &mut App, config: PortForwardConfig) -> Result<()> {
  let name = config.name.clone();
  let log_sender = app.log_sender.clone();

  // Build the command based on forward type
  let (program, args, env) = match config.forward_type {
    ForwardType::Ssh => {
      let builder = SshCommandBuilder::new();
      let cmd = builder.build_port_forward_command(&config);
      (cmd.0, cmd.1, Vec::new())
    }
    ForwardType::Kubectl => {
      // Get kubectl path and kubeconfig from app config
      let kubectl_path = app
        .config_service
        .load_kubectl_path()
        .unwrap_or_else(|_| "kubectl".to_string());
      let kubeconfig = app.config_service.load_kubeconfig_path().ok().flatten();

      let builder = KubectlCommandBuilder::new(kubectl_path, kubeconfig);
      let cmd = builder.build_port_forward_command(&config);
      cmd
    }
  };

  // Spawn the process
  let executor = TokioCommandExecutor::new();
  let env_tuples: Vec<(String, String)> = env;
  let (handle, mut rx) = executor.spawn(&program, &args, &env_tuples).await?;

  let pid = handle.pid;
  app.running_services.insert(name.clone(), pid);

  // Update process manager state
  let _ = app.process_manager.add_process(name.clone(), pid, config);

  app.set_status(format!("Started {} (pid {})", name, pid));

  // Spawn task to read output
  let service_name = name.clone();
  tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
      match event {
        ProcessEvent::Stdout(data) => {
          let line = String::from_utf8_lossy(&data).to_string();
          let _ = log_sender
            .send((
              service_name.clone(),
              LogEntry {
                line,
                is_stderr: false,
              },
            ))
            .await;
        }
        ProcessEvent::Stderr(data) => {
          let line = String::from_utf8_lossy(&data).to_string();
          let _ = log_sender
            .send((
              service_name.clone(),
              LogEntry {
                line,
                is_stderr: true,
              },
            ))
            .await;
        }
        ProcessEvent::Terminated { code } => {
          let msg = format!("Process exited with code {:?}", code);
          let _ = log_sender
            .send((
              service_name.clone(),
              LogEntry {
                line: msg,
                is_stderr: true,
              },
            ))
            .await;
          break;
        }
        ProcessEvent::Error(e) => {
          let _ = log_sender
            .send((
              service_name.clone(),
              LogEntry {
                line: e,
                is_stderr: true,
              },
            ))
            .await;
        }
      }
    }
  });

  Ok(())
}

fn stop_port_forward(app: &mut App, name: &str) -> Result<()> {
  if let Some(pid) = app.running_services.remove(name) {
    // Kill the process
    let _ = Command::new("kill").arg(pid.to_string()).output();

    // Update process manager state
    let _ = app.process_manager.remove_process(name);
  }

  Ok(())
}

fn open_in_editor(app: &mut App) -> Result<()> {
  let config_path = app.get_config_file_path();

  let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

  // We need to restore terminal, run editor, then re-setup
  crate::tui::restore_terminal().ok();

  let status = Command::new(&editor).arg(&config_path).status();

  // Re-setup terminal (caller needs to handle this)
  // For now, just reload configs after editor
  if let Ok(status) = status {
    if status.success() {
      app.load_configs()?;
      app.set_status("Reloaded configuration");
    }
  }

  Ok(())
}
