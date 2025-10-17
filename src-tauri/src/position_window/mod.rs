use anyhow::{anyhow, Context};
use tauri::{Manager, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::position_window::windows::get_caret;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::position_window::macos::get_caret;

pub fn position_window(window: &WebviewWindow) {
    if let Err(e) = try_position_window(window) {
        log::warn!("Was not able to position window at caret: {:#}", e);

        let cursor_position_res = get_cursor_position(window);
        log::debug!("cursor position: {:?}", cursor_position_res);
        if let Ok(cursor_position) = cursor_position_res  {
            log::debug!("Cursor position: {:?}", cursor_position);
            _ = window.set_position(cursor_position);
        }
    }
}

fn get_cursor_position(window: &WebviewWindow) -> anyhow::Result<PhysicalPosition<i32>> {
    let cursor_position = window.app_handle().cursor_position()?;
    let monitor = get_monitor_for_point(window, cursor_position.cast())?;
    let window_size = window.outer_size().unwrap();

    clamp_position_to_monitor(
        cursor_position.x as i32,
        cursor_position.y as i32,
        window_size,
        &monitor,
    )
}

fn try_position_window(window: &WebviewWindow) -> anyhow::Result<()> {
    let caret = get_caret().context("Could not get caret position")?;
    if caret.position.x == 0 && caret.position.y == 0 {
        return Err(anyhow!("Caret is not visible"));
    }

    let window_position = calculate_window_position(caret, window)?;

    log::debug!("Caret position: {:?}", window_position);

    window
        .set_position(window_position)
        .context("Could not set window position")?;

    Ok(())
}

fn calculate_window_position(
    caret: PhysicalRect<i32, u32>,
    window: &WebviewWindow,
) -> anyhow::Result<PhysicalPosition<i32>> {
    let monitor = get_monitor_for_point(window, caret.position)?;
    let window_size = window.outer_size().unwrap();

    let x = caret.position.x;
    let mut y = caret.position.y + caret.size.height as i32;

    if y + window_size.height as i32 > monitor.size().height as i32 {
        y = caret.position.y - window_size.height as i32;
    }

    clamp_position_to_monitor(x, y, window_size, &monitor)
}

fn get_monitor_for_point(
    window: &WebviewWindow,
    point: PhysicalPosition<i32>,
) -> anyhow::Result<tauri::Monitor> {
    window
        .monitor_from_point(point.x as f64, point.y as f64)
        .with_context(|| format!("Could not get monitor for point {:?}", point))?
        .ok_or_else(|| anyhow!("Could not get monitor for point {:?}", point))
}

fn clamp_position_to_monitor(
    mut x: i32,
    mut y: i32,
    window_size: PhysicalSize<u32>,
    monitor: &tauri::Monitor,
) -> anyhow::Result<PhysicalPosition<i32>> {
    let monitor_size = monitor.size();
    if x + window_size.width as i32 > monitor_size.width as i32 {
        x = monitor_size.width as i32 - window_size.width as i32;
    }
    if y + window_size.height as i32 > monitor_size.height as i32 {
        y = monitor_size.height as i32 - window_size.height as i32;
    }

    Ok(PhysicalPosition {
        x: x.max(0),
        y: y.max(0),
    })
}
