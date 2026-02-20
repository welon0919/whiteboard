use eframe::{egui, epaint::StrokeKind};
use egui::{Color32, Pos2, Stroke};

struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

pub struct WhiteboardApp {
    lines: Vec<Line>,
    current_line: Vec<Pos2>,
    // 將單一顏色替換為五個顏色的陣列
    palette: [Color32; 5],
    // 追蹤目前選取的顏色索引
    active_color_index: usize,
    stroke_width: f32,
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current_line: Vec::new(),
            // 預設提供五種不同的顏色選項
            palette: [
                Color32::WHITE,
                Color32::RED,
                Color32::YELLOW,
                Color32::GREEN,
                Color32::BLUE,
            ],
            active_color_index: 0,
            stroke_width: 3.0,
        }
    }
}

impl eframe::App for WhiteboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Brush settings");

            ui.add_space(5.0);
            ui.label("Select color:");

            ui.horizontal(|ui| {
                for i in 0..5 {
                    let is_selected = i == self.active_color_index;

                    let frame = if is_selected {
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(
                                2.0,
                                ui.visuals().text_color(),
                            ))
                            .inner_margin(2.0)
                            .rounding(4.0)
                    } else {
                        egui::Frame::none().inner_margin(4.0)
                    };

                    frame.show(ui, |ui| {
                        if is_selected {
                            ui.color_edit_button_srgba(&mut self.palette[i]);
                        } else {
                            let size = egui::vec2(
                                ui.spacing().interact_size.y,
                                ui.spacing().interact_size.y,
                            );
                            let (rect, response) = ui.allocate_exact_size(
                                size,
                                egui::Sense::click(),
                            );

                            if ui.is_rect_visible(rect) {
                                let rounding = 2.0;
                                ui.painter().rect_filled(
                                    rect,
                                    rounding,
                                    self.palette[i],
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    rounding,
                                    egui::Stroke::new(
                                        1.0,
                                        ui.visuals()
                                            .widgets
                                            .inactive
                                            .bg_stroke
                                            .color,
                                    ),
                                    StrokeKind::Outside,
                                );
                            }

                            if response.clicked() {
                                self.active_color_index = i;
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);

            ui.add(
                egui::Slider::new(&mut self.stroke_width, 1.0..=20.0)
                    .text("Stroke width"),
            );

            ui.add_space(20.0);

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
                        color: self.palette[self.active_color_index], // 採用當前選取的調色盤顏色
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
                    // 繪製預覽線條時也採用當前選取的顏色
                    Stroke::new(
                        self.stroke_width,
                        self.palette[self.active_color_index],
                    ),
                ));
            }
        });
    }
}
