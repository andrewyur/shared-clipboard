use crate::clipboard_files;
use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_STANDARD, Engine};
use image::{imageops::thumbnail, ImageBuffer, ImageFormat, Rgba};
use serde::ser::SerializeStruct;
use serde_json::json;
use std::fs;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::{io::Cursor, path::PathBuf};
use tauri::Manager;
use tauri::{image::Image, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_store::StoreExt;

const THUMBNAIL_HEIGHT: u32 = 300;
const PINNED_STORE: &str = "pinned.json";
static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub enum Contents {
    FilePath {
        paths: Vec<PathBuf>,
        id: u32,
    },
    Image {
        thumbnail: String,
        image: Image<'static>,
        id: u32,
    },
    Text {
        text: String,
        id: u32
    },
}

impl Contents {
    pub fn try_from_clipboard(app: &AppHandle) -> Option<Self> {
        let (tx, rx) = mpsc::channel();
        let app_clone = app.clone();

        if let Err(e) = app.run_on_main_thread(move || {
            let _ = tx.send(Self::_try_from_clipboard(&app_clone));
        }) {
            log::error!(
                "Could not run 'Item::_try_from_clipboard' on main thread: {}",
                e
            );
            return None;
        }

        match rx.recv() {
            Err(e) => {
                log::error!(
                    "Error recieving value from main thread in 'Item::_try_from_clipboard': {}",
                    e
                );
                None
            }
            Ok(res) => res,
        }
    }

    fn _try_from_clipboard(app: &AppHandle) -> Option<Self> {
        log::info!("Clipboard handler checking clipboard");
        match clipboard_files::read() {
            Ok(paths) => return Some(Self::FilePath { paths, id: NEXT_ID.fetch_add(1, Ordering::Relaxed) }),
            Err(clipboard_files::ClipboardError::NoFiles) => {}
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
                    return Some(Self::Image {
                        thumbnail,
                        image: image.to_owned(),
                        id: NEXT_ID.fetch_add(1, Ordering::Relaxed) 
                    });
                }
            }
            // no way to tell if this is because the clipboard has no images in it or because an actual error occured...
            Err(e) => {
                log::warn!("Possible error pasting image from clipboard: {e}");
            }
        }

        match app.clipboard().read_text() {
            Ok(text) => return Some(Self::Text { text, id: NEXT_ID.fetch_add(1, Ordering::Relaxed) }),
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

    pub fn id(&self) -> u32 {
        match self {
            Contents::FilePath { id, .. } => *id,
            Contents::Image { id, .. } => *id,
            Contents::Text { id , ..} => *id
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
            Self::Image { thumbnail, id, .. } => {
                s.serialize_field("content", thumbnail)?;
                s.serialize_field("kind", "image")?;
                s.serialize_field("id", id)?;
            }
            Self::FilePath { paths, id } => {
                s.serialize_field("content", paths)?;
                s.serialize_field("kind", "paths")?;
                s.serialize_field("id", id)?;
            }
            Self::Text { text, id } => {
                s.serialize_field("content", text)?;
                s.serialize_field("kind", "text")?;
                s.serialize_field("id", id)?;
            }
        };
        s.end()
    }
}

impl PartialEq for Contents {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Contents::FilePath { paths: p1, .. }, Contents::FilePath { paths: p2, .. }) => p1 == p2,
            (Contents::Image { thumbnail: t1, .. }, Contents::Image { thumbnail: t2, .. }) => t1 == t2,
            (Contents::Text { text: t1, .. }, Contents::Text { text: t2, .. }) => t1 == t2,
            _ => false
        }
    }
}

impl Eq for Contents {}

impl Hash for Contents {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Contents::FilePath { paths , ..} => paths.hash(state),
            Contents::Image { thumbnail, .. } => thumbnail.hash(state),
            Contents::Text { text , ..} => text.hash(state),
        }
    }
}

fn create_base64_thumbnail(image: &Image) -> Result<String, anyhow::Error> {
    let width = image.width();
    let height = image.height();

    let mut buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, Vec::from(image.rgba())).ok_or(anyhow!(
            "Could not convert provided image to an image buffer"
        ))?;

    if height > THUMBNAIL_HEIGHT {
        let new_height = THUMBNAIL_HEIGHT;
        let new_width = ((width as f32 / height as f32) * THUMBNAIL_HEIGHT as f32).round() as u32;
        buffer = thumbnail(&buffer, new_width, new_height);
    }

    let mut encoded = Cursor::new(Vec::new());
    buffer
        .write_to(&mut encoded, ImageFormat::Png)
        .with_context(|| anyhow!("Could not convert image to png"))?;

    Ok(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(encoded.into_inner())
    ))
}

