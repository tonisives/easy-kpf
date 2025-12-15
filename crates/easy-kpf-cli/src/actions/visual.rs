use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use easy_kpf_core::Result;

use super::port_forward::{start_port_forward, stop_port_forward};

pub async fn handle_visual_mode(app: &mut App, key: KeyEvent) -> Result<()> {
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

  app.set_status(format!(
    "Toggled {} (started {}, stopped {})",
    started + stopped,
    started,
    stopped
  ));
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
