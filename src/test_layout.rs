use eframe::egui;
pub fn test(ctx: &egui::Context) {
    let _galley = ctx.fonts(|f| f.layout_no_wrap("hello".to_string(), egui::FontId::proportional(12.0), egui::Color32::RED));
}
