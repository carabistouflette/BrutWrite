use crate::models::{Manifest, NodeMetadataUpdate, ProjectMetadata};
use crate::storage;
use crate::validation;
use crate::AppState;
use std::path::Path;
use tauri::State;
use uuid::Uuid;

fn rollback_failure(
    root_path: &Path,
    operation_error: crate::errors::Error,
    rollback_error: crate::errors::Error,
) -> crate::errors::Error {
    crate::errors::Error::InvalidStructure {
        path: root_path.to_path_buf(),
        reason: format!(
            "{}; restoring chapter files also failed: {}",
            operation_error, rollback_error
        ),
    }
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> crate::errors::Result<ProjectMetadata> {
    state
        .projects
        .mutate_project(project_id, |metadata| {
            metadata.manifest = metadata.manifest.merge_structure_update(manifest)?;
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
            let Some(index) = metadata
                .manifest
                .chapters
                .iter()
                .position(|chapter| chapter.id == node_id)
            else {
                return Err(crate::errors::Error::ChapterNotFound { id: node_id });
            };

            let chapter = &mut metadata.manifest.chapters[index];
            if let Some(title) = update.title {
                chapter.title = title;
            }
            if let Some(date) = update.chronological_date {
                chapter.chronological_date = date;
            }
            if let Some(timeframe) = update.abstract_timeframe {
                chapter.abstract_timeframe = timeframe;
            }
            if let Some(duration) = update.duration {
                chapter.duration = duration;
            }
            if let Some(plotline) = update.plotline_tag {
                chapter.plotline_tag = plotline;
            }
            if let Some(dependency) = update.depends_on {
                chapter.depends_on = dependency;
            }
            if let Some(pov_character_id) = update.pov_character_id {
                chapter.pov_character_id = pov_character_id;
            }

            Ok(())
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
    validation::validate_chapter_title(&name)?;

    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    metadata.manifest.validate_parent(parent_id.as_deref())?;

    let new_chapter = metadata.manifest.create_chapter(parent_id, name);
    let mut updated = metadata.clone();
    updated.manifest.chapters.push(new_chapter.clone());
    updated.manifest.validate()?;
    updated.updated_at = chrono::Utc::now();

    let repo = storage::LocalFileRepository;
    storage::write_chapter_file(&repo, &root_path, &new_chapter.filename, "").await?;
    if let Err(save_error) = storage::save_project_metadata(&root_path, &updated).await {
        return match storage::delete_chapter_file(&repo, &root_path, &new_chapter.filename).await {
            Ok(()) => Err(save_error),
            Err(rollback_error) => Err(rollback_failure(&root_path, save_error, rollback_error)),
        };
    }

    *metadata = updated.clone();
    Ok(updated)
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
) -> crate::errors::Result<ProjectMetadata> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    let mut updated = metadata.clone();
    let removed = updated.manifest.remove_node_recursively(&id)?;
    updated.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &updated).await?;
    *metadata = updated.clone();

    let repo = storage::LocalFileRepository;
    let mut cleanup_failures = Vec::new();
    for (chapter_id, filename) in removed {
        if let Err(error) = storage::delete_chapter_file(&repo, &root_path, &filename).await {
            cleanup_failures.push(format!("chapter file {filename}: {error}"));
        }
        if let Err(error) = storage::delete_chapter_snapshots(&repo, &root_path, &chapter_id).await
        {
            cleanup_failures.push(format!("snapshots for {chapter_id}: {error}"));
        }
    }

    if !cleanup_failures.is_empty() {
        return Err(crate::errors::Error::InvalidStructure {
            path: root_path,
            reason: format!(
                "Chapter tree was saved, but filesystem cleanup failed: {}",
                cleanup_failures.join("; ")
            ),
        });
    }

    Ok(updated)
}
