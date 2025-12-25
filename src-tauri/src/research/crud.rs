use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use std::path::PathBuf;

pub async fn create_note(
    state: &ResearchState,
    name: String,
) -> crate::errors::Result<ResearchArtifact> {
    let mut inner = state.inner.lock().await;
    // Clone root path to release borrow on inner
    let root = inner
        .root_path
        .as_ref()
        .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
        .clone();

    let mut final_name = name;
    if !final_name.ends_with(".md") {
        final_name.push_str(".md");
    }

    let file_path = root.join(&final_name);
    if file_path.exists() {
        return Err(crate::errors::Error::Research(
            "Note already exists".to_string(),
        ));
    }

    tokio::fs::write(&file_path, "").await?;

    let artifact = ResearchArtifact::new(
        file_path.to_string_lossy().to_string(),
        final_name,
        "text".to_string(),
    );

    inner
        .artifacts
        .insert(artifact.id.clone(), artifact.clone());
    crate::storage::save_index(&root, &inner.artifacts).await?;

    Ok(artifact)
}

pub async fn update_content(
    state: &ResearchState,
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

pub async fn rename_artifact(
    state: &ResearchState,
    id: String,
    new_name: String,
) -> crate::errors::Result<()> {
    let mut inner = state.inner.lock().await;
    // Check root exists
    let root = inner
        .root_path
        .as_ref()
        .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
        .clone();

    if let Some(artifact) = inner.artifacts.get_mut(&id) {
        let old_path = PathBuf::from(&artifact.path);

        let mut new_filename = new_name.clone();
        if let Some(ext) = old_path.extension() {
            let ext_str = ext.to_string_lossy();
            if !new_name
                .to_lowercase()
                .ends_with(&format!(".{}", ext_str.to_lowercase()))
            {
                new_filename.push('.');
                new_filename.push_str(&ext_str);
            }
        }

        let new_path = root.join(&new_filename);
        if new_path.exists() {
            return Err(crate::errors::Error::Research(
                "Destination already exists".to_string(),
            ));
        }

        tokio::fs::rename(&old_path, &new_path).await?;

        artifact.name = new_name;
        artifact.path = new_path.to_string_lossy().to_string();

        crate::storage::save_index(&root, &inner.artifacts).await?;
        Ok(())
    } else {
        Err(crate::errors::Error::ArtifactNotFound(id))
    }
}

pub async fn delete_artifact(state: &ResearchState, id: String) -> crate::errors::Result<()> {
    let mut inner = state.inner.lock().await;
    let root = inner
        .root_path
        .as_ref()
        .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
        .clone();

    if let Some(artifact) = inner.artifacts.remove(&id) {
        let path = PathBuf::from(&artifact.path);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        crate::storage::save_index(&root, &inner.artifacts).await?;
        Ok(())
    } else {
        Err(crate::errors::Error::ArtifactNotFound(id))
    }
}
