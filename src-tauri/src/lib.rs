mod clipboard_files;
mod clipboard_manager;
mod commands;
mod contents;
mod hook_manager;
mod position_window;
mod watcher;

use std::error::Error;
use std::sync::Mutex;

use clipboard_master::Master;
use tauri::{App, Emitter, Manager as TauriManager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::clipboard_manager::ClipboardManager;
use crate::commands::*;
use crate::hook_manager::HookManager;
use crate::watcher::Watcher;

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let window = app
        .get_webview_window("main")
        .expect("Could not get window handle during setup function");

    #[cfg(target_os = "macos")]
    {
        // app.set_dock_visibility(false);

        // app.set_activation_policy accessory
        // window set_visible_on_all_workspaces(

        // refocus app when workspace is switched to the window's active workspace, dock visibility
        use block2::RcBlock;
        use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSWorkspace};
        use objc2_foundation::{NSNotification, NSNotificationName, NSOperationQueue};

        let apphandle = app.handle().clone();
        let name = NSNotificationName::from_str("NSWorkspaceActiveSpaceDidChangeNotification");
        let block = RcBlock::new(move |_notifcation: std::ptr::NonNull<NSNotification>| {
            apphandle.get_webview_window("main").map(|w| {
                let ns_window_ptr = w.ns_window().unwrap();
                let ns_window = unsafe { &mut *(ns_window_ptr as *mut NSWindow) };
                if unsafe { ns_window.isOnActiveSpace() } {
                    let _ = w.set_focus();
                }
            });
        });
        unsafe {
            let center = NSWorkspace::sharedWorkspace().notificationCenter();
            let queue = NSOperationQueue::mainQueue();
            center.addObserverForName_object_queue_usingBlock(
                Some(&name),
                None,
                Some(&queue),
                &*block,
            );
        }

        // set window to move into active workspace when shown
        let ns_window_ptr = window.ns_window().unwrap();
        unsafe {
            let ns_window = &mut *(ns_window_ptr as *mut NSWindow);
            ns_window.setCollectionBehavior(NSWindowCollectionBehavior::MoveToActiveSpace);
        }
    }

    #[cfg(not(target_os = "macos"))]
    let _ = window.set_skip_taskbar(true);

    app.manage(Mutex::new(Some(ClipboardManager::new(app.handle()))));
    app.manage(HookManager::new(app.handle()));

    // start clipboard change watcher
    let mut watcher = Master::new(Watcher::new(app.handle()));
    std::thread::spawn(move || {
        if let Err(e) = watcher.run() {
            log::error!("Clipboard change handler failed to start: {e}");
        } else {
            log::info!("Sucessfully started Clipboard change handler");
        }
    });

    // register global shortcut
    let open_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if shortcut == &open_shortcut && event.state == ShortcutState::Pressed {
                    let window = app.get_webview_window("main").unwrap();
                    position_window::position_window(&window);
                    show(app);
                    app.emit("window-shown", {})
                        .expect("Could not emit clipboard-changed event");
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(open_shortcut)?;

    // app is shown on startup
    let keyboard_hook_manager = app.state::<HookManager>();
    keyboard_hook_manager.install();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "macos")]
    {
        app.plugin(tauri_plugin_macos_permissions::init())
    }

    app.setup(setup)
        .invoke_handler(tauri::generate_handler![
            paste_item,
            pin_item,
            unpin_item,
            request_update,
            show_window,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
