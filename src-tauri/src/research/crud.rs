use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use std::path::PathBuf;

fn validate_filename(name: &str) -> crate::errors::Result<()> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(crate::errors::Error::Validation(
            "Invalid name: path traversal and subdirectories are not allowed".to_string(),
        ));
    }
    Ok(())
}

pub async fn create_note(
    state: &ResearchState,
    name: String,
) -> crate::errors::Result<ResearchArtifact> {
    let root = state.get_root_path_safe().await?;

    validate_filename(&name)?;

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

    // IO without lock
    tokio::fs::write(&file_path, "").await?;

    let artifact = ResearchArtifact::new(
        file_path.to_string_lossy().to_string(),
        final_name,
        "text".to_string(),
    );

    state.persist_artifact(artifact.clone()).await?;
    Ok(artifact)
}

pub async fn update_content(
    state: &ResearchState,
    id: String,
    content: String,
) -> crate::errors::Result<()> {
    // Read artifact path under lock
    let artifact_path = {
        let inner = state.inner.lock().await;
        inner.artifacts.get(&id).map(|a| a.path.clone())
    };

    if let Some(path) = artifact_path {
        tokio::fs::write(path, content).await?;
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
    let root = state.get_root_path_safe().await?;

    validate_filename(&new_name)?;

    let (old_path, ext) = {
        let inner = state.inner.lock().await;
        let artifact = inner
            .artifacts
            .get(&id)
            .ok_or_else(|| crate::errors::Error::ArtifactNotFound(id.clone()))?;
        let path = PathBuf::from(&artifact.path);
        (
            path.clone(),
            path.extension().map(|e| e.to_string_lossy().to_string()),
        )
    };

    let mut new_filename = new_name.clone();
    if let Some(ext_str) = ext {
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

    // IO without lock
    tokio::fs::rename(&old_path, &new_path).await?;

    // Update state
    state
        .mutate_and_persist(|inner| {
            if let Some(artifact) = inner.artifacts.get_mut(&id) {
                inner.path_map.remove(&artifact.path);
                artifact.name = new_filename.clone();
                artifact.path = new_path.to_string_lossy().to_string();
                inner
                    .path_map
                    .insert(artifact.path.clone(), artifact.id.clone());
            }
            Ok(())
        })
        .await?;
    Ok(())
}

pub async fn delete_artifact(state: &ResearchState, id: String) -> crate::errors::Result<()> {
    let path_to_delete = {
        let inner = state.inner.lock().await;
        inner.artifacts.get(&id).map(|a| PathBuf::from(&a.path))
    }
    .ok_or_else(|| crate::errors::Error::ArtifactNotFound(id.clone()))?;

    // IO without lock
    if path_to_delete.exists() {
        tokio::fs::remove_file(&path_to_delete).await?;
    }

    // Update state
    state
        .mutate_and_persist(|inner| {
            if let Some(artifact) = inner.artifacts.remove(&id) {
                inner.path_map.remove(&artifact.path);
            }
            Ok(())
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::ResearchState;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_concurrent_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await.unwrap();

        let mut handles = vec![];
        for i in 0..10 {
            let state_clone = state.clone();
            handles.push(tokio::spawn(async move {
                let name = format!("note_{}", i);
                create_note(&state_clone, name).await
            }));
        }

        for handle in handles {
            handle.await.unwrap().expect("Failed to create note");
        }

        let all = state.get_all().await;
        assert_eq!(all.len(), 10);
    }

    #[tokio::test]
    async fn test_path_traversal_protection() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await.unwrap();

        // Test create with traversal
        let result = create_note(&state, "../evil".to_string()).await;
        assert!(matches!(result, Err(crate::errors::Error::Validation(_))));

        // Test create with separator
        let result = create_note(&state, "sub/dir".to_string()).await;
        assert!(matches!(result, Err(crate::errors::Error::Validation(_))));

        // Create valid note for rename test
        let note = create_note(&state, "valid".to_string()).await.unwrap();

        // Test rename with traversal
        let result = rename_artifact(&state, note.id, "../evil".to_string()).await;
        assert!(matches!(result, Err(crate::errors::Error::Validation(_))));
    }

    #[tokio::test]
    async fn test_rename_artifact() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await.unwrap();

        let note = create_note(&state, "original".to_string()).await.unwrap();
        let original_path = PathBuf::from(&note.path);

        rename_artifact(&state, note.id.clone(), "renamed".to_string())
            .await
            .unwrap();

        // Verify old file gone
        assert!(!original_path.exists());

        // Verify state up to date
        let files = state.get_all().await;
        let renamed = files.iter().find(|a| a.id == note.id).unwrap();
        assert_eq!(renamed.name, "renamed.md");

        // Verify new file exists
        assert!(PathBuf::from(&renamed.path).exists());
    }

    #[tokio::test]
    async fn test_delete_artifact() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await.unwrap();

        let note = create_note(&state, "to_delete".to_string()).await.unwrap();
        let note_path = PathBuf::from(&note.path);
        assert!(note_path.exists());

        delete_artifact(&state, note.id.clone()).await.unwrap();

        assert!(!note_path.exists());
    }
}
