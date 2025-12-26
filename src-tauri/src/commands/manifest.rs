use crate::models::{Manifest, NodeMetadataUpdate, ProjectMetadata};
use crate::storage;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> crate::errors::Result<ProjectMetadata> {
    state
        .projects
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
    update: NodeMetadataUpdate,
) -> crate::errors::Result<ProjectMetadata> {
    state
        .projects
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
                Err(crate::errors::Error::ChapterNotFound { id: node_id })
            }
        })
        .await
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    parent_id: Option<String>,
    name: String,
) -> crate::errors::Result<ProjectMetadata> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    // 1. Create entry in manifest (Domain Logic)
    let new_chapter = metadata.manifest.create_chapter(parent_id, name);

    // 2. Create physical file (Storage Logic)
    let repo = storage::LocalFileRepository;
    storage::write_chapter_file(&repo, &root_path, &new_chapter.filename, "").await?;

    // 3. Save Metadata
    metadata.manifest.chapters.push(new_chapter);

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).await?;

    Ok(metadata.clone())
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
) -> crate::errors::Result<ProjectMetadata> {
    let (root_path, _) = state.projects.get_context(project_id).await?;

    let mut filenames = Vec::new();

    // 1. Remove from manifest recursively and get filenames
    let new_metadata = state
        .projects
        .mutate_project(project_id, |metadata| {
            filenames = metadata.manifest.remove_node_recursively(id);
            Ok(())
        })
        .await?;

    // 2. Delete files from disk (Async)
    let repo = storage::LocalFileRepository;
    for filename in filenames {
        // Ignore errors during deletion (logging would be good here)
        let _ = storage::delete_chapter_file(&repo, &root_path, &filename).await;
    }

    Ok(new_metadata)
}
