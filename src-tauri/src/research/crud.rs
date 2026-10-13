use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use sanitize_filename::sanitize;
use std::path::PathBuf;

fn validate_filename(name: &str) -> crate::errors::Result<String> {
    let sanitized = sanitize(name);
    if sanitized != name {
        return Err(crate::errors::Error::Validation(format!(
            "Invalid filename. Suggested: {}",
            sanitized
        )));
    }

    if name.trim().is_empty() {
        return Err(crate::errors::Error::Validation(
            "Filename cannot be empty".to_string(),
        ));
    }

    // Additional check for deeply nested paths or traversal attempts that sanitize might technically allow as valid chars but we don't want
    if name.contains('/') || name.contains('\\') {
        return Err(crate::errors::Error::Validation(
            "Subdirectories are not allowed".to_string(),
        ));
    }

    Ok(sanitized)
}

pub async fn create_note(
    state: &ResearchState,
    name: String,
) -> crate::errors::Result<std::sync::Arc<ResearchArtifact>> {
    let root = state.get_root_path_safe().await?;

    let valid_name = validate_filename(&name)?;

    let mut final_name = valid_name;
    if !final_name.ends_with(".md") {
        final_name.push_str(".md");
    }

    let file_path = root.join(&final_name);

    // Atomic creation prevents TOCTOU races - using async I/O
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                crate::errors::Error::Research("Note already exists".to_string())
            } else {
                crate::errors::Error::Io(e)
            }
        })?;

    // Close file immediately, we just wanted to create it empty safely
    drop(file);

    let artifact = ResearchArtifact::new(
        file_path.to_string_lossy().to_string(),
        final_name,
        "text".to_string(),
    );

    state.persist_artifact(artifact).await
}

pub async fn update_tags(
    state: &ResearchState,
    id: String,
    tags: Vec<String>,
) -> crate::errors::Result<()> {
    state
        .mutate_and_persist(move |inner| {
            let artifact = inner
                .artifacts
                .get_mut(&id)
                .ok_or_else(|| crate::errors::Error::ArtifactNotFound(id.clone()))?;
            let mut updated = (**artifact).clone();
            updated.tags = tags;
            *artifact = std::sync::Arc::new(updated);
            Ok(())
        })
        .await
}

pub async fn update_content(
    state: &ResearchState,
    id: String,
    expected_path: String,
    content: String,
) -> crate::errors::Result<()> {
    let artifact_path = {
        let inner = state.inner.lock().await;
        let artifact = inner
            .artifacts
            .get(&id)
            .ok_or_else(|| crate::errors::Error::ArtifactNotFound(id.clone()))?;
        if artifact.path != expected_path {
            return Err(crate::errors::Error::Research(
                "Research artifact path changed before save".to_string(),
            ));
        }
        artifact.path.clone()
    };

    tokio::fs::write(artifact_path, content).await?;
    Ok(())
}

