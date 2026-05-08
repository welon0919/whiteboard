use eframe::egui::{self, Pos2, Rect, Response, vec2, pos2};

use crate::app::WhiteboardApp;
use crate::element::Element;
use crate::tools::Tool;
use crate::utils::{ResizeCorner, distance_point_to_segment};

impl WhiteboardApp {
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
                                    Element::Image(img) => {
                                        img.pos += delta;
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

        if self.current_tool == Tool::Image {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            if response.hovered() {
                response.clone().on_hover_text("Click to put image here");
            }
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
                        (Element::Image(img), Element::Image(orig_img)) => {
                            img.pos.x = new_bbox.min.x + (orig_img.pos.x - orig_bbox.min.x) * scale_x;
                            img.pos.y = new_bbox.min.y + (orig_img.pos.y - orig_bbox.min.y) * scale_y;
                            img.size.x = (orig_img.size.x * scale_x).max(5.0);
                            img.size.y = (orig_img.size.y * scale_y).max(5.0);
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
                Element::Text(_) | Element::Image(_) => {
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
}
