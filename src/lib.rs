mod colors;
mod state;
mod tools;
mod undo;

use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
};

use directories::UserDirs;
use eframe::egui;
use egui::{Color32, Painter, Pos2, Rect, Response, Stroke, Ui, pos2, vec2};

use crate::{
    colors::ColorPalette,
    state::WhiteboardState,
    tools::{TOOLS, Tool},
    undo::{UndoAction, UndoStack},
};

#[derive(Debug, Clone)]
struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct WhiteboardApp {
    lines: Vec<Line>,
    current_line: Vec<Pos2>,
    palette: ColorPalette,
    stroke_width: f32,
    current_tool: Tool,
    undo_stack: UndoStack,
    whiteboard_file: Option<PathBuf>,

    // Selection tool state
    selection_start: Option<Pos2>,
    selection_current: Option<Pos2>,
    selected_lines: HashSet<usize>,
    is_moving_selection: bool,
    last_mouse_pos: Option<Pos2>,
    resizing_corner: Option<ResizeCorner>,
    resize_original_bbox: Option<Rect>,
    resize_original_lines: Vec<(usize, Line)>,
}

impl WhiteboardApp {
    fn set_window_title(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Simple Whiteboard - {}",
            self.whiteboard_file
                .as_ref()
                .map_or("Untitled.wb".to_owned(), |s| s.display().to_string())
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
                            self.selected_lines.clear();
                        }
                        egui::Key::B if !modifiers.command => {
                            self.current_tool = Tool::Brush;
                        }
                        egui::Key::E if !modifiers.command => {
                            self.current_tool = Tool::Eraser;
                        }
                        egui::Key::S if !modifiers.command => {
                            if self.current_tool != Tool::Selection {
                                self.selected_lines.clear();
                                self.selection_start = None;
                                self.selection_current = None;
                                self.is_moving_selection = false;
                                self.resizing_corner = None;
                                self.resize_original_bbox = None;
                                self.resize_original_lines.clear();
                                self.current_tool = Tool::Selection;
                            }
                        }
                        egui::Key::S if modifiers.command => {
                            should_save = true;
                        }
                        egui::Key::O if modifiers.command => {
                            should_open = true;
                        }
                        egui::Key::Num1 => {
                            self.palette.set_active_color_index(0);
                        }
                        egui::Key::Num2 => {
                            self.palette.set_active_color_index(1);
                        }
                        egui::Key::Num3 => {
                            self.palette.set_active_color_index(2);
                        }
                        egui::Key::Num4 => {
                            self.palette.set_active_color_index(3);
                        }
                        egui::Key::Num5 => {
                            self.palette.set_active_color_index(4);
                        }
                        egui::Key::Num6 => {
                            self.palette.set_active_color_index(5);
                        }
                        egui::Key::Num7 => {
                            self.palette.set_active_color_index(6);
                        }
                        egui::Key::Num8 => {
                            self.palette.set_active_color_index(7);
                        }
                        egui::Key::Num9 => {
                            self.palette.set_active_color_index(8);
                        }
                        egui::Key::Delete => {
                            if !self.selected_lines.is_empty() {
                                let mut indices: Vec<_> = self
                                    .selected_lines
                                    .iter()
                                    .copied()
                                    .collect();
                                indices.sort_unstable_by(|a, b| b.cmp(a)); // sort descending

                                let mut deleted_lines = Vec::new();
                                for index in indices {
                                    if index < self.lines.len() {
                                        deleted_lines
                                            .push(self.lines.remove(index));
                                    }
                                }
                                // Reverse to maintain original order for undo if needed,
                                // though simple push is fine.
                                // For undo, we need to add them back.
                                // Since we remove by index descending, the last removed was the first in original list.
                                // We can just add them to undo stack as Erase action.
                                self.undo_stack.extend_erase(deleted_lines);
                                self.selected_lines.clear();
                            }
                        }
                        egui::Key::Escape => {
                            self.selected_lines.clear();
                            self.selection_start = None;
                            self.selection_current = None;
                            self.is_moving_selection = false;
                            self.resizing_corner = None;
                            self.resize_original_bbox = None;
                            self.resize_original_lines.clear();
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
                    .set_description(format!("Failed to read: {e}",))
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
        self.selected_lines.clear();
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
                .set_description(format!("Failed to save whiteboard: {e}",))
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
                user_dirs.download_dir().map(Path::to_path_buf)
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
                    self.lines = state.lines.iter().map(Into::into).collect();
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
    fn handle_selection(&mut self, response: &Response, pointer_pos: Pos2) {
        {
            // Check if we are interacting with existing selection
            let selection_info = self.get_selection_info();
            let (bounding_box, expanded_bbox, corners) = match selection_info {
                Some(info) => info,
                None => (Rect::NOTHING, Rect::NOTHING, [Pos2::ZERO; 4]),
            };

            let corner_size = vec2(10.0, 10.0);
            let tl_rect = Rect::from_center_size(corners[0], corner_size);
            let tr_rect = Rect::from_center_size(corners[1], corner_size);
            let bl_rect = Rect::from_center_size(corners[2], corner_size);
            let br_rect = Rect::from_center_size(corners[3], corner_size);

            if response.drag_started() {
                if !self.selected_lines.is_empty()
                    && tl_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::TopLeft, bounding_box);
                } else if !self.selected_lines.is_empty()
                    && tr_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::TopRight, bounding_box);
                } else if !self.selected_lines.is_empty()
                    && bl_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::BottomLeft, bounding_box);
                } else if !self.selected_lines.is_empty()
                    && br_rect.contains(pointer_pos)
                {
                    self.start_resizing(
                        ResizeCorner::BottomRight,
                        bounding_box,
                    );
                } else if expanded_bbox.contains(pointer_pos)
                    && !self.selected_lines.is_empty()
                {
                    self.is_moving_selection = true;
                    self.last_mouse_pos = Some(pointer_pos);
                } else {
                    self.selected_lines.clear();
                    self.selection_start = Some(pointer_pos);
                    self.selection_current = Some(pointer_pos);
                }
            } else if response.dragged() {
                if let Some(corner) = self.resizing_corner {
                    self.update_resizing(pointer_pos, corner);
                } else if self.is_moving_selection {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let delta = pointer_pos - last_pos;
                        for i in &self.selected_lines {
                            if let Some(line) = self.lines.get_mut(*i) {
                                for p in &mut line.points {
                                    *p += delta;
                                }
                            }
                        }
                        self.last_mouse_pos = Some(pointer_pos);
                    }
                } else if self.selection_start.is_some() {
                    self.selection_current = Some(pointer_pos);
                }
            } else if response.drag_stopped() {
                if self.resizing_corner.is_some() {
                    self.resizing_corner = None;
                    self.resize_original_bbox = None;
                    self.resize_original_lines.clear();
                } else if self.is_moving_selection {
                    self.is_moving_selection = false;
                    self.last_mouse_pos = None;
                } else if let (Some(start), Some(current)) =
                    (self.selection_start, self.selection_current)
                {
                    let rect = Rect::from_two_pos(start, current);
                    self.selected_lines.clear();
                    for (i, line) in self.lines.iter().enumerate() {
                        // Check if line is inside rect
                        // Simple check: if bounding box intersects
                        let mut line_bbox = Rect::NOTHING;
                        for p in &line.points {
                            line_bbox.extend_with(*p);
                        }
                        if rect.intersects(line_bbox) {
                            // More precise check: at least one point inside?
                            // Or just keep intersection. Intersection is usually good enough for "Select Area".
                            self.selected_lines.insert(i);
                        }
                    }
                    self.selection_start = None;
                    self.selection_current = None;
                }
            } else if response.clicked() {
                // Click outside selection to clear
                if !expanded_bbox.contains(pointer_pos) {
                    self.selected_lines.clear();
                }
            }
        }
    }

    fn get_selection_info(&self) -> Option<(Rect, Rect, [Pos2; 4])> {
        if self.selected_lines.is_empty() {
            return None;
        }

        let mut bounding_box = Rect::NOTHING;
        for &i in &self.selected_lines {
            if let Some(line) = self.lines.get(i) {
                for p in &line.points {
                    bounding_box.extend_with(*p);
                }
            }
        }

        if bounding_box == Rect::NOTHING {
            return None;
        }

        let expanded_bbox = bounding_box.expand(5.0);
        let corners = [
            expanded_bbox.left_top(),
            expanded_bbox.right_top(),
            expanded_bbox.left_bottom(),
            expanded_bbox.right_bottom(),
        ];

        Some((bounding_box, expanded_bbox, corners))
    }

    fn update_cursor(&self, ctx: &egui::Context, response: &Response) {
        if self.current_tool != Tool::Selection {
            return;
        }

        // Handle cursor during interaction
        if let Some(corner) = self.resizing_corner {
            match corner {
                ResizeCorner::TopLeft | ResizeCorner::BottomRight => {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                }
                ResizeCorner::TopRight | ResizeCorner::BottomLeft => {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                }
            }
            return;
        }

        if self.is_moving_selection {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            return;
        }

        // Handle cursor during hover
        if let Some(pointer_pos) = response.hover_pos() {
            if let Some((_, expanded_bbox, corners)) = self.get_selection_info()
            {
                let hit_size = vec2(10.0, 10.0);
                let tl_rect = Rect::from_center_size(corners[0], hit_size);
                let tr_rect = Rect::from_center_size(corners[1], hit_size);
                let bl_rect = Rect::from_center_size(corners[2], hit_size);
                let br_rect = Rect::from_center_size(corners[3], hit_size);

                if tl_rect.contains(pointer_pos)
                    || br_rect.contains(pointer_pos)
                {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                } else if tr_rect.contains(pointer_pos)
                    || bl_rect.contains(pointer_pos)
                {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                } else if expanded_bbox.contains(pointer_pos) {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        }
    }

    fn start_resizing(&mut self, corner: ResizeCorner, bbox: Rect) {
        self.resizing_corner = Some(corner);
        self.resize_original_bbox = Some(bbox);
        self.resize_original_lines.clear();
        for &i in &self.selected_lines {
            if let Some(line) = self.lines.get(i) {
                self.resize_original_lines.push((i, line.clone()));
            }
        }
    }

    fn update_resizing(&mut self, pointer_pos: Pos2, corner: ResizeCorner) {
        if let Some(orig_bbox) = self.resize_original_bbox {
            let mut new_bbox = orig_bbox;
            match corner {
                ResizeCorner::TopLeft => {
                    new_bbox.min = pointer_pos + vec2(5.0, 5.0);
                }
                ResizeCorner::TopRight => {
                    new_bbox.max.x = pointer_pos.x - 5.0;
                    new_bbox.min.y = pointer_pos.y + 5.0;
                }
                ResizeCorner::BottomLeft => {
                    new_bbox.min.x = pointer_pos.x + 5.0;
                    new_bbox.max.y = pointer_pos.y - 5.0;
                }
                ResizeCorner::BottomRight => {
                    new_bbox.max = pointer_pos - vec2(5.0, 5.0);
                }
            }

            let scale_x = if orig_bbox.width() > 0.0 {
                new_bbox.width() / orig_bbox.width()
            } else {
                1.0
            };
            let scale_y = if orig_bbox.height() > 0.0 {
                new_bbox.height() / orig_bbox.height()
            } else {
                1.0
            };

            for (i, orig_line) in &self.resize_original_lines {
                if let Some(line) = self.lines.get_mut(*i) {
                    for (p, orig_p) in
                        line.points.iter_mut().zip(&orig_line.points)
                    {
                        let nx = new_bbox.min.x
                            + (orig_p.x - orig_bbox.min.x) * scale_x;
                        let ny = new_bbox.min.y
                            + (orig_p.y - orig_bbox.min.y) * scale_y;
                        *p = pos2(nx, ny);
                    }
                }
            }
        }
    }

    fn handle_eraser(&mut self, pointer_pos: Pos2) {
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
        if !deleted_lines.is_empty() {
            self.selected_lines.clear();
            self.undo_stack.extend_erase(deleted_lines);
        }
    }

    fn push_line(&mut self) {
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

    fn draw_previous_lines(&self, painter: &Painter, i: &usize, line: &Line) {
        if line.points.len() >= 2 {
            let points = line.points.clone();
            let color = if self.selected_lines.contains(&i) {
                // Highlight selected lines? Or just leave them as is and draw box?
                // Maybe slight tint?
                // For now, let's just keep original color, but maybe we can draw a highlight.
                line.color
            } else {
                line.color
            };

            painter
                .add(egui::Shape::line(points, Stroke::new(line.width, color)));
        }
    }

    fn draw_selections(&self, painter: &Painter) {
        // Draw selection rect (drag area)
        if let (Some(start), Some(current)) =
            (self.selection_start, self.selection_current)
            && self.current_tool == Tool::Selection
        {
            let rect = Rect::from_two_pos(start, current);
            draw_dotted_rect(&painter, rect, Stroke::new(1.0, Color32::GRAY));
        }

        // Draw bounding box around selected lines
        if self.current_tool == Tool::Selection {
            if let Some((_, expanded, corners)) = self.get_selection_info() {
                draw_dotted_rect(
                    &painter,
                    expanded,
                    Stroke::new(1.0, Color32::BLUE),
                );

                let corner_size = vec2(8.0, 8.0);
                for &corner in &corners {
                    let rect = Rect::from_center_size(corner, corner_size);
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

    fn draw_tool_bar(&mut self, ui: &mut Ui) {
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

            selection_start: None,
            selection_current: None,
            selected_lines: HashSet::new(),
            is_moving_selection: false,
            last_mouse_pos: None,
            resizing_corner: None,
            resize_original_bbox: None,
            resize_original_lines: Vec::new(),
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
            self.draw_tool_bar(ui);

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // color selection (only when brush is selected)
            ui.add_enabled_ui(self.current_tool == Tool::Brush, |ui| {
                self.palette.draw(ui);
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

            self.update_cursor(ctx, &response);

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                match self.current_tool {
                    Tool::Brush => {
                        if response.dragged()
                            && self.current_line.last() != Some(&pointer_pos)
                        {
                            self.current_line.push(pointer_pos);
                        }
                    }
                    Tool::Eraser => {
                        // 支援點擊或拖曳時刪除線條
                        if response.clicked() || response.dragged() {
                            self.handle_eraser(pointer_pos);
                        }
                    }
                    Tool::Selection => {
                        self.handle_selection(&response, pointer_pos)
                    }
                }
            }

            // 畫筆模式下，放開拖曳時儲存線條
            if response.drag_stopped()
                && self.current_tool == Tool::Brush
                && !self.current_line.is_empty()
            {
                self.push_line();
            }

            // 繪製所有已存檔的線條
            for (i, line) in self.lines.iter().enumerate() {
                self.draw_previous_lines(&painter, &i, line);
            }

            self.draw_selections(&painter);

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

fn draw_dotted_rect(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let dash_len = 5.0;
    let gap_len = 5.0;
    let points = vec![
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    painter.add(egui::Shape::dashed_line(&points, stroke, dash_len, gap_len));
}
