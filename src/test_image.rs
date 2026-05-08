use eframe::egui;
pub fn test(ui: &mut egui::Ui) {
    let bytes: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![0; 10].into_boxed_slice());
    ui.add(egui::Image::from_bytes("bytes://test.png", bytes));
}
