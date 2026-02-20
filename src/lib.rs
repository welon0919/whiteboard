mod tools;
mod undo;

use eframe::{egui, epaint::StrokeKind};
use egui::{Color32, Pos2, Stroke, pos2, vec2};

use crate::{
    tools::Tool,
    undo::{UndoAction, UndoStack},
};

struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

pub struct WhiteboardApp {
    lines: Vec<Line>,
    current_line: Vec<Pos2>,
    palette: [Color32; 5],
    active_color_index: usize,
    stroke_width: f32,
    current_tool: Tool,
    undo_stack: UndoStack,
}

impl WhiteboardApp {
    fn handle_keyboard_event(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    match key {
                        egui::Key::Z if modifiers.command => {
                            self.undo();
                        }
                        egui::Key::C if !modifiers.command => {
                            self.lines.clear();
                        }
                        egui::Key::B if !modifiers.command => {
                            self.current_tool = Tool::Brush;
                        }
                        egui::Key::E if !modifiers.command => {
                            self.current_tool = Tool::Eraser;
                        }
                        egui::Key::Num1 => self.active_color_index = 0,
                        egui::Key::Num2 => self.active_color_index = 1,
                        egui::Key::Num3 => self.active_color_index = 2,
                        egui::Key::Num4 => self.active_color_index = 3,
                        egui::Key::Num5 => self.active_color_index = 4,
                        _ => {}
                    }
                }
            }
        })
    }
    fn undo(&mut self) {
        match self.undo_stack.pop() {
            None => {}
            Some(action) => match action {
                UndoAction::Erase(line) => {
                    self.lines.push(line);
                }
                UndoAction::Draw(_line) => {
                    self.lines.pop();
                }
            },
        }
    }
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
            current_tool: Tool::default(),
            undo_stack: UndoStack::default(),
        }
    }
}
// helper function to calculate the distance from a point to a line
fn distance_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let l2 = a.distance_sq(b);
    if l2 == 0.0 {
        return p.distance(a);
    }
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = pos2(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    p.distance(projection)
}

impl eframe::App for WhiteboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_event(ctx);
        // 設定側邊控制面板
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("toolbar");
            ui.add_space(5.0);

            // 工具切換區塊
            ui.horizontal(|ui| {
                let tools = [
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
                ];

                for (tool, path, tooltip) in tools {
                    let is_selected = self.current_tool == tool;

                    let frame = if is_selected {
                        egui::Frame::new()
                            .stroke(egui::Stroke::new(
                                2.0,
                                ui.visuals().text_color(),
                            ))
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

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // 當使用畫筆時才允許調整顏色
            ui.add_enabled_ui(self.current_tool == Tool::Brush, |ui| {
                ui.label("Color selection");
                ui.horizontal(|ui| {
                    for i in 0..5 {
                        let is_selected = i == self.active_color_index;

                        let frame = if is_selected {
                            egui::Frame::new()
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    ui.visuals().text_color(),
                                ))
                                .inner_margin(2.0)
                                .corner_radius(4.0)
                        } else {
                            egui::Frame::new().inner_margin(4.0)
                        };

                        frame.show(ui, |ui| {
                            if is_selected {
                                ui.color_edit_button_srgba(
                                    &mut self.palette[i],
                                );
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
            });

            ui.add_space(10.0);

            ui.add(
                egui::Slider::new(&mut self.stroke_width, 1.0..=20.0)
                    .text("Stroke Width"),
            );

            ui.add_space(20.0);

            if ui.button("Clear").clicked() {
                self.lines.clear();
            }
        });

        // 畫布區域
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::drag());

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                match self.current_tool {
                    Tool::Brush => {
                        if response.dragged() {
                            if self.current_line.last() != Some(&pointer_pos) {
                                self.current_line.push(pointer_pos);
                            }
                        }
                    }
                    Tool::Eraser => {
                        // 支援點擊或拖曳時刪除線條
                        if response.clicked() || response.dragged() {
                            let erase_radius = self.stroke_width + 5.0; // 給予一點點擊容差

                            let (kept, deleted): (Vec<_>, Vec<_>) =
                                self.lines.drain(..).partition(|line| {
                                    for window in line.points.windows(2) {
                                        if distance_point_to_segment(
                                            pointer_pos,
                                            window[0],
                                            window[1],
                                        ) < erase_radius
                                        {
                                            return false; // false 會進 deleted
                                        }
                                    }
                                    true
                                });

                            self.lines = kept;
                            let deleted_lines = deleted;
                            self.undo_stack.extend_erase(deleted_lines);
                        }
                    }
                }
            }

            // 畫筆模式下，放開拖曳時儲存線條
            if response.drag_stopped() && self.current_tool == Tool::Brush {
                if !self.current_line.is_empty() {
                    self.lines.push(Line {
                        points: self.current_line.clone(),
                        color: self.palette[self.active_color_index],
                        width: self.stroke_width,
                    });
                    self.undo_stack.add_draw(Line {
                        points: self.current_line.clone(),
                        color: self.palette[self.active_color_index],
                        width: self.stroke_width,
                    });
                    self.current_line.clear();
                }
            }

            // 繪製所有已存檔的線條
            for line in &self.lines {
                if line.points.len() >= 2 {
                    let points = line.points.clone();
                    painter.add(egui::Shape::line(
                        points,
                        Stroke::new(line.width, line.color),
                    ));
                }
            }

            // 繪製正在畫的線條（僅限畫筆模式）
            if self.current_tool == Tool::Brush && self.current_line.len() >= 2
            {
                painter.add(egui::Shape::line(
                    self.current_line.clone(),
                    Stroke::new(
                        self.stroke_width,
                        self.palette[self.active_color_index],
                    ),
                ));
            }
        });
    }
}
