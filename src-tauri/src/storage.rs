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

pub fn update_project_settings<P: AsRef<Path>>(
    root_path: P,
    settings: crate::models::ProjectSettings,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let mut metadata = load_project_metadata(root)?;

    metadata.settings = settings;
    metadata.updated_at = chrono::Utc::now();

    let metadata_path = root.join("project.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
}

pub fn delete_chapter_file<P: AsRef<Path>>(root_path: P, filename: &str) -> Result<()> {
    let root = root_path.as_ref();
    let file_path = root.join("manuscript").join(filename);

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Helper to find chapter filename from metadata and return full path
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
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Chapter {} not found in manifest", chapter_id),
        )))
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

pub fn save_chapter_content<P: AsRef<Path>>(
    root_path: P,
    chapter_id: &str,
    content: &str,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let mut metadata = load_project_metadata(root)?;
    let chapter_path = resolve_chapter_path(root, &metadata, chapter_id)?;

    // Ensure manuscript directory exists
    let manuscript_dir = root.join("manuscript");
    if !manuscript_dir.exists() {
        fs::create_dir_all(&manuscript_dir)?;
    }

    // 1. Write content
    fs::write(&chapter_path, content)?;

    // 2. Calculate word count server-side
    let word_count = content.split_whitespace().count() as u32;

    // 3. Update Manifest Word Count
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = word_count;
    }

    // 4. Save metadata
    metadata.updated_at = chrono::Utc::now();
    let metadata_path = root.join("project.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
}

pub fn save_character<P: AsRef<Path>>(
    root_path: P,
    character: crate::models::Character,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let mut metadata = load_project_metadata(root)?;

    if let Some(idx) = metadata
        .characters
        .iter()
        .position(|c| c.id == character.id)
    {
        metadata.characters[idx] = character;
    } else {
        metadata.characters.push(character);
    }

    metadata.updated_at = chrono::Utc::now();

    let metadata_path = root.join("project.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
}

pub fn delete_character<P: AsRef<Path>>(
    root_path: P,
    character_id: uuid::Uuid,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();
    let mut metadata = load_project_metadata(root)?;

    metadata.characters.retain(|c| c.id != character_id);
    metadata.updated_at = chrono::Utc::now();

    let metadata_path = root.join("project.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, json)?;

    Ok(metadata)
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
    fn test_save_chapter_by_filename() {
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
                word_count: 0,
                order: 0,
                chronological_date: None,
                abstract_timeframe: None,
                duration: None,
                plotline_tag: None,
                depends_on: None,
                pov_character_id: None,
            }],
        };
        update_project_manifest(&project_path, manifest).unwrap();

        // Save Content using chapter_id
        let content = "# Chapter 1\nIt was a dark and stormy night.";
        let updated_meta = save_chapter_content(&project_path, "c1", content).unwrap();

        // Read Content
        let read_back = read_chapter_content(&project_path, &updated_meta, "c1").unwrap();
        assert_eq!(read_back, content);

        // Verify Manifest Match
        let chapter = updated_meta
            .manifest
            .chapters
            .iter()
            .find(|c| c.id == "c1")
            .unwrap();
        assert_eq!(chapter.word_count, 10);
    }
    #[test]
    fn test_save_character_and_load() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("CharTest");

        // Create
        create_project_structure(&project_path, "Char Book", "Me").unwrap();

        // Save Character
        let char_id = uuid::Uuid::new_v4();
        let character = crate::models::Character {
            id: char_id,
            name: "Hero".to_string(),
            role: crate::models::CharacterRole::Protagonist,
            archetype: "".to_string(),
            description: "A brave hero".to_string(),
            engine: Default::default(),
            physical_features: "".to_string(),
            traits: vec![],
            arc: "".to_string(),
            notes: "".to_string(),
        };

        save_character(&project_path, character.clone()).unwrap();

        // Load and Verify
        let loaded = load_project_metadata(&project_path).unwrap();
        assert_eq!(loaded.characters.len(), 1);
        assert_eq!(loaded.characters[0].name, "Hero");
        assert_eq!(loaded.characters[0].id, char_id);
    }
}
