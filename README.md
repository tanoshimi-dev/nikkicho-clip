# Nikkicho Clip

A clipboard history manager for Windows, built with Rust and [egui](https://github.com/emilk/egui).

## Features

- **Clipboard monitoring** - Automatically captures text and image copies in the background
- **Search** - Filter clipboard history by keyword
- **Pin & Favorite** - Pin important entries to prevent deletion, or mark them as favorites for quick filtering
- **Image support** - Captures and previews copied images (PNG/JPEG), stored as base64
- **Re-copy** - Click any history entry to copy it back to the clipboard
- **Persistent storage** - History is saved to disk as JSON and survives restarts
- **Global hotkey** - Press `Ctrl+Shift+V` to bring the window to focus
- **System tray** - Runs in the background with a tray icon (Show / Quit)
- **Dark theme** - Uses egui's dark visuals by default

## Requirements

- Windows 10/11
- Rust toolchain (edition 2021)

## Building

```bash
cd sys/win
cargo build --release
```

The binary will be at `sys/win/target/release/nikkicho_clip.exe`.

## Project Structure

```
sys/win/
  src/
    main.rs        # Entry point, hotkey & tray setup
    app.rs         # egui application, UI rendering
    clip_entry.rs  # ClipEntry / ClipContent data types
    history.rs     # ClipHistory - persistence, search, pin/fav logic
    monitor.rs     # Background clipboard polling thread
```

## Data Storage

History is stored at the platform data directory provided by the `directories` crate:

```
%APPDATA%/nikkicho/clip/data/history.json
```

Up to 500 entries are kept. Pinned and favorited entries are preserved when the limit is exceeded or when clearing history.
