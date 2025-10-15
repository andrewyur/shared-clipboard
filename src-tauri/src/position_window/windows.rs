use anyhow::{anyhow, Context};
use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize};
use windows::core::Interface;
use windows::Win32::UI::Accessibility::{AccessibleObjectFromWindow, IAccessible};
use windows::Win32::UI::WindowsAndMessaging::*;

// caret as a physical rect, position as 0,0 if not available.
pub fn get_caret() -> anyhow::Result<PhysicalRect<i32, u32>> {
    unsafe {
        let hwind = GetForegroundWindow();
        if hwind.0.is_null() {
            return Err(anyhow!("No window currently in foreground"));
        }

        let mut pid = 0;
        let tid = GetWindowThreadProcessId(hwind, Some(&mut pid));

        let mut gui_info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };

        GetGUIThreadInfo(tid, &mut gui_info)
            .with_context(|| "Could not get info about current thread GUI")?;

        if gui_info.hwndFocus.0.is_null() {
            return Err(anyhow!("No window currently has keyboard focus"));
        }

        let mut p_acc: Option<IAccessible> = None;
        AccessibleObjectFromWindow(
            gui_info.hwndFocus,
            OBJID_CARET.0 as u32,
            &IAccessible::IID,
            &mut p_acc as *mut _ as *mut _,
        )
        .with_context(|| "Unable to get Accessible object from window")?;

        let acc = p_acc
            .ok_or_else(|| anyhow!("accessibilty element could not be extracted from pointer"))?;

        use windows::Win32::System::Variant::VARIANT;

        let mut left = 0;
        let mut top = 0;
        let mut width = 0;
        let mut height = 0;

        acc.accLocation(
            &mut left,
            &mut top,
            &mut width,
            &mut height,
            &VARIANT::from(CHILDID_SELF as i32),
        )
        .with_context(|| "Unable to get location from accessibility element")?;

        Ok(PhysicalRect {
            position: PhysicalPosition { x: left, y: top },
            size: PhysicalSize {
                width: width as u32,
                height: height as u32,
            },
        })
    }
}
