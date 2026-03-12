// Prevent console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clip_entry;
mod history;
mod monitor;
mod settings;

use app::NikkichoClipApp;
use eframe::egui;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
};

/// Create a 32x32 RGBA tray icon (clipboard shape)
fn create_tray_icon() -> Icon {
    const S: usize = 32;
    let mut rgba = vec![0u8; S * S * 4];

    for y in 0..S {
        for x in 0..S {
            let i = (y * S + x) * 4;

            // Clipboard body: rounded rectangle (6..26 x 8..30)
            let in_body = x >= 6 && x <= 25 && y >= 8 && y <= 29;

            // Clip at top center (12..20 x 3..12)
            let in_clip = x >= 12 && x <= 19 && y >= 3 && y <= 11;

            // Clip hole (14..18 x 3..7)
            let in_clip_hole = x >= 14 && x <= 17 && y >= 3 && y <= 6;

            // Text lines on clipboard
            let in_line = in_body
                && x >= 9
                && x <= 22
                && (y == 15 || y == 19 || y == 23);

            if in_line {
                // White text lines
                rgba[i] = 255;
                rgba[i + 1] = 255;
                rgba[i + 2] = 255;
                rgba[i + 3] = 255;
            } else if in_clip_hole {
                // Transparent hole in clip
                rgba[i + 3] = 0;
            } else if in_clip {
                // Clip: lighter color
                rgba[i] = 140;
                rgba[i + 1] = 180;
                rgba[i + 2] = 230;
                rgba[i + 3] = 255;
            } else if in_body {
                // Body: blue-ish
                rgba[i] = 70;
                rgba[i + 1] = 130;
                rgba[i + 2] = 210;
                rgba[i + 3] = 255;
            }
            // else: transparent (alpha = 0)
        }
    }

    Icon::from_rgba(rgba, S as u32, S as u32).expect("Failed to create tray icon")
}

fn find_app_window() -> Option<HWND> {
    let title: Vec<u16> = "Nikkicho Clip\0".encode_utf16().collect();
    unsafe {
        FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr()))
            .ok()
            .filter(|h| *h != HWND::default())
    }
}

/// Find the app window by title and toggle its visibility using Win32 API.
pub fn toggle_window(visible: &AtomicBool) {
    if let Some(hwnd) = find_app_window() {
        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_HIDE);
                visible.store(false, Ordering::SeqCst);
            } else {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
                visible.store(true, Ordering::SeqCst);
            }
        }
    }
}

/// Show the app window using Win32 API.
pub fn show_window(visible: &AtomicBool) {
    if let Some(hwnd) = find_app_window() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            visible.store(true, Ordering::SeqCst);
        }
    }
}

/// Hide the app window using Win32 API.
pub fn hide_window(visible: &AtomicBool) {
    if let Some(hwnd) = find_app_window() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
            visible.store(false, Ordering::SeqCst);
        }
    }
}

fn main() -> eframe::Result<()> {
    // Load user settings and register global hotkey
    let settings = settings::AppSettings::load();
    let hotkey_str = settings.hotkey_string();
    let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let hotkey = HotKey::from_str(&hotkey_str)
        .unwrap_or_else(|_| HotKey::from_str("ctrl+shift+v").expect("Failed to parse fallback hotkey"));
    hotkey_manager
        .register(hotkey)
        .unwrap_or_else(|_| eprintln!("Failed to register hotkey: {}", hotkey_str));

    // Build system tray
    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Show", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&show_item).ok();
    tray_menu.append(&quit_item).ok();

    let tray_icon_image = create_tray_icon();
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(tray_icon_image)
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

    // Shared state
    let visible = Arc::new(AtomicBool::new(true));
    let quit_requested = Arc::new(AtomicBool::new(false));

    eframe::run_native(
        "Nikkicho Clip",
        options,
        Box::new(move |cc| {
            // Set up dark theme
            let mut visuals = egui::Visuals::dark();
            visuals.window_corner_radius = egui::CornerRadius::same(8);
            cc.egui_ctx.set_visuals(visuals);

            // Handle hotkey events - toggle show/hide
            let ctx = cc.egui_ctx.clone();
            let hotkey_id = hotkey.id();
            let visible_hotkey = visible.clone();
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                    if event.id() == hotkey_id {
                        toggle_window(&visible_hotkey);
                        ctx.request_repaint();
                    }
                }
            });

            // Handle tray menu events
            let ctx2 = cc.egui_ctx.clone();
            let visible_tray = visible.clone();
            let quit_tray = quit_requested.clone();
            std::thread::spawn(move || loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id() == &show_item_id {
                        show_window(&visible_tray);
                        ctx2.request_repaint();
                    } else if event.id() == &quit_item_id {
                        quit_tray.store(true, Ordering::SeqCst);
                        ctx2.request_repaint();
                    }
                }
            });

            Ok(Box::new(NikkichoClipApp::new(cc, visible, quit_requested)))
        }),
    )
}
