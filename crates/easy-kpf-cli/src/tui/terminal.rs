use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> io::Result<Tui> {
  enable_raw_mode()?;
  execute!(stdout(), EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout());
  Terminal::new(backend)
}

pub fn restore_terminal() -> io::Result<()> {
  disable_raw_mode()?;
  execute!(stdout(), LeaveAlternateScreen)?;
  Ok(())
}
