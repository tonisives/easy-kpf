mod confirm;
mod edit;
mod help;
mod normal;
mod port_forward;
mod search;
mod visual;

use crate::app::{App, Mode};
use crossterm::event::KeyEvent;
use easy_kpf_core::Result;

// Re-export for external use (e.g., from main.rs if needed)
#[allow(unused_imports)]
pub use port_forward::{start_port_forward, stop_port_forward};

pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
  match app.mode {
    Mode::Normal => normal::handle_normal_mode(app, key).await,
    Mode::Visual => visual::handle_visual_mode(app, key).await,
    Mode::Search => search::handle_search_mode(app, key),
    Mode::Help => help::handle_help_mode(app, key),
    Mode::Edit | Mode::Create => edit::handle_edit_mode(app, key),
    Mode::Confirm => confirm::handle_confirm_mode(app, key).await,
  }
}
