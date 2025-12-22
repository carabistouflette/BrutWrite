use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
