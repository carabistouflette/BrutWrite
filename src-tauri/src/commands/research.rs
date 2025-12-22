use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use crate::storage;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_research_artifacts(
    state: State<'_, ResearchState>,
) -> crate::errors::Result<Vec<ResearchArtifact>> {
    let inner = state.inner.lock().await;
    Ok(inner.artifacts.values().cloned().collect())
}

#[tauri::command]
pub async fn add_research_files(
    state: State<'_, ResearchState>,
    paths: Vec<String>,
) -> crate::errors::Result<()> {
    let root_path = {
        let inner = state.inner.lock().await;
        inner.root_path.clone()
    };

    if let Some(root) = root_path {
        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                let file_name = path.file_name().ok_or_else(|| {
                    crate::errors::Error::Validation("Invalid file path".to_string())
                })?;
                let dest = root.join(file_name);
                tokio::fs::copy(path, dest).await?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn update_research_artifact(
    state: State<'_, ResearchState>,
    artifact: ResearchArtifact,
) -> crate::errors::Result<()> {
    let mut inner = state.inner.lock().await;
    if inner.artifacts.contains_key(&artifact.id) {
        inner.artifacts.insert(artifact.id.clone(), artifact);

        // Persist
        if let Some(path) = &inner.root_path {
            storage::save_index(path, &inner.artifacts)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_research_note(
    state: State<'_, ResearchState>,
    name: String,
) -> crate::errors::Result<ResearchArtifact> {
    let mut inner = state.inner.lock().await;
    if let Some(path) = inner.root_path.clone() {
        let mut final_name = name;
        if !final_name.ends_with(".md") {
            final_name.push_str(".md");
        }

        let file_path = path.join(&final_name);
        if file_path.exists() {
            return Err(crate::errors::Error::Research(
                "Note already exists".to_string(),
            ));
        }

        tokio::fs::write(&file_path, "").await?;

        // Manually create artifact to return immediately
        let artifact = ResearchArtifact::new(
            file_path.to_string_lossy().to_string(),
            final_name,
            "text".to_string(),
        );

        // The watcher will eventually pick it up, but we can add it to state now for responsiveness
        inner
            .artifacts
            .insert(artifact.id.clone(), artifact.clone());
        storage::save_index(&path, &inner.artifacts)?;

        return Ok(artifact);
    }
    Err(crate::errors::Error::ResearchVaultNotInitialized)
}

#[tauri::command]
pub async fn update_note_content(
    state: State<'_, ResearchState>,
    id: String,
    content: String,
) -> crate::errors::Result<()> {
    let artifact = {
        let inner = state.inner.lock().await;
        inner.artifacts.get(&id).cloned()
    };

    if let Some(artifact) = artifact {
        tokio::fs::write(&artifact.path, content).await?;
        Ok(())
    } else {
        Err(crate::errors::Error::ArtifactNotFound(id))
    }
}
#[tauri::command]
pub async fn rename_research_artifact(
    state: State<'_, ResearchState>,
    id: String,
    new_name: String,
) -> crate::errors::Result<()> {
    let mut inner = state.inner.lock().await;
    if let Some(artifact) = inner.artifacts.get_mut(&id) {
        let old_path = PathBuf::from(&artifact.path);
        let parent = old_path
            .parent()
            .ok_or_else(|| crate::errors::Error::Research("Invalid artifact path".to_string()))?;

        let mut new_filename = new_name.clone();
        // Preserving extension if missing in new name
        if let Some(ext) = old_path.extension() {
            if !new_name
                .to_lowercase()
                .ends_with(&ext.to_string_lossy().to_lowercase())
            {
                new_filename.push('.');
                new_filename.push_str(&ext.to_string_lossy());
            }
        }

        let new_path = parent.join(&new_filename);
        if new_path.exists() {
            return Err(crate::errors::Error::Research(
                "Destination already exists".to_string(),
            ));
        }

        tokio::fs::rename(&old_path, &new_path).await?;

        artifact.name = new_name;
        artifact.path = new_path.to_string_lossy().to_string();

        // Persist
        if let Some(path) = &inner.root_path {
            storage::save_index(path, &inner.artifacts)?;
        }
        Ok(())
    } else {
        Err(crate::errors::Error::ArtifactNotFound(id))
    }
}
