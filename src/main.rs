use eframe::icon_data::from_png_bytes;
use egui::{IconData, Style, Visuals};
use whiteboard::WhiteboardApp;
fn load_icon() -> Result<IconData, String> {
    let png_bytes = include_bytes!("../assets/icon.png");
    from_png_bytes(png_bytes).map_err(|err| err.to_string())
}
fn main() -> eframe::Result<()> {
    let icon = load_icon().expect("Failed to load icon");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_maximized(true)
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Simple Whiteboard",
        native_options,
        Box::new(|ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            ctx.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            let style = Style {
                visuals: Visuals::dark(),
                ..Default::default()
            };
            ctx.egui_ctx.set_style(style);
            Ok(Box::new(WhiteboardApp::default()))
        }),
    )
}
