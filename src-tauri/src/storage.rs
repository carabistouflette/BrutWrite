use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::path::{Path, PathBuf};

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
    let dirs = ["manuscript", "characters", "research", ".snapshots"];
    for dir in dirs {
        tokio::fs::create_dir(root.join(dir)).await?;
    }

    // Initialize project metadata
    let metadata = ProjectMetadata::new(title.to_string(), author.to_string());
    let metadata_path = root.join("project.json");

    let json = serde_json::to_string_pretty(&metadata)?;
    tokio::fs::write(metadata_path, json).await?;

    Ok(metadata)
}

pub async fn load_project_metadata<P: AsRef<Path>>(root_path: P) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let metadata_path = root.join("project.json");

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
        Ok(root.join("manuscript").join(fname))
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
    let metadata_path = root_path.as_ref().join("project.json");
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(metadata_path, json).await?;
    Ok(())
}

pub async fn create_chapter_node<P: AsRef<Path>>(
    root_path: P,
    metadata: &mut ProjectMetadata,
    parent_id: Option<String>,
    name: String,
) -> Result<()> {
    let root = root_path.as_ref();

    // 1. Generate ID and Filename
    let new_id = format!("chapter-{}", uuid::Uuid::new_v4());
    let filename = format!("{}.md", new_id);

    // 2. Calculate Order (last child + 1)
    let siblings: Vec<&crate::models::Chapter> = metadata
        .manifest
        .chapters
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .collect();

    let max_order = siblings.iter().map(|c| c.order).max().unwrap_or(0);
    let new_order = if siblings.is_empty() {
        0
    } else {
        max_order + 1
    };

    // 3. Create Chapter Object
    let new_chapter = crate::models::Chapter {
        id: new_id.clone(),
        parent_id,
        title: name,
        filename: filename.clone(),
        word_count: 0,
        order: new_order,
        chronological_date: None,
        abstract_timeframe: None,
        duration: None,
        plotline_tag: None,
        depends_on: None,
        pov_character_id: None,
    };

    // 4. Create File
    let manuscript_dir = root.join("manuscript");
    if !manuscript_dir.exists() {
        tokio::fs::create_dir_all(&manuscript_dir).await?;
    }
    let file_path = manuscript_dir.join(&filename);
    tokio::fs::write(&file_path, "").await?;

    // 5. Update Metadata
    metadata.manifest.chapters.push(new_chapter);
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
