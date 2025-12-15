use crate::app::{App, Mode};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
  Frame,
};

pub fn draw_edit_form(frame: &mut Frame, app: &App) {
  let area = centered_rect(85, 90, frame.area());
  frame.render_widget(Clear, area);

  let title = if app.mode == Mode::Create {
    " New Port Forward "
  } else {
    " Edit Port Forward "
  };

  let block = Block::default()
    .title(title)
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(Color::Cyan));

  let inner = block.inner(area);
  frame.render_widget(block, area);

  // Split into left (fields) and right (suggestions) panels
  let h_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(inner);

  // Left panel: fields + instructions
  let field_count = app.edit_field_count();
  let field_constraints: Vec<Constraint> = (0..field_count)
    .map(|_| Constraint::Length(3))
    .chain(std::iter::once(Constraint::Min(1))) // Instructions
    .collect();

  let left_chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints(field_constraints)
    .split(h_chunks[0]);

  for i in 0..field_count {
    let is_selected = app.edit_field_index == i;
    let label = app.edit_field_name(i);
    let mut description = app.edit_field_description(i).to_string();

    // For Name field in Create mode, show auto-generated indicator
    if i == 0 && app.mode == Mode::Create && !app.name_manually_edited {
      description = "auto-generated".to_string();
    }

    let is_typing = app.autocomplete.typing && is_selected;
    let is_vim_mode = app.is_vim_edit_mode() && is_selected;
    let is_editing = is_typing || is_vim_mode;

    let style = if is_editing {
      Style::default().fg(Color::Green)
    } else if is_selected {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default().fg(Color::White)
    };

    let border_style = if is_editing {
      Style::default().fg(Color::Green)
    } else if is_selected {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default().fg(Color::DarkGray)
    };

    // Build title with optional description
    let title = if description.is_empty() {
      format!(" {} ", label)
    } else {
      format!(" {} ({}) ", label, description)
    };

    // Render vim textarea or regular field
    if is_vim_mode {
      if let Some(textarea) = &app.vim_textarea {
        // Clone and style the textarea for rendering
        let mut ta = textarea.clone();
        ta.set_block(
          Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style),
        );
        frame.render_widget(&ta, left_chunks[i]);
      }
    } else {
      let value = if is_selected {
        if is_typing {
          // Show cursor at the correct position
          let before = &app.edit_field_value[..app.edit_cursor_pos];
          let after = &app.edit_field_value[app.edit_cursor_pos..];
          format!("{}|{}", before, after)
        } else {
          app.edit_field_value.clone()
        }
      } else {
        app.get_edit_field_value(i)
      };

      let line = Line::from(vec![Span::styled(&value, style)]);
      let field = Paragraph::new(line).block(
        Block::default()
          .title(title)
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .border_style(border_style),
      );

      frame.render_widget(field, left_chunks[i]);
    }
  }

  // Instructions at the bottom of left panel
  let instructions = if app.command_mode {
    // Show command line when in command mode
    Paragraph::new(Line::from(vec![
      Span::styled(":", Style::default().fg(Color::Yellow)),
      Span::styled(&app.command_buffer, Style::default().fg(Color::White)),
      Span::styled("_", Style::default().fg(Color::Yellow)),
    ]))
  } else if app.is_vim_edit_mode() {
    // Vim mode - minimal instructions like Telescope
    Paragraph::new(Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" done  "),
      Span::styled(" i ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" insert"),
    ]))
  } else if app.autocomplete.typing {
    Paragraph::new(Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" stop typing  "),
      Span::styled(" Tab ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" next  "),
      Span::styled(" :w ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" save"),
    ]))
  } else if app.autocomplete.focused {
    Paragraph::new(Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" back to field"),
    ]))
  } else {
    Paragraph::new(Line::from(vec![
      Span::styled(" i ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" type  "),
      Span::styled(" e ", Style::default().fg(Color::Black).bg(Color::Magenta)),
      Span::raw(" vim  "),
      Span::styled(" j/k ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" nav  "),
      Span::styled(" :w ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" save"),
    ]))
  }
  .style(Style::default().fg(Color::DarkGray));

  if field_count < left_chunks.len() {
    frame.render_widget(instructions, left_chunks[field_count]);
  }

  // Right panel: suggestions list
  let right_area = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([Constraint::Min(1)])
    .split(h_chunks[1])[0];

  let suggestions = app.get_autocomplete_suggestions();
  let can_load = matches!(app.edit_field_index, 1 | 2 | 3 | 4 | 6);
  let suggestions_focused = app.autocomplete.focused;

  if !suggestions.is_empty() {
    // Calculate visible window around selected item
    let visible_height = right_area.height.saturating_sub(2) as usize; // -2 for borders
    let selected = app.autocomplete.selected_index;
    let total = suggestions.len();

    // Determine scroll offset to keep selected item visible
    let scroll_offset = if selected < visible_height / 2 {
      0
    } else if selected > total.saturating_sub(visible_height / 2) {
      total.saturating_sub(visible_height)
    } else {
      selected.saturating_sub(visible_height / 2)
    };

    let items: Vec<ListItem> = suggestions
      .iter()
      .enumerate()
      .skip(scroll_offset)
      .take(visible_height)
      .map(|(idx, s)| {
        let is_selected = idx == app.autocomplete.selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
          if suggestions_focused {
            Style::default()
              .fg(Color::Yellow)
              .add_modifier(Modifier::BOLD)
          } else {
            Style::default()
              .fg(Color::Cyan)
              .add_modifier(Modifier::BOLD)
          }
        } else {
          Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, s), style)))
      })
      .collect();

    let title = if suggestions_focused {
      format!(" Suggestions ({}/{}) j/k nav, Enter accept ", selected + 1, total)
    } else if app.autocomplete.typing {
      format!(" Suggestions ({}/{}) ", selected + 1, total)
    } else {
      format!(" Suggestions ({}/{}) j/k nav, Enter accept ", selected + 1, total)
    };

    let border_color = if suggestions_focused {
      Color::Yellow
    } else {
      Color::Cyan
    };

    let list = List::new(items).block(
      Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(list, right_area);
  } else if app.autocomplete.loading {
    let loading = Paragraph::new(Line::from(Span::styled(
      "Loading...",
      Style::default().fg(Color::DarkGray),
    )))
    .block(
      Block::default()
        .title(" Suggestions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(loading, right_area);
  } else {
    let hint_text = if can_load {
      "Suggestions auto-load on Tab\nOr press Ctrl+o to reload"
    } else {
      "No suggestions for this field"
    };
    let hint = Paragraph::new(hint_text)
      .style(Style::default().fg(Color::DarkGray))
      .block(
        Block::default()
          .title(" Suggestions ")
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .border_style(Style::default().fg(Color::DarkGray)),
      );
    frame.render_widget(hint, right_area);
  }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
