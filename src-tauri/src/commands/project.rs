use crate::models::{Plotline, ProjectMetadata, ProjectSettings};
use crate::AppState;
use crate::{research, storage};
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn create_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
    name: String,
    author: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::create_project_structure(&root_path, &name, &author)
        .await
        .map_err(|e| e.to_string())?;

    state
        .register_project(metadata.id, root_path.clone(), metadata.clone())
        .await;

    research::init_research_watcher(&app, root_path);

    Ok(metadata)
}

#[tauri::command]
pub async fn load_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path)
        .await
        .map_err(|e| e.to_string())?;

    state
        .register_project(metadata.id, root_path.clone(), metadata.clone())
        .await;

    research::init_research_watcher(&app, root_path);

    Ok(metadata)
}

#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    project_id: Uuid,
    settings: ProjectSettings,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.settings = settings;
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn update_plotlines(
    state: State<'_, AppState>,
    project_id: Uuid,
    plotlines: Vec<Plotline>,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.plotlines = plotlines;
            Ok(())
        })
        .await
}
