use crate::app::App;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Clear, Paragraph},
  Frame,
};

pub fn draw_help_popup(frame: &mut Frame, _app: &App) {
  let area = centered_rect(60, 70, frame.area());
  frame.render_widget(Clear, area);

  let help_text = vec![
    Line::from(vec![
      Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("  j / Down    ", Style::default().fg(Color::Yellow)),
      Span::raw("Move selection down"),
    ]),
    Line::from(vec![
      Span::styled("  k / Up      ", Style::default().fg(Color::Yellow)),
      Span::raw("Move selection up"),
    ]),
    Line::from(vec![
      Span::styled("  h / Left    ", Style::default().fg(Color::Yellow)),
      Span::raw("Focus service list"),
    ]),
    Line::from(vec![
      Span::styled("  l / Right   ", Style::default().fg(Color::Yellow)),
      Span::raw("Focus log panel"),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("  Space/Enter ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle port forward on/off"),
    ]),
    Line::from(vec![
      Span::styled("  a           ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle all port forwards"),
    ]),
    Line::from(vec![
      Span::styled("  n           ", Style::default().fg(Color::Yellow)),
      Span::raw("Create new config"),
    ]),
    Line::from(vec![
      Span::styled("  e           ", Style::default().fg(Color::Yellow)),
      Span::raw("Edit selected config"),
    ]),
    Line::from(vec![
      Span::styled("  d / Delete  ", Style::default().fg(Color::Yellow)),
      Span::raw("Delete selected config"),
    ]),
    Line::from(vec![
      Span::styled("  r           ", Style::default().fg(Color::Yellow)),
      Span::raw("Refresh/sync processes"),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("Other", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("  /           ", Style::default().fg(Color::Yellow)),
      Span::raw("Search/filter services"),
    ]),
    Line::from(vec![
      Span::styled("  E           ", Style::default().fg(Color::Yellow)),
      Span::raw("Open config in $EDITOR"),
    ]),
    Line::from(vec![
      Span::styled("  ?           ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle this help"),
    ]),
    Line::from(vec![
      Span::styled("  q / Ctrl+c  ", Style::default().fg(Color::Yellow)),
      Span::raw("Quit"),
    ]),
    Line::from(""),
    Line::from(vec![
      Span::styled("  Press Esc or ? to close", Style::default().fg(Color::DarkGray)),
    ]),
  ];

  let popup = Paragraph::new(help_text).block(
    Block::default()
      .title(" Help ")
      .borders(Borders::ALL)
      .border_style(Style::default().fg(Color::Cyan)),
  );

  frame.render_widget(popup, area);
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
