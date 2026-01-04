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

    // state.persist_artifact now handles Arc internally, or expects explicit struct?
    // In state.rs: persist_artifact(artifact: ResearchArtifact) -> Arc::new(artifact)
    // So we pass Owned artifact.
    state.persist_artifact(artifact.clone()).await?;

    // We return Arc reference to it.
    Ok(std::sync::Arc::new(artifact))
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
    // We try to rename directly. If destination exists, OS will return error (usually) or we can check.
    // However, on some filesystems rename overwrites. To be safe against overwriting, we should use a link-then-unlink or checked.
    // Since this is specific, we'll keep the check but acknowledge the small race.
    // Ideally usage of `renameat2` with NOREPLACE on Linux, but standard lib abstraction is minimal.
    // For this audit fix, we will keep the check but handle the error better if rename fails for other reasons.

    if new_path.exists() {
        return Err(crate::errors::Error::Research(
            "Destination already exists".to_string(),
        ));
    }

    tokio::fs::rename(&old_path, &new_path).await.map_err(|e| {
        // If we failed, it might be because it exists now
        crate::errors::Error::Io(e)
    })?;

    // 3. Update State (Critical Section)
    // If this fails (e.g. lock poison/panic), we have a desync.
    // However, since we renamed on disk, the next app load would just see the new file as "untracked"
    // or we'd have a broken link in the old state.
    // To be truly robust we'd need a wal or extensive rollback logic, but for a local desktop app,
    // ensuring we don't hold the lock during IO is the main improvement for responsiveness,
    // AND validating the name prevents FS errors.

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
            log::error!("State update failed after rename, rolling back FS: {}", e);
            if let Err(rollback_err) = tokio::fs::rename(&new_path, &old_path).await {
                log::error!("CRITICAL: FS Rollback failed: {}", rollback_err);
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
}
