use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use crate::{errors::Result, storage};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_research_artifacts(
    state: State<'_, ResearchState>,
) -> Result<Vec<ResearchArtifact>> {
    let artifacts = state.artifacts.lock().await;
    Ok(artifacts.values().cloned().collect())
}

#[tauri::command]
pub async fn add_research_files(state: State<'_, ResearchState>, paths: Vec<String>) -> Result<()> {
    let root_path = state.root_path.lock().await.clone();
    if let Some(root) = root_path {
        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                let file_name = path.file_name().unwrap();
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
) -> Result<()> {
    let mut artifacts = state.artifacts.lock().await;
    if artifacts.contains_key(&artifact.id) {
        artifacts.insert(artifact.id.clone(), artifact);

        // Persist
        let root = state.root_path.lock().await.clone();
        if let Some(path) = root {
            storage::save_index(&path, &artifacts)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_research_note(
    state: State<'_, ResearchState>,
    name: String,
) -> Result<ResearchArtifact> {
    let root = state.root_path.lock().await.clone();
    if let Some(path) = root {
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
        {
            let mut artifacts = state.artifacts.lock().await;
            artifacts.insert(artifact.id.clone(), artifact.clone());
            storage::save_index(&path, &artifacts)?;
        }

        return Ok(artifact);
    }
    Err(crate::errors::Error::ResearchVaultNotInitialized)
}

#[tauri::command]
pub async fn update_note_content(
    state: State<'_, ResearchState>,
    id: String,
    content: String,
) -> Result<()> {
    let artifact = {
        let artifacts = state.artifacts.lock().await;
        artifacts.get(&id).cloned()
    };

    if let Some(artifact) = artifact {
        tokio::fs::write(&artifact.path, content).await?;
        Ok(())
    } else {
        Err(crate::errors::Error::ArtifactNotFound(id))
    }
}
