mod app;
mod clip_entry;
mod history;
mod monitor;
mod settings;

use app::NikkichoClipApp;
use eframe::egui;
use settings::AppSettings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

/// Create a simple 32x32 RGBA icon programmatically
fn create_tray_icon() -> Icon {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let in_body = x >= 6 && x <= 25 && y >= 6 && y <= 28;
            let in_clip = x >= 11 && x <= 20 && y >= 2 && y <= 8;

            if in_body || in_clip {
                rgba[idx] = 80;      // R
                rgba[idx + 1] = 200; // G
                rgba[idx + 2] = 80;  // B
                rgba[idx + 3] = 255; // A
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

/// Register a GNOME custom keyboard shortcut via dconf/gsettings
fn register_gnome_shortcut(hotkey: &str) {
    let binding = hotkey_to_gnome_binding(hotkey);

    // Check existing custom keybindings
    let existing = std::process::Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output();

    let path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/nikkicho-clip/";

    if let Ok(output) = existing {
        let current = String::from_utf8_lossy(&output.stdout).to_string();

        // Add our path if not already present
        if !current.contains("nikkicho-clip") {
            let mut paths: Vec<String> = if current.trim() == "@as []" || current.trim().is_empty()
            {
                vec![]
            } else {
                // Parse existing paths from ['path1', 'path2'] format
                current
                    .trim()
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(|s| s.trim().trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            paths.push(path.to_string());

            let paths_str = format!(
                "[{}]",
                paths
                    .iter()
                    .map(|p| format!("'{}'", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            let _ = std::process::Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.settings-daemon.plugins.media-keys",
                    "custom-keybindings",
                    &paths_str,
                ])
                .output();
        }
    }

    // Set the shortcut properties
    let schema = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
    let schema_path = format!("{}:{}", schema, path);

    let _ = std::process::Command::new("gsettings")
        .args([
            "set",
            &schema_path,
            "name",
            "Nikkicho Clip Toggle",
        ])
        .output();

    let _ = std::process::Command::new("gsettings")
        .args([
            "set",
            &schema_path,
            "command",
            "pkill -USR1 nikkicho_clip",
        ])
        .output();

    let _ = std::process::Command::new("gsettings")
        .args(["set", &schema_path, "binding", &binding])
        .output();

    eprintln!("GNOME shortcut registered: {} -> pkill -USR1 nikkicho_clip", binding);
}

/// Convert hotkey string like "ctrl+alt+v" to GNOME binding format "<Ctrl><Alt>v"
fn hotkey_to_gnome_binding(hotkey: &str) -> String {
    let parts: Vec<&str> = hotkey.split('+').collect();
    let mut result = String::new();
    for part in &parts {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "control" => result.push_str("<Ctrl>"),
            "alt" => result.push_str("<Alt>"),
            "shift" => result.push_str("<Shift>"),
            "super" | "meta" | "cmd" => result.push_str("<Super>"),
            key => result.push_str(key),
        }
    }
    result
}

fn main() -> eframe::Result<()> {
    // Force X11 backend - Wayland doesn't support window hide/show/focus
    std::env::remove_var("WAYLAND_DISPLAY");
    std::env::remove_var("WAYLAND_SOCKET");
    std::env::set_var("GDK_BACKEND", "x11");

    // Initialize GTK (required for tray icon on Linux)
    gtk::init().expect("Failed to initialize GTK");

    // Load settings and register GNOME keyboard shortcut
    let settings = AppSettings::load();
    register_gnome_shortcut(&settings.hotkey_string);

    // Set up SIGUSR1 signal handler for toggle
    let sigusr1_flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&sigusr1_flag))
        .expect("Failed to register SIGUSR1 handler");

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
        .with_title("NC")
        .build()
        .expect("Failed to build tray icon");

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

            // Load Japanese/CJK font as fallback
            let mut fonts = egui::FontDefinitions::default();
            let font_paths = [
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/OTF/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJKjp-Regular.otf",
                "/usr/share/fonts/noto-cjk/NotoSansCJKjp-Regular.otf",
                "/usr/share/fonts/google-noto-cjk/NotoSansCJKjp-Regular.otf",
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

            // Shared visibility state
            let visible = Arc::new(AtomicBool::new(true));
            let force_quit = Arc::new(AtomicBool::new(false));

            // Thread: handle SIGUSR1 signal toggle
            let ctx_sig = cc.egui_ctx.clone();
            let visible_sig = Arc::clone(&visible);
            let sigusr1 = Arc::clone(&sigusr1_flag);
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if sigusr1.swap(false, Ordering::SeqCst) {
                    toggle_visibility(&visible_sig, &ctx_sig);
                }
            });

            // Thread: handle tray menu events
            let ctx2 = cc.egui_ctx.clone();
            let visible_tray = Arc::clone(&visible);
            let force_quit_tray = Arc::clone(&force_quit);
            std::thread::spawn(move || loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id() == &show_item_id {
                        visible_tray.store(true, Ordering::SeqCst);
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx2.request_repaint();
                    } else if event.id() == &quit_item_id {
                        force_quit_tray.store(true, Ordering::SeqCst);
                        ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

            Ok(Box::new(NikkichoClipApp::new(
                cc,
                settings,
                Arc::clone(&visible),
                Arc::clone(&force_quit),
            )))
        }),
    )
}

/// Toggle window visibility
fn toggle_visibility(visible: &AtomicBool, ctx: &egui::Context) {
    let is_visible = visible.load(Ordering::SeqCst);
    if is_visible {
        visible.store(false, Ordering::SeqCst);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    } else {
        visible.store(true, Ordering::SeqCst);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
    ctx.request_repaint();
}
