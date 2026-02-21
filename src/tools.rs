use serde::Serialize;

#[derive(PartialEq, Default, Serialize)]
pub enum Tool {
    #[default]
    Brush,
    Eraser,
    Selection,
}
