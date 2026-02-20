use egui::{Style, Visuals};
use whiteboard::WhiteboardApp;
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Simple Whiteboard",
        native_options,
        Box::new(|ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            let style = Style {
                visuals: Visuals::dark(),
                ..Default::default()
            };
            ctx.egui_ctx.set_style(style);
            Ok(Box::new(WhiteboardApp::default()))
        }),
    )
}
