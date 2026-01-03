use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Project already exists at path: {0}")]
    ProjectExists(String),

    #[error("Invalid project structure at {path:?}: {reason}")]
    InvalidStructure {
        path: std::path::PathBuf,
        reason: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Chapter `{id}` not found")]
    ChapterNotFound { id: String },

    #[error("Character `{id}` not found")]
    CharacterNotFound { id: uuid::Uuid },

    #[error("Research error: {0}")]
    Research(String),

    #[error("Research vault not initialized")]
    ResearchVaultNotInitialized,

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("Intelligence engine error: {0}")]
    Intelligence(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::Io(_) => "IO_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::ProjectExists(_) => "PROJECT_EXISTS",
            Error::InvalidStructure { .. } => "INVALID_STRUCTURE",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::ChapterNotFound { .. } => "CHAPTER_NOT_FOUND",
            Error::CharacterNotFound { .. } => "CHARACTER_NOT_FOUND",
            Error::Research(_) => "RESEARCH_ERROR",
            Error::ResearchVaultNotInitialized => "RESEARCH_NOT_INITIALIZED",
            Error::ArtifactNotFound(_) => "ARTIFACT_NOT_FOUND",
            Error::Intelligence(_) => "INTELLIGENCE_ERROR",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
