use eframe::{emath::Pos2, epaint::Color32};
use serde::{Deserialize, Serialize};

use crate::{Line, WhiteboardApp};

#[derive(Serialize, Deserialize)]
struct Pos {
    x: f32,
    y: f32,
}
impl From<&Pos2> for Pos {
    fn from(pos: &Pos2) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}
impl From<Pos> for Pos2 {
    fn from(pos: Pos) -> Pos2 {
        Pos2::new(pos.x, pos.y)
    }
}
impl From<&Pos> for Pos2 {
    fn from(pos: &Pos) -> Self {
        Pos2::new(pos.x, pos.y)
    }
}
#[derive(Serialize, Deserialize, Copy, Clone)]
pub(crate) struct Color(pub(crate) [u8; 4]);
impl From<&Color32> for Color {
    fn from(c: &Color32) -> Self {
        Self([c[0], c[1], c[2], c[3]])
    }
}

impl From<Color32> for Color {
    fn from(c: Color32) -> Self {
        Self([c[0], c[1], c[2], c[3]])
    }
}
impl From<Color> for Color32 {
    fn from(color: Color) -> Self {
        Color32::from_rgba_unmultiplied(
            color.0[0], color.0[1], color.0[2], color.0[3],
        )
    }
}
#[derive(Serialize, Deserialize)]
pub struct LineState {
    points: Vec<Pos>,
    color: Color,
    width: f32,
}
impl From<&Line> for LineState {
    fn from(line: &Line) -> Self {
        Self {
            points: line.points.iter().map(Into::into).collect(),
            color: line.color.into(),
            width: line.width,
        }
    }
}
impl From<&LineState> for Line {
    fn from(state: &LineState) -> Self {
        Line {
            points: state.points.iter().map(Into::into).collect(),
            color: state.color.into(),
            width: state.width,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct WhiteboardState {
    pub lines: Vec<LineState>,
    pub(crate) palette: Vec<Color>,
}
impl WhiteboardState {
    pub fn new(app: &WhiteboardApp) -> Self {
        Self {
            lines: app.lines.iter().map(Into::into).collect(),
            palette: app
                .palette
                .get_palette_vec()
                .iter()
                .map(Color::from)
                .collect(),
        }
    }
}
