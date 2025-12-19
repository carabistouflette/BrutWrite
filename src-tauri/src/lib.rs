pub mod models;
pub mod errors;
pub mod storage;

use std::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;
use models::ProjectMetadata;

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
    author: String
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::create_project_structure(&root_path, &name, &author)
        .map_err(|e| e.to_string())?;

    let mut projects = state.open_projects.lock().map_err(|_| "Failed to lock state")?;
    projects.insert(metadata.id, root_path);

    Ok(metadata)
}

#[tauri::command]
async fn load_project(
    state: State<'_, AppState>,
    path: String
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path)
        .map_err(|e| e.to_string())?;

    let mut projects = state.open_projects.lock().map_err(|_| "Failed to lock state")?;
    projects.insert(metadata.id, root_path);

    Ok(metadata)
}

#[tauri::command]
async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String
) -> Result<(), String> {
    let projects = state.open_projects.lock().map_err(|_| "Failed to lock state")?;
    
    let root_path = projects.get(&project_id)
        .ok_or_else(|| "Project not loaded".to_string())?;

    // In a real app, we might want to cache the manifest or look it up efficiently.
    // For now, let's load metadata to find the filename for the chapter ID.
    // Optimization: The client *could* pass the filename, but passing ID is safer/cleaner for the API contract.
    let metadata = storage::load_project_metadata(root_path).map_err(|e| e.to_string())?;
    
    let filename = metadata.manifest.chapters.iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.filename.clone());

    if let Some(fname) = filename {
        storage::save_chapter_content(root_path, &fname, &content)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        // Fallback: if chapter not found in manifest, maybe we should error?
        // Or is this a new chapter? For now, error.
        Err(format!("Chapter {} not found in project manifest", chapter_id))
    }
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
            save_chapter
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
