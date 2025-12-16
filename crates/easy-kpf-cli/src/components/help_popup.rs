use crate::app::App;
use crate::theme::Theme;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::Modifier,
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Clear, Paragraph},
  Frame,
};

pub fn draw_help_popup(frame: &mut Frame, app: &App) {
  let theme = &app.theme;
  let area = centered_rect(65, 85, frame.area());
  frame.render_widget(Clear, area);

  let help_text = build_help_text(theme);

  let popup = Paragraph::new(help_text).block(
    Block::default()
      .title(" Help ")
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(theme.border_focused()),
  );

  frame.render_widget(popup, area);
}

fn build_help_text(theme: &Theme) -> Vec<Line<'static>> {
  let mut lines = Vec::new();

  lines.extend(build_navigation_section(theme));
  lines.extend(build_actions_section(theme));
  lines.extend(build_visual_mode_section(theme));
  lines.extend(build_edit_mode_section(theme));
  lines.extend(build_other_section(theme));

  lines.push(Line::from(vec![Span::styled(
    "  Press Esc or ? to close",
    theme.text_tertiary(),
  )]));

  lines
}

fn section_header(theme: &Theme, text: &'static str, is_alternate: bool) -> Line<'static> {
  let style = if is_alternate {
    theme.accent_secondary().add_modifier(Modifier::BOLD)
  } else {
    theme.accent().add_modifier(Modifier::BOLD)
  };
  Line::from(vec![Span::styled(text, style)])
}

fn help_line(theme: &Theme, key: &'static str, description: &'static str) -> Line<'static> {
  Line::from(vec![
    Span::styled(key, theme.warning()),
    Span::styled(description, theme.text()),
  ])
}

fn build_navigation_section(theme: &Theme) -> Vec<Line<'static>> {
  vec![
    section_header(theme, "Navigation", false),
    Line::from(""),
    help_line(theme, "  j . Down    ", "Move selection down"),
    help_line(theme, "  k , Up      ", "Move selection up"),
    help_line(theme, "  < / >       ", "Jump to start/end"),
    help_line(theme, "  h / Left    ", "Focus service list"),
    help_line(theme, "  l / Right   ", "Focus log panel"),
    Line::from(""),
  ]
}

fn build_actions_section(theme: &Theme) -> Vec<Line<'static>> {
  vec![
    section_header(theme, "Actions", false),
    Line::from(""),
    help_line(theme, "  Space/Enter ", "Toggle port forward on/off"),
    help_line(
      theme,
      "  a           ",
      "Start/stop all (confirms before start)",
    ),
    help_line(theme, "  n           ", "Create new config"),
    help_line(theme, "  e           ", "Edit selected config"),
    help_line(theme, "  d / Delete  ", "Delete selected config"),
    help_line(theme, "  r           ", "Refresh/sync processes"),
    help_line(theme, "  v           ", "Enter visual mode (multi-select)"),
    Line::from(""),
  ]
}

fn build_visual_mode_section(theme: &Theme) -> Vec<Line<'static>> {
  vec![
    section_header(theme, "Visual Mode (v)", true),
    Line::from(""),
    help_line(theme, "  j/k         ", "Extend selection up/down"),
    help_line(theme, "  < / >       ", "Extend to start/end"),
    help_line(theme, "  Space/Enter ", "Toggle selected services"),
    help_line(theme, "  s           ", "Start all selected"),
    help_line(theme, "  x           ", "Stop all selected"),
    help_line(theme, "  Esc / v     ", "Exit visual mode"),
    Line::from(""),
  ]
}

fn build_edit_mode_section(theme: &Theme) -> Vec<Line<'static>> {
  vec![
    section_header(theme, "Edit Mode (n/e)", false),
    Line::from(""),
    help_line(theme, "  Tab/S-Tab   ", "Next/previous field (auto-loads)"),
    help_line(theme, "  Enter       ", "Focus suggestions list"),
    help_line(
      theme,
      "  j/k         ",
      "Navigate suggestions (when focused)",
    ),
    help_line(theme, "  Ctrl+s      ", "Save config"),
    help_line(theme, "  Esc         ", "Back / Cancel edit"),
    Line::from(""),
  ]
}

fn build_other_section(theme: &Theme) -> Vec<Line<'static>> {
  vec![
    section_header(theme, "Other", false),
    Line::from(""),
    help_line(theme, "  /           ", "Search/filter services"),
    help_line(theme, "  E           ", "Open config in $EDITOR"),
    help_line(theme, "  ?           ", "Toggle this help"),
    help_line(theme, "  q / Ctrl+c  ", "Quit"),
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
