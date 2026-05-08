pub mod app;
pub mod colors;
pub mod draw;
pub mod element;
pub mod input;
pub mod selection;
pub mod state;
pub mod storage;
pub mod tools;
pub mod undo;
pub mod utils;

pub use app::WhiteboardApp;
pub use element::{Element, Line, TextElement};
pub mod test_paste_img;
