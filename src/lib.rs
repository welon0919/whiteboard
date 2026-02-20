use eframe::egui;
use egui::{Color32, Pos2, Stroke};

struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

pub struct WhiteboardApp {
    lines: Vec<Line>,
    current_line: Vec<Pos2>,
    stroke_color: Color32,
    stroke_width: f32,
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current_line: Vec::new(),
            stroke_color: Color32::LIGHT_BLUE,
            stroke_width: 3.0,
        }
    }
}

impl eframe::App for WhiteboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 設定側邊控制面板
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Brush settings");
            ui.color_edit_button_srgba(&mut self.stroke_color);
            ui.add(
                egui::Slider::new(&mut self.stroke_width, 1.0..=20.0)
                    .text("Stroke width"),
            );

            if ui.button("Clear").clicked() {
                self.lines.clear();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::drag());

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if response.dragged() {
                    // 如果正在拖曳，將點加入當前線條
                    if self.current_line.last() != Some(&pointer_pos) {
                        self.current_line.push(pointer_pos);
                    }
                }
            }

            if response.drag_stopped() {
                if !self.current_line.is_empty() {
                    self.lines.push(Line {
                        points: self.current_line.clone(),
                        color: self.stroke_color,
                        width: self.stroke_width,
                    });
                    self.current_line.clear();
                }
            }

            for line in &self.lines {
                if line.points.len() >= 2 {
                    let points = line.points.clone();
                    painter.add(egui::Shape::line(
                        points,
                        Stroke::new(line.width, line.color),
                    ));
                }
            }

            if self.current_line.len() >= 2 {
                painter.add(egui::Shape::line(
                    self.current_line.clone(),
                    Stroke::new(self.stroke_width, self.stroke_color),
                ));
            }
        });
    }
}
