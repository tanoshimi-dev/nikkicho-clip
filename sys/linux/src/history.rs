use crate::clip_entry::{ClipContent, ClipEntry};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_SIZE: usize = 500;

pub struct ClipHistory {
    pub entries: Vec<ClipEntry>,
    storage_path: PathBuf,
    pub max_size: usize,
}

impl ClipHistory {
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        let entries = Self::load_from_disk(&storage_path);
        Self {
            entries,
            storage_path,
            max_size: MAX_HISTORY_SIZE,
        }
    }

    fn get_storage_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "nikkicho", "clip") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("history.json")
        } else {
            PathBuf::from("clipboard_history.json")
        }
    }

    fn load_from_disk(path: &PathBuf) -> Vec<ClipEntry> {
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn save_to_disk(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
            fs::write(&self.storage_path, data).ok();
        }
    }

    pub fn add_text(&mut self, text: String) -> bool {
        if text.trim().is_empty() {
            return false;
        }
        // Deduplicate: if the same text is already the most recent, skip
        if let Some(last) = self.entries.first() {
            if let ClipContent::Text(ref t) = last.content {
                if t == &text {
                    return false;
                }
            }
        }
        let entry = ClipEntry::new_text(text);
        self.entries.insert(0, entry);
        self.enforce_max_size();
        self.save_to_disk();
        true
    }

    pub fn add_image(&mut self, width: u32, height: u32, png_data: Vec<u8>) -> bool {
        let entry = ClipEntry::new_image(width, height, png_data);
        self.entries.insert(0, entry);
        self.enforce_max_size();
        self.save_to_disk();
        true
    }

    fn enforce_max_size(&mut self) {
        // Keep pinned/favorited items, remove oldest non-pinned if over limit
        while self.entries.len() > self.max_size {
            // Find last non-pinned entry to remove
            if let Some(pos) = self
                .entries
                .iter()
                .rposition(|e| !e.pinned && !e.favorite)
            {
                self.entries.remove(pos);
            } else {
                break; // All are pinned/favorited
            }
        }
    }

    pub fn toggle_pin(&mut self, id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.pinned = !entry.pinned;
            self.save_to_disk();
        }
    }

    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.favorite = !entry.favorite;
            self.save_to_disk();
        }
    }

    pub fn delete(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
        self.save_to_disk();
    }

    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|e| e.pinned || e.favorite);
        self.save_to_disk();
    }

    pub fn search(&self, query: &str) -> Vec<&ClipEntry> {
        self.entries
            .iter()
            .filter(|e| e.matches_search(query))
            .collect()
    }
}
