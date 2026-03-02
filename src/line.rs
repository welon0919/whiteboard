use egui::{Color32, Pos2};

#[derive(Debug, Clone)]
pub struct Line {
    pub points: Vec<Pos2>,
    pub color: Color32,
    pub width: f32,
}
