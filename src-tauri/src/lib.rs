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
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_updater::UpdaterExt;

use crate::clipboard_manager::ClipboardManager;
use crate::commands::*;
use crate::hook_manager::HookManager;
use crate::watcher::Watcher;

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        update
            .download_and_install(
                |_chunk_length, _content_length| {},
                || {
                    log::info!("download finished");
                },
            )
            .await?;

        log::info!("Update installed, restarting");
        app.restart();
    }

    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let window = app
        .get_webview_window("main")
        .expect("Could not get window handle during setup function");

    #[cfg(target_os = "macos")]
    {
        app.set_dock_visibility(false);
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        window.set_visible_on_all_workspaces(true)?;

        use objc2_app_kit::{
            NSWindow, NSWindowButton, NSWindowCollectionBehavior, NSWindowStyleMask, NSFloatingWindowLevel
        };

        let ns_window_ptr = window.ns_window().unwrap();
        unsafe {
            let ns_window = &mut *(ns_window_ptr as *mut NSWindow);
            ns_window.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );

            let mut mask = ns_window.styleMask();
            mask.insert(NSWindowStyleMask::FullSizeContentView);
            ns_window.setStyleMask(mask);

            ns_window.setTitlebarAppearsTransparent(true);

            ns_window
                .standardWindowButton(NSWindowButton::CloseButton)
                .map(|b| b.setHidden(true));
            ns_window
                .standardWindowButton(NSWindowButton::MiniaturizeButton)
                .map(|b| b.setHidden(true));
            ns_window
                .standardWindowButton(NSWindowButton::ZoomButton)
                .map(|b| b.setHidden(true));

            ns_window.setLevel(NSFloatingWindowLevel);
        }

        app.handle()
            .plugin(tauri_plugin_macos_permissions::init())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        window.set_skip_taskbar(true)?;
        window.set_always_on_top(true)?;
        window.set_decorations(false)?;
    }

    let autostart_manager = app.autolaunch();
    if !cfg!(dev) {
        let _ = autostart_manager
            .enable()
            .map_err(|e| log::warn!("Could not enable autostart: {:#}", e));

        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            _ = update(handle)
                .await
                .map_err(|e| log::error!("Could not install update: {:#}", e));
        });
    }

    app.manage(Mutex::new(Some(ClipboardManager::new(app.handle()))));
    app.manage(Mutex::new(HookManager::new(app.handle())));

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
    let open_shortcut = Shortcut::new(
        Some(Modifiers::CONTROL | Modifiers::ALT),
        if cfg!(dev) { Code::KeyB } else { Code::KeyV },
    );
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if shortcut == &open_shortcut && event.state == ShortcutState::Pressed {
                    let window = app.get_webview_window("main").unwrap();
                    position_window::position_window(&window);
                    // sleep to let window position call be registered before window shown, so they happen in order
                    // i know this is bad, not sure how else to do it though, tokio fails for some reason
                    #[cfg(target_os = "macos")]
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    show(&app);
                    app.emit("window-shown", {})
                        .expect("Could not emit clipboard-changed event");
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(open_shortcut)?;

    show(app.handle());
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
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
