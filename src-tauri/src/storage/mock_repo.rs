use super::traits::{FileMetadata, FileRepository};
use crate::errors::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct MockFileRepository {
    files: Arc<Mutex<HashMap<PathBuf, String>>>,
    exists_override: Arc<Mutex<HashMap<PathBuf, bool>>>,
}

impl MockFileRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_content(&self, path: PathBuf, content: String) {
        self.files
            .lock()
            .expect("mutex poisoned")
            .insert(path, content);
    }

    pub fn get_content(&self, path: &Path) -> Option<String> {
        self.files
            .lock()
            .expect("mutex poisoned")
            .get(path)
            .cloned()
    }

    pub fn set_exists(&self, path: PathBuf, exists: bool) {
        self.exists_override
            .lock()
            .expect("mutex poisoned")
            .insert(path, exists);
    }
}

#[async_trait]
impl FileRepository for MockFileRepository {
    async fn read_file(&self, path: &Path) -> Result<String> {
        self.files
            .lock()
            .expect("mutex poisoned")
            .get(path)
            .cloned()
            .ok_or_else(|| {
                crate::errors::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found in mock",
                ))
            })
    }

    async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        self.files
            .lock()
            .expect("mutex poisoned")
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        if let Some(&exists) = self
            .exists_override
            .lock()
            .expect("mutex poisoned")
            .get(path)
        {
            return Ok(exists);
        }
        Ok(self
            .files
            .lock()
            .expect("mutex poisoned")
            .contains_key(path))
    }

    async fn delete(&self, path: &Path) -> Result<()> {
        self.files.lock().expect("mutex poisoned").remove(path);
        Ok(())
    }

    async fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let files = self.files.lock().expect("mutex poisoned");
        let paths: Vec<PathBuf> = files
            .keys()
            .filter(|p| p.starts_with(path))
            .cloned()
            .collect();
        Ok(paths)
    }

    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata> {
        // Ensure file exists first
        if !self.exists(path).await? {
            return Err(crate::errors::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found in mock",
            )));
        }

        // Return dummy metadata since our mock doesn't track modification time/size strictly
        // or we could calculate size from content.
        let len = self.get_content(path).map(|c| c.len() as u64).unwrap_or(0);
        Ok(FileMetadata {
            len,
            modified: 1000,
        })
    }
}
