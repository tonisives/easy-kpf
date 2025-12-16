use crate::app::{App, Mode, Panel};
use ratatui::{
  layout::Rect,
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, List, ListItem},
  Frame,
};

pub fn draw_service_list(frame: &mut Frame, app: &App, area: Rect) {
  let theme = &app.theme;
  let is_focused = app.active_panel == Panel::ServiceList;
  let is_visual_mode = app.mode == Mode::Visual;

  let border_style = if is_visual_mode {
    theme.border_visual()
  } else if is_focused {
    theme.border_focused()
  } else {
    theme.border()
  };

  let groups = app.configs_by_context();
  let mut items: Vec<ListItem> = Vec::new();

  for (context, configs) in &groups {
    // Context header
    items.push(ListItem::new(Line::from(vec![Span::styled(
      format!(" [{}]", context),
      theme.warning().add_modifier(Modifier::BOLD),
    )])));

    for (visual_idx, _config_idx, config) in configs {
      // Selection is based on visual index now
      let is_cursor = app.selected_index == *visual_idx && is_focused;
      let is_in_visual = app.is_in_visual_selection(*visual_idx);
      let is_running = app.running_services.contains_key(&config.name);

      let checkbox = if is_running { "[x]" } else { "[ ]" };
      let status = if is_running { "Run" } else { "Stop" };
      let status_style = if is_running {
        theme.success()
      } else {
        theme.text_tertiary()
      };

      // Style: visual selection gets purple bg, cursor gets highlight
      let style = if is_cursor && is_in_visual {
        // Cursor within visual selection
        theme.visual_cursor()
      } else if is_in_visual {
        // Visual selection (not cursor)
        theme.visual_selection()
      } else if is_cursor {
        // Normal cursor (not in visual mode)
        theme.cursor()
      } else {
        Style::default()
      };

      // Format ports as a single string
      let ports_str = config.ports.join(", ");

      // Show selection indicator for cursor or visual selection
      let indicator = if is_cursor {
        " > "
      } else if is_in_visual {
        " * "
      } else {
        "   "
      };

      let line = Line::from(vec![
        Span::raw(indicator),
        Span::styled(
          checkbox,
          if is_running {
            theme.success()
          } else {
            theme.text_tertiary()
          },
        ),
        Span::raw(" "),
        Span::styled(&config.name, theme.text()),
        Span::raw("  "),
        Span::styled(ports_str, theme.accent()),
        Span::raw("  "),
        Span::styled(status, status_style),
      ]);

      items.push(ListItem::new(line).style(style));
    }
  }

  if items.is_empty() {
    items.push(ListItem::new(Line::from(vec![Span::styled(
      "  No port forwards configured. Press 'n' to create one.",
      theme.text_tertiary(),
    )])));
  }

  // Title shows mode indicator
  let title = if is_visual_mode {
    if let Some((start, end)) = app.visual_selection_range() {
      format!(" Services [VISUAL {}] ", end - start + 1)
    } else {
      " Services [VISUAL] ".to_string()
    }
  } else {
    " Services ".to_string()
  };

  let list = List::new(items).block(
    Block::default()
      .title(title)
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(border_style),
  );

  frame.render_widget(list, area);
}
