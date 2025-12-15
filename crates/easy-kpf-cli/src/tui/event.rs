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
          match event::read() {
            Ok(CrosstermEvent::Key(key)) => {
              // Only handle key press events (not release)
              if key.kind == KeyEventKind::Press {
                if event_tx.blocking_send(Event::Key(key)).is_err() {
                  break;
                }
              }
            }
            Ok(CrosstermEvent::Resize(w, h)) => {
              if event_tx.blocking_send(Event::Resize(w, h)).is_err() {
                break;
              }
            }
            _ => {}
          }
        } else {
          // Tick event for periodic updates
          if event_tx.blocking_send(Event::Tick).is_err() {
            break;
          }
        }
      }
    });

    Self { rx, _tx: tx }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }
}
