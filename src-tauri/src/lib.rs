mod clip;
mod handler;

use std::error::Error;
use std::sync::Mutex;

use clipboard_master::Master;
use tauri::{App, AppHandle, Emitter, Manager};

use crate::{clip::Clip, handler::Handler};

// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[tauri::command]
async fn get_clipboard_contents(app: AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<Clip>>();
    let clip = state.lock().map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    app.emit("clipboard-changed", clip.values()).map_err(|e| format!("Could not emit clipboard-changed event {}", e))?;
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.manage(Mutex::new(Clip::new(app.handle())));

    let mut watcher = Master::new(Handler::new(app.handle().clone()));
    std::thread::spawn(move || {
        if let Err(e) = watcher.run() {
            log::error!("Clipboard change handler failed to start: {e}");
        }
    });

    log::info!("Sucessfully started Clipboard change handler");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![get_clipboard_contents])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
