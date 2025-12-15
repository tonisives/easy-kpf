use crate::app::{App, ConfirmAction, Mode};
use crate::vim::{VimMode, VimTransition};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use easy_kpf_core::Result;
use tui_textarea::Input;

pub fn handle_edit_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  // Check if in command mode (: commands like :w, :q)
  if app.command_mode {
    return handle_command_mode(app, key);
  }

  // Check if in vim edit mode for the current field
  if app.is_vim_edit_mode() {
    return handle_vim_edit_mode(app, key);
  }

  // Check if suggestions panel is focused
  if app.autocomplete.focused {
    return handle_suggestions_focused(app, key);
  }

  // Check if in typing mode - then all keys go to the field
  if app.autocomplete.typing {
    return handle_typing_mode(app, key);
  }

  // Default mode: j/k navigate suggestions, i enters typing mode
  match key.code {
    KeyCode::Esc => {
      // Sync current field value before checking for changes
      app.set_edit_field_value(app.edit_field_value.clone());

      // Only show confirmation if there are unsaved changes
      if app.has_unsaved_changes() {
        let current_mode = app.mode;
        app.confirm_action = Some(ConfirmAction::CancelEdit(current_mode));
        app.mode = Mode::Confirm;
      } else {
        app.cancel_edit();
      }
    }
    // : to enter command mode (vim-style :w, :q)
    KeyCode::Char(':') => {
      app.enter_command_mode();
    }
    // i to enter typing mode (vim-style insert)
    KeyCode::Char('i') => {
      app.enter_typing_mode();
    }
    // e to enter vim edit mode for the current field
    KeyCode::Char('e') => {
      app.enter_vim_edit_mode();
    }
    // j/Down to go to next suggestion
    KeyCode::Char('j') | KeyCode::Down => {
      app.autocomplete_next();
    }
    // k/Up to go to previous suggestion
    KeyCode::Char('k') | KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Enter accepts current suggestion
    KeyCode::Enter => {
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
    }
    // Ctrl+s to save
    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.save_edit()?;
      app.set_status("Configuration saved");
    }
    KeyCode::Tab => {
      // Accept current suggestion if any, then move to next field
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
      app.next_edit_field();
    }
    KeyCode::BackTab => {
      app.prev_edit_field();
    }
    // Ctrl+o to reload suggestions (mnemonic: "options")
    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.load_autocomplete();
    }
    _ => {}
  }
  Ok(())
}

fn handle_typing_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc exits typing mode
    KeyCode::Esc => {
      app.exit_typing_mode();
    }
    // Ctrl+s to save
    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.save_edit()?;
      app.set_status("Configuration saved");
    }
    KeyCode::Tab => {
      app.exit_typing_mode();
      app.next_edit_field();
    }
    KeyCode::BackTab => {
      app.exit_typing_mode();
      app.prev_edit_field();
    }
    // Up/Down arrow keys navigate suggestions while in typing mode
    KeyCode::Down => {
      app.autocomplete_next();
    }
    KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Left/Right arrow keys move cursor within the field
    KeyCode::Left => {
      app.cursor_left();
    }
    KeyCode::Right => {
      app.cursor_right();
    }
    KeyCode::Backspace => {
      app.delete_char_before_cursor();
      // Mark name as manually edited when user types/deletes in name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
    }
    // Ctrl+o to reload suggestions
    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.load_autocomplete();
    }
    // Enter accepts current suggestion
    KeyCode::Enter => {
      if !app.get_autocomplete_suggestions().is_empty() {
        app.accept_autocomplete();
      }
    }
    // : at start of empty field or Ctrl+: enters command mode
    KeyCode::Char(':') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.exit_typing_mode();
      app.enter_command_mode();
    }
    KeyCode::Char(c) => {
      app.insert_char(c);
      // Mark name as manually edited when user types in name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
    }
    _ => {}
  }
  Ok(())
}

fn handle_vim_edit_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  let input: Input = key.into();

  // Get mutable references to both textarea and vim_state
  let (textarea, vim_state) = match (&mut app.vim_textarea, &mut app.vim_state) {
    (Some(ta), Some(vs)) => (ta, vs),
    _ => {
      app.exit_vim_edit_mode();
      return Ok(());
    }
  };

  let transition = vim_state.transition(input, textarea);

  match transition {
    VimTransition::Exit => {
      // Mark name as edited if this was the name field
      if app.edit_field_index == 0 {
        app.name_manually_edited = true;
      }
      app.exit_vim_edit_mode();
    }
    VimTransition::Mode(mode) if vim_state.mode != mode => {
      textarea.set_cursor_style(mode.cursor_style());
      vim_state.mode = mode;
      // Mark as edited when text changes (entering insert mode means editing)
      if app.edit_field_index == 0 && mode == VimMode::Insert {
        app.name_manually_edited = true;
      }
    }
    VimTransition::Nop | VimTransition::Mode(_) => {}
    VimTransition::Pending(_input) => {
      // Store pending input for multi-key sequences (like gg)
      // For now we ignore pending - single line doesn't need gg/G
    }
  }

  Ok(())
}

fn handle_suggestions_focused(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc goes back to field editing
    KeyCode::Esc => {
      app.unfocus_suggestions();
    }
    // j/Down to go to next suggestion
    KeyCode::Char('j') | KeyCode::Down => {
      app.autocomplete_next();
    }
    // k/Up to go to previous suggestion
    KeyCode::Char('k') | KeyCode::Up => {
      app.autocomplete_prev();
    }
    // Enter to accept suggestion and go back to field
    KeyCode::Enter => {
      app.accept_autocomplete();
      app.unfocus_suggestions();
    }
    // Tab to accept and move to next field
    KeyCode::Tab => {
      app.accept_autocomplete();
      app.next_edit_field();
    }
    _ => {}
  }
  Ok(())
}

fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<()> {
  match key.code {
    // Esc exits command mode
    KeyCode::Esc => {
      app.exit_command_mode();
    }
    // Enter executes the command
    KeyCode::Enter => {
      let cmd = app.command_buffer.trim().to_lowercase();
      app.exit_command_mode();

      match cmd.as_str() {
        "w" | "write" => {
          // Save and stay in edit mode
          app.save_edit()?;
          app.set_status("Configuration saved");
        }
        "q" | "quit" => {
          // Quit without saving - show confirmation only if there are changes
          app.set_edit_field_value(app.edit_field_value.clone());
          if app.has_unsaved_changes() {
            let current_mode = app.mode;
            app.confirm_action = Some(ConfirmAction::CancelEdit(current_mode));
            app.mode = Mode::Confirm;
          } else {
            app.cancel_edit();
          }
        }
        "wq" | "x" => {
          // Save and quit
          app.save_edit()?;
          app.set_status("Configuration saved");
        }
        "q!" => {
          // Force quit without confirmation
          app.cancel_edit();
        }
        _ => {
          app.set_status(format!("Unknown command: {}", cmd));
        }
      }
    }
    // Backspace deletes from command buffer
    KeyCode::Backspace => {
      if app.command_buffer.is_empty() {
        // Exit command mode if backspace on empty buffer
        app.exit_command_mode();
      } else {
        app.command_buffer.pop();
      }
    }
    // Type characters into command buffer
    KeyCode::Char(c) => {
      app.command_buffer.push(c);
    }
    _ => {}
  }
  Ok(())
}
