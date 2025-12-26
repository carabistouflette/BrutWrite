use crate::errors::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn read_file(&self, path: &Path) -> Result<String>;
    async fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    async fn exists(&self, path: &Path) -> Result<bool>;
    async fn delete(&self, path: &Path) -> Result<()>;
    async fn create_dir_all(&self, path: &Path) -> Result<()>;
}
