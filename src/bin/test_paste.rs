use arboard::Clipboard;

fn main() {
    match Clipboard::new() {
        Ok(mut cb) => {
            match cb.get_image() {
                Ok(img) => println!("Got image: {}x{}", img.width, img.height),
                Err(e) => println!("Image error: {:?}", e),
            }
            match cb.get_text() {
                Ok(text) => println!("Got text: {}", text),
                Err(e) => println!("Text error: {:?}", e),
            }
        }
        Err(_) => {}
    }
}
