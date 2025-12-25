use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use std::path::PathBuf;

pub async fn import_files(state: &ResearchState, paths: Vec<String>) -> crate::errors::Result<()> {
    let root = {
        let inner = state.inner.lock().await;
        inner
            .root_path
            .clone()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
    };

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            let file_name = path
                .file_name()
                .ok_or_else(|| crate::errors::Error::Validation("Invalid file path".to_string()))?;
            let dest = root.join(file_name);
            tokio::fs::copy(path, dest).await?;
        }
    }
    Ok(())
}

pub async fn handle_fs_change(
    state: &ResearchState,
    event: notify::Event,
) -> crate::errors::Result<()> {
    // 1. Determine root path safely
    let root = state.get_root_path_safe().await.ok();
    // If vault is not initialized, we can't process events (and probably shouldn't receive them, but safe guard)
    let root = match root {
        Some(r) => r,
        None => return Ok(()),
    };

    // We only care about file events in the root directory
    for path in event.paths {
        // Ignore if not in root or is index file
        if path.parent() != Some(&root) {
            continue;
        }
        if path
            .file_name()
            .map(|n| n == ".research-index.json")
            .unwrap_or(false)
        {
            continue;
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let path_str = path.to_string_lossy().to_string();

        if path.exists() {
            // Updated or Created
            state
                .mutate_and_persist(move |inner| {
                    // Check if exists by path using O(1) lookup
                    let existing_id = inner.path_map.get(&path_str).cloned();

                    if let Some(_id) = existing_id {
                        // Exists, do nothing
                    } else {
                        // New file detected
                        let file_type = ResearchArtifact::determine_type(&file_name);
                        let artifact =
                            ResearchArtifact::new(path_str.clone(), file_name, file_type);
                        inner.path_map.insert(path_str.clone(), artifact.id.clone());
                        inner.artifacts.insert(artifact.id.clone(), artifact);
                    }
                    Ok(())
                })
                .await?;
        } else {
            // Deleted
            state
                .mutate_and_persist(move |inner| {
                    let found_id = inner.path_map.get(&path_str).cloned();

                    if let Some(id) = found_id {
                        inner.artifacts.remove(&id);
                        inner.path_map.remove(&path_str);
                    }
                    Ok(())
                })
                .await?;
        }
    }
    Ok(())
}
