use anyhow::{anyhow, Context};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::position_window::windows::get_caret;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::position_window::macos::get_caret;

pub fn position_window(window: &WebviewWindow) {
    if let Err(e) = move_to_caret(window) {
        log::warn!("Was not able to position window at caret: {:#}", e);

        if let Err(e) = move_to_cursor(window) {
            log::warn!("Was not able to position window at cursor: {:#}", e);
        }
    }
}

fn move_to_cursor(window: &WebviewWindow) -> anyhow::Result<()> {
    let cursor_position = window.app_handle().cursor_position()?;
    let monitor = get_monitor_for_point(window, cursor_position.cast())?;
    let window_size = window.outer_size().unwrap();

    let window_position = clamp_position_to_monitor(
        cursor_position.x as i32,
        cursor_position.y as i32,
        window_size,
        &monitor,
    )?;

    move_window(window, window_position)
}

fn move_to_caret(window: &WebviewWindow) -> anyhow::Result<()> {
    let caret = get_caret().context("Could not get caret position")?;
    if caret.position.x == 0 && caret.position.y == 0 {
        return Err(anyhow!("Caret is not visible"));
    }
    let monitor = get_monitor_for_point(window, caret.position)?;
    let window_size = window.outer_size().unwrap();

    let window_position = clamp_position_to_monitor(
        caret.position.x,
        caret.position.y + caret.size.height as i32,
        window_size,
        &monitor,
    )?;

    move_window(window, window_position)
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
    x: i32,
    y: i32,
    window_size: PhysicalSize<u32>,
    monitor: &tauri::Monitor,
) -> anyhow::Result<PhysicalPosition<i32>> {
    let monitor_position = monitor.position().to_logical::<i32>(monitor.scale_factor());
    let monitor_size = monitor.size().to_logical::<i32>(monitor.scale_factor());
    let window_size = window_size.to_logical::<i32>(monitor.scale_factor());

    Ok(PhysicalPosition { 
        x: x.clamp(monitor_position.x, monitor_position.x + monitor_size.width as i32 - window_size.width as i32), 
        y: y.clamp(monitor_position.y, monitor_position.y + monitor_size.height as i32 - window_size.height as i32),
    })
}

fn move_window(window: &WebviewWindow, position: PhysicalPosition<i32>) -> anyhow::Result<()> {
    let main_monitor = window
        .primary_monitor()
        .context("could not get the primary monitor")?
        .ok_or_else(|| anyhow!("No primary monitor"))?;

    let logical = position.to_logical::<i32>(main_monitor.scale_factor());

    window
        .set_position(logical)
        .context("Could not set window position")
}
