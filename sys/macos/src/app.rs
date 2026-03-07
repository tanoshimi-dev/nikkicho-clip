use crate::clip_entry::ClipContent;
use crate::history::ClipHistory;
use crate::monitor::{self, ClipEvent};
use arboard::Clipboard;
use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc;

pub struct NikkichoClipApp {
    history: ClipHistory,
    clip_rx: mpsc::Receiver<ClipEvent>,
    search_query: String,
    show_favorites_only: bool,
    image_textures: HashMap<String, egui::TextureHandle>,
    status_message: Option<(String, std::time::Instant)>,
}

impl NikkichoClipApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let clip_rx = monitor::start_monitor();
        Self {
            history: ClipHistory::new(),
            clip_rx,
            search_query: String::new(),
            show_favorites_only: false,
            image_textures: HashMap::new(),
            status_message: None,
        }
    }

    fn process_clipboard_events(&mut self) {
        while let Ok(event) = self.clip_rx.try_recv() {
            match event {
                ClipEvent::Text(text) => {
                    self.history.add_text(text);
                }
                ClipEvent::Image {
                    width,
                    height,
                    png_data,
                } => {
                    self.history.add_image(width, height, png_data);
                }
            }
        }
    }

    fn copy_to_clipboard(&mut self, entry_id: &str) {
        let entry = self.history.entries.iter().find(|e| e.id == entry_id);
        if let Some(entry) = entry {
            let mut clipboard = match Clipboard::new() {
                Ok(c) => c,
                Err(_) => return,
            };
            match &entry.content {
                ClipContent::Text(text) => {
                    if clipboard.set_text(text.clone()).is_ok() {
                        self.status_message =
                            Some(("Copied to clipboard!".into(), std::time::Instant::now()));
                    }
                }
                ClipContent::Image {
                    width,
                    height,
                    png_base64,
                } => {
                    use base64::Engine;
                    if let Ok(png_data) =
                        base64::engine::general_purpose::STANDARD.decode(png_base64)
                    {
                        if let Ok(img) = image::load_from_memory(&png_data) {
                            let rgba = img.to_rgba8();
                            let img_data = arboard::ImageData {
                                width: *width as usize,
                                height: *height as usize,
                                bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                            };
                            if clipboard.set_image(img_data).is_ok() {
                                self.status_message = Some((
                                    "Image copied to clipboard!".into(),
                                    std::time::Instant::now(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_or_load_texture(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        png_base64: &str,
    ) -> Option<egui::TextureId> {
        if let Some(handle) = self.image_textures.get(id) {
            return Some(handle.id());
        }

        use base64::Engine;
        let png_data = base64::engine::general_purpose::STANDARD
            .decode(png_base64)
            .ok()?;
        let img = image::load_from_memory(&png_data).ok()?;
        let rgba = img.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
        let handle = ctx.load_texture(id, color_image, egui::TextureOptions::default());
        let tex_id = handle.id();
        self.image_textures.insert(id.to_string(), handle);
        Some(tex_id)
    }
}

impl eframe::App for NikkichoClipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_clipboard_events();

        // Request repaint periodically to check for new clipboard content
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // Top panel with search and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Nikkicho Clip");
                ui.separator();
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Clear search").clicked() {
                    self.search_query.clear();
                }
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_favorites_only, "Favorites only");
                ui.separator();
                ui.label(format!("{} items", self.history.entries.len()));
                ui.separator();
                if ui.button("Clear all (keep pinned)").clicked() {
                    self.history.clear_unpinned();
                    self.image_textures.clear();
                }
            });

            // Status message
            if let Some((msg, time)) = &self.status_message {
                if time.elapsed() < std::time::Duration::from_secs(2) {
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 80), msg);
                } else {
                    self.status_message = None;
                }
            }
            ui.add_space(4.0);
        });

        // Main content - scrollable list of clipboard entries
        egui::CentralPanel::default().show(ctx, |ui| {
            let entries: Vec<_> = self
                .history
                .entries
                .iter()
                .filter(|e| e.matches_search(&self.search_query))
                .filter(|e| !self.show_favorites_only || e.favorite)
                .cloned()
                .collect();

            if entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No clipboard entries yet. Copy something!");
                });
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let mut action: Option<EntryAction> = None;

                    for entry in &entries {
                        let is_pinned = entry.pinned;
                        let is_fav = entry.favorite;

                        let frame = egui::Frame::group(ui.style())
                            .inner_margin(8.0)
                            .outer_margin(egui::Margin::symmetric(0, 2));

                        if is_pinned {
                            let frame = frame.fill(egui::Color32::from_rgb(40, 45, 55));
                            frame.show(ui, |ui| {
                                Self::render_entry_ui(ui, entry, is_pinned, is_fav, &mut action);
                            });
                        } else {
                            frame.show(ui, |ui| {
                                Self::render_entry_ui(ui, entry, is_pinned, is_fav, &mut action);
                            });
                        }
                    }

                    // Process actions after iteration
                    if let Some(act) = action {
                        match act {
                            EntryAction::Copy(id) => self.copy_to_clipboard(&id),
                            EntryAction::Pin(id) => self.history.toggle_pin(&id),
                            EntryAction::Favorite(id) => self.history.toggle_favorite(&id),
                            EntryAction::Delete(id) => {
                                self.image_textures.remove(&id);
                                self.history.delete(&id);
                            }
                        }
                    }
                });

            // Load image textures for visible entries
            let to_load: Vec<_> = self
                .history
                .entries
                .iter()
                .filter_map(|entry| {
                    if let ClipContent::Image { png_base64, .. } = &entry.content {
                        if !self.image_textures.contains_key(&entry.id) {
                            return Some((entry.id.clone(), png_base64.clone()));
                        }
                    }
                    None
                })
                .collect();
            for (id, png_base64) in to_load {
                self.get_or_load_texture(ctx, &id, &png_base64);
            }
        });
    }
}

