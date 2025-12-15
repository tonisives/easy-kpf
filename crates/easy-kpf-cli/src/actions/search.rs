use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use easy_kpf_core::Result;

pub fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<()> {
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
