use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use easy_kpf_core::Result;

pub fn handle_help_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
      app.mode = Mode::Normal;
    }
    _ => {}
  }
  Ok(())
}
