use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey_string: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey_string: "ctrl+shift+v".to_string(),
        }
    }
}

impl AppSettings {
    fn get_settings_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "nikkicho", "clip") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("settings.json")
        } else {
            PathBuf::from("nikkicho_clip_settings.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::get_settings_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::get_settings_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(&path, data).ok();
        }
    }
}
