use crate::models::research::ResearchArtifact;
use notify::RecommendedWatcher;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;

use std::sync::Arc;

pub struct ResearchInner {
    pub watcher: Option<RecommendedWatcher>,
    pub artifacts: HashMap<String, Arc<ResearchArtifact>>,
    /// Reverse index for O(1) lookup by path (Path -> ID)
    pub path_map: HashMap<String, String>,
    pub root_path: Option<PathBuf>,
    pub version: u64,
}

pub struct PersistenceState {
    pub last_saved_version: u64,
}

pub struct ResearchState {
    pub inner: Mutex<ResearchInner>,
    pub persistence: Mutex<PersistenceState>,
    /// Prevents concurrent initializations during rapid project switching
    pub init_lock: Mutex<()>,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ResearchInner {
                watcher: None,
                artifacts: HashMap::new(),
                path_map: HashMap::new(),
                root_path: None,
                version: 0,
            }),
            persistence: Mutex::new(PersistenceState {
                last_saved_version: 0,
            }),
            init_lock: Mutex::new(()),
        }
    }
}

impl ResearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn initialize(&self, path: PathBuf) -> crate::errors::Result<()> {
        crate::research::lifecycle::initialize(self, path).await
    }

    pub async fn set_watcher(&self, watcher: RecommendedWatcher) {
        crate::research::lifecycle::set_watcher(self, watcher).await;
    }

    pub async fn get_all(&self) -> Vec<Arc<ResearchArtifact>> {
        crate::research::lifecycle::get_all(self).await
    }

    pub async fn create_note(&self, name: String) -> crate::errors::Result<Arc<ResearchArtifact>> {
        crate::research::crud::create_note(self, name).await
    }

    pub async fn import_files(&self, paths: Vec<String>) -> crate::errors::Result<()> {
        crate::research::io::import_files(self, paths).await
    }

    pub async fn update_content(
        &self,
        id: String,
        expected_path: String,
        content: String,
    ) -> crate::errors::Result<()> {
        crate::research::crud::update_content(self, id, expected_path, content).await
    }

    pub async fn rename_artifact(&self, id: String, new_name: String) -> crate::errors::Result<()> {
        crate::research::crud::rename_artifact(self, id, new_name).await
    }

    pub async fn update_tags(&self, id: String, tags: Vec<String>) -> crate::errors::Result<()> {
        crate::research::crud::update_tags(self, id, tags).await
    }

    pub async fn delete_artifact(&self, id: String) -> crate::errors::Result<()> {
        crate::research::crud::delete_artifact(self, id).await
    }

    pub async fn handle_fs_change(&self, event: notify::Event) -> crate::errors::Result<()> {
        crate::research::io::handle_fs_change(self, event).await
    }

    pub async fn stop(&self) {
        let mut inner = self.inner.lock().await;
        inner.watcher = None;
        inner.root_path = None;
        inner.artifacts.clear();
        inner.path_map.clear();
    }
    pub async fn get_root_path_safe(&self) -> crate::errors::Result<PathBuf> {
        let inner = self.inner.lock().await;
        inner
            .root_path
            .as_ref()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)
            .cloned()
    }

    /// Helper to mutate state and persist changes safely
    /// This avoids holding the main state lock during I/O
    pub async fn mutate_and_persist<F>(&self, mutation: F) -> crate::errors::Result<()>
    where
        F: FnOnce(&mut ResearchInner) -> crate::errors::Result<()>,
    {
        // 1. Mutate securely and capture snapshot
        let (root, artifacts, version) = {
            let mut inner = self.inner.lock().await;
            mutation(&mut inner)?;
            inner.version += 1;
            let root = inner
                .root_path
                .as_ref()
                .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
                .clone();
            (root, inner.artifacts.clone(), inner.version)
        };

        // 2. Persist with serialization
        let mut persistence = self.persistence.lock().await;
        if version <= persistence.last_saved_version {
            // Already saved a newer or equal version
            return Ok(());
        }

        crate::storage::save_index(&root, &artifacts).await?;
        persistence.last_saved_version = version;

        Ok(())
    }

    /// Helper to insert an artifact and save the index
    pub async fn persist_artifact(
        &self,
        artifact: ResearchArtifact,
    ) -> crate::errors::Result<Arc<ResearchArtifact>> {
        let artifact = Arc::new(artifact);
        let mut registered = None;

        self.mutate_and_persist(|inner| {
            if let Some(existing_id) = inner.path_map.get(&artifact.path) {
                if let Some(existing) = inner.artifacts.get(existing_id) {
                    registered = Some(Arc::clone(existing));
                    return Ok(());
                }
            }

            inner
                .path_map
                .insert(artifact.path.clone(), artifact.id.clone());
            inner
                .artifacts
                .insert(artifact.id.clone(), Arc::clone(&artifact));
            registered = Some(Arc::clone(&artifact));
            Ok(())
        })
        .await?;

        Ok(registered.expect("artifact registration always returns an artifact"))
    }
}
