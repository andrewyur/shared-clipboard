use std::sync::Mutex;

use tauri::{AppHandle, Manager as TauriManager};

use crate::{hook_manager::{send_ctrl_v, HookManager}, clipboard_manager::ClipboardManager};

#[tauri::command]
pub async fn paste_item(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Option<ClipboardManager>>>,
    id: u32,
) -> Result<(), String> {
    log::info!("copying item with id {} from history to clipboard and pasting", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.as_mut().map(|s| s.copy(id));
    send_ctrl_v();
    hide(&app);
    Ok(())
}

#[tauri::command]
pub async fn request_update(state: tauri::State<'_, Mutex<Option<ClipboardManager>>>) -> Result<(), String> {
    let clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.as_ref().map(|s| s.emit());
    Ok(())
}

#[tauri::command]
pub async fn pin_item(
    state: tauri::State<'_, Mutex<Option<ClipboardManager>>>,
    id: u32,
) -> Result<(), String> {
    log::info!("pinning item with id: {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.as_mut().map(|s| s.pin(id));
    Ok(())
}

#[tauri::command]
pub async fn unpin_item(
    state: tauri::State<'_, Mutex<Option<ClipboardManager>>>,
    id: u32,
) -> Result<(), String> {
    log::info!("unpinning item with id: {}", id);
    let mut clip = state
        .lock()
        .map_err(|e| format!("Could not access the clipboard handler {}", e))?;
    clip.as_mut().map(|s| s.unpin(id));
    Ok(())
}


// want to listen to show and hide window events: https://github.com/tauri-apps/tauri/issues/14061
#[tauri::command]
pub async fn show_window(app: AppHandle) {
    show(&app);
}

pub fn show(app: &AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    let hook_manager = app.state::<Mutex<HookManager>>();
    _ = hook_manager.lock().as_mut().map(|h| h.install());
    _ = window.show();

    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSWindow};
        let ns_window_ptr = window.ns_window().unwrap();
        unsafe {
            let ns_window = &mut *(ns_window_ptr as *mut NSWindow);
            ns_window.orderFrontRegardless();
        }
    }
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) {
    hide(&app);
}

pub fn hide(app: &AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    let hook_manager = app.state::<Mutex<HookManager>>();
    _ = hook_manager.lock().as_mut().map(|h| h.uninstall());
    _ = window.hide();
}