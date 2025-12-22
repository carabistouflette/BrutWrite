use super::consts::MANUSCRIPT_DIR;
use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::path::{Path, PathBuf};

pub fn resolve_chapter_path<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<PathBuf> {
    let root = root_path.as_ref();

    let filename = metadata
        .manifest
        .chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.filename.clone());

    if let Some(fname) = filename {
        Ok(root.join(MANUSCRIPT_DIR).join(fname))
    } else {
        Err(Error::ChapterNotFound(chapter_id.to_string()))
    }
}

pub async fn read_chapter_content<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<String> {
    let chapter_path = resolve_chapter_path(root_path, metadata, chapter_id)?;

    if !chapter_path.exists() {
        return Ok(String::new());
    }

    let content = tokio::fs::read_to_string(chapter_path).await?;
    Ok(content)
}

pub async fn write_chapter_file<P: AsRef<Path>>(
    root_path: P,
    filename: &str,
    content: &str,
) -> Result<()> {
    let root = root_path.as_ref();
    let manuscript_dir = root.join(MANUSCRIPT_DIR);

    if !manuscript_dir.exists() {
        tokio::fs::create_dir_all(&manuscript_dir).await?;
    }

    let file_path = manuscript_dir.join(filename);
    tokio::fs::write(file_path, content).await?;
    Ok(())
}

pub async fn delete_chapter_file<P: AsRef<Path>>(root_path: P, filename: &str) -> Result<()> {
    let file_path = root_path.as_ref().join(MANUSCRIPT_DIR).join(filename);
    if tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
        tokio::fs::remove_file(file_path).await?;
    }
    Ok(())
}
