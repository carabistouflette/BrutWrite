use crate::models::research::ResearchArtifact;
use notify::RecommendedWatcher;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct ResearchInner {
    pub watcher: Option<RecommendedWatcher>,
    pub artifacts: HashMap<String, ResearchArtifact>,
    pub root_path: Option<PathBuf>,
}

pub struct ResearchState {
    pub inner: Mutex<ResearchInner>,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ResearchInner {
                watcher: None,
                artifacts: HashMap::new(),
                root_path: None,
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

    /// Helper to insert an artifact and save the index
    pub async fn persist_artifact(&self, artifact: ResearchArtifact) -> crate::errors::Result<()> {
        let mut inner = self.inner.lock().await;
        let root = inner
            .root_path
            .as_ref()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
            .clone();

        inner.artifacts.insert(artifact.id.clone(), artifact);
        crate::storage::save_index(&root, &inner.artifacts).await?;
        Ok(())
    }
}
