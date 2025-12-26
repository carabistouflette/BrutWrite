use super::traits::FileRepository;
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
        self.files.lock().unwrap().insert(path, content);
    }

    pub fn get_content(&self, path: &Path) -> Option<String> {
        self.files.lock().unwrap().get(path).cloned()
    }

    pub fn set_exists(&self, path: PathBuf, exists: bool) {
        self.exists_override.lock().unwrap().insert(path, exists);
    }
}

#[async_trait]
impl FileRepository for MockFileRepository {
    async fn read_file(&self, path: &Path) -> Result<String> {
        self.files
            .lock()
            .unwrap()
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
            .unwrap()
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        if let Some(&exists) = self.exists_override.lock().unwrap().get(path) {
            return Ok(exists);
        }
        Ok(self.files.lock().unwrap().contains_key(path))
    }

    async fn delete(&self, path: &Path) -> Result<()> {
        self.files.lock().unwrap().remove(path);
        Ok(())
    }

    async fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}