enum EntryAction {
    Copy(String),
    Pin(String),
    Favorite(String),
    Delete(String),
}

impl NikkichoClipApp {
    fn render_entry_ui(
        ui: &mut egui::Ui,
        entry: &crate::clip_entry::ClipEntry,
        is_pinned: bool,
        is_fav: bool,
        action: &mut Option<EntryAction>,
    ) {
        ui.horizontal(|ui| {
            // Timestamp
            ui.label(
                egui::RichText::new(entry.timestamp.format("%m/%d %H:%M").to_string())
                    .small()
                    .color(egui::Color32::GRAY),
            );

            // Badges
            if is_pinned {
                ui.label(egui::RichText::new("[PIN]").small().strong());
            }
            if is_fav {
                ui.label(
                    egui::RichText::new("[FAV]")
                        .small()
                        .color(egui::Color32::from_rgb(255, 200, 50)),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(egui::RichText::new("X").color(egui::Color32::from_rgb(200, 80, 80)))
                    .on_hover_text("Delete")
                    .clicked()
                {
                    *action = Some(EntryAction::Delete(entry.id.clone()));
                }
                let fav_label = if is_fav { "Unfav" } else { "Fav" };
                if ui.button(fav_label).on_hover_text("Toggle favorite").clicked() {
                    *action = Some(EntryAction::Favorite(entry.id.clone()));
                }
                let pin_label = if is_pinned { "Unpin" } else { "Pin" };
                if ui.button(pin_label).on_hover_text("Toggle pin").clicked() {
                    *action = Some(EntryAction::Pin(entry.id.clone()));
                }
                if ui.button("Copy").on_hover_text("Copy to clipboard").clicked() {
                    *action = Some(EntryAction::Copy(entry.id.clone()));
                }
            });
        });

        // Content preview
        match &entry.content {
            ClipContent::Text(text) => {
                let preview = if text.chars().count() > 300 {
                    let truncated: String = text.chars().take(300).collect();
                    format!("{}...", truncated)
                } else {
                    text.clone()
                };
                ui.label(&preview);
            }
            ClipContent::Image {
                width, height, ..
            } => {
                ui.label(format!("Image ({}x{})", width, height));
            }
        }
    }
}
