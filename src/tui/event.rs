use crate::error::{Error, Result};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Quit,
    Key(KeyEvent),
    Search(String),
}

#[allow(dead_code)]
pub struct EventHandler {
    pub sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));

        let handler = {
            let sender = sender.clone();
            let running = running.clone();
            thread::spawn(move || {
                while running.load(Ordering::Relaxed) {
                    if event::poll(Duration::from_millis(250)).expect("no events available") {
                        match event::read().expect("unable to read events") {
                            CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                            _ => unimplemented!(),
                        }
                        .expect("failed to send a event to terminal")
                    }
                }
            })
        };

        EventHandler {
            sender,
            receiver,
            handler,
            running,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .map_err(|e| Error::Unknown { msg: e.to_string() })
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
