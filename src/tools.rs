use egui::ImageSource;
use serde::Serialize;

pub(super) const TOOLS: [(Tool, ImageSource, &str); 3] = [
    (
        Tool::Brush,
        egui::include_image!("../assets/tools/brush.png"),
        "brush",
    ),
    (
        Tool::Eraser,
        egui::include_image!("../assets/tools/eraser.png"),
        "eraser",
    ),
    (
        Tool::Selection,
        egui::include_image!("../assets/tools/select.png"),
        "Selection Tool",
    ),
];

#[derive(PartialEq, Default, Serialize)]
pub enum Tool {
    #[default]
    Brush,
    Eraser,
    Selection,
}
