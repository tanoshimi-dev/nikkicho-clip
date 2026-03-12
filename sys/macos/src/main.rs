mod app;
mod clip_entry;
mod history;
mod monitor;
mod settings;

use app::NikkichoClipApp;
use eframe::egui;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use settings::AppSettings;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

fn main() -> eframe::Result<()> {
    // Load settings
    let settings = AppSettings::load();

    // Register global hotkey from settings
    let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let hotkey = HotKey::from_str(&settings.hotkey_string).expect("Failed to parse hotkey");
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey");

    let hotkey_manager = Arc::new(Mutex::new(hotkey_manager));
    let current_hotkey_id = Arc::new(AtomicU32::new(hotkey.id()));

    // Build system tray
    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Show", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&show_item).ok();
    tray_menu.append(&quit_item).ok();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Nikkicho Clip - Clipboard History")
        .build()
        .ok();

    let show_item_id = show_item.id().clone();
    let quit_item_id = quit_item.id().clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_min_inner_size([350.0, 400.0])
            .with_title("Nikkicho Clip"),
        ..Default::default()
    };

    let hotkey_manager_app = Arc::clone(&hotkey_manager);
    let current_hotkey_id_app = Arc::clone(&current_hotkey_id);

    eframe::run_native(
        "Nikkicho Clip",
        options,
        Box::new(move |cc| {
            // Set up dark theme
            let mut visuals = egui::Visuals::dark();
            visuals.window_corner_radius = egui::CornerRadius::same(8);
            cc.egui_ctx.set_visuals(visuals);

            // Load Japanese font (Hiragino Sans W3) as fallback
            let mut fonts = egui::FontDefinitions::default();
            let font_paths = [
                "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            ];
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "japanese_font".to_owned(),
                        egui::FontData::from_owned(font_data).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("japanese_font".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("japanese_font".to_owned());
                    break;
                }
            }
            cc.egui_ctx.set_fonts(fonts);

            // Shared visibility state for hotkey toggle
            let visible = Arc::new(AtomicBool::new(true));

            // Handle hotkey events - toggle show/hide
            let ctx = cc.egui_ctx.clone();
            let current_id = Arc::clone(&current_hotkey_id);
            let visible_hotkey = Arc::clone(&visible);
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                    let expected_id = current_id.load(Ordering::SeqCst);
                    if event.id() == expected_id && event.state() == HotKeyState::Pressed {
                        let is_visible = visible_hotkey.load(Ordering::SeqCst);
                        if is_visible {
                            visible_hotkey.store(false, Ordering::SeqCst);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        } else {
                            visible_hotkey.store(true, Ordering::SeqCst);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        }
                        ctx.request_repaint();
                    }
                }
            });

            // Handle tray menu events
            let ctx2 = cc.egui_ctx.clone();
            let visible_tray = Arc::clone(&visible);
            std::thread::spawn(move || loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id() == &show_item_id {
                        visible_tray.store(true, Ordering::SeqCst);
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx2.request_repaint();
                    } else if event.id() == &quit_item_id {
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

            Ok(Box::new(NikkichoClipApp::new(
                cc,
                settings,
                hotkey_manager_app,
                current_hotkey_id_app,
            )))
        }),
    )
}
