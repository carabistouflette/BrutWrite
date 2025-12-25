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

pub struct ProjectContext {
    pub path: PathBuf,
    pub metadata: Arc<Mutex<models::ProjectMetadata>>,
}

pub struct AppState {
    pub projects: Mutex<HashMap<Uuid, ProjectContext>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_context(
        &self,
        project_id: Uuid,
    ) -> crate::errors::Result<(PathBuf, Arc<Mutex<models::ProjectMetadata>>)> {
        let projects = self.projects.lock().await;
        let context = projects.get(&project_id).ok_or_else(|| {
            crate::errors::Error::InvalidStructure("Project not loaded".to_string())
        })?;

        Ok((context.path.clone(), context.metadata.clone()))
    }

    pub async fn mutate_project<F>(
        &self,
        project_id: Uuid,
        mutation: F,
    ) -> crate::errors::Result<models::ProjectMetadata>
    where
        F: FnOnce(&mut models::ProjectMetadata) -> crate::errors::Result<()> + Send,
    {
        let (root_path, metadata_arc) = self.get_context(project_id).await?;

        let mut metadata = metadata_arc.lock().await;

        mutation(&mut metadata)?;

        metadata.updated_at = chrono::Utc::now();

        storage::save_project_metadata(&root_path, &metadata).await?;

        Ok(metadata.clone())
    }
    pub async fn register_project(
        &self,
        id: Uuid,
        path: PathBuf,
        metadata: models::ProjectMetadata,
    ) {
        let mut projects = self.projects.lock().await;
        projects.insert(
            id,
            ProjectContext {
                path,
                metadata: Arc::new(Mutex::new(metadata)),
            },
        );
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
            commands::update_note_content,
            commands::rename_research_artifact,
            commands::delete_research_artifact
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
