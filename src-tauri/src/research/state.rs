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
        // Ensure directory exists
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
        }

        // Load index and scan disk
        let index_data = crate::storage::load_index(&path).await;
        // logic from old scan_artifacts replaced by utils
        let disk_files = crate::storage::scan_on_disk(&path).await;

        // REFACTORED: Use utility
        let current_artifacts = crate::research::utils::reconcile_index(disk_files, index_data);

        // Save reconciled
        crate::storage::save_index(&path, &current_artifacts).await?;

        let mut inner = self.inner.lock().await;
        inner.root_path = Some(path);
        inner.artifacts = current_artifacts;
        Ok(())
    }

    pub async fn set_watcher(&self, watcher: RecommendedWatcher) {
        let mut inner = self.inner.lock().await;
        inner.watcher = Some(watcher);
    }

    pub async fn get_all(&self) -> Vec<ResearchArtifact> {
        let inner = self.inner.lock().await;
        inner.artifacts.values().cloned().collect()
    }

    pub async fn create_note(&self, name: String) -> crate::errors::Result<ResearchArtifact> {
        let mut inner = self.inner.lock().await;
        // Clone root path to release borrow on inner
        let root = inner
            .root_path
            .as_ref()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
            .clone();

        let mut final_name = name;
        if !final_name.ends_with(".md") {
            final_name.push_str(".md");
        }

        let file_path = root.join(&final_name);
        if file_path.exists() {
            return Err(crate::errors::Error::Research(
                "Note already exists".to_string(),
            ));
        }

        tokio::fs::write(&file_path, "").await?;

        let artifact = ResearchArtifact::new(
            file_path.to_string_lossy().to_string(),
            final_name,
            "text".to_string(),
        );

        inner
            .artifacts
            .insert(artifact.id.clone(), artifact.clone());
        crate::storage::save_index(&root, &inner.artifacts).await?;

        Ok(artifact)
    }

    pub async fn import_files(&self, paths: Vec<String>) -> crate::errors::Result<()> {
        // We need to release the lock during IO to avoid blocking readers,
        // but for safety of `root_path` we might hold it or clone it.
        // Cloning root_path is better.
        let root = {
            let inner = self.inner.lock().await;
            inner
                .root_path
                .clone()
                .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
        };

        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                let file_name = path.file_name().ok_or_else(|| {
                    crate::errors::Error::Validation("Invalid file path".to_string())
                })?;
                let dest = root.join(file_name);
                tokio::fs::copy(path, dest).await?;
                // The watcher or a manual scan should pick this up.
                // But since we are moving to immediate consistency, we should probably add it here too.
                // However, without metadata like "type", it's slightly annoying to duplicate the logic.
                // For now, relying on the watcher for imported files is "okay" but better to be explicit
                // if we want avoiding races.
                // IMPLEMENTATION DECISION: Let's assume the Watcher (via handle_fs_change) picks it up.
                // OR we reuse the logic. Let's rely on watcher for imports for now to keep this method simple,
                // as imports are external mostly.
                // ACTUALLY, the plan said "handle incremental updates". So the watcher WILL call handle_fs_change.
            }
        }
        Ok(())
    }

    pub async fn update_content(&self, id: String, content: String) -> crate::errors::Result<()> {
        let artifact = {
            let inner = self.inner.lock().await;
            inner.artifacts.get(&id).cloned()
        };

        if let Some(artifact) = artifact {
            tokio::fs::write(&artifact.path, content).await?;
            Ok(())
        } else {
            Err(crate::errors::Error::ArtifactNotFound(id))
        }
    }

    pub async fn rename_artifact(&self, id: String, new_name: String) -> crate::errors::Result<()> {
        let mut inner = self.inner.lock().await;
        // Check root exists
        let root = inner
            .root_path
            .as_ref()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
            .clone();

        if let Some(artifact) = inner.artifacts.get_mut(&id) {
            let old_path = PathBuf::from(&artifact.path);

            let mut new_filename = new_name.clone();
            if let Some(ext) = old_path.extension() {
                let ext_str = ext.to_string_lossy();
                if !new_name
                    .to_lowercase()
                    .ends_with(&format!(".{}", ext_str.to_lowercase()))
                {
                    new_filename.push('.');
                    new_filename.push_str(&ext_str);
                }
            }

            let new_path = root.join(&new_filename);
            if new_path.exists() {
                return Err(crate::errors::Error::Research(
                    "Destination already exists".to_string(),
                ));
            }

            tokio::fs::rename(&old_path, &new_path).await?;

            artifact.name = new_name;
            artifact.path = new_path.to_string_lossy().to_string();

            crate::storage::save_index(&root, &inner.artifacts).await?;
            Ok(())
        } else {
            Err(crate::errors::Error::ArtifactNotFound(id))
        }
    }

    pub async fn delete_artifact(&self, id: String) -> crate::errors::Result<()> {
        let mut inner = self.inner.lock().await;
        let root = inner
            .root_path
            .as_ref()
            .ok_or(crate::errors::Error::ResearchVaultNotInitialized)?
            .clone();

        if let Some(artifact) = inner.artifacts.remove(&id) {
            let path = PathBuf::from(&artifact.path);
            if path.exists() {
                tokio::fs::remove_file(path).await?;
            }
            crate::storage::save_index(&root, &inner.artifacts).await?;
            Ok(())
        } else {
            Err(crate::errors::Error::ArtifactNotFound(id))
        }
    }

    // Handles incremental changes from the watcher
    pub async fn handle_fs_change(&self, event: notify::Event) -> crate::errors::Result<()> {
        let mut inner = self.inner.lock().await;
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

            // Determine if it's a create/modify or delete
            // Notify events can be tricky (Rename is often two events or one).
            // Simplest robust approach: check if file exists.

            if path.exists() {
                // Upsert
                // Find if we already have an artifact with this path
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
                    // It exists, no action needed for content changes in index
                } else {
                    // New file detected!
                    // REFACTORED: Use utility
                    let file_type = crate::research::utils::get_file_type_from_name(&file_name);

                    let artifact = ResearchArtifact::new(path_str, file_name, file_type);
                    inner.artifacts.insert(artifact.id.clone(), artifact);
                    // Persist immediately on detection
                    crate::storage::save_index(&root, &inner.artifacts).await?;
                }
            } else {
                // File gone?
                // Find artifact by path
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
}
