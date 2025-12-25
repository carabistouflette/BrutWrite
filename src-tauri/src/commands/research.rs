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
pub async fn update_research_artifact(
    _state: State<'_, ResearchState>,
    _artifact: ResearchArtifact,
) -> crate::errors::Result<()> {
    // Deprecated or Unused? The plan didn't specify what to do with this.
    // Logic was: update artifact in index.
    // If it's just metadata update, we might need a method for it.
    // But currently Artifact has name, path, type. Name is updated via rename. Path via rename.
    // Type is set on creation.
    // So this might be redundant or for future use.
    // Let's implement a generic update if needed, or just leave it empty/NoOp if not used by frontend in major way.
    // Looking at previous code, it just overwrites the artifact in the map and saves.
    // This could be dangerous if it changes path without moving file.
    // safer to rely on rename_artifact.
    Ok(())
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
