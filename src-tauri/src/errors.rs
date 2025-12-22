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

    #[error("Invalid project structure: {0}")]
    InvalidStructure(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Chapter not found: {0}")]
    ChapterNotFound(String),

    #[error("Character not found: {0}")]
    CharacterNotFound(String),

    #[error("Research error: {0}")]
    Research(String),

    #[error("Research vault not initialized")]
    ResearchVaultNotInitialized,

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
