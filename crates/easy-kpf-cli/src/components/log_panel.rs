use crate::app::{App, Panel};
use ratatui::{
  layout::Rect,
  style::{Color, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Paragraph, Wrap},
  Frame,
};

pub fn draw_log_panel(frame: &mut Frame, app: &App, area: Rect) {
  let is_focused = app.active_panel == Panel::Logs;
  let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

  let title = app
    .selected_name()
    .map(|k| format!(" Logs: {} ", k))
    .unwrap_or_else(|| " Logs ".to_string());

  let logs = app.get_logs_for_selected();

  let lines: Vec<Line> = if logs.is_empty() {
    vec![Line::from(Span::styled(
      "  No logs yet. Start a port forward to see output.",
      Style::default().fg(Color::DarkGray),
    ))]
  } else {
    let visible_height = area.height.saturating_sub(2) as usize;
    let start = logs.len().saturating_sub(visible_height);
    logs[start..]
      .iter()
      .map(|entry| {
        let color = if entry.is_stderr {
          Color::Red
        } else {
          Color::White
        };
        Line::from(Span::styled(&entry.line, Style::default().fg(color)))
      })
      .collect()
  };

  let paragraph = Paragraph::new(lines)
    .block(
      Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color)),
    )
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
}
