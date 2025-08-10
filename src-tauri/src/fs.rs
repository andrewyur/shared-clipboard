use tauri::Manager;
use tauri::{image::Image, AppHandle};
use std::fs;
use std::path::PathBuf;

pub fn save_image(image: Image, app: AppHandle) -> Result<(), String> {
    // let mut path = app.path().app_local_data_dir().map_err(|e| format!("Could not get app local data dir: {}", e))?;
    // path.push("images");
    // fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    // fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}