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
    let current_artifacts = crate::research::utils::reconcile_index(disk_files, index_data);

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
