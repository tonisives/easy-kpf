use crate::app::{App, ConfirmAction, Mode, Panel};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use easy_kpf_core::Result;
use std::process::Command;

use super::port_forward::{start_port_forward, stop_port_forward};

pub async fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<()> {
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
