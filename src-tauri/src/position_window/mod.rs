use anyhow::Context;
use tauri::{PhysicalPosition, PhysicalRect, WebviewWindow};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::position_window::windows::get_caret;

pub fn position_window(window: &WebviewWindow) {
    if let Err(e) = _position_window(window) {
        log::warn!("Was not able to position window at cursor: {:#}", e);
        let _ = window.center();
    }
}

fn _position_window(window: &WebviewWindow) -> anyhow::Result<()>{ 
    let caret = get_caret().with_context(|| "Could not get caret position")?;
    if caret.position.x == 0 && caret.position.y == 0 {
        return Err(anyhow::anyhow!("Caret is not visible"))
    }
    let window_position = calculate_window_position(caret, window)?;
    window.set_position(window_position).with_context(|| "could not set window position")
}

fn calculate_window_position(caret: PhysicalRect<i32, u32>, window: &WebviewWindow) -> anyhow::Result<PhysicalPosition<i32>> {
    let monitor = window.monitor_from_point(caret.position.x as f64, caret.position.y as f64)
        .with_context(|| format!("could not get window monitor from point: {:?}, {:?}", caret.position.x, caret.position.y))?
        .ok_or_else(|| anyhow::anyhow!("could not get monitor from point: {:?}, {:?}", caret.position.x, caret.position.y))?;
    let window_size = window.outer_size().unwrap();

    let mut y = caret.position.y;
    let mut x = caret.position.x;

    if y + window_size.height as i32 > monitor.size().height as i32 {
        y -= (window_size.height + caret.size.height) as i32
    }

    if x + window_size.width as i32 > monitor.size().width as i32 {
        x = (monitor.size().width - window_size.width) as i32
    }

    Ok(PhysicalPosition { x: x - caret.size.width as i32 , y: y + caret.size.height as i32 })
}
