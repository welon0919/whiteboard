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
    element::{Element, Line, TextElement},
    state::WhiteboardState,
    tools::{TOOLS, Tool},
    undo::{UndoAction, UndoStack},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub const CANVAS_SIZE: f32 = 5000.0;

pub struct WhiteboardApp {
    pub elements: Vec<Element>,
    pub current_line: Vec<Pos2>,
    pub palette: ColorPalette,
    pub stroke_width: f32,
    pub current_tool: Tool,
    pub undo_stack: UndoStack,
    pub whiteboard_file: Option<PathBuf>,

    // View state
    pub view_offset: egui::Vec2,
    pub initialized: bool,

    // Selection tool state
    pub selection_start: Option<Pos2>,
    pub selection_current: Option<Pos2>,
    pub selected_elements: HashSet<usize>,
    pub is_moving_selection: bool,
    pub last_mouse_pos: Option<Pos2>,
    pub resizing_corner: Option<ResizeCorner>,
    pub resize_original_bbox: Option<Rect>,
    pub resize_original_elements: Vec<(usize, Element)>,

    // Text tool state
    pub editing_text: Option<usize>,
    pub editing_text_original: Option<Element>,
    pub focus_text_editor: bool,
}

impl WhiteboardApp {
    pub fn set_window_title(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Simple Whiteboard - {}",
            self.whiteboard_file
                .as_ref()
                .map_or("Untitled.wb".to_owned(), |s| s.display().to_string())
        )));
    }

    pub fn handle_keyboard_event(&mut self, ctx: &egui::Context) {
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
                        egui::Key::A if modifiers.command => {
                            if self.editing_text.is_none() {
                                self.selected_elements.clear();
                                for i in 0..self.elements.len() {
                                    self.selected_elements.insert(i);
                                }
                                self.selection_start = None;
                                self.selection_current = None;
                                self.is_moving_selection = false;
                                self.resizing_corner = None;
                                self.resize_original_bbox = None;
                                self.resize_original_elements.clear();
                                self.current_tool = Tool::Selection;
                            }
                        }
                        egui::Key::Z if modifiers.command => {
                            if self.editing_text.is_none() {
                                self.undo();
                            }
                        }
                        egui::Key::C if !modifiers.command => {
                            if self.editing_text.is_none() {
                                self.elements.clear();
                                self.selected_elements.clear();
                            }
                        }
                        egui::Key::B if !modifiers.command => {
                            if self.editing_text.is_none() {
                                self.current_tool = Tool::Brush;
                            }
                        }
                        egui::Key::E if !modifiers.command => {
                            if self.editing_text.is_none() {
                                self.current_tool = Tool::Eraser;
                            }
                        }
                        egui::Key::S if !modifiers.command => {
                            if self.editing_text.is_none() && self.current_tool != Tool::Selection {
                                self.selected_elements.clear();
                                self.selection_start = None;
                                self.selection_current = None;
                                self.is_moving_selection = false;
                                self.resizing_corner = None;
                                self.resize_original_bbox = None;
                                self.resize_original_elements.clear();
                                self.current_tool = Tool::Selection;
                            }
                        }
                        egui::Key::T if !modifiers.command => {
                            if self.editing_text.is_none() {
                                self.current_tool = Tool::Text;
                            }
                        }
                        egui::Key::S if modifiers.command => {
                            should_save = true;
                        }
                        egui::Key::O if modifiers.command => {
                            should_open = true;
                        }
                        egui::Key::Num1 => self.palette.set_active_color_index(0),
                        egui::Key::Num2 => self.palette.set_active_color_index(1),
                        egui::Key::Num3 => self.palette.set_active_color_index(2),
                        egui::Key::Num4 => self.palette.set_active_color_index(3),
                        egui::Key::Num5 => self.palette.set_active_color_index(4),
                        egui::Key::Num6 => self.palette.set_active_color_index(5),
                        egui::Key::Num7 => self.palette.set_active_color_index(6),
                        egui::Key::Num8 => self.palette.set_active_color_index(7),
                        egui::Key::Num9 => self.palette.set_active_color_index(8),
                        egui::Key::Delete | egui::Key::Backspace => {
                            if self.editing_text.is_none() && !self.selected_elements.is_empty() {
                                let mut indices: Vec<_> = self
                                    .selected_elements
                                    .iter()
                                    .copied()
                                    .collect();
                                indices.sort_unstable_by(|a, b| b.cmp(a)); // sort descending

                                let mut deleted_elems = Vec::new();
                                for index in indices {
                                    if index < self.elements.len() {
                                        deleted_elems.push((
                                            index,
                                            self.elements.remove(index),
                                        ));
                                    }
                                }
                                self.undo_stack.add_erase(deleted_elems);
                                self.selected_elements.clear();
                            }
                        }
                        egui::Key::Escape => {
                            if self.editing_text.is_none() {
                                self.selected_elements.clear();
                                self.selection_start = None;
                                self.selection_current = None;
                                self.is_moving_selection = false;
                                self.resizing_corner = None;
                                self.resize_original_bbox = None;
                                self.resize_original_elements.clear();
                            }
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

    pub fn undo(&mut self) {
        self.selected_elements.clear();
        match self.undo_stack.pop() {
            None => {}
            Some(action) => match action {
                UndoAction::Erase(mut elems) => {
                    elems.sort_by_key(|(idx, _)| *idx);
                    for (index, elem) in elems {
                        if index <= self.elements.len() {
                            self.elements.insert(index, elem);
                        } else {
                            self.elements.push(elem);
                        }
                    }
                }
                UndoAction::Draw => {
                    self.elements.pop();
                }
                UndoAction::Modify(elems) => {
                    for (index, elem) in elems {
                        if let Some(target) = self.elements.get_mut(index) {
                            *target = elem;
                        }
                    }
                }
            },
        }
    }

    pub fn write_whiteboard(&mut self, file_path: PathBuf, json: String) {
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

    pub fn save_whiteboard(&mut self) {
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

    pub fn open_whiteboard_file(&mut self) -> io::Result<()> {
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
                    self.elements = state.elements.iter().map(Into::into).collect();
                    self.initialized = false;
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

    pub fn handle_selection(&mut self, ctx: &egui::Context, response: &Response, pointer_pos: Pos2) {
        {
            // Check if we are interacting with existing selection
            let selection_info = self.get_selection_info(ctx);
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
                if !self.selected_elements.is_empty()
                    && tl_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::TopLeft, bounding_box);
                } else if !self.selected_elements.is_empty()
                    && tr_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::TopRight, bounding_box);
                } else if !self.selected_elements.is_empty()
                    && bl_rect.contains(pointer_pos)
                {
                    self.start_resizing(ResizeCorner::BottomLeft, bounding_box);
                } else if !self.selected_elements.is_empty()
                    && br_rect.contains(pointer_pos)
                {
                    self.start_resizing(
                        ResizeCorner::BottomRight,
                        bounding_box,
                    );
                } else if expanded_bbox.contains(pointer_pos)
                    && !self.selected_elements.is_empty()
                {
                    self.is_moving_selection = true;
                    self.last_mouse_pos = Some(pointer_pos);
                    self.resize_original_elements.clear();
                    for &i in &self.selected_elements {
                        if let Some(elem) = self.elements.get(i) {
                            self.resize_original_elements.push((i, elem.clone()));
                        }
                    }
                } else {
                    self.selected_elements.clear();
                    self.selection_start = Some(pointer_pos);
                    self.selection_current = Some(pointer_pos);
                }
            } else if response.dragged() {
                if let Some(corner) = self.resizing_corner {
                    self.update_resizing(pointer_pos, corner);
                } else if self.is_moving_selection {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let delta = pointer_pos - last_pos;
                        for i in &self.selected_elements {
                            if let Some(elem) = self.elements.get_mut(*i) {
                                match elem {
                                    Element::Line(line) => {
                                        for p in &mut line.points {
                                            *p += delta;
                                        }
                                    }
                                    Element::Text(text) => {
                                        text.pos += delta;
                                    }
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
                    self.undo_stack
                        .add_modify(self.resize_original_elements.clone());
                    self.resizing_corner = None;
                    self.resize_original_bbox = None;
                    self.resize_original_elements.clear();
                } else if self.is_moving_selection {
                    self.undo_stack
                        .add_modify(self.resize_original_elements.clone());
                    self.is_moving_selection = false;
                    self.last_mouse_pos = None;
                    self.resize_original_elements.clear();
                } else if let (Some(start), Some(current)) =
                    (self.selection_start, self.selection_current)
                {
                    let rect = Rect::from_two_pos(start, current);
                    self.selected_elements.clear();
                    for (i, elem) in self.elements.iter().enumerate() {
                        let elem_bbox = elem.bounding_box(ctx);
                        if rect.intersects(elem_bbox) {
                            self.selected_elements.insert(i);
                        }
                    }
                    self.selection_start = None;
                    self.selection_current = None;
                }
            } else if response.clicked() {
                if !expanded_bbox.contains(pointer_pos) {
                    self.selected_elements.clear();
                }
            }
        }
    }

    pub fn get_selection_info(&self, ctx: &egui::Context) -> Option<(Rect, Rect, [Pos2; 4])> {
        if self.selected_elements.is_empty() {
            return None;
        }

        let mut bounding_box = Rect::NOTHING;
        for &i in &self.selected_elements {
            if let Some(elem) = self.elements.get(i) {
                bounding_box = bounding_box.union(elem.bounding_box(ctx));
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

    pub fn update_cursor(&self, ctx: &egui::Context, response: &Response) {
        if self.current_tool == Tool::Move {
            if response.dragged() {
                ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else {
                ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
            return;
        }

        if self.current_tool == Tool::Text {
            ctx.set_cursor_icon(egui::CursorIcon::Text);
            return;
        }

        if self.current_tool != Tool::Selection {
            return;
        }

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

        if let Some(pointer_pos) = response.hover_pos() {
            let canvas_pos = pointer_pos + self.view_offset;
            if let Some((_, expanded_bbox, corners)) = self.get_selection_info(ctx) {
                let hit_size = vec2(10.0, 10.0);
                let tl_rect = Rect::from_center_size(corners[0], hit_size);
                let tr_rect = Rect::from_center_size(corners[1], hit_size);
                let bl_rect = Rect::from_center_size(corners[2], hit_size);
                let br_rect = Rect::from_center_size(corners[3], hit_size);

                if tl_rect.contains(canvas_pos) || br_rect.contains(canvas_pos) {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                } else if tr_rect.contains(canvas_pos) || bl_rect.contains(canvas_pos) {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                } else if expanded_bbox.contains(canvas_pos) {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        }
    }

    pub fn start_resizing(&mut self, corner: ResizeCorner, bbox: Rect) {
        self.resizing_corner = Some(corner);
        self.resize_original_bbox = Some(bbox);
        self.resize_original_elements.clear();
        for &i in &self.selected_elements {
            if let Some(elem) = self.elements.get(i) {
                self.resize_original_elements.push((i, elem.clone()));
            }
        }
    }

    pub fn update_resizing(&mut self, pointer_pos: Pos2, corner: ResizeCorner) {
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

            for (i, orig_elem) in &self.resize_original_elements {
                if let Some(elem) = self.elements.get_mut(*i) {
                    match (elem, orig_elem) {
                        (Element::Line(line), Element::Line(orig_line)) => {
                            for (p, orig_p) in line.points.iter_mut().zip(&orig_line.points) {
                                let nx = new_bbox.min.x + (orig_p.x - orig_bbox.min.x) * scale_x;
                                let ny = new_bbox.min.y + (orig_p.y - orig_bbox.min.y) * scale_y;
                                *p = pos2(nx, ny);
                            }
                        }
                        (Element::Text(text), Element::Text(orig_text)) => {
                            text.pos.x = new_bbox.min.x + (orig_text.pos.x - orig_bbox.min.x) * scale_x;
                            text.pos.y = new_bbox.min.y + (orig_text.pos.y - orig_bbox.min.y) * scale_y;
                            text.size = (orig_text.size * scale_y).max(5.0);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn handle_eraser(&mut self, pointer_pos: Pos2, ctx: &egui::Context) {
        let erase_radius = self.stroke_width + 5.0;

        let mut kept = Vec::new();
        let mut deleted_elems = Vec::new();
        for (i, elem) in self.elements.drain(..).enumerate() {
            let mut hit = false;
            match &elem {
                Element::Line(line) => {
                    for window in line.points.windows(2) {
                        if distance_point_to_segment(pointer_pos, window[0], window[1])
                            < erase_radius
                        {
                            hit = true;
                            break;
                        }
                    }
                }
                Element::Text(text) => {
                    if elem.bounding_box(ctx).expand(erase_radius).contains(pointer_pos) {
                        hit = true;
                    }
                }
            }
            if hit {
                deleted_elems.push((i, elem));
            } else {
                kept.push(elem);
            }
        }

        self.elements = kept;
        if !deleted_elems.is_empty() {
            self.selected_elements.clear();
            self.undo_stack.add_erase(deleted_elems);
        }
    }

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
        }
    }
}

pub fn distance_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
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
                                let new_text = TextElement {
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
                self.draw_previous_elements(ctx, &painter, &i, elem);
            }

            self.draw_selections(ctx, &painter);

            if self.current_tool == Tool::Brush && self.current_line.len() >= 2
            {
                let points: Vec<Pos2> = self
                    .current_line
                    .iter()
                    .map(|&p| p - self.view_offset)
                    .collect();
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(
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

pub fn draw_dotted_rect(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
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
