use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use notify::RecommendedWatcher;
use std::path::PathBuf;

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

    // Save reconciled
    crate::storage::save_index(&path, &current_artifacts).await?;

    let mut inner = state.inner.lock().await;
    inner.root_path = Some(path);
    inner.artifacts = current_artifacts;
    Ok(())
}

pub async fn set_watcher(state: &ResearchState, watcher: RecommendedWatcher) {
    let mut inner = state.inner.lock().await;
    inner.watcher = Some(watcher);
}

pub async fn get_all(state: &ResearchState) -> Vec<ResearchArtifact> {
    let inner = state.inner.lock().await;
    inner.artifacts.values().cloned().collect()
}

fn reconcile_index(
    disk_files: std::collections::HashMap<String, String>,
    mut index: std::collections::HashMap<String, ResearchArtifact>,
) -> std::collections::HashMap<String, ResearchArtifact> {
    let mut current_artifacts = std::collections::HashMap::new();

    for (file_path, file_name) in disk_files {
        let existing_id = {
            index.iter().find_map(|(id, art)| {
                if art.path == file_path {
                    Some(id.clone())
                } else {
                    None
                }
            })
        };

        if let Some(id) = existing_id {
            if let Some(mut artifact) = index.remove(&id) {
                artifact.name = file_name;
                current_artifacts.insert(id, artifact);
            }
        } else {
            let file_type = ResearchArtifact::determine_type(&file_name);
            let artifact = ResearchArtifact::new(file_path, file_name, file_type);
            current_artifacts.insert(artifact.id.clone(), artifact);
        }
    }
    current_artifacts
}
