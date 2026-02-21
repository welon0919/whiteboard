mod colors;
mod state;
mod tools;
mod undo;

use std::{io, path::PathBuf};

use directories::UserDirs;
use eframe::egui;
use egui::{Color32, Pos2, Stroke, pos2, vec2};

use crate::{
    colors::ColorPalette,
    state::WhiteboardState,
    tools::Tool,
    undo::{UndoAction, UndoStack},
};

#[derive(Debug, Clone)]
struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

pub struct WhiteboardApp {
    lines: Vec<Line>,
    current_line: Vec<Pos2>,
    palette: ColorPalette,
    stroke_width: f32,
    current_tool: Tool,
    undo_stack: UndoStack,
    whiteboard_file: Option<PathBuf>,
}

impl WhiteboardApp {
    fn set_window_title(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Simple Whiteboard - {}",
            self.whiteboard_file
                .as_ref()
                .map(|s| s.display().to_string())
                .unwrap_or("Untitled.wb".to_owned())
        )));
    }
    fn handle_keyboard_event(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_open = false;
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
                        egui::Key::S if modifiers.command => {
                            should_save = true;
                        }
                        egui::Key::O if modifiers.command => {
                            should_open = true;
                        }
                        egui::Key::Num1 => {
                            self.palette.set_active_color_index(0)
                        }
                        egui::Key::Num2 => {
                            self.palette.set_active_color_index(1)
                        }
                        egui::Key::Num3 => {
                            self.palette.set_active_color_index(2)
                        }
                        egui::Key::Num4 => {
                            self.palette.set_active_color_index(3)
                        }
                        egui::Key::Num5 => {
                            self.palette.set_active_color_index(4)
                        }
                        _ => {}
                    }
                }
            }
        });
        if should_open {
            if let Err(e) = self.open_whiteboard_file() {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Failed to read")
                    .set_description(format!("Failed to read: {}", e))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            } else {
                self.set_window_title(ctx);
            }
        }
        if should_save {
            self.save_whiteboard();
            self.set_window_title(ctx);
        }
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
    fn write_whiteboard(&mut self, file_path: PathBuf, json: String) {
        if let Err(e) = std::fs::write(&file_path, json) {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to save whiteboard")
                .set_description(format!("Failed to save whiteboard: {}", e))
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return;
        }
        if self.whiteboard_file.is_none() {
            self.whiteboard_file = Some(file_path);
        }
    }
    fn save_whiteboard(&mut self) {
        let default_path = UserDirs::new()
            .and_then(|user_dirs| {
                user_dirs.download_dir().map(|p| p.to_path_buf())
            })
            .unwrap_or(std::env::current_dir().unwrap_or_default());
        let whiteboard_state = WhiteboardState::new(self);
        let json = serde_json::to_string(&whiteboard_state).unwrap();
        if let Some(file_path) = self.whiteboard_file.clone() {
            self.write_whiteboard(file_path, json);
        } else {
            let files = rfd::FileDialog::new()
                .add_filter("Whiteboard file", &["wb"])
                .add_filter("All files", &["*"])
                .set_directory(default_path)
                .set_file_name("Untitled.wb")
                .save_file();
            if let Some(file_path) = files {
                self.write_whiteboard(file_path, json);
            }
        }
    }
    fn open_whiteboard_file(&mut self) -> io::Result<()> {
        let files = rfd::FileDialog::new()
            .add_filter("Whiteboard file", &["wb"])
            .set_title("Select whiteboard file")
            .pick_file();
        if let Some(file_path) = files {
            let json = std::fs::read_to_string(&file_path)?;
            let state = serde_json::from_str::<WhiteboardState>(&json);
            match state {
                Ok(state) => {
                    self.whiteboard_file = Some(file_path);
                    self.palette = state
                        .palette
                        .iter()
                        .map(|&color| color.into())
                        .collect::<Vec<_>>()
                        .into();
                    self.lines = state.lines.iter().map(|l| l.into()).collect();
                }
                Err(_) => {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Invalid whiteboard file")
                        .set_description(format!(
                            "{} is not a whiteboard file",
                            &file_path.to_string_lossy()
                        ))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
        }
        Ok(())
    }
}
impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current_line: Vec::new(),
            // 預設提供五種不同的顏色選項
            palette: ColorPalette::default(),
            stroke_width: 3.0,
            current_tool: Tool::Brush,
            undo_stack: UndoStack::default(),
            whiteboard_file: None,
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

            // Tool selection
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

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // color selection (only when brush is selected)
            ui.add_enabled_ui(self.current_tool == Tool::Brush, |ui| {
                self.palette.draw(ui)
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
                        color: self.palette.get_current_color(),
                        width: self.stroke_width,
                    });
                    self.undo_stack.add_draw(Line {
                        points: self.current_line.clone(),
                        color: self.palette.get_current_color(),
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
                        self.palette.get_current_color(),
                    ),
                ));
            }
        });
    }
}
