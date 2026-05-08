use arboard::Clipboard;
use image::{RgbaImage, ImageEncoder};

pub fn test() {
    let mut cb = Clipboard::new().unwrap();
    if let Ok(img) = cb.get_image() {
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder.write_image(&img.bytes, img.width as u32, img.height as u32, image::ExtendedColorType::Rgba8).unwrap();
    }
}
