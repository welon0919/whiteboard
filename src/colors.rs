use eframe::{
    emath::vec2,
    epaint::{Stroke, StrokeKind},
};
use egui::Color32;

pub struct ColorPalette {
    pub colors: Vec<Color32>,
    active_color_index: usize,
}
impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            colors: vec![
                Color32::WHITE,
                Color32::RED,
                Color32::YELLOW,
                Color32::GREEN,
                Color32::BLUE,
            ],
            active_color_index: 0,
        }
    }
}
impl ColorPalette {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        {
            ui.label("Color selection");
            ui.horizontal(|ui| {
                // draw color palette
                for i in 0..self.colors.len() {
                    let is_selected = i == self.active_color_index;
                    let is_add_btn = i == self.colors.len();

                    let frame = if is_selected {
                        egui::Frame::new()
                            .stroke(Stroke::new(2.0, ui.visuals().text_color()))
                            .inner_margin(2.0)
                            .corner_radius(4.0)
                    } else {
                        egui::Frame::new().inner_margin(4.0)
                    };

                    frame.show(ui, |ui| {
                        if is_selected {
                            ui.color_edit_button_srgba(&mut self.colors[i]);
                        } else {
                            let size = vec2(
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
                                    self.colors[i],
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    rounding,
                                    Stroke::new(
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
                // draw add color button
                let add_btn_size = vec2(
                    ui.spacing().interact_size.y,
                    ui.spacing().interact_size.y,
                );
                let (rect, response) = ui.allocate_at_least(
                    add_btn_size + vec2(8.0, 8.0),
                    egui::Sense::click(),
                );
                let btn_rect = rect.shrink(4.0);
                if ui.is_rect_visible(btn_rect) {
                    let visuals = ui.style().interact(&response);
                    let rounding = 2.0;
                    ui.painter().rect_filled(
                        btn_rect,
                        rounding,
                        visuals.bg_fill,
                    );
                    ui.painter().rect_stroke(
                        btn_rect,
                        rounding,
                        visuals.bg_stroke,
                        StrokeKind::Outside,
                    );
                    // draw the plus sign
                    let stroke = Stroke::new(2.0, visuals.fg_stroke.color);
                    let center = btn_rect.center();
                    let radius = btn_rect.width() / 4.0;
                    ui.painter().line_segment(
                        [
                            center - vec2(radius, 0.0),
                            center + vec2(radius, 0.0),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            center - vec2(0.0, radius),
                            center + vec2(0.0, radius),
                        ],
                        stroke,
                    );
                }
                if response.clicked() {
                    self.colors.push(self.get_current_color());
                    self.active_color_index = self.colors.len() - 1;
                }
            });
        }
    }
    pub fn set_active_color_index(&mut self, active_color_index: usize) {
        self.active_color_index = active_color_index;
    }
    pub fn get_current_color(&self) -> Color32 {
        self.colors[self.active_color_index]
    }
    pub fn get_palette_vec(&self) -> &[Color32] {
        &self.colors
    }
}
impl From<Vec<Color32>> for ColorPalette {
    fn from(colors: Vec<Color32>) -> Self {
        Self {
            colors,
            active_color_index: 0,
        }
    }
}
