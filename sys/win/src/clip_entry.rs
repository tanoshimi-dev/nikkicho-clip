use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClipContent {
    Text(String),
    Image {
        width: u32,
        height: u32,
        png_base64: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipEntry {
    pub id: String,
    pub content: ClipContent,
    pub timestamp: DateTime<Local>,
    pub pinned: bool,
    pub favorite: bool,
}

impl ClipEntry {
    pub fn new_text(text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: ClipContent::Text(text),
            timestamp: Local::now(),
            pinned: false,
            favorite: false,
        }
    }

    pub fn new_image(width: u32, height: u32, png_data: Vec<u8>) -> Self {
        use base64::Engine;
        Self {
            id: Uuid::new_v4().to_string(),
            content: ClipContent::Image {
                width,
                height,
                png_base64: base64::engine::general_purpose::STANDARD.encode(&png_data),
            },
            timestamp: Local::now(),
            pinned: false,
            favorite: false,
        }
    }

    pub fn preview_text(&self) -> String {
        match &self.content {
            ClipContent::Text(t) => {
                if t.chars().count() > 200 {
                    let truncated: String = t.chars().take(200).collect();
                    format!("{}...", truncated)
                } else {
                    t.clone()
                }
            }
            ClipContent::Image { width, height, .. } => {
                format!("[Image {}x{}]", width, height)
            }
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        match &self.content {
            ClipContent::Text(t) => t.to_lowercase().contains(&q),
            ClipContent::Image { .. } => "image".contains(&q),
        }
    }
}
