use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
  Key(KeyEvent),
  Tick,
  #[allow(dead_code)]
  Resize(u16, u16),
}

pub struct EventHandler {
  rx: mpsc::Receiver<Event>,
  _tx: mpsc::Sender<Event>,
}

impl EventHandler {
  pub fn new(tick_rate: Duration) -> Self {
    let (tx, rx) = mpsc::channel(100);
    let event_tx = tx.clone();

    std::thread::spawn(move || {
      loop {
        if event::poll(tick_rate).unwrap_or(false) {
          let send_result = match event::read() {
            // Only handle key press events (not release)
            Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
              event_tx.blocking_send(Event::Key(key))
            }
            Ok(CrosstermEvent::Resize(w, h)) => event_tx.blocking_send(Event::Resize(w, h)),
            _ => Ok(()),
          };
          if send_result.is_err() {
            break;
          }
        } else if event_tx.blocking_send(Event::Tick).is_err() {
          // Tick event for periodic updates
          break;
        }
      }
    });

    Self { rx, _tx: tx }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }
}
