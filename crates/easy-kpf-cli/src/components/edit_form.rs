use crate::app::{App, Mode};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Clear, Paragraph},
  Frame,
};

pub fn draw_edit_form(frame: &mut Frame, app: &App) {
  let area = centered_rect(60, 60, frame.area());
  frame.render_widget(Clear, area);

  let title = if app.mode == Mode::Create {
    " New Port Forward "
  } else {
    " Edit Port Forward "
  };

  let block = Block::default()
    .title(title)
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Cyan));

  let inner = block.inner(area);
  frame.render_widget(block, area);

  let field_count = app.edit_field_count();
  let constraints: Vec<Constraint> = (0..field_count)
    .map(|_| Constraint::Length(3))
    .chain(std::iter::once(Constraint::Min(1)))
    .collect();

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints(constraints)
    .split(inner);

  for i in 0..field_count {
    let is_selected = app.edit_field_index == i;
    let label = app.edit_field_name(i);
    let value = if is_selected {
      format!("{}|", app.edit_field_value)
    } else {
      app.get_edit_field_value(i)
    };

    let style = if is_selected {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default().fg(Color::White)
    };

    let border_style = if is_selected {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default().fg(Color::DarkGray)
    };

    let field = Paragraph::new(Line::from(vec![
      Span::styled(&value, style),
    ]))
    .block(
      Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style),
    );

    frame.render_widget(field, chunks[i]);
  }

  // Instructions at the bottom
  let instructions = Paragraph::new(Line::from(vec![
    Span::styled(" Tab ", Style::default().fg(Color::Black).bg(Color::Cyan)),
    Span::raw(" next  "),
    Span::styled(" S-Tab ", Style::default().fg(Color::Black).bg(Color::Cyan)),
    Span::raw(" prev  "),
    Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Cyan)),
    Span::raw(" save  "),
    Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Cyan)),
    Span::raw(" cancel"),
  ]))
  .style(Style::default().fg(Color::DarkGray));

  if field_count < chunks.len() {
    frame.render_widget(instructions, chunks[field_count]);
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
