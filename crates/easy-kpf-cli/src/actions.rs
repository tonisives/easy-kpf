use crate::app::{App, ConfirmAction, LogEntry, Mode, Panel};
use crate::executor::TokioCommandExecutor;
use crate::vim::VimTransition;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use easy_kpf_core::{
  services::{KubectlCommandBuilder, SshCommandBuilder},
  traits::ProcessEvent,
  CommandExecutor, ForwardType, PortForwardConfig, Result,
};
use std::process::Command;
use tui_textarea::Input;

pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
  match app.mode {
    Mode::Normal => handle_normal_mode(app, key).await,
    Mode::Visual => handle_visual_mode(app, key).await,
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
    KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('.') => {
      if app.active_panel == Panel::ServiceList {
        app.move_selection(1);
      }
    }
    KeyCode::Char('k') | KeyCode::Up | KeyCode::Char(',') => {
      if app.active_panel == Panel::ServiceList {
        app.move_selection(-1);
      }
    }
    // Jump to start/end
    KeyCode::Char('<') => {
      if app.active_panel == Panel::ServiceList {
        app.select_first();
      }
    }
    KeyCode::Char('>') => {
      if app.active_panel == Panel::ServiceList {
        app.select_last();
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

    // Enter visual mode
    KeyCode::Char('v') => {
      if app.active_panel == Panel::ServiceList && !app.visual_order.is_empty() {
        app.enter_visual_mode();
      }
    }

    // Toggle all - show confirmation
    KeyCode::Char('a') => {
      if app.running_services.is_empty() && !app.configs.is_empty() {
        // Starting all - show confirmation
        app.confirm_action = Some(ConfirmAction::StartAll);
        app.mode = Mode::Confirm;
      } else if !app.running_services.is_empty() {
        // Stopping all - show confirmation
        app.confirm_action = Some(ConfirmAction::StopAll);
        app.mode = Mode::Confirm;
      }
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

async fn handle_visual_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Exit visual mode
    KeyCode::Esc | KeyCode::Char('v') => {
      app.exit_visual_mode();
    }

    // Quit
    KeyCode::Char('q') => {
      app.exit_visual_mode();
      app.should_quit = true;
    }
    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.exit_visual_mode();
      app.should_quit = true;
    }

    // Navigation - extends selection
    KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('.') => {
      app.move_selection(1);
    }
    KeyCode::Char('k') | KeyCode::Up | KeyCode::Char(',') => {
      app.move_selection(-1);
    }
    // Jump to start/end
    KeyCode::Char('<') => {
      app.select_first();
    }
    KeyCode::Char('>') => {
      app.select_last();
    }

    // Toggle all selected services
    KeyCode::Char(' ') | KeyCode::Enter => {
      toggle_visual_selection(app).await?;
      app.exit_visual_mode();
    }

    // Start all selected
    KeyCode::Char('s') => {
      start_visual_selection(app).await?;
      app.exit_visual_mode();
    }

    // Stop all selected
    KeyCode::Char('x') => {
      stop_visual_selection(app)?;
      app.exit_visual_mode();
    }

    // Help
    KeyCode::Char('?') => {
      app.mode = Mode::Help;
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
  // Check if in command mode (: commands like :w, :q)
  if app.command_mode {
    return handle_command_mode(app, key);
  }

  // Check if in vim edit mode for the current field
  if app.is_vim_edit_mode() {
    return handle_vim_edit_mode(app, key);
  }

  // Check if suggestions panel is focused
  if app.autocomplete.focused {
    return handle_suggestions_focused(app, key);
  }

  // Check if in typing mode - then all keys go to the field
  if app.autocomplete.typing {
    return handle_typing_mode(app, key);
  }

  // Default mode: j/k navigate suggestions, i enters typing mode
  match key.code {
    KeyCode::Esc => {
      // Show confirmation dialog before canceling
      let current_mode = app.mode;
      app.confirm_action = Some(ConfirmAction::CancelEdit(current_mode));
      app.mode = Mode::Confirm;
    }
    // : to enter command mode (vim-style :w, :q)
    KeyCode::Char(':') => {
      app.enter_command_mode();
    }
    // i to enter typing mode (vim-style insert)
    KeyCode::Char('i') => {
      app.enter_typing_mode();
    }
    // e to enter vim edit mode for the current field
    KeyCode::Char('e') => {
      app.enter_vim_edit_mode();
    }
    // j/Down to go to next suggestion
    KeyCode::Char('j') | KeyCode::Down => {
      app.autocomplete_next();
    }
    // k/Up to go to previous suggestion
    KeyCode::Char('k') | KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Enter accepts current suggestion
    KeyCode::Enter => {
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
    }
    // Ctrl+s to save
    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.save_edit()?;
      app.set_status("Configuration saved");
    }
    KeyCode::Tab => {
      // Accept current suggestion if any, then move to next field
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
      app.next_edit_field();
    }
    KeyCode::BackTab => {
      app.prev_edit_field();
    }
    // Ctrl+o to reload suggestions (mnemonic: "options")
    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.load_autocomplete();
    }
    _ => {}
  }
  Ok(())
}

fn handle_typing_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc exits typing mode
    KeyCode::Esc => {
      app.exit_typing_mode();
    }
    // Ctrl+s to save
    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.save_edit()?;
      app.set_status("Configuration saved");
    }
    KeyCode::Tab => {
      app.exit_typing_mode();
      app.next_edit_field();
    }
    KeyCode::BackTab => {
      app.exit_typing_mode();
      app.prev_edit_field();
    }
    // Up/Down arrow keys navigate suggestions while in typing mode
    KeyCode::Down => {
      app.autocomplete_next();
    }
    KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Left/Right arrow keys move cursor within the field
    KeyCode::Left => {
      app.cursor_left();
    }
    KeyCode::Right => {
      app.cursor_right();
    }
    KeyCode::Backspace => {
      app.delete_char_before_cursor();
      // Mark name as manually edited when user types/deletes in name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
    }
    // Ctrl+o to reload suggestions
    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.load_autocomplete();
    }
    // Enter accepts current suggestion
    KeyCode::Enter => {
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
    }
    // : at start of empty field or Ctrl+: enters command mode
    KeyCode::Char(':') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.exit_typing_mode();
      app.enter_command_mode();
    }
    KeyCode::Char(c) => {
      app.insert_char(c);
      // Mark name as manually edited when user types in name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
    }
    _ => {}
  }
  Ok(())
}

