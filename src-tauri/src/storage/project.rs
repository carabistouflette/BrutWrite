use super::consts::METADATA_FILENAME;
use crate::models::ProjectMetadata;
use crate::storage::fs_repo::LocalFileRepository;
use crate::storage::traits::FileRepository;
use std::path::Path;

pub async fn create_project_structure(
    root_path: &Path,
    name: &str,
    author: &str,
) -> crate::errors::Result<ProjectMetadata> {
    let repo = LocalFileRepository;

    if repo.exists(root_path).await? {
        // Check if directory is empty or not? For now invoke error if exists
        // Actually typically "create" fails if folder exists.
        return Err(crate::errors::Error::ProjectExists(
            root_path.to_string_lossy().to_string(),
        ));
    }

    repo.create_dir_all(root_path).await?;

    let metadata = ProjectMetadata::new(name.to_string(), author.to_string());
    save_project_metadata(root_path, &metadata).await?;

    Ok(metadata)
}

pub async fn load_project_metadata(root_path: &Path) -> crate::errors::Result<ProjectMetadata> {
    let repo = LocalFileRepository;
    let file_path = root_path.join(METADATA_FILENAME);

    if !repo.exists(&file_path).await? {
        return Err(crate::errors::Error::InvalidStructure {
            path: root_path.to_path_buf(),
            reason: "Missing project.json".to_string(),
        });
    }

    let content = repo.read_file(&file_path).await?;
    let metadata: ProjectMetadata = serde_json::from_str(&content)?;

    Ok(metadata)
}

pub async fn save_project_metadata(
    root_path: &Path,
    metadata: &ProjectMetadata,
) -> crate::errors::Result<()> {
    let repo = LocalFileRepository;
    let file_path = root_path.join(METADATA_FILENAME);
    let content = serde_json::to_string_pretty(metadata)?;

    repo.write_file(&file_path, &content).await?;
    Ok(())
}