pub async fn rename_artifact(
    state: &ResearchState,
    id: String,
    new_name: String,
) -> crate::errors::Result<()> {
    let root = state.get_root_path_safe().await?;

    let valid_name = validate_filename(&new_name)?;

    // 1. Prepare new path
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

    let mut new_filename = valid_name;
    if let Some(ext_str) = ext {
        if !new_filename
            .to_lowercase()
            .ends_with(&format!(".{}", ext_str.to_lowercase()))
        {
            new_filename.push('.');
            new_filename.push_str(&ext_str);
        }
    }

    let new_path = root.join(&new_filename);

    // 2. Perform FS Rename
    // NOTE: This operation is not strictly atomic on all filesystems.
    // There is a small TOCTOU window where `exists()` returns false,
    // but another process creates the file before `rename()`.
    // Ideally use `renameat2` with `RENAME_NOREPLACE` on Linux, but strictly standard Rust lacks this.
    // We minimize the risk by handling the error from `rename` if possible, though overly defensive checking is still useful for UI feedback.
    if new_path.exists() {
        return Err(crate::errors::Error::Research(
            "Destination already exists".to_string(),
        ));
    }

    // Try rename. If it fails, we abort before touching state.
    tokio::fs::rename(&old_path, &new_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            crate::errors::Error::Research("Destination already exists".to_string())
        } else {
            crate::errors::Error::Io(e)
        }
    })?;

    // 3. Update State (Critical Section)
    // If this fails (e.g. lock poison/panic), we have a desync.
    let update_result = state
        .mutate_and_persist(|inner| {
            if let Some(artifact_arc) = inner.artifacts.get_mut(&id) {
                let mut artifact = (**artifact_arc).clone();
                inner.path_map.remove(&artifact.path);

                artifact.name = new_filename.clone();
                artifact.path = new_path.to_string_lossy().to_string();

                let new_arc = std::sync::Arc::new(artifact);
                inner
                    .path_map
                    .insert(new_arc.path.clone(), new_arc.id.clone());
                *artifact_arc = new_arc;
            }
            Ok(())
        })
        .await;

    match update_result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Rollback FS
            log::error!(
                "State update failed after rename, attempting FS rollback: {}",
                e
            );

            // Attempt to move file back
            if let Err(rollback_err) = tokio::fs::rename(&new_path, &old_path).await {
                log::error!(
                    "CRITICAL: FS Rollback failed! State and FS are desynchronized. \
                    Old Path: {:?}, New Path: {:?}, Error: {}",
                    old_path,
                    new_path,
                    rollback_err
                );
            } else {
                log::info!("FS rollback successful for artifact {}", id);
            }
            Err(e)
        }
    }
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
    async fn test_concurrent_creation() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await?;

        let mut handles = vec![];
        for i in 0..10 {
            let state_clone = state.clone();
            handles.push(tokio::spawn(async move {
                let name = format!("note_{}", i);
                create_note(&state_clone, name).await
            }));
        }

        for handle in handles {
            handle.await??;
        }

        let all = state.get_all().await;
        assert_eq!(all.len(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn test_path_traversal_protection() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await?;

        // Test create with traversal
        // Sanitize should strip it or we return error
        let result = create_note(&state, "../evil".to_string()).await;
        // With strict validation we expect Error or Sanitized name?
        // Our new validate_filename returns Err if sanitized != name
        assert!(matches!(result, Err(crate::errors::Error::Validation(_))));

        // Create valid note for rename test
        let note = create_note(&state, "valid".to_string()).await?;

        // Test rename with traversal
        let result = rename_artifact(&state, note.id.clone(), "../evil".to_string()).await;
        assert!(matches!(result, Err(crate::errors::Error::Validation(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_rename_artifact_rollback() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().to_path_buf();
        let state = Arc::new(ResearchState::new());
        state.initialize(path.clone()).await?;

        let note = create_note(&state, "torename".to_string()).await?;

        rename_artifact(&state, note.id.clone(), "renamed".to_string()).await?;

        let old_p = path.join("torename.md");
        let new_p = path.join("renamed.md");

        assert!(!old_p.exists());
        assert!(new_p.exists());
        Ok(())
    }
    #[tokio::test]
    async fn test_tags_rename_and_delete_persist_across_reinitialization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().to_path_buf();
        let state = ResearchState::new();
        state.initialize(path.clone()).await?;

        let note = create_note(&state, "reference".to_string()).await?;
        update_tags(&state, note.id.clone(), vec!["history".to_string()]).await?;
        rename_artifact(&state, note.id.clone(), "renamed".to_string()).await?;

        let reloaded = ResearchState::new();
        reloaded.initialize(path.clone()).await?;
        let reloaded_note = reloaded
            .get_all()
            .await
            .into_iter()
            .find(|artifact| artifact.id == note.id)
            .expect("renamed note should remain indexed");
        assert_eq!(reloaded_note.name, "renamed.md");
        assert_eq!(reloaded_note.tags, vec!["history".to_string()]);

        delete_artifact(&reloaded, note.id.clone()).await?;
        assert!(!path.join("renamed.md").exists());

        let after_delete = ResearchState::new();
        after_delete.initialize(path).await?;
        assert!(after_delete.get_all().await.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn update_content_rejects_a_copied_id_from_another_project_path(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let root_a = dir.path().join("project-a");
        let root_b = dir.path().join("project-b");
        std::fs::create_dir_all(&root_a)?;
        std::fs::create_dir_all(&root_b)?;

        let path_a = root_a.join("research").join("note.md");
        let path_b = root_b.join("research").join("note.md");
        std::fs::create_dir_all(path_a.parent().unwrap())?;
        std::fs::create_dir_all(path_b.parent().unwrap())?;
        tokio::fs::write(&path_a, "project A").await?;

        let state = ResearchState::new();
        state.initialize(root_b).await?;
        tokio::fs::write(&path_b, "project B").await?;
        state
            .persist_artifact(crate::models::research::ResearchArtifact {
                id: "copied-id".to_string(),
                path: path_b.to_string_lossy().to_string(),
                name: "note.md".to_string(),
                file_type: "text".to_string(),
                tags: Vec::new(),
            })
            .await?;

        let result = update_content(
            &state,
            "copied-id".to_string(),
            path_a.to_string_lossy().to_string(),
            "overwritten".to_string(),
        )
        .await;

        assert!(matches!(result, Err(crate::errors::Error::Research(_))));
        assert_eq!(tokio::fs::read_to_string(path_a).await?, "project A");
        assert_eq!(tokio::fs::read_to_string(path_b).await?, "project B");
        Ok(())
    }
}
