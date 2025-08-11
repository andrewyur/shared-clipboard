use std::io;
use std::sync::Mutex;

use clipboard_master::{CallbackResult, ClipboardHandler};
use tauri::{AppHandle, Manager as TauriManager};

use crate::manager::Manager;

pub struct Watcher {
    handle: AppHandle,
}

impl Watcher {
    pub fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl ClipboardHandler for Watcher {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let state = self.handle.state::<Mutex<Option<Manager>>>();
        match state.lock() {
            Ok(mut manager) => {
                manager.as_mut().map(|m| m.check());
            }
            Err(e) => {
                log::error!("Couldn't access clipboard manager: {}", e);
            }
        };
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("Error: {}", error);
        CallbackResult::Next
    }
}
