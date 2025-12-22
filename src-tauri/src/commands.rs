use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;

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
        .insert(metadata.id, Arc::new(Mutex::new(metadata.clone())));

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
        .insert(metadata.id, Arc::new(Mutex::new(metadata.clone())));

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    state.mutate_project(project_id, |metadata| {
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
    state.mutate_project(project_id, |metadata| {
        if let Some(node) = metadata
            .manifest
            .chapters
            .iter_mut()
            .find(|c| c.id == node_id)
        {
            if let Some(t) = update.title {
                node.title = t;
            }
            if let Some(d) = update.chronological_date {
                node.chronological_date = Some(d);
            }
            if let Some(a) = update.abstract_timeframe {
                node.abstract_timeframe = Some(a);
            }
            if let Some(dur) = update.duration {
                node.duration = Some(dur);
            }
            if let Some(p) = update.plotline_tag {
                node.plotline_tag = Some(p);
            }
            if let Some(dep) = update.depends_on {
                node.depends_on = Some(dep);
            }
            if let Some(pov) = update.pov_character_id {
                node.pov_character_id = Some(pov);
            }
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
    state.mutate_project(project_id, |metadata| {
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
    state.mutate_project(project_id, |metadata| {
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
    let (root_path, metadata_arc) = state.get_context(project_id)?;
    let metadata = metadata_arc.lock().map_err(|_| "Failed to lock metadata")?;
    storage::read_chapter_content(root_path, &metadata, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = state.get_context(project_id)?;

    // 1. Resolve Path and Write content (using storage helper to avoid duplication of path logic if possible,
    // but save_chapter_content in storage.rs also tries to modify metadata which we want to control here to keep mutate_project_metadata)

    state.mutate_project(project_id, |metadata| {
        let chapter_path = storage::resolve_chapter_path(&root_path, metadata, &chapter_id)
            .map_err(|e| e.to_string())?;

        std::fs::write(&chapter_path, content.clone()).map_err(|e| e.to_string())?;

        if let Some(chapter) = metadata
            .manifest
            .chapters
            .iter_mut()
            .find(|c| c.id == chapter_id)
        {
            // Strip HTML tags before counting words
            chapter.word_count = crate::models::count_words(&content);
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
    let (root_path, _) = state.get_context(project_id)?;

    state.mutate_project(project_id, |metadata| {
        storage::create_chapter_node(&root_path, metadata, parent_id, name)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = state.get_context(project_id)?;

    let mut filenames = Vec::new();

    // 1. Remove from manifest recursively and get filenames
    let new_metadata = state.mutate_project(project_id, |metadata| {
        filenames = metadata.remove_node_recursively(id);
        Ok(())
    })?;

    // 2. Delete files from disk
    for filename in filenames {
        let file_path = root_path.join("manuscript").join(filename);
        if file_path.exists() {
            // Log error but don't fail the request if file deletion fails (orphaned file is better than broken state)
            let _ = std::fs::remove_file(file_path);
        }
    }

    Ok(new_metadata)
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    state.mutate_project(project_id, |metadata| {
        metadata.add_or_update_character(character);
        Ok(())
    })
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    state.mutate_project(project_id, |metadata| {
        metadata.remove_character(character_id)
    })
}
