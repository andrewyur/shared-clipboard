mod commands;
mod contents;
mod manager;
mod watcher;
mod clipboard_files;

use std::error::Error;
use std::sync::Mutex;

use clipboard_master::Master;
use tauri::{App, Emitter, Manager as TauriManager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::commands::{copy_item, pin_item, request_update, unpin_item};
use crate::manager::Manager;
use crate::watcher::Watcher;

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.set_dock_visibility(false);

    // refocus app when workspace is switched to the window's active workspace
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSWorkspace, NSWindow};
        use objc2_foundation::{NSNotificationName, NSNotification, NSOperationQueue};
        use block2::RcBlock;
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
            center.addObserverForName_object_queue_usingBlock(Some(&name), None, Some(&queue), &*block);
        }
    }

    // set window to move into active workspace when shown
    app.get_webview_window("main").map(|w| {
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
            let ns_window_ptr = w.ns_window().unwrap();
            unsafe {
                let ns_window = &mut *(ns_window_ptr as *mut NSWindow);
                ns_window.setCollectionBehavior(NSWindowCollectionBehavior::MoveToActiveSpace);
            }
        }
    });

    app.manage(Mutex::new(Some(Manager::new(app.handle().clone()))));

    // start clipboard change watcher
    let mut watcher = Master::new(Watcher::new(app.handle().clone()));
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
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            copy_item,
            pin_item,
            unpin_item,
            request_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
