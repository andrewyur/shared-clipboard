use serde::ser::SerializeStruct;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};
use tauri::image::Image;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub enum Item {
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
        id: u32,
    },
}

pub trait HasId {
    fn id(&self) -> u32;
}

impl Item {
    pub fn new_file_path(paths: Vec<PathBuf>) -> Self {
        Self::FilePath {
            paths,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
    pub fn new_image(thumbnail: String, image: Image<'static>) -> Self {
        Self::Image {
            thumbnail,
            image,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
    pub fn new_text(text: String) -> Self {
        Self::Text {
            text,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl HasId for Item {
    fn id(&self) -> u32 {
        match self {
            Self::FilePath { id, .. } | Self::Image { id, .. } | Self::Text { id, .. } => *id,
        }
    }
}

impl serde::Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ClipItem", 3)?;
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
        s.serialize_field("id", &self.id())?;
        s.end()
    }
}
