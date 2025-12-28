use crate::storage;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_snapshots(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> crate::errors::Result<Vec<String>> {
    let (root_path, _metadata) = state.projects.get_context(project_id).await?;
    let repo = storage::LocalFileRepository;
    storage::list_snapshots(&repo, &root_path, &chapter_id).await
}

#[tauri::command]
pub async fn load_snapshot_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    filename: String,
) -> crate::errors::Result<String> {
    let (root_path, _metadata) = state.projects.get_context(project_id).await?;
    let repo = storage::LocalFileRepository;
    storage::read_snapshot_content(&repo, &root_path, &chapter_id, &filename).await
}

#[tauri::command]
pub async fn create_snapshot(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> crate::errors::Result<()> {
    let (root_path, _metadata) = state.projects.get_context(project_id).await?;
    let repo = storage::LocalFileRepository;
    storage::create_snapshot(&repo, &root_path, &chapter_id, &content).await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_snapshot(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    snapshot_filename: String,
) -> crate::errors::Result<String> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;

    // We need current content to create a safety snapshot
    let repo = storage::LocalFileRepository;

    // Get chapter filename from metadata
    let chapter_filename = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)?
        .file_name()
        .ok_or_else(|| crate::errors::Error::InvalidStructure {
            path: root_path.clone(),
            reason: "Invalid chapter path".to_string(),
        })?
        .to_string_lossy()
        .to_string();

    let current_content =
        storage::read_chapter_content(&repo, &root_path, &metadata, &chapter_id).await?;

    let new_content = storage::restore_snapshot(
        &repo,
        &root_path,
        &chapter_id,
        &snapshot_filename,
        &current_content,
        &chapter_filename,
    )
    .await?;

    // Update word count
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = crate::models::count_words(&new_content);
    }

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).await?;

    Ok(new_content)
}

#[tauri::command]
pub async fn branch_snapshot(
    state: State<'_, AppState>,
    project_id: Uuid,
    snapshot_chapter_id: String,
    snapshot_filename: String,
) -> crate::errors::Result<crate::models::ProjectMetadata> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;
    let repo = storage::LocalFileRepository;

    // 1. Read snapshot content
    let content =
        storage::read_snapshot_content(&repo, &root_path, &snapshot_chapter_id, &snapshot_filename)
            .await?;

    // 2. Create new chapter in manifest (Branching off)
    // We can name it "Branch from <date>" or similar.
    let timestamp = snapshot_filename.split('_').next().unwrap_or("snapshot");
    let name = format!("Branch from {}", timestamp);

    // Find parent_id of the original chapter to place the branch as a sibling
    let parent_id = metadata
        .manifest
        .chapters
        .iter()
        .find(|c| c.id == snapshot_chapter_id)
        .and_then(|c| c.parent_id.clone());

    let new_chapter = metadata.manifest.create_chapter(parent_id, name);

    // 3. Write file
    storage::write_chapter_file(&repo, &root_path, &new_chapter.filename, &content).await?;

    // 4. Update metadata
    // Update word count for the new chapter
    let mut chapter_to_update = new_chapter; // It was returned by value
    chapter_to_update.word_count = crate::models::count_words(&content);

    metadata.manifest.chapters.push(chapter_to_update);
    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).await?;

    Ok(metadata.clone())
}
