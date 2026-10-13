use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use notify::RecommendedWatcher;
use std::path::PathBuf;

use std::sync::Arc;

pub async fn initialize(state: &ResearchState, path: PathBuf) -> crate::errors::Result<()> {
    // Ensure directory exists
    if !path.exists() {
        tokio::fs::create_dir_all(&path).await?;
    }

    // Load index and scan disk
    let index_data = crate::storage::load_index(&path).await;
    let disk_files = crate::storage::scan_on_disk(&path).await;

    // Use utility
    let current_artifacts = reconcile_index(disk_files, index_data);

    // Build path map
    let path_map: std::collections::HashMap<String, String> = current_artifacts
        .values()
        .map(|a| (a.path.clone(), a.id.clone()))
        .collect();

    // Save reconciled
    // We need to deref for saving until storage is updated or map it.
    // Ideally storage should accept the map as is if we update it.
    // For now, let's assume we update storage.rs next.
    crate::storage::save_index(&path, &current_artifacts).await?;

    let mut inner = state.inner.lock().await;
    inner.root_path = Some(path);
    inner.artifacts = current_artifacts;
    inner.path_map = path_map;
    Ok(())
}

pub async fn set_watcher(state: &ResearchState, watcher: RecommendedWatcher) {
    let mut inner = state.inner.lock().await;
    inner.watcher = Some(watcher);
}

pub async fn get_all(state: &ResearchState) -> Vec<Arc<ResearchArtifact>> {
    let inner = state.inner.lock().await;
    inner.artifacts.values().cloned().collect()
}

fn reconcile_index(
    disk_files: std::collections::HashMap<String, String>,
    mut index: std::collections::HashMap<String, ResearchArtifact>,
) -> std::collections::HashMap<String, Arc<ResearchArtifact>> {
    let mut current_artifacts = std::collections::HashMap::new();

    // Research files are flat; filenames remain stable when a project is copied or moved.
    let filename_to_id: std::collections::HashMap<String, String> = index
        .iter()
        .filter_map(|(id, artifact)| {
            let name = std::path::Path::new(&artifact.path).file_name()?.to_str()?;
            Some((name.to_owned(), id.clone()))
        })
        .collect();

    for (file_path, file_name) in disk_files {
        let existing_id = filename_to_id.get(&file_name);

        if let Some(id) = existing_id {
            if let Some(mut artifact) = index.remove(id) {
                artifact.path = file_path;
                artifact.name = file_name;
                current_artifacts.insert(id.clone(), Arc::new(artifact));
            }
        } else {
            let file_type = ResearchArtifact::determine_type(&file_name);
            let artifact = ResearchArtifact::new(file_path, file_name, file_type);
            current_artifacts.insert(artifact.id.clone(), Arc::new(artifact));
        }
    }
    current_artifacts
}
#[cfg(test)]
mod tests {
    use crate::research::ResearchState;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_reconcile_index_path_map() {
        let dir = tempdir().expect("Failed to create temp dir");
        let path = dir.path().to_path_buf();

        // Create dummy files
        let file1 = path.join("note1.md");
        tokio::fs::write(&file1, "content")
            .await
            .expect("Failed to write note1");
        let file2 = path.join("note2.md");
        tokio::fs::write(&file2, "content")
            .await
            .expect("Failed to write note2");

        let state = Arc::new(ResearchState::new());

        // Initialize (this calls reconcile_index and builds path_map)
        state
            .initialize(path.clone())
            .await
            .expect("Failed to initialize state");

        let inner = state.inner.lock().await;

        // Verify artifacts exist
        assert_eq!(inner.artifacts.len(), 2);

        // Verify path_map is populated correctly
        assert_eq!(inner.path_map.len(), 2);

        let path1 = file1.to_string_lossy().to_string();
        let path2 = file2.to_string_lossy().to_string();

        assert!(inner.path_map.contains_key(&path1));
        assert!(inner.path_map.contains_key(&path2));

        let id1 = inner.path_map.get(&path1).expect("Path1 should exist");
        let artifact1 = inner.artifacts.get(id1).expect("Artifact1 should exist");
        assert_eq!(artifact1.path, path1);

        let id2 = inner.path_map.get(&path2).expect("Path2 should exist");
        let artifact2 = inner.artifacts.get(id2).expect("Artifact2 should exist");
        assert_eq!(artifact2.path, path2);
    }

    #[tokio::test]
    async fn relocated_project_preserves_research_ids_and_tags() -> crate::errors::Result<()> {
        let source = tempdir()?;
        let destination = tempdir()?;
        let original_state = ResearchState::new();
        original_state
            .initialize(source.path().to_path_buf())
            .await?;
        let note = original_state.create_note("Reference".to_string()).await?;
        original_state
            .update_tags(note.id.clone(), vec!["archive".to_string()])
            .await?;
        tokio::fs::copy(&note.path, destination.path().join(&note.name)).await?;
        tokio::fs::copy(
            source.path().join(".research-index.json"),
            destination.path().join(".research-index.json"),
        )
        .await?;

        let relocated_state = ResearchState::new();
        relocated_state
            .initialize(destination.path().to_path_buf())
            .await?;
        let artifacts = relocated_state.get_all().await;
        let relocated = artifacts
            .iter()
            .find(|artifact| artifact.id == note.id)
            .unwrap();
        assert_eq!(relocated.tags, vec!["archive"]);
        assert_eq!(
            relocated.path,
            destination.path().join(&note.name).to_string_lossy()
        );
        let persisted = crate::storage::load_index(destination.path()).await;
        assert_eq!(persisted[&note.id].path, relocated.path);
        Ok(())
    }
}
