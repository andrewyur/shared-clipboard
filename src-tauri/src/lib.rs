mod manager;
mod item;
mod watcher;
mod commands;

use std::error::Error;
use std::sync::Mutex;

use clipboard_master::Master;
use tauri::{App, Emitter, Manager as TauriManager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::commands::{pin_item, get_pinned, unpin_item, copy_item, get_history};
use crate::manager::Manager;
use crate::watcher::Watcher;

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.manage(Mutex::new(Manager::new(app.handle().clone())));

    let mut watcher = Master::new(Watcher::new(app.handle().clone()));
    std::thread::spawn(move || {
        if let Err(e) = watcher.run() {
            log::error!("Clipboard change handler failed to start: {e}");
        }
    });

    let open_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, _event| {
                if shortcut == &open_shortcut {
                    app.emit("window-shown", {})
                        .expect("Could not emit clipboard-changed event");
                    app.get_webview_window("main").map(|w| {
                        w.show()?;
                        w.set_focus()
                    });
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(open_shortcut)?;

    log::info!("Sucessfully started Clipboard change handler");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Debug).build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![get_history, copy_item, pin_item, get_pinned, unpin_item])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
