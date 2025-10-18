use anyhow::{anyhow, Context};
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewWindow};

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
    let physical_cursor = window.app_handle().cursor_position()?;
    let main_monitor =     window
        .primary_monitor()
        .context("Could not get primary monitor")?
        .ok_or_else(|| anyhow!("No primary monitor"))?;
    let logical_cursor = physical_cursor.to_logical(main_monitor.scale_factor());

    let cursor_monitor = get_monitor_for_point(window, logical_cursor)?;

    let window_size = window
        .outer_size()
        .unwrap()
        .to_logical(cursor_monitor.scale_factor());
    let window_position = clamp_position_to_monitor(logical_cursor, window_size, &cursor_monitor, 0);

    window
        .set_position(window_position)
        .context("Could not set window position")
}

fn move_to_caret(window: &WebviewWindow) -> anyhow::Result<()> {
    let caret = get_caret().context("Could not get caret position")?;
    if caret.position.x == 0 && caret.position.y == 0 {
        return Err(anyhow!("Caret is not visible"));
    }

    let caret_monitor = get_monitor_for_point(window, caret.position)?;
    let window_size = window.outer_size().unwrap().to_logical::<u32>(caret_monitor.scale_factor());

    let window_position = clamp_position_to_monitor(
        caret.position,
        window_size,
        &caret_monitor,
        caret.size.height
    );

        window
        .set_position(window_position)
        .context("Could not set window position")
}

fn get_monitor_for_point(
    window: &WebviewWindow,
    point: LogicalPosition<i32>,
) -> anyhow::Result<tauri::Monitor> {
    window
        .monitor_from_point(point.x as f64, point.y as f64)
        .with_context(|| format!("Could not get monitor for point {:?}", point))?
        .ok_or_else(|| anyhow!("Could not get monitor for point {:?}", point))
        .inspect(|m| log::debug!("got monitor: {}", m.name().unwrap()))
}

fn clamp_position_to_monitor(
    window_position: LogicalPosition<i32>,
    window_size: LogicalSize<u32>,
    monitor: &tauri::Monitor,
    flip_height: u32,
) -> LogicalPosition<i32> {
    let monitor_position = monitor.position().to_logical::<i32>(monitor.scale_factor());
    let monitor_size = monitor.size().to_logical::<i32>(monitor.scale_factor());

    let mut x = window_position.x;
    let mut y = window_position.y;
    
    if x > monitor_position.x + monitor_size.width as i32 - window_size.width as i32 {
        x -= window_size.width as i32
    }

    if y > monitor_position.y + monitor_size.height as i32 - window_size.height as i32 {
        y -= (flip_height + window_size.height) as i32
    }

    LogicalPosition {
        x: x.clamp(
            monitor_position.x,
            monitor_position.x + monitor_size.width as i32 - window_size.width as i32,
        ),
        y: y.clamp(
            monitor_position.y,
            monitor_position.y + monitor_size.height as i32 - window_size.height as i32,
        ),
    }
}