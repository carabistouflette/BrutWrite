use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn count_words(content: &str) -> u32 {
    let re = HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
    let plain_text = re.replace_all(content, " ");
    plain_text.split_whitespace().count() as u32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ProjectMetadata {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub settings: ProjectSettings,
    pub manifest: Manifest,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub plotlines: Vec<Plotline>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ProjectSettings {
    pub daily_target: u32,
    #[serde(default = "default_word_target")]
    pub word_target: u32,
}

fn default_word_target() -> u32 {
    50000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Chapter {
    pub id: String, // chap-1, chap-2, etc.
    pub parent_id: Option<String>,
    pub title: String,
    pub filename: String,
    #[serde(default)]
    pub word_count: u32,
    pub order: u32,
    /// ISO 8601 date/time for chronological placement
    #[serde(default)]
    pub chronological_date: Option<String>,
    /// Abstract timeframe (e.g., "Day 1", "Year 5") for fantasy/sci-fi
    #[serde(default)]
    pub abstract_timeframe: Option<String>,
    /// Estimated in-world duration (e.g., "2 hours", "3 days")
    #[serde(default)]
    pub duration: Option<String>,
    /// Plotline/subplot tag for swimlane grouping
    #[serde(default)]
    pub plotline_tag: Option<String>,
    /// Scene that must occur before this one (for causality checking)
    #[serde(default)]
    pub depends_on: Option<String>,
    /// POV character ID (for simultaneous-scene paradox detection)
    #[serde(default)]
    pub pov_character_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct NodeMetadataUpdate {
    pub title: Option<String>,
    pub chronological_date: Option<String>,
    pub abstract_timeframe: Option<String>,
    pub duration: Option<String>,
    pub plotline_tag: Option<String>,
    pub depends_on: Option<String>,
    pub pov_character_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Plotline {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub role: CharacterRole,
    #[serde(default)]
    pub archetype: String,
    pub description: String,
    #[serde(default)]
    pub engine: CharacterEngine,
    #[serde(default)]
    pub physical_features: String,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub arc: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct CharacterEngine {
    pub desire: String,
    pub fear: String,
    pub wound: String,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CharacterRole {
    Protagonist,
    Antagonist,
    Secondary,
    Extra,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            daily_target: 2000,
            word_target: default_word_target(),
        }
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            chapters: Vec::new(),
        }
    }
}

impl ProjectMetadata {
    pub fn new(title: String, author: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            author,
            created_at: now,
            updated_at: now,
            settings: ProjectSettings::default(),
            manifest: Manifest::default(),
            characters: Vec::new(),
            plotlines: vec![Plotline {
                id: "main".to_string(),
                name: "Main Plot".to_string(),
                color: "#3b82f6".to_string(),
            }],
        }
    }

    pub fn add_or_update_character(&mut self, character: Character) {
        if let Some(idx) = self.characters.iter().position(|c| c.id == character.id) {
            self.characters[idx] = character;
        } else {
            self.characters.push(character);
        }
    }

    pub fn remove_character(&mut self, character_id: Uuid) -> Result<(), String> {
        let initial_len = self.characters.len();
        self.characters.retain(|c| c.id != character_id);

        if self.characters.len() == initial_len {
            Err("Character not found".to_string())
        } else {
            Ok(())
        }
    }

    pub fn create_chapter(&self, parent_id: Option<String>, title: String) -> Chapter {
        let new_id = format!("chapter-{}", Uuid::new_v4());
        let filename = format!("{}.md", new_id);

        let siblings: Vec<&Chapter> = self
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

        Chapter {
            id: new_id.clone(),
            parent_id,
            title,
            filename: filename.clone(),
            word_count: 0,
            order: new_order,
            chronological_date: None,
            abstract_timeframe: None,
            duration: None,
            plotline_tag: None,
            depends_on: None,
            pov_character_id: None,
        }
    }

    pub fn remove_node_recursively(&mut self, node_id: String) -> Vec<String> {
        // Build efficient lookup
        let mut children_map: std::collections::HashMap<Option<String>, Vec<String>> =
            std::collections::HashMap::new();

        for c in &self.manifest.chapters {
            children_map
                .entry(c.parent_id.clone())
                .or_default()
                .push(c.id.clone());
        }

        let mut ids_to_remove = std::collections::HashSet::new();
        let mut stack = vec![node_id];

        while let Some(current_id) = stack.pop() {
            if ids_to_remove.insert(current_id.clone()) {
                if let Some(children) = children_map.get(&Some(current_id)) {
                    stack.extend(children.clone());
                }
            }
        }

        // Collect filenames and remove chapters
        // efficient retain
        let mut filenames = Vec::new();
        self.manifest.chapters.retain(|c| {
            if ids_to_remove.contains(&c.id) {
                filenames.push(c.filename.clone());
                false
            } else {
                true
            }
        });

        filenames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("<p>Hello world</p>"), 2);
        assert_eq!(count_words("<p>Hello</p><p>world</p>"), 2);
        assert_eq!(count_words("Hello  world"), 2); // multiple spaces
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("<div>Nested <span>content</span></div>"), 2);
    }

    fn create_dummy_chapter(id: &str, parent: Option<&str>, filename: &str) -> Chapter {
        Chapter {
            id: id.to_string(),
            parent_id: parent.map(|s| s.to_string()),
            title: "Title".to_string(),
            filename: filename.to_string(),
            word_count: 0,
            order: 0,
            chronological_date: None,
            abstract_timeframe: None,
            duration: None,
            plotline_tag: None,
            depends_on: None,
            pov_character_id: None,
        }
    }

    #[test]
    fn test_remove_node_recursively() {
        let mut metadata = ProjectMetadata::new("Test".to_string(), "Author".to_string());

        // Tree:
        // root
        //  - c1 (c1.md)
        //     - c1_1 (c1_1.md)
        //  - c2 (c2.md)

        metadata
            .manifest
            .chapters
            .push(create_dummy_chapter("c1", None, "c1.md"));
        metadata
            .manifest
            .chapters
            .push(create_dummy_chapter("c1_1", Some("c1"), "c1_1.md"));
        metadata
            .manifest
            .chapters
            .push(create_dummy_chapter("c2", None, "c2.md"));

        let removed_files = metadata.remove_node_recursively("c1".to_string());

        assert_eq!(removed_files.len(), 2);
        assert!(removed_files.contains(&"c1.md".to_string()));
        assert!(removed_files.contains(&"c1_1.md".to_string()));
        assert!(!removed_files.contains(&"c2.md".to_string()));

        assert_eq!(metadata.manifest.chapters.len(), 1);
        assert_eq!(metadata.manifest.chapters[0].id, "c2");
    }
}