fn handle_vim_edit_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  use crate::vim::VimMode;

  let input: Input = key.into();

  // Get mutable references to both textarea and vim_state
  let (textarea, vim_state) = match (&mut app.vim_textarea, &mut app.vim_state) {
    (Some(ta), Some(vs)) => (ta, vs),
    _ => {
      app.exit_vim_edit_mode();
      return Ok(());
    }
  };

  let transition = vim_state.transition(input, textarea);

  match transition {
    VimTransition::Exit => {
      // Mark name as edited if this was the name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
      app.exit_vim_edit_mode();
    }
    VimTransition::Mode(mode) if vim_state.mode != mode => {
      textarea.set_cursor_style(mode.cursor_style());
      vim_state.mode = mode;
      // Mark as edited when text changes (entering insert mode means editing)
      if app.edit_field_index == 0 && mode == VimMode::Insert {
        app.name_manually_edited = true;
      }
    }
    VimTransition::Nop | VimTransition::Mode(_) => {}
    VimTransition::Pending(_input) => {
      // Store pending input for multi-key sequences (like gg)
      // For now we ignore pending - single line doesn't need gg/G
    }
  }

  Ok(())
}

fn handle_suggestions_focused(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc goes back to field editing
    KeyCode::Esc => {
      app.unfocus_suggestions();
    }
    // j/Down to go to next suggestion
    KeyCode::Char('j') | KeyCode::Down => {
      app.autocomplete_next();
    }
    // k/Up to go to previous suggestion
    KeyCode::Char('k') | KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Enter to accept suggestion and go back to field
    KeyCode::Enter => {
      app.accept_autocomplete();
      app.unfocus_suggestions();
    }
    // Tab to accept and move to next field
    KeyCode::Tab => {
      app.accept_autocomplete();
      app.next_edit_field();
    }
    _ => {}
  }
  Ok(())
}

fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc exits command mode
    KeyCode::Esc => {
      app.exit_command_mode();
    }
    // Enter executes the command
    KeyCode::Enter => {
      let cmd = app.command_buffer.trim().to_lowercase();
      app.exit_command_mode();

      match cmd.as_str() {
        "w" | "write" => {
          // Save and stay in edit mode
          app.save_edit()?;
          app.set_status("Configuration saved");
        }
        "q" | "quit" => {
          // Quit without saving - show confirmation
          let current_mode = app.mode;
          app.confirm_action = Some(ConfirmAction::CancelEdit(current_mode));
          app.mode = Mode::Confirm;
        }
        "wq" | "x" => {
          // Save and quit
          app.save_edit()?;
          app.set_status("Configuration saved");
        }
        "q!" => {
          // Force quit without confirmation
          app.cancel_edit();
        }
        _ => {
          app.set_status(format!("Unknown command: {}", cmd));
        }
      }
    }
    // Backspace deletes from command buffer
    KeyCode::Backspace => {
      if app.command_buffer.is_empty() {
        // Exit command mode if backspace on empty buffer
        app.exit_command_mode();
      } else {
        app.command_buffer.pop();
      }
    }
    // Type characters into command buffer
    KeyCode::Char(c) => {
      app.command_buffer.push(c);
    }
    _ => {}
  }
  Ok(())
}

