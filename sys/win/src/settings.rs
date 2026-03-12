use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey_modifiers: HotkeyModifiers,
    pub hotkey_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey_modifiers: HotkeyModifiers {
                ctrl: true,
                shift: true,
                alt: false,
                super_key: false,
            },
            hotkey_key: "V".to_string(),
        }
    }
}

impl AppSettings {
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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(&path, data).ok();
        }
    }

    fn get_settings_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "nikkicho", "clip") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    /// Build the hotkey string for `global-hotkey` crate (e.g. "ctrl+shift+v")
    pub fn hotkey_string(&self) -> String {
        let mut parts = Vec::new();
        if self.hotkey_modifiers.ctrl {
            parts.push("ctrl");
        }
        if self.hotkey_modifiers.shift {
            parts.push("shift");
        }
        if self.hotkey_modifiers.alt {
            parts.push("alt");
        }
        if self.hotkey_modifiers.super_key {
            parts.push("super");
        }
        parts.push(&self.hotkey_key);
        parts.join("+").to_lowercase()
    }

    /// Human-readable display of the hotkey
    pub fn hotkey_display(&self) -> String {
        let mut parts = Vec::new();
        if self.hotkey_modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.hotkey_modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.hotkey_modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.hotkey_modifiers.super_key {
            parts.push("Win".to_string());
        }
        parts.push(self.hotkey_key.to_uppercase());
        parts.join(" + ")
    }
}

/// Keys available for hotkey binding
pub const AVAILABLE_KEYS: &[&str] = &[
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
];
