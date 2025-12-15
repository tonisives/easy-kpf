use ratatui::style::{Color, Modifier, Style};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
  Normal,
  Insert,
  Visual,
  Operator(char),
}

impl VimMode {
  pub fn cursor_style(&self) -> Style {
    let color = match self {
      Self::Normal => Color::Reset,
      Self::Insert => Color::LightBlue,
      Self::Visual => Color::LightYellow,
      Self::Operator(_) => Color::LightGreen,
    };
    Style::default().fg(color).add_modifier(Modifier::REVERSED)
  }
}

pub enum VimTransition {
  Nop,
  Mode(VimMode),
  Pending(Input),
  Exit, // Exit vim mode (Enter in normal mode)
}

pub struct VimState {
  pub mode: VimMode,
  #[allow(dead_code)]
  pending: Input,
}

impl VimState {
  pub fn new() -> Self {
    Self {
      mode: VimMode::Normal,
      pending: Input::default(),
    }
  }

  #[allow(dead_code)]
  fn with_pending(self, pending: Input) -> Self {
    Self {
      mode: self.mode,
      pending,
    }
  }

  pub fn transition(&self, input: Input, textarea: &mut TextArea<'_>) -> VimTransition {
    if input.key == Key::Null {
      return VimTransition::Nop;
    }

    match self.mode {
      VimMode::Normal | VimMode::Visual | VimMode::Operator(_) => {
        match input {
          // Exit vim mode with Enter (in normal mode only)
          Input {
            key: Key::Enter, ..
          } if self.mode == VimMode::Normal => {
            textarea.cancel_selection();
            return VimTransition::Exit;
          }
          // Exit vim mode with Esc (in normal mode only)
          Input { key: Key::Esc, .. } if self.mode == VimMode::Normal => {
            textarea.cancel_selection();
            return VimTransition::Exit;
          }
          // Basic movement
          Input {
            key: Key::Char('h'),
            ..
          }
          | Input {
            key: Key::Left, ..
          } => textarea.move_cursor(CursorMove::Back),
          Input {
            key: Key::Char('l'),
            ..
          }
          | Input {
            key: Key::Right, ..
          } => textarea.move_cursor(CursorMove::Forward),
          // Word movement
          Input {
            key: Key::Char('w'),
            ..
          } => textarea.move_cursor(CursorMove::WordForward),
          Input {
            key: Key::Char('e'),
            ctrl: false,
            ..
          } => {
            textarea.move_cursor(CursorMove::WordEnd);
            if matches!(self.mode, VimMode::Operator(_)) {
              textarea.move_cursor(CursorMove::Forward);
            }
          }
          Input {
            key: Key::Char('b'),
            ctrl: false,
            ..
          } => textarea.move_cursor(CursorMove::WordBack),
          // Line movement
          Input {
            key: Key::Char('^'),
            ..
          }
          | Input {
            key: Key::Char('0'),
            ..
          } => textarea.move_cursor(CursorMove::Head),
          Input {
            key: Key::Char('$'),
            ..
          } => textarea.move_cursor(CursorMove::End),
          // Delete operations
          Input {
            key: Key::Char('D'),
            ..
          } => {
            textarea.delete_line_by_end();
            return VimTransition::Mode(VimMode::Normal);
          }
          Input {
            key: Key::Char('C'),
            ..
          } => {
            textarea.delete_line_by_end();
            textarea.cancel_selection();
            return VimTransition::Mode(VimMode::Insert);
          }
          Input {
            key: Key::Char('x'),
            ..
          } => {
            textarea.delete_next_char();
            return VimTransition::Mode(VimMode::Normal);
          }
          // Paste
          Input {
            key: Key::Char('p'),
            ..
          } => {
            textarea.paste();
            return VimTransition::Mode(VimMode::Normal);
          }
          // Undo/Redo
          Input {
            key: Key::Char('u'),
            ctrl: false,
            ..
          } => {
            textarea.undo();
            return VimTransition::Mode(VimMode::Normal);
          }
          Input {
            key: Key::Char('r'),
            ctrl: true,
            ..
          } => {
            textarea.redo();
            return VimTransition::Mode(VimMode::Normal);
          }
          // Enter insert mode
          Input {
            key: Key::Char('i'),
            ..
          } => {
            textarea.cancel_selection();
            return VimTransition::Mode(VimMode::Insert);
          }
          Input {
            key: Key::Char('a'),
            ..
          } => {
            textarea.cancel_selection();
            textarea.move_cursor(CursorMove::Forward);
            return VimTransition::Mode(VimMode::Insert);
          }
          Input {
            key: Key::Char('A'),
            ..
          } => {
            textarea.cancel_selection();
            textarea.move_cursor(CursorMove::End);
            return VimTransition::Mode(VimMode::Insert);
          }
          Input {
            key: Key::Char('I'),
            ..
          } => {
            textarea.cancel_selection();
            textarea.move_cursor(CursorMove::Head);
            return VimTransition::Mode(VimMode::Insert);
          }
          // Visual mode
          Input {
            key: Key::Char('v'),
            ctrl: false,
            ..
          } if self.mode == VimMode::Normal => {
            textarea.start_selection();
            return VimTransition::Mode(VimMode::Visual);
          }
          // Exit visual mode
          Input { key: Key::Esc, .. }
          | Input {
            key: Key::Char('v'),
            ctrl: false,
            ..
          } if self.mode == VimMode::Visual => {
            textarea.cancel_selection();
            return VimTransition::Mode(VimMode::Normal);
          }
          // Operators: y, d, c
          Input {
            key: Key::Char(c),
            ctrl: false,
            ..
          } if self.mode == VimMode::Operator(c) => {
            // Handle yy, dd, cc - select entire line
            textarea.move_cursor(CursorMove::Head);
            textarea.start_selection();
            textarea.move_cursor(CursorMove::End);
          }
          Input {
            key: Key::Char(op @ ('y' | 'd' | 'c')),
            ctrl: false,
            ..
          } if self.mode == VimMode::Normal => {
            textarea.start_selection();
            return VimTransition::Mode(VimMode::Operator(op));
          }
          // Visual mode operations
          Input {
            key: Key::Char('y'),
            ctrl: false,
            ..
          } if self.mode == VimMode::Visual => {
            textarea.move_cursor(CursorMove::Forward);
            textarea.copy();
            return VimTransition::Mode(VimMode::Normal);
          }
          Input {
            key: Key::Char('d'),
            ctrl: false,
            ..
          } if self.mode == VimMode::Visual => {
            textarea.move_cursor(CursorMove::Forward);
            textarea.cut();
            return VimTransition::Mode(VimMode::Normal);
          }
          Input {
            key: Key::Char('c'),
            ctrl: false,
            ..
          } if self.mode == VimMode::Visual => {
            textarea.move_cursor(CursorMove::Forward);
            textarea.cut();
            return VimTransition::Mode(VimMode::Insert);
          }
          // Exit operator mode on Esc
          Input { key: Key::Esc, .. } if matches!(self.mode, VimMode::Operator(_)) => {
            textarea.cancel_selection();
            return VimTransition::Mode(VimMode::Normal);
          }
          input => return VimTransition::Pending(input),
        }

        // Handle pending operator
        match self.mode {
          VimMode::Operator('y') => {
            textarea.copy();
            VimTransition::Mode(VimMode::Normal)
          }
          VimMode::Operator('d') => {
            textarea.cut();
            VimTransition::Mode(VimMode::Normal)
          }
          VimMode::Operator('c') => {
            textarea.cut();
            VimTransition::Mode(VimMode::Insert)
          }
          _ => VimTransition::Nop,
        }
      }
      VimMode::Insert => match input {
        Input { key: Key::Esc, .. }
        | Input {
          key: Key::Char('c'),
          ctrl: true,
          ..
        } => VimTransition::Mode(VimMode::Normal),
        input => {
          textarea.input(input);
          VimTransition::Mode(VimMode::Insert)
        }
      },
    }
  }

  #[allow(dead_code)]
  pub fn apply_transition(self, transition: VimTransition) -> Option<Self> {
    match transition {
      VimTransition::Mode(mode) if self.mode != mode => Some(Self {
        mode,
        pending: Input::default(),
      }),
      VimTransition::Nop | VimTransition::Mode(_) => Some(self),
      VimTransition::Pending(input) => Some(self.with_pending(input)),
      VimTransition::Exit => None,
    }
  }
}
