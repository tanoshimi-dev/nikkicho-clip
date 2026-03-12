mod app;
mod clip_entry;
mod history;
mod monitor;

use app::NikkichoClipApp;
use eframe::egui;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

fn main() -> eframe::Result<()> {
    // Register global hotkey: Ctrl+Shift+V
    let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let hotkey = HotKey::from_str("ctrl+shift+v").expect("Failed to parse hotkey");
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey Ctrl+Shift+V");

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

    eframe::run_native(
        "Nikkicho Clip",
        options,
        Box::new(move |cc| {
            // Set up dark theme
            let mut visuals = egui::Visuals::dark();
            visuals.window_corner_radius = egui::CornerRadius::same(8);
            cc.egui_ctx.set_visuals(visuals);

            // Shared visibility state for hotkey toggle
            let visible = Arc::new(AtomicBool::new(true));

            // Handle hotkey events - toggle show/hide
            let ctx = cc.egui_ctx.clone();
            let hotkey_id = hotkey.id();
            let visible_hotkey = Arc::clone(&visible);
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                    if event.id() == hotkey_id && event.state() == HotKeyState::Pressed {
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

            Ok(Box::new(NikkichoClipApp::new(cc)))
        }),
    )
}