pub fn store_pinned(contents: &[Arc<Contents>], app: &AppHandle) -> Result<(), anyhow::Error> {
    let store = app
        .store(PINNED_STORE)
        .with_context(|| "failed to get pinned items store")?;
    log::debug!("created store");

    let mut images_directory = app
        .path()
        .app_local_data_dir()
        .with_context(|| "failed to get app local data dir")?;

    images_directory.push("images");

    if images_directory.exists() {
        log::debug!("clearing images dir ");
        fs::remove_dir_all(&images_directory).with_context(|| {
            format!(
                "failed clear the image data dir at {}",
                images_directory.display()
            )
        })?;
    }
    log::debug!("creating image data dir ");
    fs::create_dir_all(&images_directory).with_context(|| {
        format!(
            "failed create the image data dir at {}",
            images_directory.display()
        )
    })?;

    let mut image_file_name = 0;

    let serialized = contents
        .iter()
        .map(|c| match c.as_ref() {
            Contents::FilePath { paths , ..} => Ok(json!({"type": "paths", "content": paths})),
            Contents::Text { text, ..} => Ok(json!({"type": "text", "content": text})),
            Contents::Image { image, .. } => {
                image_file_name += 1;
                let mut image_path = images_directory.clone();
                image_path.push(image_file_name.to_string());
                log::debug!("writing image data at {}", &image_path.display());
                fs::write(&image_path, image.rgba()).with_context(|| {
                    format!("failed to write image data to {}", image_path.display())
                })?;
                Ok(json!({ "type": "image", "content": {
                    "file": image_file_name.to_string(),
                    "height": image.height(),
                    "width": image.width(),
                }}))
            }
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    log::debug!("storing pinned data");
    store.set("pinned", serialized);

    Ok(())
}

pub fn load_pinned(app: &AppHandle) -> Result<Vec<Contents>, anyhow::Error> {
    let store = app
        .store(PINNED_STORE)
        .with_context(|| "failed to get or create pinned items store")?;

    let mut images_directory = app
        .path()
        .app_local_data_dir()
        .with_context(|| "failed to get app local data dir")?;

    images_directory.push("images");

    if let Some(pinned) = store.get("pinned") {
        store.close_resource();
        let items = pinned
            .as_array()
            .ok_or_else(|| anyhow!("Value for pinned key was not an array"))?;
        items.iter().map(|i| {
                let json_obj = i.as_object().ok_or_else(|| anyhow!("item in pinned array was not object"))?;
                let type_str = json_obj
                    .get("type").ok_or_else(|| anyhow!("pinned array object did not have 'type' key"))?
                    .as_str().ok_or_else(|| anyhow!("value for 'type' key in pinned array object was not a string"))?;
                let content_obj = json_obj.get("content").ok_or_else(|| anyhow!("pinned array object did not have 'content' key"))?;
                match type_str {
                    "text" => {
                        let text = content_obj.as_str().ok_or_else(|| anyhow!("Value for 'content' was not a string for 'text' item"))?.to_string();
                        Ok(Contents::Text { text, id: NEXT_ID.fetch_add(1, Ordering::Relaxed)  })
                    },
                    "paths" => {
                        let path_arr = content_obj.as_array().ok_or_else(|| anyhow!("Value for 'content' was not an array for 'paths' item"))?;
                        let paths = path_arr.iter().map(|p|p.as_str().map(PathBuf::from)).collect::<Option<Vec<_>>>().ok_or_else(|| anyhow!("Not all items in paths array were strings"))?;
                        Ok(Contents::FilePath { paths, id: NEXT_ID.fetch_add(1, Ordering::Relaxed)  })
                    },
                    "image" => {
                        let image_data_obj = content_obj.as_object().ok_or_else(|| anyhow!("Value for 'content' was not an object for 'image' item"))?;
                        let file_name = image_data_obj
                            .get("file").ok_or_else(|| anyhow!("Image data object did not have 'file' key"))?
                            .as_str().ok_or_else(|| anyhow!("Value for 'file' key for image data object was not a string"))?;
                        let height = image_data_obj
                            .get("height").ok_or_else(|| anyhow!("Image data object did not have a 'height' key"))?
                            .as_u64().ok_or_else(|| anyhow!("Value for 'height' in image data object could not be cast as a u64"))?;
                        let width = image_data_obj
                            .get("width").ok_or_else(|| anyhow!("Image data object did not have a 'width' key"))?
                            .as_u64().ok_or_else(|| anyhow!("Value for 'width' in image data object could not be cast as a u64"))?;

                        let mut image_path = images_directory.clone();
                        image_path.push(file_name);
                        let rgba = fs::read(&image_path).with_context(|| format!("Failed to read image data from {}", image_path.display()))?;
                        let image = Image::new_owned(rgba, width as u32, height as u32);
                        let thumbnail = create_base64_thumbnail(&image)?;
                        Ok(Contents::Image { thumbnail, image, id: NEXT_ID.fetch_add(1, Ordering::Relaxed)  })
                    },
                    _ => Err(anyhow!("type for pinned object not 'image', 'paths', or 'text'"))
                }
            }).collect::<Result<Vec<Contents>, anyhow::Error>>()
    } else {
        store.close_resource();
        Ok(vec![])
    }
}
