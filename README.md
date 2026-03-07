# Nikkicho Clip

A cross-platform clipboard history manager built with Rust and [egui](https://github.com/emilk/egui).

## Features

- **Clipboard monitoring** - Automatically captures text and image copies in a background thread (polls every 500ms)
- **Search** - Filter clipboard history by keyword
- **Pin & Favorite** - Pin important entries to prevent deletion, or mark them as favorites for quick filtering
- **Image support** - Captures and previews copied images (PNG/JPEG), stored as base64
- **Re-copy** - Click any history entry to copy it back to the clipboard
- **Persistent storage** - History is saved to disk as JSON and survives restarts
- **Global hotkey** - Bring the window to focus instantly (see [Hotkeys](#global-hotkeys))
- **System tray** - Runs in the background with a tray icon (Show / Quit)
- **Dark theme** - Uses egui's dark visuals by default

## Supported Platforms

| Platform | Status |
|----------|--------|
| Windows 10/11 | Supported |
| macOS | Supported |
| Linux (X11/Wayland) | Supported (requires GTK 3) |

## Requirements

- Rust toolchain (edition 2021)
- **Linux only:** GTK 3 development libraries
  ```bash
  # Debian/Ubuntu
  sudo apt install libgtk-3-dev

  # Fedora
  sudo dnf install gtk3-devel

  # Arch
  sudo pacman -S gtk3
  ```

## Building

Each platform has its own crate under `sys/`:

```bash
# Windows
cd sys/win
cargo build --release

# macOS
cd sys/macos
cargo build --release

# Linux
cd sys/linux
cargo build --release
```

The binary will be at `sys/<platform>/target/release/nikkicho_clip` (`.exe` on Windows).

## Global Hotkeys

| Platform | Hotkey |
|----------|--------|
| Windows | `Ctrl+Shift+V` |
| macOS | `Cmd+Shift+V` |
| Linux | `Ctrl+Shift+V` |

## Project Structure

```
sys/
  win/                 # Windows build
  macos/               # macOS build
  linux/               # Linux build (requires GTK)
  <platform>/src/
    main.rs            # Entry point, hotkey & tray setup
    app.rs             # egui application, UI rendering
    clip_entry.rs      # ClipEntry / ClipContent data types
    history.rs         # ClipHistory - persistence, search, pin/fav logic
    monitor.rs         # Background clipboard polling thread
```

## Data Storage

History is persisted as JSON using the [`directories`](https://crates.io/crates/directories) crate for platform-appropriate paths:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\nikkicho\clip\data\history.json` |
| macOS | `~/Library/Application Support/com.nikkicho.clip/history.json` |
| Linux | `~/.local/share/clip/history.json` (or `$XDG_DATA_HOME/clip/`) |

Up to **500 entries** are kept. Pinned and favorited entries are preserved when the limit is exceeded or when clearing history.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` | GUI framework |
| `arboard` | Cross-platform clipboard access |
| `global-hotkey` | System-wide hotkey registration |
| `tray-icon` | System tray integration |
| `image` | PNG/JPEG encoding for clipboard images |
| `serde` / `serde_json` | JSON serialization for history persistence |
| `chrono` | Timestamps on clipboard entries |
| `directories` | Platform-specific data directory resolution |
| `uuid` | Unique IDs for each clipboard entry |
| `base64` | Encoding image data for storage |

## License

See [LICENSE](LICENSE) for details.
