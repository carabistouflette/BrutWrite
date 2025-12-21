use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

// --- Helpers ---

fn get_project_context(
    state: &State<'_, AppState>,
    project_id: Uuid,
) -> Result<(PathBuf, ProjectMetadata), String> {
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

    let cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    Ok((root_path, metadata))
}

fn mutate_project_metadata<F>(
    state: &State<'_, AppState>,
    project_id: Uuid,
    mutation: F,
) -> Result<ProjectMetadata, String>
where
    F: FnOnce(&mut ProjectMetadata) -> Result<(), String>,
{
    let (root_path, mut metadata) = get_project_context(state, project_id)?;

    mutation(&mut metadata)?;

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

// --- Commands ---

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

    state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock projects")?
        .insert(metadata.id, root_path);

    state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?
        .insert(metadata.id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn load_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path).map_err(|e| e.to_string())?;

    state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock projects")?
        .insert(metadata.id, root_path);

    state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?
        .insert(metadata.id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        metadata.manifest = manifest;
        Ok(())
    })
}

#[tauri::command]
pub async fn update_node_metadata(
    state: State<'_, AppState>,
    project_id: Uuid,
    node_id: String,
    update: crate::models::NodeMetadataUpdate,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        if let Some(node) = metadata
            .manifest
            .chapters
            .iter_mut()
            .find(|c| c.id == node_id)
        {
            if let Some(t) = update.title { node.title = t; }
            if let Some(d) = update.chronological_date { node.chronological_date = Some(d); }
            if let Some(a) = update.abstract_timeframe { node.abstract_timeframe = Some(a); }
            if let Some(dur) = update.duration { node.duration = Some(dur); }
            if let Some(p) = update.plotline_tag { node.plotline_tag = Some(p); }
            if let Some(dep) = update.depends_on { node.depends_on = Some(dep); }
            if let Some(pov) = update.pov_character_id { node.pov_character_id = Some(pov); }
            Ok(())
        } else {
            Err("Node not found".to_string())
        }
    })
}

#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    project_id: Uuid,
    settings: crate::models::ProjectSettings,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        metadata.settings = settings;
        Ok(())
    })
}

#[tauri::command]
pub async fn update_plotlines(
    state: State<'_, AppState>,
    project_id: Uuid,
    plotlines: Vec<crate::models::Plotline>,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        metadata.plotlines = plotlines;
        Ok(())
    })
}

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> Result<String, String> {
    let (root_path, metadata) = get_project_context(&state, project_id)?;
    storage::read_chapter_content(root_path, &metadata, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = get_project_context(&state, project_id)?;

    // 1. Resolve Path and Write content (using storage helper to avoid duplication of path logic if possible, 
    // but save_chapter_content in storage.rs also tries to modify metadata which we want to control here to keep mutate_project_metadata)
    
    mutate_project_metadata(&state, project_id, |metadata| {
        let chapter_path = storage::resolve_chapter_path(&root_path, metadata, &chapter_id)
            .map_err(|e| e.to_string())?;

        std::fs::write(&chapter_path, content.clone()).map_err(|e| e.to_string())?;

        if let Some(chapter) = metadata
            .manifest
            .chapters
            .iter_mut()
            .find(|c| c.id == chapter_id)
        {
            chapter.word_count = content.split_whitespace().count() as u32;
            Ok(())
        } else {
            Err("Chapter not found in manifest".to_string())
        }
    })
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    parent_id: Option<String>,
    name: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = get_project_context(&state, project_id)?;

    mutate_project_metadata(&state, project_id, |metadata| {
        storage::create_chapter_node(&root_path, metadata, parent_id, name).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
    filenames_to_delete: Vec<String>,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = get_project_context(&state, project_id)?;

    // 1. Delete files from disk
    for filename in filenames_to_delete {
        let file_path = root_path.join("manuscript").join(filename);
        if file_path.exists() {
            std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
        }
    }

    // 2. Remove from manifest recursively
    mutate_project_metadata(&state, project_id, |metadata| {
        let mut ids_to_remove = vec![id.clone()];
        let mut i = 0;
        while i < ids_to_remove.len() {
            let current = ids_to_remove[i].clone();
            for c in &metadata.manifest.chapters {
                if c.parent_id == Some(current.clone()) {
                    if !ids_to_remove.contains(&c.id) {
                         ids_to_remove.push(c.id.clone());
                    }
                }
            }
            i += 1;
        }
        
        metadata.manifest.chapters.retain(|c| !ids_to_remove.contains(&c.id));
        Ok(())
    })
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        if let Some(idx) = metadata
            .characters
            .iter()
            .position(|c| c.id == character.id)
        {
            metadata.characters[idx] = character;
        } else {
            metadata.characters.push(character);
        }
        Ok(())
    })
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    mutate_project_metadata(&state, project_id, |metadata| {
        let initial_len = metadata.characters.len();
        metadata.characters.retain(|c| c.id != character_id);
        
        if metadata.characters.len() == initial_len {
            return Err("Character not found".to_string());
        }
        Ok(())
    })
}

