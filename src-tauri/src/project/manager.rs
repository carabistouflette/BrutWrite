use crate::models;
use crate::storage;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock}; // RwLock added
use uuid::Uuid;

pub struct ProjectContext {
    pub path: PathBuf,
    pub metadata: Arc<Mutex<models::ProjectMetadata>>,
}

/// Manages lifecycle of loaded projects.
///
/// # Concurrency
/// Uses `RwLock` for the project registry to allow concurrent reads (getting context)
/// while ensuring exclusive access for writes (loading/unloading projects).
/// Each project's metadata is protected by its own `Mutex`.
pub struct ProjectManager {
    pub projects: RwLock<HashMap<Uuid, ProjectContext>>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_context(
        &self,
        project_id: Uuid,
    ) -> crate::errors::Result<(PathBuf, Arc<Mutex<models::ProjectMetadata>>)> {
        let projects = self.projects.read().await;
        let context =
            projects
                .get(&project_id)
                .ok_or_else(|| crate::errors::Error::InvalidStructure {
                    path: PathBuf::new(),
                    reason: "Project not loaded".to_string(),
                })?;

        Ok((context.path.clone(), context.metadata.clone()))
    }

    pub async fn mutate_project<F>(
        &self,
        project_id: Uuid,
        mutation: F,
    ) -> crate::errors::Result<models::ProjectMetadata>
    where
        F: FnOnce(&mut models::ProjectMetadata) -> crate::errors::Result<()> + Send,
    {
        let (root_path, metadata_arc) = self.get_context(project_id).await?;

        let mut metadata = metadata_arc.lock().await;

        mutation(&mut metadata)?;

        metadata.updated_at = chrono::Utc::now();

        storage::save_project_metadata(&root_path, &metadata).await?;

        Ok(metadata.clone())
    }

    pub async fn create_project(
        &self,
        path: PathBuf,
        name: String,
        author: String,
    ) -> crate::errors::Result<models::ProjectMetadata> {
        let metadata = storage::create_project_structure(&path, &name, &author).await?;
        self.register_project(metadata.id, path.clone(), metadata.clone())
            .await;
        Ok(metadata)
    }

    pub async fn load_project(
        &self,
        path: PathBuf,
    ) -> crate::errors::Result<models::ProjectMetadata> {
        let metadata = storage::load_project_metadata(&path).await?;
        self.register_project(metadata.id, path.clone(), metadata.clone())
            .await;
        Ok(metadata)
    }

    pub async fn register_project(
        &self,
        id: Uuid,
        path: PathBuf,
        metadata: models::ProjectMetadata,
    ) {
        let mut projects = self.projects.write().await;
        projects.insert(
            id,
            ProjectContext {
                path,
                metadata: Arc::new(Mutex::new(metadata)),
            },
        );
    }

    pub async fn unload_project(&self, project_id: Uuid) {
        let mut projects = self.projects.write().await;
        projects.remove(&project_id);
    }

    pub async fn is_loaded(&self, project_id: Uuid) -> bool {
        let projects = self.projects.read().await;
        projects.contains_key(&project_id)
    }

    pub async fn get_all_loaded(&self) -> Vec<Uuid> {
        let projects = self.projects.read().await;
        projects.keys().cloned().collect()
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_project() {
        let manager = ProjectManager::new();
        let project_id = Uuid::new_v4();
        let path = PathBuf::from("/tmp/test_project");
        let metadata =
            models::ProjectMetadata::new("Test Project".to_string(), "Tester".to_string());

        manager
            .register_project(project_id, path.clone(), metadata)
            .await;

        let (retrieved_path, _metadata_arc) = manager.get_context(project_id).await.unwrap();
        assert_eq!(retrieved_path, path);
    }

    #[tokio::test]
    async fn test_get_nonexistent_project() {
        let manager = ProjectManager::new();
        let project_id = Uuid::new_v4();

        let result = manager.get_context(project_id).await;
        assert!(result.is_err());
    }
}
