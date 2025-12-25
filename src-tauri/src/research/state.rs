use crate::models::research::ResearchArtifact;
use notify::RecommendedWatcher;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct ResearchInner {
    pub watcher: Option<RecommendedWatcher>,
    pub artifacts: HashMap<String, ResearchArtifact>,
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

    pub async fn get_all(&self) -> Vec<ResearchArtifact> {
        crate::research::lifecycle::get_all(self).await
    }

    pub async fn create_note(&self, name: String) -> crate::errors::Result<ResearchArtifact> {
        crate::research::crud::create_note(self, name).await
    }

    pub async fn import_files(&self, paths: Vec<String>) -> crate::errors::Result<()> {
        crate::research::io::import_files(self, paths).await
    }

    pub async fn update_content(&self, id: String, content: String) -> crate::errors::Result<()> {
        crate::research::crud::update_content(self, id, content).await
    }

    pub async fn rename_artifact(&self, id: String, new_name: String) -> crate::errors::Result<()> {
        crate::research::crud::rename_artifact(self, id, new_name).await
    }

    pub async fn delete_artifact(&self, id: String) -> crate::errors::Result<()> {
        crate::research::crud::delete_artifact(self, id).await
    }

    pub async fn handle_fs_change(&self, event: notify::Event) -> crate::errors::Result<()> {
        crate::research::io::handle_fs_change(self, event).await
    }

    /// Helper to safely get the root path without holding the lock for long
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
    pub async fn persist_artifact(&self, artifact: ResearchArtifact) -> crate::errors::Result<()> {
        self.mutate_and_persist(|inner| {
            inner
                .path_map
                .insert(artifact.path.clone(), artifact.id.clone());
            inner.artifacts.insert(artifact.id.clone(), artifact);
            Ok(())
        })
        .await
    }
}
