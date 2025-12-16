use crate::app::App;
use ratatui::{
  layout::Rect,
  widgets::{Block, BorderType, Borders, Paragraph},
  Frame,
};

pub fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
  let theme = &app.theme;
  let search_text = format!(" /{}_ ", app.search_query);

  let paragraph = Paragraph::new(search_text).style(theme.warning()).block(
    Block::default()
      .title(" Search ")
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(theme.warning()),
  );

  frame.render_widget(paragraph, area);
}
