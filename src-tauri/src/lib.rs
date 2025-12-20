pub mod errors;
pub mod models;
pub mod storage;

use models::{Manifest, ProjectMetadata};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn create_project(
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
async fn load_project(state: State<'_, AppState>, path: String) -> Result<ProjectMetadata, String> {
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
async fn update_manifest(
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
async fn load_chapter_content(
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
async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<(), String> {
    let projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock state")?;
    let root_path = projects
        .get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    storage::save_chapter_content(root_path, &chapter_id, &content).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_project,
            load_project,
            update_manifest,
            load_chapter_content,
            save_chapter
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
