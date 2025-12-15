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
        self.handle_normal_visual_operator(input, textarea)
      }
      VimMode::Insert => handle_insert_mode(input, textarea),
    }
  }

  fn handle_normal_visual_operator(
    &self,
    input: Input,
    textarea: &mut TextArea<'_>,
  ) -> VimTransition {
    // Handle exit conditions first
    if let Some(transition) = self.try_exit(&input, textarea) {
      return transition;
    }

    // Handle movement
    if let Some(transition) = handle_movement(&input, textarea, self.mode) {
      return transition;
    }

    // Handle delete/change operations
    if let Some(transition) = handle_delete_change(&input, textarea, self.mode) {
      return transition;
    }

    // Handle paste and undo/redo
    if let Some(transition) = handle_clipboard_undo(&input, textarea) {
      return transition;
    }

    // Handle insert mode entry
    if let Some(transition) = handle_enter_insert(&input, textarea) {
      return transition;
    }

    // Handle visual mode
    if let Some(transition) = handle_visual_mode(&input, textarea, self.mode) {
      return transition;
    }

    // Handle operators (y, d, c)
    if let Some(transition) = self.handle_operators(&input, textarea) {
      return transition;
    }

    // Pending input for unrecognized
    VimTransition::Pending(input)
  }

  fn try_exit(&self, input: &Input, textarea: &mut TextArea<'_>) -> Option<VimTransition> {
    match input {
      Input {
        key: Key::Enter, ..
      }
      | Input { key: Key::Esc, .. }
        if self.mode == VimMode::Normal =>
      {
        textarea.cancel_selection();
        Some(VimTransition::Exit)
      }
      Input { key: Key::Esc, .. } if matches!(self.mode, VimMode::Operator(_)) => {
        textarea.cancel_selection();
        Some(VimTransition::Mode(VimMode::Normal))
      }
      _ => None,
    }
  }

  fn handle_operators(&self, input: &Input, textarea: &mut TextArea<'_>) -> Option<VimTransition> {
    match input {
      // Double operator (yy, dd, cc) - select entire line
      Input {
        key: Key::Char(c),
        ctrl: false,
        ..
      } if self.mode == VimMode::Operator(*c) => {
        textarea.move_cursor(CursorMove::Head);
        textarea.start_selection();
        textarea.move_cursor(CursorMove::End);
        Some(complete_operator(self.mode, textarea))
      }
      // Start operator mode
      Input {
        key: Key::Char(op @ ('y' | 'd' | 'c')),
        ctrl: false,
        ..
      } if self.mode == VimMode::Normal => {
        textarea.start_selection();
        Some(VimTransition::Mode(VimMode::Operator(*op)))
      }
      _ => None,
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

fn handle_movement(input: &Input, textarea: &mut TextArea<'_>, mode: VimMode) -> Option<VimTransition> {
  match input {
    // Basic movement
    Input {
      key: Key::Char('h'),
      ..
    }
    | Input {
      key: Key::Left, ..
    } => {
      textarea.move_cursor(CursorMove::Back);
      Some(complete_operator(mode, textarea))
    }
    Input {
      key: Key::Char('l'),
      ..
    }
    | Input {
      key: Key::Right, ..
    } => {
      textarea.move_cursor(CursorMove::Forward);
      Some(complete_operator(mode, textarea))
    }
    // Word movement
    Input {
      key: Key::Char('w'),
      ..
    } => {
      textarea.move_cursor(CursorMove::WordForward);
      Some(complete_operator(mode, textarea))
    }
    Input {
      key: Key::Char('e'),
      ctrl: false,
      ..
    } => {
      textarea.move_cursor(CursorMove::WordEnd);
      if matches!(mode, VimMode::Operator(_)) {
        textarea.move_cursor(CursorMove::Forward);
      }
      Some(complete_operator(mode, textarea))
    }
    Input {
      key: Key::Char('b'),
      ctrl: false,
      ..
    } => {
      textarea.move_cursor(CursorMove::WordBack);
      Some(complete_operator(mode, textarea))
    }
    // Line movement
    Input {
      key: Key::Char('^' | '0'),
      ..
    } => {
      textarea.move_cursor(CursorMove::Head);
      Some(complete_operator(mode, textarea))
    }
    Input {
      key: Key::Char('$'),
      ..
    } => {
      textarea.move_cursor(CursorMove::End);
      Some(complete_operator(mode, textarea))
    }
    _ => None,
  }
}

fn handle_delete_change(input: &Input, textarea: &mut TextArea<'_>, mode: VimMode) -> Option<VimTransition> {
  match input {
    Input {
      key: Key::Char('D'),
      ..
    } => {
      textarea.delete_line_by_end();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    Input {
      key: Key::Char('C'),
      ..
    } => {
      textarea.delete_line_by_end();
      textarea.cancel_selection();
      Some(VimTransition::Mode(VimMode::Insert))
    }
    Input {
      key: Key::Char('x'),
      ..
    } => {
      textarea.delete_next_char();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    // Visual mode delete/change/yank
    Input {
      key: Key::Char('y'),
      ctrl: false,
      ..
    } if mode == VimMode::Visual => {
      textarea.move_cursor(CursorMove::Forward);
      textarea.copy();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    Input {
      key: Key::Char('d'),
      ctrl: false,
      ..
    } if mode == VimMode::Visual => {
      textarea.move_cursor(CursorMove::Forward);
      textarea.cut();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    Input {
      key: Key::Char('c'),
      ctrl: false,
      ..
    } if mode == VimMode::Visual => {
      textarea.move_cursor(CursorMove::Forward);
      textarea.cut();
      Some(VimTransition::Mode(VimMode::Insert))
    }
    _ => None,
  }
}

fn handle_clipboard_undo(input: &Input, textarea: &mut TextArea<'_>) -> Option<VimTransition> {
  match input {
    Input {
      key: Key::Char('p'),
      ..
    } => {
      textarea.paste();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    Input {
      key: Key::Char('u'),
      ctrl: false,
      ..
    } => {
      textarea.undo();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    Input {
      key: Key::Char('r'),
      ctrl: true,
      ..
    } => {
      textarea.redo();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    _ => None,
  }
}

fn handle_enter_insert(input: &Input, textarea: &mut TextArea<'_>) -> Option<VimTransition> {
  match input {
    Input {
      key: Key::Char('i'),
      ..
    } => {
      textarea.cancel_selection();
      Some(VimTransition::Mode(VimMode::Insert))
    }
    Input {
      key: Key::Char('a'),
      ..
    } => {
      textarea.cancel_selection();
      textarea.move_cursor(CursorMove::Forward);
      Some(VimTransition::Mode(VimMode::Insert))
    }
    Input {
      key: Key::Char('A'),
      ..
    } => {
      textarea.cancel_selection();
      textarea.move_cursor(CursorMove::End);
      Some(VimTransition::Mode(VimMode::Insert))
    }
    Input {
      key: Key::Char('I'),
      ..
    } => {
      textarea.cancel_selection();
      textarea.move_cursor(CursorMove::Head);
      Some(VimTransition::Mode(VimMode::Insert))
    }
    _ => None,
  }
}

fn handle_visual_mode(input: &Input, textarea: &mut TextArea<'_>, mode: VimMode) -> Option<VimTransition> {
  match input {
    // Enter visual mode from normal
    Input {
      key: Key::Char('v'),
      ctrl: false,
      ..
    } if mode == VimMode::Normal => {
      textarea.start_selection();
      Some(VimTransition::Mode(VimMode::Visual))
    }
    // Exit visual mode
    Input { key: Key::Esc, .. }
    | Input {
      key: Key::Char('v'),
      ctrl: false,
      ..
    } if mode == VimMode::Visual => {
      textarea.cancel_selection();
      Some(VimTransition::Mode(VimMode::Normal))
    }
    _ => None,
  }
}

fn handle_insert_mode(input: Input, textarea: &mut TextArea<'_>) -> VimTransition {
  match input {
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
  }
}

/// Complete an operator motion (y/d/c) after cursor movement
fn complete_operator(mode: VimMode, textarea: &mut TextArea<'_>) -> VimTransition {
  match mode {
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
