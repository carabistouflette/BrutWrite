use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use tauri::State;

#[tauri::command]
pub async fn get_research_artifacts(
    state: State<'_, ResearchState>,
) -> crate::errors::Result<Vec<ResearchArtifact>> {
    Ok(state.get_all().await)
}

#[tauri::command]
pub async fn add_research_files(
    state: State<'_, ResearchState>,
    paths: Vec<String>,
) -> crate::errors::Result<()> {
    state.import_files(paths).await
}

#[tauri::command]
pub async fn create_research_note(
    state: State<'_, ResearchState>,
    name: String,
) -> crate::errors::Result<ResearchArtifact> {
    state.create_note(name).await
}

#[tauri::command]
pub async fn update_note_content(
    state: State<'_, ResearchState>,
    id: String,
    content: String,
) -> crate::errors::Result<()> {
    state.update_content(id, content).await
}

#[tauri::command]
pub async fn rename_research_artifact(
    state: State<'_, ResearchState>,
    id: String,
    new_name: String,
) -> crate::errors::Result<()> {
    state.rename_artifact(id, new_name).await
}

#[tauri::command]
pub async fn delete_research_artifact(
    state: State<'_, ResearchState>,
    id: String,
) -> crate::errors::Result<()> {
    state.delete_artifact(id).await
}
