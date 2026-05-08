use eframe::egui;
use image::ImageEncoder;

use crate::app::WhiteboardApp;
use crate::tools::Tool;
use crate::undo::UndoAction;

impl WhiteboardApp {
    pub fn handle_keyboard_event(&mut self, ctx: &egui::Context) {
        let mut should_save = false;
        let mut should_open = false;
        
        let mut do_paste = false;
        let mut pointer_pos_for_paste = egui::pos2(100.0, 100.0);

        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::V) {
                do_paste = true;
                pointer_pos_for_paste = i.pointer.hover_pos().unwrap_or(egui::pos2(100.0, 100.0));
            }
            for event in &i.events {
                if let egui::Event::Paste(_) = event {
                    do_paste = true;
                    pointer_pos_for_paste = i.pointer.hover_pos().unwrap_or(egui::pos2(100.0, 100.0));
                }
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

        if do_paste && self.editing_text.is_none() {
            let canvas_pos = pointer_pos_for_paste + self.view_offset;

            match arboard::Clipboard::new() {
                Ok(mut cb) => {
                    if let Ok(img) = cb.get_image() {
                        let mut png_bytes = Vec::new();
                        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                        match encoder.write_image(&img.bytes, img.width as u32, img.height as u32, image::ExtendedColorType::Rgba8) {
                            Ok(_) => {
                                let id = self.image_id_counter;
                                self.image_id_counter += 1;
                                let img_elem = crate::element::ImageElement {
                                    id,
                                    bytes: std::sync::Arc::from(png_bytes.into_boxed_slice()),
                                    pos: canvas_pos,
                                    size: eframe::egui::vec2(img.width as f32, img.height as f32),
                                };
                                let idx = self.elements.len();
                                self.elements.push(crate::element::Element::Image(img_elem));
                                self.undo_stack.add_draw();
                                self.selected_elements.clear();
                                self.selected_elements.insert(idx);
                                self.current_tool = Tool::Selection;
                            }
                            Err(e) => {
                                eprintln!("Failed to encode image to PNG: {:?}", e);
                            }
                        }
                    } else if let Ok(text) = cb.get_text() {
                        let text = text.trim();
                        if text.starts_with("file://") || std::path::Path::new(text).exists() {
                            let path = if text.starts_with("file://") {
                                text.trim_start_matches("file://").to_string()
                            } else {
                                text.to_string()
                            };
                            let path = path.trim(); // clean up newlines
                            
                            // It's a file path. Let's see if it's an image.
                            if let Ok(bytes) = std::fs::read(&path) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let id = self.image_id_counter;
                                    self.image_id_counter += 1;
                                    let img_elem = crate::element::ImageElement {
                                        id,
                                        bytes: std::sync::Arc::from(bytes.into_boxed_slice()),
                                        pos: canvas_pos,
                                        size: eframe::egui::vec2(img.width() as f32, img.height() as f32),
                                    };
                                    let idx = self.elements.len();
                                    self.elements.push(crate::element::Element::Image(img_elem));
                                    self.undo_stack.add_draw();
                                    self.selected_elements.clear();
                                    self.selected_elements.insert(idx);
                                    self.current_tool = Tool::Selection;
                                    return; // exit early, handled as image
                                }
                            }
                        }
                        
                        if !text.is_empty() {
                            let text_elem = crate::element::TextElement {
                                text: text.to_string(),
                                pos: canvas_pos,
                                size: 20.0,
                                color: self.palette.get_current_color(),
                            };
                            let idx = self.elements.len();
                            self.elements.push(crate::element::Element::Text(text_elem));
                            self.undo_stack.add_draw();
                            self.selected_elements.clear();
                            self.selected_elements.insert(idx);
                            self.current_tool = Tool::Selection;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to init clipboard: {:?}", e);
                }
            }
        }

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
}
