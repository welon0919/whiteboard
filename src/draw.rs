use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Ui, vec2};

use crate::app::WhiteboardApp;
use crate::element::{Element, Line};
use crate::tools::{TOOLS, Tool};
use crate::utils::draw_dotted_rect;

impl WhiteboardApp {
    pub fn push_line(&mut self) {
        self.elements.push(Element::Line(Line {
            points: self.current_line.clone(),
            color: self.palette.get_current_color(),
            width: self.stroke_width,
        }));
        self.undo_stack.add_draw();
        self.current_line.clear();
    }

    pub fn draw_previous_elements(
        &self,
        ui: &mut Ui,
        ctx: &egui::Context,
        painter: &Painter,
        i: &usize,
        elem: &Element,
    ) {
        match elem {
            Element::Line(line) => {
                if line.points.len() >= 2 {
                    let points: Vec<Pos2> =
                        line.points.iter().map(|&p| p - self.view_offset).collect();
                    painter.add(egui::Shape::line(points, Stroke::new(line.width, line.color)));
                }
            }
            Element::Text(text_elem) => {
                if self.editing_text == Some(*i) {
                    return;
                }
                let screen_pos = text_elem.pos - self.view_offset;
                painter.text(
                    screen_pos,
                    egui::Align2::LEFT_TOP,
                    &text_elem.text,
                    egui::FontId::proportional(text_elem.size),
                    text_elem.color,
                );
            }
            Element::Image(img) => {
                let screen_pos = img.pos - self.view_offset;
                let rect = Rect::from_min_size(screen_pos, img.size);
                let uri = format!("bytes://image_{}.png", img.id);
                let image = egui::Image::from_bytes(uri, img.bytes.clone())
                    .fit_to_exact_size(img.size);
                image.paint_at(ui, rect);
            }
        }
    }

    pub fn draw_selections(&self, ctx: &egui::Context, painter: &Painter) {
        if let (Some(start), Some(current)) =
            (self.selection_start, self.selection_current)
        {
            if self.current_tool == Tool::Selection {
                let rect = Rect::from_two_pos(start, current);
                let screen_rect = rect.translate(-self.view_offset);
                draw_dotted_rect(
                    &painter,
                    screen_rect,
                    Stroke::new(1.0, Color32::GRAY),
                );
            }
        }

        if self.current_tool == Tool::Selection {
            if let Some((_, expanded, corners)) = self.get_selection_info(ctx) {
                let screen_expanded = expanded.translate(-self.view_offset);
                draw_dotted_rect(
                    &painter,
                    screen_expanded,
                    Stroke::new(1.0, Color32::BLUE),
                );

                let corner_size = vec2(8.0, 8.0);
                for &corner in &corners {
                    let screen_corner = corner - self.view_offset;
                    let rect =
                        Rect::from_center_size(screen_corner, corner_size);
                    painter.rect_filled(rect, 0.0, Color32::GRAY);
                    painter.rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::WHITE),
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }
    }

    pub fn draw_tool_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (tool, path, tooltip) in TOOLS {
                let is_selected = self.current_tool == tool;

                let frame = if is_selected {
                    egui::Frame::new()
                        .stroke(Stroke::new(2.0, ui.visuals().text_color()))
                        .inner_margin(2.0)
                        .corner_radius(4.0)
                } else {
                    egui::Frame::new().inner_margin(4.0)
                };

                frame.show(ui, |ui| {
                    let img = egui::Image::new(path)
                        .fit_to_exact_size(vec2(30.0, 30.0));
                    if ui
                        .add(egui::Button::image(img))
                        .on_hover_text(tooltip)
                        .clicked()
                    {
                        self.current_tool = tool;
                    }
                });
            }
        });
    }
}
