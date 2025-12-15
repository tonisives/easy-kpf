use crate::app::{App, ConfirmAction, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use easy_kpf_core::Result;

use super::port_forward::{start_port_forward, stop_port_forward};

pub async fn handle_confirm_mode(app: &mut App, key: KeyEvent) -> Result<()> {
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
