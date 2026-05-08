use arboard::Clipboard;
use image::ImageEncoder;

pub fn test() {
    println!("Testing image paste...");
    match Clipboard::new() {
        Ok(mut cb) => {
            match cb.get_image() {
                Ok(img) => {
                    println!("Got image: {}x{}, {} bytes", img.width, img.height, img.bytes.len());
                    let mut png_bytes = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                    match encoder.write_image(&img.bytes, img.width as u32, img.height as u32, image::ExtendedColorType::Rgba8) {
                        Ok(_) => println!("Encoded to png successfully: {} bytes", png_bytes.len()),
                        Err(e) => println!("Error encoding png: {:?}", e),
                    }
                }
                Err(e) => println!("Error getting image: {:?}", e),
            }
        }
        Err(e) => println!("Clipboard error: {:?}", e),
    }
}
