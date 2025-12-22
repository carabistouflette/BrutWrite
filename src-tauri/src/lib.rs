pub mod commands;
pub mod errors;
pub mod models;
pub mod storage;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct AppState {
    pub open_projects: Mutex<HashMap<Uuid, PathBuf>>,
    pub project_cache: Mutex<HashMap<Uuid, Arc<Mutex<models::ProjectMetadata>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            open_projects: Mutex::new(HashMap::new()),
            project_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_context(
        &self,
        project_id: Uuid,
    ) -> Result<(PathBuf, Arc<Mutex<models::ProjectMetadata>>), String> {
        let root_path = {
            let projects = self
                .open_projects
                .lock()
                .map_err(|_| "Failed to lock projects")?;
            projects
                .get(&project_id)
                .cloned()
                .ok_or_else(|| "Project not loaded".to_string())?
        };

        let cache = self
            .project_cache
            .lock()
            .map_err(|_| "Failed to lock cache")?;
        let metadata = cache
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Metadata not in cache".to_string())?;

        Ok((root_path, metadata))
    }

    pub fn mutate_project<F>(
        &self,
        project_id: Uuid,
        mutation: F,
    ) -> Result<models::ProjectMetadata, String>
    where
        F: FnOnce(&mut models::ProjectMetadata) -> Result<(), String>,
    {
        let (root_path, metadata_arc) = self.get_context(project_id)?;

        // Lock the specific project logic
        let mut metadata = metadata_arc.lock().map_err(|_| "Failed to lock metadata")?;

        mutation(&mut metadata)?;

        metadata.updated_at = chrono::Utc::now();
        storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

        Ok(metadata.clone())
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
            commands::update_node_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
