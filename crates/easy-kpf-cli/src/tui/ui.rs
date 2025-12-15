use crate::app::{App, Mode};
use crate::components::{
  draw_edit_form, draw_help_popup, draw_log_panel, draw_search_bar, draw_service_list,
  draw_status_bar,
};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders, Clear, Paragraph},
  Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Header/search bar
      Constraint::Min(10),   // Main content
      Constraint::Length(1), // Status bar
    ])
    .split(frame.area());

  draw_header(frame, app, chunks[0]);
  draw_main_content(frame, app, chunks[1]);
  draw_status_bar(frame, app, chunks[2]);

  // Draw modal overlays
  match app.mode {
    Mode::Help => draw_help_popup(frame, app),
    Mode::Edit | Mode::Create => draw_edit_form(frame, app),
    Mode::Confirm => draw_confirm_dialog(frame, app),
    _ => {}
  }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Min(20), Constraint::Length(12)])
    .split(area);

  // Search/title area
  if app.mode == Mode::Search {
    draw_search_bar(frame, app, chunks[0]);
  } else {
    let title = Paragraph::new(" Easy KPF")
      .style(
        Style::default()
          .fg(Color::Cyan)
          .add_modifier(Modifier::BOLD),
      )
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded),
      );
    frame.render_widget(title, chunks[0]);
  }

  // Help hint
  let help_hint = Paragraph::new(" [?] Help ")
    .style(Style::default().fg(Color::DarkGray))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded),
    );
  frame.render_widget(help_hint, chunks[1]);
}

fn draw_main_content(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(area);

  draw_service_list(frame, app, chunks[0]);
  draw_log_panel(frame, app, chunks[1]);
}

fn draw_confirm_dialog(frame: &mut Frame, app: &App) {
  let area = centered_rect(50, 25, frame.area());
  frame.render_widget(Clear, area);

  let message = match &app.confirm_action {
    Some(crate::app::ConfirmAction::Delete(key)) => {
      format!("Delete '{}'?\n\n[y] Yes  [n] No", key)
    }
    Some(crate::app::ConfirmAction::StartAll) => {
      let count = app.configs.len();
      format!(
        "Start all {} port forward{}?\n\n[y] Yes  [n] No",
        count,
        if count == 1 { "" } else { "s" }
      )
    }
    Some(crate::app::ConfirmAction::StopAll) => {
      let count = app.running_services.len();
      format!(
        "Stop all {} port forward{}?\n\n[y] Yes  [n] No",
        count,
        if count == 1 { "" } else { "s" }
      )
    }
    Some(crate::app::ConfirmAction::CancelEdit(_)) => {
      "Discard changes?\n\n[y] Yes  [n] No".to_string()
    }
    None => "Confirm?".to_string(),
  };

  let popup = Paragraph::new(message)
    .style(Style::default().fg(Color::Yellow))
    .block(
      Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow)),
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
