use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager as TauriManager};

use crate::manager::Manager;

#[tauri::command]
pub async fn copy_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("copying item with {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.copy(id);
    app.get_webview_window("main").map(|w| w.hide());
    Ok(())
}

#[tauri::command]
pub async fn get_history(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
) -> Result<(), String> {
    let clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    app.emit("history", clip.history())
        .map_err(|e| format!("Could not emit history event {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_pinned(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
) -> Result<(), String> {
    let clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    app.emit("pinned", clip.pinned())
        .map_err(|e| format!("Could not emit pinned event {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn pin_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("pinning item with {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.pin(id);
    app.emit("pinned", clip.pinned())        .map_err(|e| format!("Could not emit pinned event {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn unpin_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Manager>>,
    id: u32,
) -> Result<(), String> {
    log::info!("unpinning item with {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.unpin(id);
    app.emit("pinned", clip.pinned())        .map_err(|e| format!("Could not emit pinned event {}", e))?;
    Ok(())
}
