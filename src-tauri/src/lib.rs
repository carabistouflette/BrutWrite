pub mod commands;
pub mod errors;
pub mod models;

pub mod research;
pub mod storage;

pub mod integrations;
pub mod project;

use crate::project::manager::ProjectManager;

pub struct AppState {
    pub projects: ProjectManager,
    pub research: research::ResearchState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: ProjectManager::new(),
            research: research::ResearchState::new(),
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
            commands::create_project,
            commands::load_project,
            commands::update_manifest,
            commands::load_chapter_content,
            commands::save_chapter,
            commands::delete_node,
            commands::save_character,
            commands::delete_character,
            commands::update_project_settings,
            commands::update_plotlines,
            commands::create_node,
            commands::update_node_metadata,
            commands::get_research_artifacts,
            commands::add_research_files,
            commands::create_research_note,
            commands::update_note_content,
            commands::rename_research_artifact,
            commands::delete_research_artifact,
            commands::list_snapshots,
            commands::load_snapshot_content,
            commands::create_snapshot,
            commands::restore_snapshot,
            commands::branch_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
