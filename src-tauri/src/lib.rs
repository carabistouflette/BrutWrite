pub mod commands;
pub mod errors;
pub mod models;
pub mod storage;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub open_projects: Mutex<HashMap<Uuid, PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            open_projects: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::create_project,
            commands::load_project,
            commands::update_manifest,
            commands::load_chapter_content,
            commands::save_chapter,
            commands::save_character,
            commands::delete_character
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
