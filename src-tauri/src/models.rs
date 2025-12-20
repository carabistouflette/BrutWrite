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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ProjectSettings {
    pub daily_target: u32,
    pub theme: String,
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
    pub order: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub role: CharacterRole,
    pub description: String,
    pub traits: Vec<String>,
    pub arc: String,
    pub notes: String,
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
            theme: "brutalist-dark".to_string(),
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
        }
    }
}
