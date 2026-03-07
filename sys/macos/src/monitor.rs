use arboard::Clipboard;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum ClipEvent {
    Text(String),
    Image {
        width: u32,
        height: u32,
        png_data: Vec<u8>,
    },
}

/// Monitors the system clipboard in a background thread, sending new content via channel.
pub fn start_monitor() -> mpsc::Receiver<ClipEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open clipboard: {}", e);
                return;
            }
        };

        let mut last_text: Option<String> = None;
        let mut last_image_hash: Option<u64> = None;

        loop {
            thread::sleep(Duration::from_millis(500));

            // Check for text
            if let Ok(text) = clipboard.get_text() {
                if !text.trim().is_empty() {
                    let is_new = match &last_text {
                        Some(prev) => prev != &text,
                        None => true,
                    };
                    if is_new {
                        last_text = Some(text.clone());
                        if tx.send(ClipEvent::Text(text)).is_err() {
                            break;
                        }
                        continue;
                    }
                }
            }

            // Check for image
            if let Ok(img) = clipboard.get_image() {
                let hash = simple_hash(&img.bytes);
                let is_new = match last_image_hash {
                    Some(prev) => prev != hash,
                    None => true,
                };
                if is_new {
                    last_image_hash = Some(hash);
                    // Convert RGBA to PNG
                    if let Some(png_data) =
                        rgba_to_png(&img.bytes, img.width as u32, img.height as u32)
                    {
                        if tx
                            .send(ClipEvent::Image {
                                width: img.width as u32,
                                height: img.height as u32,
                                png_data,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        }
    });

    rx
}

fn simple_hash(data: &[u8]) -> u64 {
    // FNV-1a hash for quick comparison
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data.iter().step_by(64) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    use image::{ImageBuffer, RgbaImage};
    use std::io::Cursor;

    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba.to_vec())?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).ok()?;
    Some(buf.into_inner())
}
