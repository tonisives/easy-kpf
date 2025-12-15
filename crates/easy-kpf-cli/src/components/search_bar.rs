use crate::app::App;
use ratatui::{
  layout::Rect,
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
  Frame,
};

pub fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
  let search_text = format!(" /{}_ ", app.search_query);

  let paragraph = Paragraph::new(search_text)
    .style(Style::default().fg(Color::Yellow))
    .block(
      Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow)),
    );

  frame.render_widget(paragraph, area);
}
