use std::fs;
use std::path::Path;
use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;

pub fn create_project_structure<P: AsRef<Path>>(root_path: P, title: &str, author: &str) -> Result<ProjectMetadata> {
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
            "Project metadata file not found"
        )));
    }

    let json = fs::read_to_string(metadata_path)?;
    let metadata: ProjectMetadata = serde_json::from_str(&json)?;

    Ok(metadata)
}

pub fn save_chapter_content<P: AsRef<Path>>(root_path: P, filename: &str, content: &str) -> Result<()> {
    let root = root_path.as_ref();
    let chapter_path = root.join("manuscript").join(filename);

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
        assert_eq!(loaded.author, "Test Author");
        assert!(project_path.join("project.json").exists());
    }

    #[test]
    fn test_save_chapter() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MyNovel");
        
        create_project_structure(&project_path, "Novel", "Author").unwrap();
        
        let filename = "chapter1.md";
        let content = "# Chapter 1\nIt was a dark and stormy night.";
        
        save_chapter_content(&project_path, filename, content).unwrap();
        
        let saved_path = project_path.join("manuscript").join(filename);
        assert!(saved_path.exists());
        
        let saved_content = fs::read_to_string(saved_path).unwrap();
        assert_eq!(saved_content, content);
    }
}
