use std::sync::Mutex;

use tauri::{AppHandle, Manager as TauriManager};

use crate::manager::Manager;

#[tauri::command]
pub async fn copy_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("copying item with id {} from history to clipboard", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.copy(id);
    app.get_webview_window("main").map(|w| w.hide());
    Ok(())
}

#[tauri::command]
pub async fn request_update(state: tauri::State<'_, Mutex<Manager>>,) -> Result<(), String> {
    let clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.emit();
    Ok(())
}

#[tauri::command]
pub async fn pin_item(
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("pinning item with id: {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.pin(id);
    Ok(())
}

#[tauri::command]
pub async fn unpin_item(
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("unpinning item with id: {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.unpin(id);
    Ok(())
}
