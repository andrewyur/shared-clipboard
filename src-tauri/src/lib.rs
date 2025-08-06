mod clip;
mod clip_item;
mod handler;

use std::error::Error;
use std::sync::Mutex;

use clipboard_master::Master;
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::{clip::Clip, handler::Handler};

#[tauri::command]
async fn copy_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Clip>>,
    id: u32,
) -> Result<(), String> {
    log::info!("copying item with {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.copy(id, &app);
    app.get_webview_window("main").map(|w| w.hide());
    Ok(())
}

#[tauri::command]
async fn get_clipboard_contents(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Clip>>,
) -> Result<(), String> {
    let clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    app.emit("clipboard-changed", clip.values())
        .map_err(|e| format!("Could not emit clipboard-changed event {}", e))?;
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
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![get_clipboard_contents, copy_item])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
