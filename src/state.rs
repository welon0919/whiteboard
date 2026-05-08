use eframe::{emath::Pos2, epaint::Color32};
use serde::{Deserialize, Serialize};

use crate::{Element, Line, TextElement, element::ImageElement, WhiteboardApp};

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

#[derive(Serialize, Deserialize)]
pub struct TextState {
    text: String,
    pos: Pos,
    size: f32,
    color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct ImageState {
    id: u64,
    bytes: Vec<u8>,
    pos: Pos,
    size: [f32; 2],
}

#[derive(Serialize, Deserialize)]
pub enum ElementState {
    Line(LineState),
    Text(TextState),
    Image(ImageState),
}

impl From<&Element> for ElementState {
    fn from(element: &Element) -> Self {
        match element {
            Element::Line(line) => ElementState::Line(LineState {
                points: line.points.iter().map(Into::into).collect(),
                color: line.color.into(),
                width: line.width,
            }),
            Element::Text(text) => ElementState::Text(TextState {
                text: text.text.clone(),
                pos: (&text.pos).into(),
                size: text.size,
                color: text.color.into(),
            }),
            Element::Image(img) => ElementState::Image(ImageState {
                id: img.id,
                bytes: img.bytes.to_vec(),
                pos: (&img.pos).into(),
                size: [img.size.x, img.size.y],
            }),
        }
    }
}

impl From<&ElementState> for Element {
    fn from(state: &ElementState) -> Self {
        match state {
            ElementState::Line(line_state) => Element::Line(Line {
                points: line_state.points.iter().map(Into::into).collect(),
                color: line_state.color.into(),
                width: line_state.width,
            }),
            ElementState::Text(text_state) => Element::Text(TextElement {
                text: text_state.text.clone(),
                pos: (&text_state.pos).into(),
                size: text_state.size,
                color: text_state.color.into(),
            }),
            ElementState::Image(img_state) => Element::Image(ImageElement {
                id: img_state.id,
                bytes: std::sync::Arc::from(img_state.bytes.clone()),
                pos: (&img_state.pos).into(),
                size: eframe::egui::vec2(img_state.size[0], img_state.size[1]),
            }),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WhiteboardState {
    pub elements: Vec<ElementState>,
    pub(crate) palette: Vec<Color>,
}

impl WhiteboardState {
    pub fn new(app: &WhiteboardApp) -> Self {
        Self {
            elements: app.elements.iter().map(Into::into).collect(),
            palette: app
                .palette
                .get_palette_vec()
                .iter()
                .map(Color::from)
                .collect(),
        }
    }
}
