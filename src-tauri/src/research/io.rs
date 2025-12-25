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
    let mut inner = state.inner.lock().await;
    let root = if let Some(r) = &inner.root_path {
        r.clone()
    } else {
        return Ok(());
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
            // Upsert
            let existing_id = {
                inner.artifacts.iter().find_map(|(k, v)| {
                    if v.path == path_str {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
            };

            if let Some(_id) = existing_id {
                // It exists, no action needed
            } else {
                // New file detected!
                let file_type = ResearchArtifact::determine_type(&file_name);

                let artifact = ResearchArtifact::new(path_str, file_name, file_type);
                inner.artifacts.insert(artifact.id.clone(), artifact);
                // Persist immediately on detection
                crate::storage::save_index(&root, &inner.artifacts).await?;
            }
        } else {
            // File gone?
            let found_id = {
                inner.artifacts.iter().find_map(|(k, v)| {
                    if v.path == path_str {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
            };

            if let Some(id) = found_id {
                inner.artifacts.remove(&id);
                // Persist
                crate::storage::save_index(&root, &inner.artifacts).await?;
            }
        }
    }
    Ok(())
}
