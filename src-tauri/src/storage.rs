use crate::errors::{Error, Result};
use crate::models::{Manifest, ProjectMetadata};
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

pub fn update_project_manifest<P: AsRef<Path>>(
    root_path: P,
    manifest: Manifest,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    // 1. Load existing to preserve other metadata
    let mut metadata = load_project_metadata(root)?;

    // 2. Update manifest
    metadata.manifest = manifest;
    metadata.updated_at = chrono::Utc::now(); // Update timestamp

    // 3. Save
    let metadata_path = root.join("project.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
}

/// Helper to find chapter filename from manifest and return full path
fn resolve_chapter_path<P: AsRef<Path>>(root_path: P, chapter_id: &str) -> Result<PathBuf> {
    let root = root_path.as_ref();
    let metadata = load_project_metadata(root)?;

    let filename = metadata
        .manifest
        .chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.filename.clone());

    if let Some(fname) = filename {
        Ok(root.join("manuscript").join(fname))
    } else {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Chapter {} not found in manifest", chapter_id),
        )))
    }
}

pub fn read_chapter_content<P: AsRef<Path>>(root_path: P, chapter_id: &str) -> Result<String> {
    let chapter_path = resolve_chapter_path(root_path, chapter_id)?;

    if !chapter_path.exists() {
        return Ok(String::new());
    }

    let content = fs::read_to_string(chapter_path)?;
    Ok(content)
}

pub fn save_chapter_content<P: AsRef<Path>>(
    root_path: P,
    chapter_id: &str,
    content: &str,
) -> Result<()> {
    let root = root_path.as_ref();
    let chapter_path = resolve_chapter_path(root, chapter_id)?;

    // Ensure manuscript directory exists (sanity check)
    if !root.join("manuscript").exists() {
        fs::create_dir_all(root.join("manuscript"))?;
    }

    fs::write(chapter_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Chapter, Manifest};
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
    fn test_save_chapter_by_id() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MyNovel");

        // Create Project
        create_project_structure(&project_path, "Novel", "Author").unwrap();

        // Manually update manifest to have a chapter "c1" -> "c1.md"
        let manifest = Manifest {
            chapters: vec![Chapter {
                id: "c1".to_string(),
                parent_id: None,
                title: "Ch1".to_string(),
                filename: "c1.md".to_string(),
                order: 0,
            }],
        };
        update_project_manifest(&project_path, manifest).unwrap();

        // Save Content
        let content = "# Chapter 1\nIt was a dark and stormy night.";
        save_chapter_content(&project_path, "c1", content).unwrap();

        // Read Content
        let read_back = read_chapter_content(&project_path, "c1").unwrap();
        assert_eq!(read_back, content);
    }
}
