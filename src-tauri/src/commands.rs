use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

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
        .map_err(|_| "Failed to lock projects")?;
    projects.insert(metadata.id, root_path);

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(metadata.id, metadata.clone());

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
        .map_err(|_| "Failed to lock projects")?;
    projects.insert(metadata.id, root_path);

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(metadata.id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    metadata.manifest = manifest;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    project_id: Uuid,
    settings: crate::models::ProjectSettings,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    metadata.settings = settings;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_plotlines(
    state: State<'_, AppState>,
    project_id: Uuid,
    plotlines: Vec<crate::models::Plotline>,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    metadata.plotlines = plotlines;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> Result<String, String> {
    let (root_path, metadata) = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        let cache = state
            .project_cache
            .lock()
            .map_err(|_| "Failed to lock cache")?;

        let root = projects
            .get(&project_id)
            .ok_or_else(|| "Project not loaded".to_string())?
            .clone();
        let meta = cache
            .get(&project_id)
            .ok_or_else(|| "Metadata not in cache".to_string())?
            .clone();
        (root, meta)
    };

    storage::read_chapter_content(root_path, &metadata, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    // 1. Resolve Path using cached metadata
    let chapter_path = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)
        .map_err(|e| e.to_string())?;

    // 2. Write content
    std::fs::write(&chapter_path, content.clone()).map_err(|e| e.to_string())?;

    // 3. Update metadata in cache
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = content.split_whitespace().count() as u32;
    }
    metadata.updated_at = chrono::Utc::now();

    // 4. Save metadata
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
    filenames_to_delete: Vec<String>,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    // 1. Delete files from disk
    for filename in filenames_to_delete {
        let file_path = root_path.join("manuscript").join(filename);
        if file_path.exists() {
            std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
        }
    }

    // 2. Remove from manifest
    metadata
        .manifest
        .chapters
        .retain(|c| c.id != id && c.parent_id != Some(id.clone()));

    // Note: This only handles one level deep if parent_id is used.
    // In a real tree we'd need a recursive removal from the flat list,
    // but the frontend sends the exact list of filenames to delete anyway.
    // The flat manifest list just needs to be filtered.
    metadata.manifest.chapters.retain(|c| c.id != id);

    metadata.updated_at = chrono::Utc::now();

    // 3. Save and Cache
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    if let Some(idx) = metadata
        .characters
        .iter()
        .position(|c| c.id == character.id)
    {
        metadata.characters[idx] = character;
    } else {
        metadata.characters.push(character);
    }
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let mut metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    metadata.characters.retain(|c| c.id != character_id);
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}
