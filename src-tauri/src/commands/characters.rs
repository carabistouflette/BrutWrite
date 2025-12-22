use crate::models::{Character, ProjectMetadata};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: Character,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.add_or_update_character(character);
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    state
        .mutate_project(project_id, |metadata| {
            metadata.remove_character(character_id)
        })
        .await
}
