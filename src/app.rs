use std::{
    collections::HashSet,
    path::PathBuf,
};

use eframe::egui;
use egui::vec2;

use crate::{
    colors::ColorPalette,
    element::Element,
    tools::Tool,
    undo::UndoStack,
};

use crate::utils::ResizeCorner;

pub const CANVAS_SIZE: f32 = 5000.0;

pub struct WhiteboardApp {
    pub elements: Vec<Element>,
    pub current_line: Vec<egui::Pos2>,
    pub palette: ColorPalette,
    pub stroke_width: f32,
    pub current_tool: Tool,
    pub undo_stack: UndoStack,
    pub whiteboard_file: Option<PathBuf>,

    // View state
    pub view_offset: egui::Vec2,
    pub initialized: bool,

    // Selection tool state
    pub selection_start: Option<egui::Pos2>,
    pub selection_current: Option<egui::Pos2>,
    pub selected_elements: HashSet<usize>,
    pub is_moving_selection: bool,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub resizing_corner: Option<ResizeCorner>,
    pub resize_original_bbox: Option<egui::Rect>,
    pub resize_original_elements: Vec<(usize, Element)>,

    // Text tool state
    pub editing_text: Option<usize>,
    pub editing_text_original: Option<Element>,
    pub focus_text_editor: bool,

    // Image tool state
    pub image_id_counter: u64,
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            current_line: Vec::new(),
            palette: ColorPalette::default(),
            stroke_width: 3.0,
            current_tool: Tool::Brush,
            undo_stack: UndoStack::default(),
            whiteboard_file: None,

            view_offset: egui::Vec2::ZERO,
            initialized: false,

            selection_start: None,
            selection_current: None,
            selected_elements: HashSet::new(),
            is_moving_selection: false,
            last_mouse_pos: None,
            resizing_corner: None,
            resize_original_bbox: None,
            resize_original_elements: Vec::new(),

            editing_text: None,
            editing_text_original: None,
            focus_text_editor: false,

            image_id_counter: 0,
        }
    }
}

