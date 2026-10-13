use crate::models::ProjectMetadata;
use crate::storage;
use crate::storage::FileRepository;
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
            "{}; restoring chapter content also failed: {}",
            operation_error, rollback_error
        ),
    }
}

async fn restore_chapter_file(
    repo: &storage::LocalFileRepository,
    root_path: &Path,
    filename: &str,
    original_exists: bool,
    original_content: &str,
) -> crate::errors::Result<()> {
    if original_exists {
        storage::write_chapter_file(repo, root_path, filename, original_content).await
    } else {
        storage::delete_chapter_file(repo, root_path, filename).await
    }
}

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
    validation::validate_content_size(&content)?;

    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let mut metadata = metadata_arc.lock().await;
    let file_path = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)?;
    let filename = file_path
        .file_name()
        .ok_or_else(|| crate::errors::Error::InvalidStructure {
            path: root_path.clone(),
            reason: "Invalid chapter path".to_string(),
        })?
        .to_string_lossy()
        .to_string();
    let repo = storage::LocalFileRepository;
    let original_exists = repo.exists(&file_path).await?;
    let original_content = if original_exists {
        repo.read_file(&file_path).await?
    } else {
        String::new()
    };

    let mut updated = metadata.clone();
    let chapter = updated
        .manifest
        .chapters
        .iter_mut()
        .find(|chapter| chapter.id == chapter_id)
        .ok_or_else(|| crate::errors::Error::ChapterNotFound {
            id: chapter_id.clone(),
        })?;
    chapter.word_count = crate::models::count_words(&content);
    updated.updated_at = chrono::Utc::now();

    storage::create_snapshot(&repo, &root_path, &chapter_id, &content).await?;
    if let Err(write_error) =
        storage::write_chapter_file(&repo, &root_path, &filename, &content).await
    {
        return match restore_chapter_file(
            &repo,
            &root_path,
            &filename,
            original_exists,
            &original_content,
        )
        .await
        {
            Ok(()) => Err(write_error),
            Err(rollback_error) => Err(rollback_failure(&root_path, write_error, rollback_error)),
        };
    }

    if let Err(save_error) = storage::save_project_metadata(&root_path, &updated).await {
        return match restore_chapter_file(
            &repo,
            &root_path,
            &filename,
            original_exists,
            &original_content,
        )
        .await
        {
            Ok(()) => Err(save_error),
            Err(rollback_error) => Err(rollback_failure(&root_path, save_error, rollback_error)),
        };
    }

    *metadata = updated.clone();
    Ok(updated)
}
