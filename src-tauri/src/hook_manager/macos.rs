use std::{
    ptr::NonNull,
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock,
    },
};

use anyhow::anyhow;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWindow};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop, CGPoint};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapCallBack,
    CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
    CGRectContainsPoint,
};
use tauri::{AppHandle, Manager};

use crate::{
    commands::hide,
    hook_manager::{handle_key, TargetKeys},
};

enum HookEvent {
    Keyboard(TargetKeys),
    Mouse(CGPoint),
}

static SENDER: OnceLock<Sender<HookEvent>> = OnceLock::new();
pub struct HookManager {
    data: Option<HookManagerTemp>,
    handle: AppHandle,
}

// wrapper struct for data, since creating an Event Tap can fail if app doesn't have accessibility perms
impl HookManager {
    pub fn new(app: &AppHandle) -> Self {
        match HookManagerTemp::try_new(app) {
            Ok(data) => Self {
                data: Some(data),
                handle: app.clone(),
            },
            Err(e) => {
                log::warn!("Could not create hook manager: {:#}", e);
                Self {
                    data: None,
                    handle: app.clone(),
                }
            }
        }
    }

    fn check_data(&mut self) {
        if self.data.is_none() {
            match HookManagerTemp::try_new(&self.handle) {
                Ok(data) => self.data = Some(data),
                Err(e) => {
                    log::warn!("Could not create hook manager: {:#}", e);
                }
            }
        }
    }

    pub fn install(&mut self) {
        self.check_data();
        self.data.as_ref().map(|d| d.install());
    }

    pub fn uninstall(&mut self) {
        self.check_data();
        self.data.as_ref().map(|d| d.uninstall());
    }
}
struct HookManagerTemp {
    enable_tx: Sender<bool>,
}

impl HookManagerTemp {
    fn try_new(app: &AppHandle) -> anyhow::Result<Self> {
        let (event_tx, event_rx) = channel();
        SENDER.set(event_tx).expect("Tried to set the sender when it was already set");

        // need to do this bcs CGEventTap is not Send or Sync
        let (enable_tx, enable_rx) = channel();
        let (create_tx, create_rx) = channel::<Option<anyhow::Error>>();

        std::thread::spawn(move || {
            let tap_res = create_tap();

            if let Err(e) = tap_res {
                create_tx.send(Some(e)).unwrap();
                return;
            } else {
                create_tx.send(None).unwrap();
            }

            let tap = tap_res.unwrap();

            while let Ok(enable) = enable_rx.recv() {
                CGEvent::tap_enable(&tap, enable);
            }
        });

        if let Ok(Some(e)) = create_rx.recv() {
            return Err(e);
        }

        let app_clone = app.clone();
        std::thread::spawn(move || event_message_handler(app_clone, event_rx));

        Ok(HookManagerTemp { enable_tx })
    }

    fn install(&self) {
        self.enable_tx.send(true).unwrap();
    }

    fn uninstall(&self) {
        self.enable_tx.send(false).unwrap();
    }
}

fn create_tap() -> anyhow::Result<CFRetained<CFMachPort>> {
    unsafe {
        let tap = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            1 << CGEventType::LeftMouseDown.0 | 1 << CGEventType::KeyDown.0,
            CGEventTapCallBack::Some(raw_callback),
            std::ptr::null_mut(),
        )
        .ok_or(anyhow!("Could not create event tap, check permissions"))?;

        let source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or(anyhow!("could not create loop"))?;

        let current_loop = CFRunLoop::main().unwrap();
        current_loop.add_source(Some(&source), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);

        Ok(tap)
    }
}

unsafe extern "C-unwind" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    _user_info: *mut std::os::raw::c_void,
) -> *mut CGEvent {
    if let Some(tx) = SENDER.get() {
        let event_ref = cg_event.as_ref();
        if event_type == CGEventType::LeftMouseDown {
            let point = CGEvent::location(Some(event_ref));
            tx.send(HookEvent::Mouse(point)).unwrap();
        } else if event_type == CGEventType::KeyDown {
            let keycode =
                CGEvent::integer_value_field(Some(event_ref), CGEventField::KeyboardEventKeycode);
            let target_key = match keycode {
                126 => TargetKeys::UpArrow,
                125 => TargetKeys::DownArrow,
                123 => TargetKeys::LeftArrow,
                124 => TargetKeys::RightArrow,
                36 => TargetKeys::Enter,
                _ => TargetKeys::Other,
            };

            let capture = target_key != TargetKeys::Other;

            tx.send(HookEvent::Keyboard(target_key)).unwrap();

            if capture {
                return std::ptr::null_mut();
            }
        }
    } else {
        log::error!("Tried to send an event from callback when the sender was not initialized")
    }

    cg_event.as_ptr()
    // std::ptr::null_mut()
}

fn event_message_handler(app: AppHandle, rx: Receiver<HookEvent>) {
    while let Ok(message) = rx.recv() {
        match message {
            HookEvent::Keyboard(key) => handle_key(&app, key),
            HookEvent::Mouse(point) => {
                let app_clone = app.clone();
                let _ = app.run_on_main_thread(move || {
                    let window = app_clone.get_webview_window("main").unwrap();
                    let ns_window_ptr = window.ns_window().unwrap();
                    let ns_window = unsafe { &mut *(ns_window_ptr as *mut NSWindow) };

                    let mut frame = ns_window.frame();
                    let mtm = MainThreadMarker::new().unwrap();
                    let screens = NSScreen::screens(mtm);
                    let main_screen = screens.firstObject().unwrap();

                    frame.origin.y = main_screen.frame().max().y - frame.max().y;

                    if !CGRectContainsPoint(frame, point) {
                        hide(&app_clone);
                    }
                });
            }
        }
    }
}

pub fn send_ctrl_v() {
    let v_key = 9;

    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .expect("could not get combined event source");

    let post_event = |key: u16, key_down: bool| {
        let event = CGEvent::new_keyboard_event(Some(&source.clone()), key, key_down)
            .expect("could not create keyboard event");
        if key_down {
            CGEvent::set_flags(Some(&event), CGEventFlags::MaskCommand);
        }
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    };

    post_event(v_key, true);
    post_event(v_key, false);
}
