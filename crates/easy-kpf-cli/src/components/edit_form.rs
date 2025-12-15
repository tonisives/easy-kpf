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

  draw_fields_panel(frame, app, h_chunks[0]);
  draw_suggestions_panel(frame, app, h_chunks[1]);
}

fn draw_fields_panel(frame: &mut Frame, app: &App, area: Rect) {
  let field_count = app.edit_field_count();
  let field_constraints: Vec<Constraint> = (0..field_count)
    .map(|_| Constraint::Length(3))
    .chain(std::iter::once(Constraint::Min(1))) // Instructions
    .collect();

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints(field_constraints)
    .split(area);

  for i in 0..field_count {
    draw_field(frame, app, i, chunks[i]);
  }

  // Instructions at the bottom
  let instructions = build_instructions(app);
  if field_count < chunks.len() {
    frame.render_widget(instructions, chunks[field_count]);
  }
}

fn draw_field(frame: &mut Frame, app: &App, field_index: usize, area: Rect) {
  let is_selected = app.edit_field_index == field_index;
  let label = app.edit_field_name(field_index);
  let mut description = app.edit_field_description(field_index).to_string();

  // For Name field in Create mode, show auto-generated indicator
  if field_index == 0 && app.mode == Mode::Create && !app.name_manually_edited {
    description = "auto-generated".to_string();
  }

  let is_typing = app.autocomplete.typing && is_selected;
  let is_vim_mode = app.is_vim_edit_mode() && is_selected;
  let is_editing = is_typing || is_vim_mode;

  let (style, border_style) = get_field_styles(is_selected, is_editing);

  // Build title with optional description
  let title = if description.is_empty() {
    format!(" {} ", label)
  } else {
    format!(" {} ({}) ", label, description)
  };

  // Render vim textarea or regular field
  if is_vim_mode {
    if let Some(textarea) = &app.vim_textarea {
      let mut ta = textarea.clone();
      ta.set_block(
        Block::default()
          .title(title)
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .border_style(border_style),
      );
      frame.render_widget(&ta, area);
    }
  } else {
    let value = get_field_display_value(app, field_index, is_selected, is_typing);
    let line = Line::from(vec![Span::styled(&value, style)]);
    let field = Paragraph::new(line).block(
      Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style),
    );
    frame.render_widget(field, area);
  }
}

fn get_field_styles(is_selected: bool, is_editing: bool) -> (Style, Style) {
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

  (style, border_style)
}

fn get_field_display_value(
  app: &App,
  field_index: usize,
  is_selected: bool,
  is_typing: bool,
) -> String {
  if is_selected {
    if is_typing {
      // Show cursor at the correct position
      let before = &app.edit_field_value[..app.edit_cursor_pos];
      let after = &app.edit_field_value[app.edit_cursor_pos..];
      format!("{}|{}", before, after)
    } else {
      app.edit_field_value.clone()
    }
  } else {
    app.get_edit_field_value(field_index)
  }
}

fn build_instructions(app: &App) -> Paragraph<'static> {
  let content = if app.command_mode {
    Line::from(vec![
      Span::styled(":", Style::default().fg(Color::Yellow)),
      Span::styled(
        app.command_buffer.clone(),
        Style::default().fg(Color::White),
      ),
      Span::styled("_", Style::default().fg(Color::Yellow)),
    ])
  } else if app.is_vim_edit_mode() {
    Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" done  "),
      Span::styled(" i ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" insert"),
    ])
  } else if app.autocomplete.typing {
    Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" stop typing  "),
      Span::styled(" Tab ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" next  "),
      Span::styled(" :w ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" save"),
    ])
  } else if app.autocomplete.focused {
    Line::from(vec![
      Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Yellow)),
      Span::raw(" back to field"),
    ])
  } else {
    Line::from(vec![
      Span::styled(" i ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" type  "),
      Span::styled(" e ", Style::default().fg(Color::Black).bg(Color::Magenta)),
      Span::raw(" vim  "),
      Span::styled(" j/k ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" nav  "),
      Span::styled(" :w ", Style::default().fg(Color::Black).bg(Color::Cyan)),
      Span::raw(" save"),
    ])
  };

  Paragraph::new(content).style(Style::default().fg(Color::DarkGray))
}

fn draw_suggestions_panel(frame: &mut Frame, app: &App, area: Rect) {
  let right_area = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([Constraint::Min(1)])
    .split(area)[0];

  let suggestions = app.get_autocomplete_suggestions();
  let can_load = matches!(app.edit_field_index, 1 | 2 | 3 | 4 | 6);

  if !suggestions.is_empty() {
    draw_suggestions_list(frame, app, suggestions, right_area);
  } else if app.autocomplete.loading {
    draw_loading_state(frame, right_area);
  } else {
    draw_empty_suggestions(frame, can_load, right_area);
  }
}

fn draw_suggestions_list(frame: &mut Frame, app: &App, suggestions: &[String], area: Rect) {
  let suggestions_focused = app.autocomplete.focused;
  let visible_height = area.height.saturating_sub(2) as usize;
  let selected = app.autocomplete.selected_index;
  let total = suggestions.len();

  let scroll_offset = calculate_scroll_offset(selected, total, visible_height);

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

  let title = format!(
    " Suggestions ({}/{}) j/k nav, Enter accept ",
    selected + 1,
    total
  );
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

  frame.render_widget(list, area);
}

fn calculate_scroll_offset(selected: usize, total: usize, visible_height: usize) -> usize {
  if selected < visible_height / 2 {
    0
  } else if selected > total.saturating_sub(visible_height / 2) {
    total.saturating_sub(visible_height)
  } else {
    selected.saturating_sub(visible_height / 2)
  }
}

fn draw_loading_state(frame: &mut Frame, area: Rect) {
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
  frame.render_widget(loading, area);
}

fn draw_empty_suggestions(frame: &mut Frame, can_load: bool, area: Rect) {
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
  frame.render_widget(hint, area);
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
