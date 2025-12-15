use crate::app::{App, Panel};
use ratatui::{
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, List, ListItem},
  Frame,
};

pub fn draw_service_list(frame: &mut Frame, app: &App, area: Rect) {
  let is_focused = app.active_panel == Panel::ServiceList;
  let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

  let groups = app.configs_by_context();
  let mut items: Vec<ListItem> = Vec::new();

  for (context, configs) in &groups {
    // Context header
    items.push(ListItem::new(Line::from(vec![Span::styled(
      format!(" [{}]", context),
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD),
    )])));

    for (visual_idx, _config_idx, config) in configs {
      // Selection is based on visual index now
      let is_selected = app.selected_index == *visual_idx && is_focused;
      let is_running = app.running_services.contains_key(&config.name);

      let checkbox = if is_running { "[x]" } else { "[ ]" };
      let status = if is_running { "Run" } else { "Stop" };
      let status_color = if is_running {
        Color::Green
      } else {
        Color::DarkGray
      };

      let style = if is_selected {
        Style::default()
          .bg(Color::DarkGray)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::default()
      };

      // Format ports as a single string
      let ports_str = config.ports.join(", ");

      let line = Line::from(vec![
        Span::raw(if is_selected { " > " } else { "   " }),
        Span::styled(
          checkbox,
          Style::default().fg(if is_running {
            Color::Green
          } else {
            Color::DarkGray
          }),
        ),
        Span::raw(" "),
        Span::styled(&config.name, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(ports_str, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(status, Style::default().fg(status_color)),
      ]);

      items.push(ListItem::new(line).style(style));
    }
  }

  if items.is_empty() {
    items.push(ListItem::new(Line::from(vec![Span::styled(
      "  No port forwards configured. Press 'n' to create one.",
      Style::default().fg(Color::DarkGray),
    )])));
  }

  let list = List::new(items).block(
    Block::default()
      .title(" Services ")
      .borders(Borders::ALL)
      .border_style(Style::default().fg(border_color)),
  );

  frame.render_widget(list, area);
}
