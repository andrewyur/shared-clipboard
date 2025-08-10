use base64::{prelude::BASE64_STANDARD, Engine};
use image::{imageops::thumbnail, ImageBuffer, ImageFormat, Rgba};
use serde::ser::SerializeStruct;
use tauri_plugin_clipboard_manager::ClipboardExt;
use std::{io::Cursor, path::PathBuf};
use tauri::{image::Image, AppHandle};

const THUMBNAIL_HEIGHT: u32 = 300;

#[derive(Debug, Clone)]
pub enum Contents {
    FilePath {
        paths: Vec<PathBuf>,
    },
    Image {
        thumbnail: String,
        image: Image<'static>,
    },
    Text {
        text: String,
    },
}

impl Contents {
    pub fn try_from_clipboard(app: &AppHandle) -> Option<Self> {
        log::info!("Clibboard handler checking clibpoard");
        match clipboard_files::read() {
            Ok(paths) => {
                return Some(Self::FilePath { paths })
            }
            Err(clipboard_files::ClipboardError::NoFiles) => {},
            Err(e) => {
                log::error!(
                    "System returned an error when reading file paths from clipboard: {}",
                    e
                );
            }
        } 

        match app.clipboard().read_image() {
            Ok(image) => {
                if let Ok(thumbnail) = create_base64_thumbnail(&image) {
                    return Some(Self::Image { thumbnail, image: image.to_owned() })
                }
            },
            // no way to tell if this is because the clipboard has no images in it or because an actual error occured...
            Err(e) => {
                log::warn!("Possible error pasting image from clipboard: {e}");
            }
        }

        match app.clipboard().read_text() {
            Ok(text) => {
                return Some(Self::Text { text })
            }
            Err(e) => {
                log::warn!("Possible error pasting text from clipboard: {e}");
            }
        }

        None
    }
    pub fn try_to_clipboard(&self, app: &AppHandle) {
        match self {
            Contents::FilePath { paths, .. } => match clipboard_files::write(paths) {
                Err(e) => log::error!("Error writing file paths to clipboard: {}", e),
                Ok(_) => log::info!("Successfully wrote file paths to clipboard"),
            },
            Contents::Image { image, .. } => match app.clipboard().write_image(image) {
                Err(e) => log::error!("Error writing image to clipboard: {}", e),
                Ok(_) => log::info!("Successfully wrote image to clipboard"),
            },
            Contents::Text { text, .. } => match app.clipboard().write_text(text) {
                Err(e) => log::error!("Error writing image to clipboard: {}", e),
                Ok(_) => log::info!("Successfully wrote text to clipboard"),
            },
        }
    }
}

impl serde::Serialize for Contents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ClipItem", 2)?;
        match self {
            Self::Image { thumbnail, .. } => {
                s.serialize_field("content", thumbnail)?;
                s.serialize_field("kind", "image")?;
            }
            Self::FilePath { paths, .. } => {
                s.serialize_field("content", paths)?;
                s.serialize_field("kind", "paths")?;
            }
            Self::Text { text, .. } => {
                s.serialize_field("content", text)?;
                s.serialize_field("kind", "text")?;
            }
        };
        s.end()
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
