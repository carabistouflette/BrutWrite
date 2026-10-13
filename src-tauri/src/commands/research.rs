use crate::models::research::ResearchArtifact;

use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn get_research_artifacts(
    state: State<'_, AppState>,
) -> crate::errors::Result<Vec<std::sync::Arc<ResearchArtifact>>> {
    Ok(state.research.get_all().await)
}

#[tauri::command]
pub async fn add_research_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> crate::errors::Result<()> {
    state.research.import_files(paths).await
}

#[tauri::command]
pub async fn create_research_note(
    state: State<'_, AppState>,
    name: String,
) -> crate::errors::Result<std::sync::Arc<ResearchArtifact>> {
    state.research.create_note(name).await
}

#[tauri::command]
pub async fn update_note_content(
    state: State<'_, AppState>,
    id: String,
    expected_path: String,
    content: String,
) -> crate::errors::Result<()> {
    state
        .research
        .update_content(id, expected_path, content)
        .await
}

#[tauri::command]
pub async fn update_research_artifact_tags(
    state: State<'_, AppState>,
    id: String,
    tags: Vec<String>,
) -> crate::errors::Result<()> {
    state.research.update_tags(id, tags).await
}

#[tauri::command]
pub async fn rename_research_artifact(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> crate::errors::Result<()> {
    state.research.rename_artifact(id, new_name).await
}

#[tauri::command]
pub async fn delete_research_artifact(
    state: State<'_, AppState>,
    id: String,
) -> crate::errors::Result<()> {
    state.research.delete_artifact(id).await
}
