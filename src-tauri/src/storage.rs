use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_project_structure<P: AsRef<Path>>(
    root_path: P,
    title: &str,
    author: &str,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();

    if root.exists() {
        return Err(Error::ProjectExists(root.to_string_lossy().to_string()));
    }

    // Create main project directory
    fs::create_dir_all(root)?;

    // Create subdirectories
    let dirs = ["manuscript", "characters", "research", ".snapshots"];
    for dir in dirs {
        fs::create_dir(root.join(dir))?;
    }

    // Initialize project metadata
    let metadata = ProjectMetadata::new(title.to_string(), author.to_string());
    let metadata_path = root.join("project.json");

    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
}

pub fn load_project_metadata<P: AsRef<Path>>(root_path: P) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let metadata_path = root.join("project.json");

    if !metadata_path.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Project metadata file not found",
        )));
    }

    let json = fs::read_to_string(metadata_path)?;
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

pub fn read_chapter_content<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<String> {
    let chapter_path = resolve_chapter_path(root_path, metadata, chapter_id)?;

    if !chapter_path.exists() {
        return Ok(String::new());
    }

    let content = fs::read_to_string(chapter_path)?;
    Ok(content)
}

pub fn save_project_metadata<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
) -> Result<()> {
    let metadata_path = root_path.as_ref().join("project.json");
    let json = serde_json::to_string_pretty(metadata)?;
    fs::write(metadata_path, json)?;
    Ok(())
}

pub fn create_chapter_node<P: AsRef<Path>>(
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
        fs::create_dir_all(&manuscript_dir)?;
    }
    let file_path = manuscript_dir.join(&filename);
    fs::write(&file_path, "")?;

    // 5. Update Metadata
    metadata.manifest.chapters.push(new_chapter);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_load_project() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MyBook");

        // Create
        let created = create_project_structure(&project_path, "Test Book", "Test Author").unwrap();

        // Load
        let loaded = load_project_metadata(&project_path).unwrap();

        assert_eq!(created.id, loaded.id);
        assert_eq!(loaded.title, "Test Book");
    }

    #[test]
    fn test_save_metadata_io() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MetaTest");

        // Setup
        let mut meta = create_project_structure(&project_path, "Meta", "Author").unwrap();
        meta.title = "Updated Title".to_string();

        // Save
        save_project_metadata(&project_path, &meta).unwrap();

        // Load and Verify
        let loaded = load_project_metadata(&project_path).unwrap();
        assert_eq!(loaded.title, "Updated Title");
    }
}
