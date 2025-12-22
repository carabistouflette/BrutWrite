use serde::{Deserialize, Serialize};

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
