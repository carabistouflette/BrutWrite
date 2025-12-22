use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
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
    let metadata = storage::create_project_structure(&root_path, &name, &author)
        .await
        .map_err(|e| e.to_string())?;

    state
        .open_projects
        .lock()
        .await
        .insert(metadata.id, root_path);

    state
        .project_cache
        .lock()
        .await
        .insert(metadata.id, Arc::new(Mutex::new(metadata.clone())));

    Ok(metadata)
}

#[tauri::command]
pub async fn load_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path)
        .await
        .map_err(|e| e.to_string())?;

    state
        .open_projects
        .lock()
        .await
        .insert(metadata.id, root_path);

    state
        .project_cache
        .lock()
        .await
        .insert(metadata.id, Arc::new(Mutex::new(metadata.clone())));

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.manifest = manifest;
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn update_node_metadata(
    state: State<'_, AppState>,
    project_id: Uuid,
    node_id: String,
    update: crate::models::NodeMetadataUpdate,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
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
        .await
}

#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    project_id: Uuid,
    settings: crate::models::ProjectSettings,
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
    plotlines: Vec<crate::models::Plotline>,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.plotlines = plotlines;
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> Result<String, String> {
    let (root_path, metadata_arc) = state.get_context(project_id).await?;
    let metadata = metadata_arc.lock().await;
    storage::read_chapter_content(root_path, &metadata, &chapter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, metadata_arc) = state.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    // 1. Resolve filename to ensure chapter exists
    let filename = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .map_err(|e| e.to_string())?;

    // 2. Write content
    storage::write_chapter_file(&root_path, &filename, &content)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Update word count
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = crate::models::count_words(&content);
    } else {
        return Err("Chapter not found in manifest".to_string());
    }

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata)
        .await
        .map_err(|e| e.to_string())?;

    Ok(metadata.clone())
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    parent_id: Option<String>,
    name: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, metadata_arc) = state.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    // 1. Create entry in manifest (Domain Logic)
    let new_chapter = metadata.create_and_add_chapter(parent_id, name);

    // 2. Create physical file (Storage Logic)
    storage::write_chapter_file(&root_path, &new_chapter.filename, "")
        .await
        .map_err(|e| e.to_string())?;

    // 3. Save Metadata
    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata)
        .await
        .map_err(|e| e.to_string())?;

    Ok(metadata.clone())
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, _) = state.get_context(project_id).await?;

    let mut filenames = Vec::new();

    // 1. Remove from manifest recursively and get filenames
    let new_metadata = state
        .mutate_project(project_id, |metadata| {
            filenames = metadata.remove_node_recursively(id);
            Ok(())
        })
        .await?;

    // 2. Delete files from disk (Async)
    for filename in filenames {
        // Ignore errors during deletion (logging would be good here)
        let _ = storage::delete_chapter_file(&root_path, &filename).await;
    }

    Ok(new_metadata)
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.add_or_update_character(character);
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.remove_character(character_id)
        })
        .await
}
