use std::sync::mpsc::{Receiver, Sender};

pub struct AIState {
    pub ai_tx: Option<Sender<(String, String, String)>>,
    pub ai_rx: Option<Receiver<(String, String, String)>>,
    pub processing: bool,
    pub last_error: Option<String>,
}

impl AIState {
    pub fn new(tx: Sender<(String, String, String)>, rx: Receiver<(String, String, String)>) -> Self {
        Self {
            ai_tx: Some(tx),
            ai_rx: Some(rx), 
            processing: false,
            last_error: None,
        }
    }

    pub fn handle_response(&mut self) -> Option<(String, String, String)> {
        if let Some(rx) = &self.ai_rx {
            if let Ok(response) = rx.try_recv() {
                self.processing = false;
                return Some(response);
            }
        }
        None
    }
}
