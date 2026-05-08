use eframe::egui::{self, Pos2, Rect, Stroke, pos2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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