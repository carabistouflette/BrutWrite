use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::path::{Path, PathBuf};

pub const MANUSCRIPT_DIR: &str = "manuscript";
pub const CHARACTERS_DIR: &str = "characters";
pub const RESEARCH_DIR: &str = "research";
pub const SNAPSHOTS_DIR: &str = ".snapshots";
pub const METADATA_FILENAME: &str = "project.json";

pub async fn create_project_structure<P: AsRef<Path>>(
    root_path: P,
    title: &str,
    author: &str,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();

    if root.exists() {
        return Err(Error::ProjectExists(root.to_string_lossy().to_string()));
    }

    // Create main project directory
    tokio::fs::create_dir_all(root).await?;

    // Create subdirectories
    let dirs = [MANUSCRIPT_DIR, CHARACTERS_DIR, RESEARCH_DIR, SNAPSHOTS_DIR];
    for dir in dirs {
        tokio::fs::create_dir(root.join(dir)).await?;
    }

    // Initialize project metadata
    let metadata = ProjectMetadata::new(title.to_string(), author.to_string());
    let metadata_path = root.join(METADATA_FILENAME);

    let json = serde_json::to_string_pretty(&metadata)?;
    tokio::fs::write(metadata_path, json).await?;

    Ok(metadata)
}

pub async fn load_project_metadata<P: AsRef<Path>>(root_path: P) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let metadata_path = root.join(METADATA_FILENAME);

    if !metadata_path.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Project metadata file not found",
        )));
    }

    let json = tokio::fs::read_to_string(metadata_path).await?;
    let metadata: ProjectMetadata = serde_json::from_str(&json)?;

    Ok(metadata)
}

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

pub async fn save_project_metadata<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
) -> Result<()> {
    let metadata_path = root_path.as_ref().join(METADATA_FILENAME);
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(metadata_path, json).await?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_and_load_project() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MyBook");

        // Create
        let created = create_project_structure(&project_path, "Test Book", "Test Author")
            .await
            .unwrap();

        // Load
        let loaded = load_project_metadata(&project_path).await.unwrap();

        assert_eq!(created.id, loaded.id);
        assert_eq!(loaded.title, "Test Book");
    }

    #[tokio::test]
    async fn test_save_metadata_io() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MetaTest");

        // Setup
        let mut meta = create_project_structure(&project_path, "Meta", "Author")
            .await
            .unwrap();
        meta.title = "Updated Title".to_string();

        // Save
        save_project_metadata(&project_path, &meta).await.unwrap();

        // Load and Verify
        let loaded = load_project_metadata(&project_path).await.unwrap();
        assert_eq!(loaded.title, "Updated Title");
    }
}