impl eframe::App for WhiteboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let prev_color = self.palette.get_current_color();
        self.handle_keyboard_event(ctx);

        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("toolbar");
            ui.add_space(5.0);

            self.draw_tool_bar(ui);

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            ui.add_enabled_ui(
                self.current_tool == Tool::Brush || self.current_tool == Tool::Text || !self.selected_elements.is_empty(),
                |ui| {
                    self.palette.draw(ui);
                },
            );

            ui.add_space(10.0);

            ui.add(
                egui::Slider::new(&mut self.stroke_width, 1.0..=20.0)
                    .text("Stroke Width"),
            );

            ui.add_space(20.0);

            if ui.button("Clear").clicked() {
                self.elements.clear();
            }
        });

        let new_color = self.palette.get_current_color();
        if prev_color != new_color && !self.selected_elements.is_empty() {
            let mut original_elements = Vec::new();
            for &i in &self.selected_elements {
                if let Some(elem) = self.elements.get(i) {
                    original_elements.push((i, elem.clone()));
                }
            }
            for &i in &self.selected_elements {
                if let Some(elem) = self.elements.get_mut(i) {
                    match elem {
                        Element::Line(line) => line.color = new_color,
                        Element::Text(text) => text.color = new_color,
                        Element::Image(_) => {}
                    }
                }
            }
            self.undo_stack.add_modify(original_elements);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            if !self.initialized {
                let size = response.rect.size();
                self.view_offset = vec2(
                    CANVAS_SIZE / 2.0 - size.x / 2.0,
                    CANVAS_SIZE / 2.0 - size.y / 2.0,
                );
                self.initialized = true;
            }

            let (scroll_delta, shift) =
                ui.input(|i| (i.smooth_scroll_delta, i.modifiers.shift));
            if scroll_delta != vec2(0.0, 0.0) {
                let mut dx = scroll_delta.x;
                let mut dy = scroll_delta.y;

                if shift && dx == 0.0 && dy != 0.0 {
                    dx = dy;
                    dy = 0.0;
                }

                self.view_offset.x -= dx;
                self.view_offset.y -= dy;

                let size = response.rect.size();
                self.view_offset.x =
                    self.view_offset.x.clamp(0.0, CANVAS_SIZE - size.x);
                self.view_offset.y =
                    self.view_offset.y.clamp(0.0, CANVAS_SIZE - size.y);
            }

            self.update_cursor(ctx, &response);

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = pointer_pos + self.view_offset;

                if response.double_clicked() {
                    for (i, elem) in self.elements.iter().enumerate().rev() {
                        if let Element::Text(_) = elem {
                            if elem.bounding_box(ctx).contains(canvas_pos) {
                                self.editing_text = Some(i);
                                self.editing_text_original = Some(elem.clone());
                                self.focus_text_editor = true;
                                break;
                            }
                        }
                    }
                } else if self.editing_text.is_none() {
                    match self.current_tool {
                        Tool::Brush => {
                            if response.dragged()
                                && self.current_line.last() != Some(&canvas_pos)
                            {
                                self.current_line.push(canvas_pos);
                            }
                        }
                        Tool::Eraser => {
                            if response.clicked() || response.dragged() {
                                self.handle_eraser(canvas_pos, ctx);
                            }
                        }
                        Tool::Selection => {
                            self.handle_selection(ctx, &response, canvas_pos)
                        }
                        Tool::Move => {
                            if response.dragged() {
                                let delta = response.drag_delta();
                                self.view_offset -= delta;

                                let size = response.rect.size();
                                self.view_offset.x = self
                                    .view_offset
                                    .x
                                    .clamp(0.0, CANVAS_SIZE - size.x);
                                self.view_offset.y = self
                                    .view_offset
                                    .y
                                    .clamp(0.0, CANVAS_SIZE - size.y);
                            }
                        }
                        Tool::Text => {
                            if response.clicked() {
                                let new_text = crate::element::TextElement {
                                    text: "".to_string(),
                                    pos: canvas_pos,
                                    size: 20.0,
                                    color: self.palette.get_current_color(),
                                };
                                let idx = self.elements.len();
                                self.elements.push(Element::Text(new_text.clone()));
                                self.undo_stack.add_draw();
                                self.editing_text = Some(idx);
                                self.editing_text_original = Some(Element::Text(new_text));
                                self.focus_text_editor = true;
                                self.current_tool = Tool::Selection;
                            }
                        }
                        Tool::Image => {
                            if response.clicked() {
                                if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "webp", "gif", "bmp"]).pick_file() {
                                    if let Ok(bytes) = std::fs::read(&path) {
                                        let id = self.image_id_counter;
                                        self.image_id_counter += 1;
                                        let img_elem = crate::element::ImageElement {
                                            id,
                                            bytes: std::sync::Arc::from(bytes.into_boxed_slice()),
                                            pos: canvas_pos,
                                            size: egui::vec2(200.0, 200.0),
                                        };
                                        let idx = self.elements.len();
                                        self.elements.push(Element::Image(img_elem));
                                        self.undo_stack.add_draw();
                                        
                                        self.selected_elements.clear();
                                        self.selected_elements.insert(idx);
                                        self.current_tool = Tool::Selection;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if response.drag_stopped()
                && self.current_tool == Tool::Brush
                && !self.current_line.is_empty()
            {
                self.push_line();
            }

            for (i, elem) in self.elements.iter().enumerate() {
                self.draw_previous_elements(ui, ctx, &painter, &i, elem);
            }

            self.draw_selections(ctx, &painter);

            if self.current_tool == Tool::Brush && self.current_line.len() >= 2
            {
                let points: Vec<egui::Pos2> = self
                    .current_line
                    .iter()
                    .map(|&p| p - self.view_offset)
                    .collect();
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(
                        self.stroke_width,
                        self.palette.get_current_color(),
                    ),
                ));
            }

            if let Some(idx) = self.editing_text {
                if let Some(Element::Text(text_elem)) = self.elements.get_mut(idx) {
                    let screen_pos = text_elem.pos - self.view_offset;
                    let mut done_editing = false;
                    let mut request_focus = false;

                    if self.focus_text_editor {
                        request_focus = true;
                        self.focus_text_editor = false;
                    }

                    egui::Area::new("text_editor_area".into())
                        .fixed_pos(screen_pos)
                        .show(ctx, |ui| {
                            let response = ui.add(
                                egui::TextEdit::multiline(&mut text_elem.text)
                                    .font(egui::FontId::proportional(text_elem.size))
                                    .text_color(text_elem.color)
                                    .desired_width(f32::INFINITY)
                            );
                            if request_focus {
                                response.request_focus();
                            }
                            if response.lost_focus() {
                                done_editing = true;
                            }
                        });

                    if done_editing {
                        self.editing_text = None;
                        if text_elem.text.trim().is_empty() {
                            self.elements.remove(idx);
                            if let Some(orig) = self.editing_text_original.take() {
                                if let Element::Text(orig_text) = orig {
                                    if orig_text.text.is_empty() {
                                        self.undo_stack.pop();
                                    }
                                }
                            }
                        } else {
                            if let Some(orig) = self.editing_text_original.take() {
                                if let Element::Text(orig_text) = &orig {
                                    if orig_text.text != text_elem.text {
                                        if !orig_text.text.is_empty() {
                                            self.undo_stack.add_modify(vec![(idx, orig)]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.editing_text = None;
                }
            }
        });
    }
}
