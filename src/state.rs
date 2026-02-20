use eframe::{emath::Pos2, epaint::Color32};
use serde::Serialize;

use crate::{Line, WhiteboardApp};

#[derive(Serialize)]
struct Pos {
    x: f32,
    y: f32,
}
impl From<&Pos2> for Pos {
    fn from(pos: &Pos2) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}
impl Into<Pos2> for Pos {
    fn into(self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }
}
#[derive(Serialize)]
struct Color(pub(crate) [u8; 4]);
impl From<Color32> for Color {
    fn from(c: Color32) -> Self {
        Self([c[0], c[1], c[2], c[3]])
    }
}
impl Into<Color32> for Color {
    fn into(self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.0[0], self.0[1], self.0[2], self.0[3],
        )
    }
}
#[derive(Serialize)]
pub struct LineState {
    points: Vec<Pos>,
    color: Color,
    width: f32,
}
impl From<&Line> for LineState {
    fn from(line: &Line) -> Self {
        Self {
            points: line.points.iter().map(|p| p.into()).collect(),
            color: line.color.into(),
            width: line.width,
        }
    }
}
#[derive(Serialize)]
pub struct WhiteboardState {
    lines: Vec<LineState>,
    palette: Vec<Color>,
}
impl From<&WhiteboardApp> for WhiteboardState {
    fn from(app: &WhiteboardApp) -> Self {
        Self {
            lines: app.lines.iter().map(|line| line.into()).collect(),
            palette: app
                .palette
                .into_iter()
                .map(|color| Color::from(color))
                .collect(),
        }
    }
}
