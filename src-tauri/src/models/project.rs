use super::character::Character;
use super::manifest::Manifest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub struct Plotline {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            daily_target: 2000,
            word_target: default_word_target(),
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
}
