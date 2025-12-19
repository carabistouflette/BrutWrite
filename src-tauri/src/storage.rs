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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_project_structure() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("MyBook");
        
        let result = create_project_structure(&project_path, "Test Book", "Test Author");
        assert!(result.is_ok());

        assert!(project_path.exists());
        assert!(project_path.join("manuscript").is_dir());
        assert!(project_path.join("characters").is_dir());
        assert!(project_path.join("research").is_dir());
        assert!(project_path.join(".snapshots").is_dir());
        assert!(project_path.join("project.json").is_file());

        let metadata_content = fs::read_to_string(project_path.join("project.json")).unwrap();
        let metadata: ProjectMetadata = serde_json::from_str(&metadata_content).unwrap();
        assert_eq!(metadata.title, "Test Book");
        assert_eq!(metadata.author, "Test Author");
    }
}
