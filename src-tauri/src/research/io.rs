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
        let path = PathBuf::from(path_str);
        let mut source = tokio::fs::File::open(&path).await?;
        if !source.metadata().await?.is_file() {
            return Err(crate::errors::Error::Validation(
                "Research imports must be files".to_string(),
            ));
        }

        let file_name = path
            .file_name()
            .ok_or_else(|| crate::errors::Error::Validation("Invalid file path".to_string()))?;
        let name = file_name.to_string_lossy().into_owned();
        let destination = root.join(file_name);
        let mut destination_file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .await
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    crate::errors::Error::Research(format!(
                        "A research file named {:?} already exists",
                        name
                    ))
                } else {
                    crate::errors::Error::Io(error)
                }
            })?;

        if let Err(error) = tokio::io::copy(&mut source, &mut destination_file).await {
            drop(destination_file);
            if let Err(cleanup_error) = tokio::fs::remove_file(&destination).await {
                log::warn!(
                    "Failed to remove partial research import {:?}: {}",
                    destination,
                    cleanup_error
                );
            }
            return Err(crate::errors::Error::Io(error));
        }
        drop(destination_file);

        let artifact = ResearchArtifact::new(
            destination.to_string_lossy().to_string(),
            name.clone(),
            ResearchArtifact::determine_type(&name),
        );
        if let Err(error) = state.persist_artifact(artifact).await {
            if let Err(cleanup_error) = tokio::fs::remove_file(&destination).await {
                log::warn!(
                    "Failed to remove unindexed research import {:?}: {}",
                    destination,
                    cleanup_error
                );
            }
            return Err(error);
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

        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        let path_str = path.to_string_lossy().to_string();

        if path.exists() {
            let already_indexed = {
                let inner = state.inner.lock().await;
                inner.path_map.contains_key(&path_str)
            };

            if !already_indexed {
                let file_type = ResearchArtifact::determine_type(&file_name);
                let artifact = ResearchArtifact::new(path_str, file_name, file_type);
                state.persist_artifact(artifact).await?;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn import_is_indexed_immediately_and_never_overwrites_existing_files(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault = dir.path().join("research");
        tokio::fs::create_dir(&vault).await?;
        let source = dir.path().join("source.pdf");
        tokio::fs::write(&source, b"%PDF-1.7 reference").await?;

        let state = ResearchState::new();
        state.initialize(vault.clone()).await?;
        import_files(&state, vec![source.to_string_lossy().into_owned()]).await?;

        let imported = state
            .get_all()
            .await
            .into_iter()
            .next()
            .expect("imported file should be indexed before the command returns");
        assert_eq!(imported.file_type, "pdf");
        assert_eq!(
            tokio::fs::read(&imported.path).await?,
            b"%PDF-1.7 reference"
        );

        let duplicate = import_files(&state, vec![source.to_string_lossy().into_owned()]).await;
        assert!(matches!(duplicate, Err(crate::errors::Error::Research(_))));
        assert_eq!(
            tokio::fs::read(vault.join("source.pdf")).await?,
            b"%PDF-1.7 reference"
        );

        let reloaded = ResearchState::new();
        reloaded.initialize(vault).await?;
        let persisted = reloaded.get_all().await;
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].id, imported.id);
        Ok(())
    }
}
