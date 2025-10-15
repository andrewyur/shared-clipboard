use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{send_ctrl_v, HookManager};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{send_ctrl_v, HookManager};

use crate::commands::hide;

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
enum TargetKeys {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    Enter,
    Other,
}

#[allow(dead_code)]
fn handle_key(app: &AppHandle, key: TargetKeys) {
    match key {
        TargetKeys::Other => hide(&app),
        _ => {
            if let Err(e) = app.emit("key", key) {
                log::error!("could not emit key: {:#}", e);
            }
        }
    }
}
