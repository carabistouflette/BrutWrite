use crate::models::ProjectMetadata;
use crate::storage;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

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
