use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchArtifact {
    pub id: String,
    pub path: String,
    pub name: String,
    pub file_type: String, // "pdf", "image", "markdown", "text", "other"
    pub tags: Vec<String>,
}

impl ResearchArtifact {
    pub fn new(path: String, name: String, file_type: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            name,
            file_type,
            tags: Vec::new(),
        }
    }
}
