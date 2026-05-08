use egui::{Color32, Pos2, Rect};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Line {
    pub points: Vec<Pos2>,
    pub color: Color32,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct TextElement {
    pub text: String,
    pub pos: Pos2,
    pub size: f32,
    pub color: Color32,
}

#[derive(Debug, Clone)]
pub struct ImageElement {
    pub id: u64,
    pub bytes: Arc<[u8]>,
    pub pos: Pos2,
    pub size: egui::Vec2,
}

#[derive(Debug, Clone)]
pub enum Element {
    Line(Line),
    Text(TextElement),
    Image(ImageElement),
}

impl Element {
    pub fn bounding_box(&self, ctx: &egui::Context) -> Rect {
        match self {
            Element::Line(line) => {
                let mut bbox = Rect::NOTHING;
                for p in &line.points {
                    bbox.extend_with(*p);
                }
                bbox
            }
            Element::Text(text_elem) => {
                let galley = ctx.fonts_mut(|f| f.layout(
                    text_elem.text.clone(),
                    egui::FontId::proportional(text_elem.size),
                    text_elem.color,
                    f32::INFINITY
                ));
                Rect::from_min_size(text_elem.pos, galley.size())
            }
            Element::Image(img) => {
                Rect::from_min_size(img.pos, img.size)
            }
        }
    }
}
