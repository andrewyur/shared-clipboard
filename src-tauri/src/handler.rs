use std::io;
use std::sync::Mutex;

use clipboard_master::{CallbackResult, ClipboardHandler};
use tauri::{AppHandle, Emitter, Manager};

use crate::clip::Clip;

pub struct Handler {
    handle: AppHandle,
}

impl Handler {
    pub fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let state = self.handle.state::<Mutex<Clip>>();
        match state.lock() {
            Ok(mut clip) => {
                clip.check(&self.handle);
                if let Err(e) = self.handle.emit("clipboard-changed", clip.values()) {
                    log::error!("Couldn't emit clipboard-changed event to frontend {}", e);
                } else {
                    log::info!("successfully emitted clipboard-changed event")
                }
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
