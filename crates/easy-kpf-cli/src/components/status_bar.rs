use crate::app::{App, Mode, Panel};
use ratatui::{
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::Paragraph,
  Frame,
};

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  let bindings = match app.mode {
    Mode::Normal => {
      if app.active_panel == Panel::ServiceList {
        vec![
          ("j/k", "navigate"),
          ("Space", "toggle"),
          ("a", "toggle all"),
          ("n", "new"),
          ("e", "edit"),
          ("d", "delete"),
          ("/", "search"),
          ("E", "$EDITOR"),
          ("l", "logs"),
          ("?", "help"),
          ("q", "quit"),
        ]
      } else {
        vec![
          ("j/k", "scroll"),
          ("h", "services"),
          ("?", "help"),
          ("q", "quit"),
        ]
      }
    }
    Mode::Search => vec![("Enter", "confirm"), ("Esc", "cancel")],
    Mode::Edit | Mode::Create => vec![
      ("Tab", "next field"),
      ("S-Tab", "prev field"),
      ("Enter", "save"),
      ("Esc", "cancel"),
    ],
    Mode::Help => vec![("Esc/q/?", "close")],
    Mode::Confirm => vec![("y", "yes"), ("n", "no"), ("Esc", "cancel")],
  };

  let spans: Vec<Span> = bindings
    .iter()
    .flat_map(|(key, desc)| {
      let spans = vec![
        Span::styled(
          format!(" {} ", key),
          Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}  ", desc), Style::default().fg(Color::Gray)),
      ];
      spans
    })
    .collect();

  let mut line_spans = spans;

  // Add status message if present
  if let Some(msg) = &app.status_message {
    line_spans.push(Span::raw("  "));
    line_spans.push(Span::styled(
      msg,
      Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
    ));
  }

  let paragraph = Paragraph::new(Line::from(line_spans)).style(Style::default().bg(Color::Black));

  frame.render_widget(paragraph, area);
}
