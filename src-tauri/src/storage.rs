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
    modify_metadata(root_path, |metadata| {
        metadata.manifest = manifest;
        Ok(())
    })
}

pub fn update_project_settings<P: AsRef<Path>>(
    root_path: P,
    settings: crate::models::ProjectSettings,
) -> Result<ProjectMetadata> {
    modify_metadata(root_path, |metadata| {
        metadata.settings = settings;
        Ok(())
    })
}

pub fn delete_chapter_file<P: AsRef<Path>>(root_path: P, filename: &str) -> Result<()> {
    let root = root_path.as_ref();
    let file_path = root.join("manuscript").join(filename);

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Internal helper to load, modify, and save project metadata in one go.
fn modify_metadata<P, F>(root_path: P, modifier: F) -> Result<ProjectMetadata>
where
    P: AsRef<Path>,
    F: FnOnce(&mut ProjectMetadata) -> Result<()>,
{
    let root = root_path.as_ref();
    let mut metadata = load_project_metadata(root)?;

    modifier(&mut metadata)?;

    metadata.updated_at = chrono::Utc::now();
    save_project_metadata(root, &metadata)?;

    Ok(metadata)
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

pub fn save_chapter_content<P: AsRef<Path>>(
    root_path: P,
    chapter_id: &str,
    content: &str,
) -> Result<ProjectMetadata> {
    let root = root_path.as_ref();

    modify_metadata(root, |metadata| {
        let chapter_path = resolve_chapter_path(root, metadata, chapter_id)?;

        // Ensure manuscript directory exists
        let manuscript_dir = root.join("manuscript");
        if !manuscript_dir.exists() {
            fs::create_dir_all(&manuscript_dir)?;
        }

        // 1. Write content
        fs::write(&chapter_path, content)?;

        // 2. Calculate word count server-side (strip HTML)
        let word_count = crate::models::count_words(content);

        // 3. Update Manifest Word Count
        if let Some(chapter) = metadata
            .manifest
            .chapters
            .iter_mut()
            .find(|c| c.id == chapter_id)
        {
            chapter.word_count = word_count;
            Ok(())
        } else {
            Err(Error::ChapterNotFound(chapter_id.to_string()))
        }
    })
}

pub fn save_character<P: AsRef<Path>>(
    root_path: P,
    character: crate::models::Character,
) -> Result<ProjectMetadata> {
    modify_metadata(root_path, |metadata| {
        if let Some(idx) = metadata
            .characters
            .iter()
            .position(|c| c.id == character.id)
        {
            metadata.characters[idx] = character;
        } else {
            metadata.characters.push(character);
        }
        Ok(())
    })
}

pub fn delete_character<P: AsRef<Path>>(
    root_path: P,
    character_id: uuid::Uuid,
) -> Result<ProjectMetadata> {
    modify_metadata(root_path, |metadata| {
        let initial_len = metadata.characters.len();
        metadata.characters.retain(|c| c.id != character_id);

        if metadata.characters.len() == initial_len {
            return Err(Error::CharacterNotFound(character_id.to_string()));
        }
        Ok(())
    })
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

pub fn update_plotlines<P: AsRef<Path>>(
    root_path: P,
    plotlines: Vec<crate::models::Plotline>,
) -> Result<ProjectMetadata> {
    modify_metadata(root_path, |metadata| {
        metadata.plotlines = plotlines;
        Ok(())
    })
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
