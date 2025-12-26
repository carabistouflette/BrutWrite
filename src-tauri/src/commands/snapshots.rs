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
