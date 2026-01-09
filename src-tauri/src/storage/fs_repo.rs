use super::traits::{FileMetadata, FileRepository};
use crate::errors::Result;
use async_trait::async_trait;
use std::path::Path;

#[derive(Default, Clone)]
pub struct LocalFileRepository;

#[async_trait]
impl FileRepository for LocalFileRepository {
    async fn read_file(&self, path: &Path) -> Result<String> {
        Ok(tokio::fs::read_to_string(path).await?)
    }

    async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        Ok(tokio::fs::try_exists(path).await?)
    }

    async fn delete(&self, path: &Path) -> Result<()> {
        if self.exists(path).await? {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut paths = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            paths.push(entry.path());
        }
        Ok(paths)
    }

    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let meta = tokio::fs::metadata(path).await?;
        let len = meta.len();
        let modified = meta
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(FileMetadata { len, modified })
    }
}
