use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::OnceLock;
use anyhow::Context;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM, POINT};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::commands::hide;
use crate::hook_manager::{handle_key, TargetKeys};

const WM_INSTALL_HOOK: u32 = WM_USER + 1;
const WM_UNINSTALL_HOOK: u32 = WM_USER + 2;

enum HookEvent {
    Keyboard(TargetKeys),
    Mouse(POINT)
}

static SENDER: OnceLock<Sender<HookEvent>> = OnceLock::new();
pub struct HookManager {
    thread_id: u32
}

impl HookManager {
    pub fn new(app: &AppHandle) -> Self {
        let ( thread_id_tx, thread_id_rx ) = channel();

        std::thread::spawn(move || unsafe {
            if let Err(e) = run_hook_handler(thread_id_tx) {
                log::error!("Error running hook thread: {:#}", e);
            }
        });

        let ( app_tx, app_rx ) = channel();
        SENDER.set(app_tx).unwrap();

        let app_clone = app.clone();

        std::thread::spawn(move || run_action_handler(app_clone, app_rx));

        let thread_id = thread_id_rx.recv().expect("Could not get thread of id");
        
        HookManager { thread_id }
    }

    pub fn uninstall(&self) {
        log::info!("uninstalling hooks");
        if let Err(e) = unsafe { PostThreadMessageW(self.thread_id, WM_UNINSTALL_HOOK, WPARAM(0), LPARAM(0)) } {
            log::error!("Could not post message to thread: {:#}", e);
        }
    }

    pub fn install(&self) {
        log::info!("installing hooks");
        if let Err(e) = unsafe { PostThreadMessageW(self.thread_id, WM_INSTALL_HOOK, WPARAM(0), LPARAM(0)) } {
            log::error!("Could not post message to thread: {:#}", e);
        }
    }
}

impl Drop for HookManager {
    fn drop(&mut self) {
        if let Err(e) = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) } {
            log::error!("Could not post message to thread: {:#}", e);
        }
    }
}

// SetWindowsHook and UnhookWindowsHook must be called on their own thread
unsafe fn run_hook_handler(tx: Sender<u32>) -> Result<(), anyhow::Error> {
    tx.send(GetCurrentThreadId()).with_context(|| "Could not send current thread id!")?;

    let mut keyboard_hook: Option<HHOOK> = None;
    let mut mouse_hook: Option<HHOOK> = None;

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).into() {
        match msg.message {
            WM_INSTALL_HOOK => {
                if keyboard_hook.is_none() {
                    let hhk = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)
                        .with_context(|| "Could not install keyboard hook")?;
                    keyboard_hook = Some(hhk)
                } else {
                    log::info!("did not install keyboard hook, already installed");
                }
                if mouse_hook.is_none() {
                    let hhk = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
                        .with_context(|| "Could not install mouse hook")?;
                    mouse_hook = Some(hhk)
                } else {
                    log::info!("did not install mouse hook, already installed");
                }
            },
            WM_UNINSTALL_HOOK => {
                if let Some(hhk) = keyboard_hook {
                    UnhookWindowsHookEx(hhk)?;
                    keyboard_hook = None;
                } else {
                    log::info!("did not uninstall keyboard hook, already uninstalled");
                }
                if let Some(hhk) = mouse_hook {
                    UnhookWindowsHookEx(hhk)?;
                    mouse_hook = None;
                } else {
                    log::info!("did not uninstall mouse hook, already uninstalled");
                }
            },
            WM_QUIT => {
                break;
            }
            _ => {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    if let Some(hhk) = keyboard_hook {
        UnhookWindowsHookEx(hhk)?;
    }
    Ok(())
}

// once tx is automatically dropped when main process stops, channel will close and thread will exit
fn run_action_handler(app: AppHandle, rx: Receiver<HookEvent>) {
    while let Ok(event) = rx.recv() {
        match event {
            HookEvent::Keyboard(key) => handle_key(&app, key),
            HookEvent::Mouse(point) => {
                let window =  app.get_webview_window("main").unwrap();
                let clicked = unsafe { WindowFromPoint(point) }; 
                match window.hwnd() {
                    Ok(window_hwnd) => {
                        let root_hwind = unsafe { GetAncestor(window_hwnd, GA_ROOT) };
                        let root_clicked = unsafe { GetAncestor(clicked, GA_ROOT) };
                        log::info!("recieved click event: {:?}, {:?}", root_hwind, root_clicked);
                        if root_hwind != root_clicked {
                            hide(&app);
                        }
                    },
                    Err(e) => log::error!("Could not get hwind from current window: {:#}", e)
                }
            }
        };
    }
}

// this needs to be extremely lightweight, hence the actions handling on a separate thread
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {

    if code == HC_ACTION as i32 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);

        if wparam.0 as u32 == WM_KEYDOWN {
            let key = match VIRTUAL_KEY(kb.vkCode as _) {
                VK_RETURN => TargetKeys::Enter,
                VK_LEFT => TargetKeys::LeftArrow,
                VK_RIGHT => TargetKeys::RightArrow,
                VK_UP => TargetKeys::UpArrow,
                VK_DOWN => TargetKeys::DownArrow,
                _ => TargetKeys::Other
            };
            let intercept = key != TargetKeys::Other;

            let tx = SENDER.get().unwrap();
            tx.send(HookEvent::Keyboard(key)).expect("Could not send key to listener thread");
            if intercept {
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {

    if code == HC_ACTION as i32 {
        let ms = *(lparam.0 as *const MSLLHOOKSTRUCT);
        if wparam.0 as u32 == WM_RBUTTONUP || wparam.0 as u32 == WM_LBUTTONUP {
            let tx = SENDER.get().unwrap();
            tx.send(HookEvent::Mouse(ms.pt)).expect("Could not send key to listener thread");
        }
    }

    CallNextHookEx(None, code, wparam, lparam)

}


pub fn send_ctrl_v() {
    let create_key_input = |k: VIRTUAL_KEY, release: bool | {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: k,
                    wScan: unsafe { MapVirtualKeyW(k.0 as u32, MAPVK_VK_TO_VSC) } as u16,
                    dwFlags: if release { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                    time: 0,
                    dwExtraInfo: 0 
                }
            }
        }
    };


    let inputs: [INPUT; 4] = [
        create_key_input(VK_CONTROL, false),
        create_key_input(VK_V, false),
        create_key_input(VK_CONTROL, true),
        create_key_input(VK_V, true)
    ];

    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}
