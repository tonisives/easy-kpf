use crate::app::{App, Mode, Panel};
use ratatui::{
  layout::Rect,
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::Paragraph,
  Frame,
};

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  let theme = &app.theme;

  let bindings = match app.mode {
    Mode::Normal => {
      if app.active_panel == Panel::ServiceList {
        vec![
          ("j/k", "navigate"),
          ("Space", "toggle"),
          ("v", "visual"),
          ("a", "all"),
          ("n", "new"),
          ("e", "edit"),
          ("d", "del"),
          ("/", "search"),
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
    Mode::Visual => vec![
      ("j/k", "extend"),
      ("Space", "toggle"),
      ("s", "start"),
      ("x", "stop"),
      ("Esc", "exit"),
    ],
  };

  let spans: Vec<Span> = bindings
    .iter()
    .flat_map(|(key, desc)| {
      vec![
        Span::styled(
          format!(" {} ", key),
          theme.status_key().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}  ", desc), theme.text_secondary()),
      ]
    })
    .collect();

  let mut line_spans = spans;

  // Add status message if present
  if let Some(msg) = &app.status_message {
    line_spans.push(Span::raw("  "));
    line_spans.push(Span::styled(
      msg,
      theme.warning().add_modifier(Modifier::ITALIC),
    ));
  }

  let paragraph =
    Paragraph::new(Line::from(line_spans)).style(Style::default().bg(theme.status_bg));

  frame.render_widget(paragraph, area);
}
