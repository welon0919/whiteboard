use std::io;
use std::path::{Path, PathBuf};
use directories::UserDirs;
use eframe::egui;

use crate::app::WhiteboardApp;
use crate::state::WhiteboardState;

impl WhiteboardApp {
    pub fn set_window_title(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Simple Whiteboard - {}",
            self.whiteboard_file
                .as_ref()
                .map_or("Untitled.wb".to_owned(), |s| s.display().to_string())
        )));
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
}
