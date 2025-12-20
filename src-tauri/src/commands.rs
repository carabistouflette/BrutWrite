use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    path: String,
    name: String,
    author: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata =
        storage::create_project_structure(&root_path, &name, &author).map_err(|e| e.to_string())?;

    let mut projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    projects.insert(metadata.id, root_path);

    Ok(metadata)
}

#[tauri::command]
pub async fn load_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path).map_err(|e| e.to_string())?;

    let mut projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    projects.insert(metadata.id, root_path);

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::update_project_manifest(root_path, manifest).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> Result<String, String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::read_chapter_content(root_path, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    filename: String,
    content: String,
) -> Result<(), String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::save_chapter_content(root_path, &filename, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::save_character(root_path, character).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::delete_character(root_path, character_id).map_err(|e| e.to_string())
}
