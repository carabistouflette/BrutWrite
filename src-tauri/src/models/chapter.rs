use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_nullable<'de, D, T>(
    deserializer: D,
) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
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
/// `None` means omitted; `Some(None)` means explicit JSON null for nullable values.
pub struct NodeMetadataUpdate {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub chronological_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub abstract_timeframe: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub duration: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub plotline_tag: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub depends_on: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub pov_character_id: Option<Option<String>>,
}

#[cfg(test)]
mod tests {
    use super::NodeMetadataUpdate;

    #[test]
    fn nullable_metadata_distinguishes_omitted_null_and_value() {
        let omitted: NodeMetadataUpdate = serde_json::from_str("{}").unwrap();
        assert_eq!(omitted.chronological_date, None);

        let cleared: NodeMetadataUpdate =
            serde_json::from_str(r#"{"chronological_date":null}"#).unwrap();
        assert_eq!(cleared.chronological_date, Some(None));

        let dependency_clear: NodeMetadataUpdate =
            serde_json::from_str(r#"{"depends_on":null}"#).unwrap();
        assert_eq!(dependency_clear.depends_on, Some(None));

        let set: NodeMetadataUpdate =
            serde_json::from_str(r#"{"chronological_date":"2040-01-02"}"#).unwrap();
        assert_eq!(set.chronological_date, Some(Some("2040-01-02".to_string())));
    }
}
