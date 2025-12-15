mod actions;
mod app;
mod components;
mod executor;
mod tui;

use app::App;
use std::time::Duration;
use tui::{restore_terminal, setup_terminal, Event, EventHandler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Initialize logging
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

  // Setup terminal
  let mut terminal = setup_terminal()?;

  // Create app state
  let mut app = match App::new() {
    Ok(app) => app,
    Err(e) => {
      restore_terminal()?;
      eprintln!("Failed to initialize app: {}", e);
      std::process::exit(1);
    }
  };

  // Take the log receiver from app
  let mut log_receiver = app.log_receiver.take();

  // Create event handler
  let mut events = EventHandler::new(Duration::from_millis(100));

  // Main loop
  let result = run_app(&mut terminal, &mut app, &mut events, &mut log_receiver).await;

  // Restore terminal
  restore_terminal()?;

  if let Err(e) = result {
    eprintln!("Error: {}", e);
    std::process::exit(1);
  }

  Ok(())
}

async fn run_app(
  terminal: &mut tui::Tui,
  app: &mut App,
  events: &mut EventHandler,
  log_receiver: &mut Option<tokio::sync::mpsc::Receiver<(String, app::LogEntry)>>,
) -> anyhow::Result<()> {
  loop {
    // Draw UI
    terminal.draw(|frame| tui::ui::draw(frame, app))?;

    // Handle events
    tokio::select! {
      // Keyboard/tick events
      Some(event) = events.next() => {
        match event {
          Event::Key(key) => {
            // Handle $EDITOR specially - need to restore terminal
            if key.code == crossterm::event::KeyCode::Char('E')
              && app.mode == app::Mode::Normal
            {
              restore_terminal()?;
              actions::handle_key(app, key).await?;
              *terminal = setup_terminal()?;
            } else {
              actions::handle_key(app, key).await?;
            }
          }
          Event::Tick => {
            // Clear old status messages
            // Could add timeout logic here
          }
          Event::Resize(_, _) => {
            // Terminal will handle resize automatically
          }
        }
      }

      // Log messages from running processes
      Some((service_key, entry)) = async {
        if let Some(rx) = log_receiver {
          rx.recv().await
        } else {
          None
        }
      } => {
        app.append_log(&service_key, entry);

        // Check if process terminated
        // This is handled in the log message itself
      }
    }

    // Check for quit
    if app.should_quit {
      break;
    }

    // Check for terminated processes and update state
    let terminated: Vec<String> = app
      .running_services
      .iter()
      .filter(|(_, &pid)| !is_process_running(pid))
      .map(|(k, _)| k.clone())
      .collect();

    for key in terminated {
      app.running_services.remove(&key);
      let _ = app.process_manager.remove_process(&key);
    }
  }

  Ok(())
}

fn is_process_running(pid: u32) -> bool {
  use std::process::Command;
  Command::new("kill")
    .args(["-0", &pid.to_string()])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}
