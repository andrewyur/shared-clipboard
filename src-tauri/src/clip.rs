use crate::clip_item::{ClipItem, HasId};
use std::collections::VecDeque;
use std::io::Cursor;

use base64::prelude::*;
use image::imageops::thumbnail;
use image::{ImageBuffer, ImageFormat, Rgba};
use tauri::{image::Image, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;

const HISTORY_LEN: usize = 20;
const THUMBNAIL_HEIGHT: u32 = 300;
pub struct Clip {
    history: VecDeque<ClipItem>,
}

impl Clip {
    pub fn new(app: &AppHandle) -> Self {
        log::info!("created clipboard manager");
        let mut self_struct = Self {
            history: VecDeque::with_capacity(HISTORY_LEN),
        };
        self_struct.check(app);
        self_struct
    }

    pub fn values(&self) -> &VecDeque<ClipItem> {
        &self.history
    }

    pub fn check(&mut self, app: &AppHandle) -> bool {
        log::info!("Clibboard handler checking clibpoard");
        if match clipboard_files::read() {
            Ok(file_paths) => {
                log::debug!("clipboard files returned: {file_paths:?}");
                self.add(ClipItem::new_file_path(file_paths));
                true
            }
            Err(clipboard_files::ClipboardError::NoFiles) => false,
            Err(e) => {
                log::error!(
                    "System returned an error when reading file paths from clipboard: {}",
                    e
                );
                false
            }
        } {
            return true;
        }

        if match app.clipboard().read_image() {
            Ok(image) => match create_base64_thumbnail(&image) {
                Ok(thumbnail) => {
                    self.add(ClipItem::new_image(thumbnail, image.to_owned()));
                    true
                }
                Err(e) => {
                    log::error!("Could not create thumbnail for image in clipboard: {}", e);
                    false
                }
            },
            // no way to tell if this is because the clipboard has no images in it or because an actual error occured...
            Err(e) => {
                log::warn!("Possible error pasting image from clipboard: {e}");
                false
            }
        } {
            return true;
        }

        match app.clipboard().read_text() {
            Ok(text) => {
                self.add(ClipItem::new_text(text));
                return true;
            }
            Err(e) => {
                log::warn!("Possible error pasting text from clipboard: {e}");
                false
            }
        }
    }

    fn add(&mut self, clip_item: ClipItem) {
        log::debug!("adding item to {:?}", clip_item);
        if self.history.len() == HISTORY_LEN {
            self.history.pop_back();
        }
        self.history.push_front(clip_item);
    }

    pub fn copy(&mut self, id: u32, app: &AppHandle) {
        let item = self.history.iter().enumerate().find(|(_i, f)| id == f.id());
        if let Some((index, item)) = item {
            match item {
                ClipItem::FilePath { paths, .. } => match clipboard_files::write(paths) {
                    Err(e) => log::error!("Error writing file paths to clipboard: {}", e),
                    Ok(_) => log::info!("Successfully wrote file paths to clipboard"),
                },
                ClipItem::Image { image, .. } => match app.clipboard().write_image(image) {
                    Err(e) => log::error!("Error writing image to clipboard: {}", e),
                    Ok(_) => log::info!("Successfully wrote image to clipboard"),
                },
                ClipItem::Text { text, .. } => match app.clipboard().write_text(text) {
                    Err(e) => log::error!("Error writing image to clipboard: {}", e),
                    Ok(_) => log::info!("Successfully wrote text to clipboard"),
                },
            }
            self.history.remove(index);
        }
    }
}

fn create_base64_thumbnail(image: &Image) -> Result<String, String> {
    let width = image.width();
    let height = image.height();

    let mut buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, Vec::from(image.rgba()))
            .ok_or("Could not convert provided image to an image buffer")?;

    if height > THUMBNAIL_HEIGHT {
        let new_height = THUMBNAIL_HEIGHT;
        let new_width = ((width as f32 / height as f32) * THUMBNAIL_HEIGHT as f32).round() as u32;
        buffer = thumbnail(&buffer, new_width, new_height);
    }

    let mut encoded = Cursor::new(Vec::new());
    buffer
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(|e| format!("Could not convert image to png: {}", e))?;

    Ok(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(encoded.into_inner())
    ))
}
