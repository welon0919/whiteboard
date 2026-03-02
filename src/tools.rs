use egui::ImageSource;
use serde::Serialize;

pub(super) const TOOLS: [(Tool, ImageSource, &str); 4] = [
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
    (
        Tool::Move,
        egui::include_image!("../assets/tools/move.png"),
        "Move Canvas Tool",
    ),
];

#[derive(PartialEq, Default, Serialize, Clone, Copy, Debug)]
pub enum Tool {
    #[default]
    Brush,
    Eraser,
    Selection,
    Move,
}
