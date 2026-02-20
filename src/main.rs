use whiteboard::WhiteboardApp;
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simple Whiteboard",
        native_options,
        Box::new(|_cc| Ok(Box::new(WhiteboardApp::default()))),
    )
}
