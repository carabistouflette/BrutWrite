pub mod commands;
pub mod errors;
pub mod models;

pub mod research;
pub mod storage;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
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

    pub async fn get_context(
        &self,
        project_id: Uuid,
    ) -> Result<(PathBuf, Arc<Mutex<models::ProjectMetadata>>), String> {
        let root_path = {
            let projects = self.open_projects.lock().await;
            projects
                .get(&project_id)
                .cloned()
                .ok_or_else(|| "Project not loaded".to_string())?
        };

        let cache = self.project_cache.lock().await;
        let metadata = cache
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Metadata not in cache".to_string())?;

        Ok((root_path, metadata))
    }

    pub async fn mutate_project<F>(
        &self,
        project_id: Uuid,
        mutation: F,
    ) -> Result<models::ProjectMetadata, String>
    where
        F: FnOnce(&mut models::ProjectMetadata) -> Result<(), String> + Send,
    {
        let (root_path, metadata_arc) = self.get_context(project_id).await?;

        // Lock the specific project logic
        let mut metadata = metadata_arc.lock().await;

        mutation(&mut metadata)?;

        metadata.updated_at = chrono::Utc::now();
        // Clone metadata to release the lock before critical IO?
        // Actually, we should hold the lock during save to ensure consistency?
        // But if save takes long, we block other operations on this project.
        // However, if we release and someone else modifies, and then we save... race condition.
        // Correct is: serialized modifications. So we SHOULD hold lock during save.
        // Since we are using tokio Mutex, we can await inside lock.
        storage::save_project_metadata(&root_path, &metadata)
            .await
            .map_err(|e| e.to_string())?;

        Ok(metadata.clone())
    }
    pub async fn register_project(
        &self,
        id: Uuid,
        path: PathBuf,
        metadata: models::ProjectMetadata,
    ) {
        self.open_projects.lock().await.insert(id, path);
        self.project_cache
            .lock()
            .await
            .insert(id, Arc::new(Mutex::new(metadata)));
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
        .manage(research::ResearchState::new())
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
            commands::update_research_artifact,
            commands::create_research_note,
            commands::update_note_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
