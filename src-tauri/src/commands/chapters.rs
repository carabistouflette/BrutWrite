use crate::models::ProjectMetadata;
use crate::storage;
use crate::validation;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> crate::errors::Result<String> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let metadata = metadata_arc.lock().await;

    let repo = storage::LocalFileRepository;

    storage::read_chapter_content(&repo, &root_path, &metadata, &chapter_id).await
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> crate::errors::Result<ProjectMetadata> {
    // Validate content size to prevent resource exhaustion
    validation::validate_content_size(&content)?;

    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    // 1. Resolve filename to ensure chapter exists
    let filename = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)?
        .file_name()
        .ok_or_else(|| crate::errors::Error::InvalidStructure {
            path: root_path.clone(),
            reason: "Invalid chapter path".to_string(),
        })?
        .to_string_lossy()
        .to_string();

    // 2. Write content
    let repo = storage::LocalFileRepository;

    // Create snapshot
    storage::create_snapshot(&repo, &root_path, &chapter_id, &content).await?;

    storage::write_chapter_file(&repo, &root_path, &filename, &content).await?;

    // 3. Update word count
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = crate::models::count_words(&content);
    } else {
        return Err(crate::errors::Error::ChapterNotFound { id: chapter_id });
    }

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).await?;

    Ok(metadata.clone())
}
