use crate::app::App;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Clear, Paragraph},
  Frame,
};

pub fn draw_help_popup(frame: &mut Frame, _app: &App) {
  let area = centered_rect(65, 85, frame.area());
  frame.render_widget(Clear, area);

  let help_text = build_help_text();

  let popup = Paragraph::new(help_text).block(
    Block::default()
      .title(" Help ")
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(Style::default().fg(Color::Cyan)),
  );

  frame.render_widget(popup, area);
}

fn build_help_text() -> Vec<Line<'static>> {
  let mut lines = Vec::new();

  lines.extend(build_navigation_section());
  lines.extend(build_actions_section());
  lines.extend(build_visual_mode_section());
  lines.extend(build_edit_mode_section());
  lines.extend(build_other_section());

  lines.push(Line::from(vec![Span::styled(
    "  Press Esc or ? to close",
    Style::default().fg(Color::DarkGray),
  )]));

  lines
}

fn section_header(text: &'static str, color: Color) -> Line<'static> {
  Line::from(vec![Span::styled(
    text,
    Style::default().add_modifier(Modifier::BOLD).fg(color),
  )])
}

fn help_line(key: &'static str, description: &'static str) -> Line<'static> {
  Line::from(vec![
    Span::styled(key, Style::default().fg(Color::Yellow)),
    Span::raw(description),
  ])
}

fn build_navigation_section() -> Vec<Line<'static>> {
  vec![
    section_header("Navigation", Color::Cyan),
    Line::from(""),
    help_line("  j . Down    ", "Move selection down"),
    help_line("  k , Up      ", "Move selection up"),
    help_line("  < / >       ", "Jump to start/end"),
    help_line("  h / Left    ", "Focus service list"),
    help_line("  l / Right   ", "Focus log panel"),
    Line::from(""),
  ]
}

fn build_actions_section() -> Vec<Line<'static>> {
  vec![
    section_header("Actions", Color::Cyan),
    Line::from(""),
    help_line("  Space/Enter ", "Toggle port forward on/off"),
    help_line("  a           ", "Start/stop all (confirms before start)"),
    help_line("  n           ", "Create new config"),
    help_line("  e           ", "Edit selected config"),
    help_line("  d / Delete  ", "Delete selected config"),
    help_line("  r           ", "Refresh/sync processes"),
    help_line("  v           ", "Enter visual mode (multi-select)"),
    Line::from(""),
  ]
}

fn build_visual_mode_section() -> Vec<Line<'static>> {
  vec![
    section_header("Visual Mode (v)", Color::Magenta),
    Line::from(""),
    help_line("  j/k         ", "Extend selection up/down"),
    help_line("  < / >       ", "Extend to start/end"),
    help_line("  Space/Enter ", "Toggle selected services"),
    help_line("  s           ", "Start all selected"),
    help_line("  x           ", "Stop all selected"),
    help_line("  Esc / v     ", "Exit visual mode"),
    Line::from(""),
  ]
}

fn build_edit_mode_section() -> Vec<Line<'static>> {
  vec![
    section_header("Edit Mode (n/e)", Color::Cyan),
    Line::from(""),
    help_line("  Tab/S-Tab   ", "Next/previous field (auto-loads)"),
    help_line("  Enter       ", "Focus suggestions list"),
    help_line("  j/k         ", "Navigate suggestions (when focused)"),
    help_line("  Ctrl+s      ", "Save config"),
    help_line("  Esc         ", "Back / Cancel edit"),
    Line::from(""),
  ]
}

fn build_other_section() -> Vec<Line<'static>> {
  vec![
    section_header("Other", Color::Cyan),
    Line::from(""),
    help_line("  /           ", "Search/filter services"),
    help_line("  E           ", "Open config in $EDITOR"),
    help_line("  ?           ", "Toggle this help"),
    help_line("  q / Ctrl+c  ", "Quit"),
    Line::from(""),
  ]
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
