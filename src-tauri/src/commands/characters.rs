use crate::models::{Character, ProjectMetadata};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: Character,
) -> crate::errors::Result<ProjectMetadata> {
    state
        .projects
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
) -> crate::errors::Result<ProjectMetadata> {
    state
        .projects
        .mutate_project(project_id, |metadata| {
            remove_character_and_pov_references(metadata, character_id)
                .map_err(crate::errors::Error::Validation)
        })
        .await
}

fn remove_character_and_pov_references(
    metadata: &mut ProjectMetadata,
    character_id: Uuid,
) -> Result<(), String> {
    metadata.remove_character(character_id)?;
    for chapter in &mut metadata.manifest.chapters {
        if chapter
            .pov_character_id
            .as_deref()
            .and_then(|pov_id| Uuid::parse_str(pov_id).ok())
            == Some(character_id)
        {
            chapter.pov_character_id = None;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::remove_character_and_pov_references;
    use crate::models::{Character, CharacterEngine, CharacterRole, ProjectMetadata};
    use uuid::Uuid;

    #[test]
    fn deleting_character_clears_only_matching_uuid_pov_references() {
        let mut metadata = ProjectMetadata::new("Project".to_string(), "Author".to_string());
        let deleted_character_id = Uuid::new_v4();
        let unrelated_character_id = Uuid::new_v4();
        let unrelated_uuid_pov_id = unrelated_character_id.to_string();
        metadata.characters.push(Character {
            id: deleted_character_id,
            name: "Deleted character".to_string(),
            role: CharacterRole::Protagonist,
            archetype: String::new(),
            description: String::new(),
            engine: CharacterEngine::default(),
            physical_features: String::new(),
            traits: Vec::new(),
            arc: String::new(),
            notes: String::new(),
            aliases: Vec::new(),
        });

        let mut matching_pov = metadata
            .manifest
            .create_chapter(None, "Matching POV".to_string());
        matching_pov.pov_character_id = Some(deleted_character_id.to_string().to_uppercase());
        let mut unrelated_uuid_pov = metadata
            .manifest
            .create_chapter(None, "Unrelated UUID POV".to_string());
        unrelated_uuid_pov.pov_character_id = Some(unrelated_uuid_pov_id.clone());
        let mut arbitrary_pov = metadata
            .manifest
            .create_chapter(None, "Arbitrary POV".to_string());
        arbitrary_pov.pov_character_id = Some("Narrator".to_string());
        metadata.manifest.chapters = vec![matching_pov, unrelated_uuid_pov, arbitrary_pov];

        remove_character_and_pov_references(&mut metadata, deleted_character_id).unwrap();

        assert!(metadata
            .characters
            .iter()
            .all(|character| character.id != deleted_character_id));
        assert_eq!(metadata.manifest.chapters[0].pov_character_id, None);
        assert_eq!(
            metadata.manifest.chapters[1].pov_character_id.as_deref(),
            Some(unrelated_uuid_pov_id.as_str())
        );
        assert_eq!(
            metadata.manifest.chapters[2].pov_character_id.as_deref(),
            Some("Narrator")
        );
    }
}