async fn handle_confirm_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc | KeyCode::Char('n') => {
      // For CancelEdit, go back to edit mode; for others, go to Normal
      if let Some(ConfirmAction::CancelEdit(prev_mode)) = app.confirm_action.take() {
        app.mode = prev_mode;
      } else {
        app.confirm_action = None;
        app.mode = Mode::Normal;
      }
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
          ConfirmAction::StartAll => {
            let configs: Vec<_> = app.configs.clone();
            for config in configs {
              start_port_forward(app, config).await?;
            }
            app.set_status("Started all port forwards");
          }
          ConfirmAction::StopAll => {
            let names: Vec<String> = app.running_services.keys().cloned().collect();
            for name in names {
              stop_port_forward(app, &name)?;
            }
            app.set_status("Stopped all port forwards");
          }
          ConfirmAction::CancelEdit(_) => {
            // User confirmed they want to discard changes
            app.cancel_edit();
            return Ok(());
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

/// Toggle all services in the visual selection (start stopped ones, stop running ones)
async fn toggle_visual_selection(app: &mut App) -> Result<()> {
  let configs = app.get_visual_selection_configs();
  if configs.is_empty() {
    return Ok(());
  }

  let mut started = 0;
  let mut stopped = 0;

  for config in configs {
    let name = config.name.clone();
    if app.running_services.contains_key(&name) {
      stop_port_forward(app, &name)?;
      stopped += 1;
    } else {
      start_port_forward(app, config).await?;
      started += 1;
    }
  }

  app.set_status(format!("Toggled {} (started {}, stopped {})", started + stopped, started, stopped));
  Ok(())
}

/// Start all stopped services in the visual selection
async fn start_visual_selection(app: &mut App) -> Result<()> {
  let configs = app.get_visual_selection_configs();
  if configs.is_empty() {
    return Ok(());
  }

  let mut started = 0;

  for config in configs {
    let name = config.name.clone();
    if !app.running_services.contains_key(&name) {
      start_port_forward(app, config).await?;
      started += 1;
    }
  }

  if started > 0 {
    app.set_status(format!("Started {} services", started));
  } else {
    app.set_status("All selected services already running");
  }
  Ok(())
}

/// Stop all running services in the visual selection
fn stop_visual_selection(app: &mut App) -> Result<()> {
  let configs = app.get_visual_selection_configs();
  if configs.is_empty() {
    return Ok(());
  }

  let mut stopped = 0;

  for config in configs {
    let name = config.name.clone();
    if app.running_services.contains_key(&name) {
      stop_port_forward(app, &name)?;
      stopped += 1;
    }
  }

  if stopped > 0 {
    app.set_status(format!("Stopped {} services", stopped));
  } else {
    app.set_status("No running services in selection");
  }
  Ok(())
}

/// Extract local port from a port mapping string (e.g., "8080:80" -> 8080, "8080" -> 8080)
fn parse_local_port(port_mapping: &str) -> Option<u16> {
  let parts: Vec<&str> = port_mapping.split(':').collect();
  parts.first()?.parse().ok()
}

/// Check if running as root/sudo
fn is_running_as_root() -> bool {
  // SAFETY: getuid() is always safe to call
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

async fn start_port_forward(app: &mut App, config: PortForwardConfig) -> Result<()> {
  let name = config.name.clone();
  let log_sender = app.log_sender.clone();

  // Check for privileged ports when not running as root
  let privileged_ports = find_privileged_ports(&config);
  if !privileged_ports.is_empty() && !is_running_as_root() {
    let ports_str = privileged_ports
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<_>>()
      .join(", ");
    let warning = format!(
      "Warning: Port(s) {} require root privileges. Run with sudo or use ports >= 1024.",
      ports_str
    );
    app.set_status(warning.clone());
    let _ = log_sender
      .send((
        name.clone(),
        LogEntry {
          line: warning,
          is_stderr: true,
        },
      ))
      .await;
    return Ok(());
  }

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
