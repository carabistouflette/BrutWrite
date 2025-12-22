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
}
