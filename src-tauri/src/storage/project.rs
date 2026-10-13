use super::consts::METADATA_FILENAME;
use crate::models::ProjectMetadata;
use crate::storage::fs_repo::LocalFileRepository;
use crate::storage::traits::FileRepository;
use std::path::Path;
use uuid::Uuid;

pub async fn create_project_structure(
    root_path: &Path,
    name: &str,
    author: &str,
) -> crate::errors::Result<ProjectMetadata> {
    let repo = LocalFileRepository;

    if repo.exists(root_path).await? {
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
    metadata.manifest.validate()?;

    Ok(metadata)
}

pub async fn save_project_metadata(
    root_path: &Path,
    metadata: &ProjectMetadata,
) -> crate::errors::Result<()> {
    metadata.manifest.validate()?;

    let file_path = root_path.join(METADATA_FILENAME);
    let temporary_path = root_path.join(format!(".{}.{}.tmp", METADATA_FILENAME, Uuid::new_v4()));
    let content = serde_json::to_string_pretty(metadata)?;

    if let Err(write_error) = tokio::fs::write(&temporary_path, content.as_bytes()).await {
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return Err(write_error.into());
    }
    if let Err(rename_error) = tokio::fs::rename(&temporary_path, &file_path).await {
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return Err(rename_error.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_project_metadata, save_project_metadata};
    use crate::models::{Chapter, ProjectMetadata};

    #[tokio::test]
    async fn metadata_round_trip_preserves_nested_order_and_word_counts() {
        let directory = tempfile::tempdir().unwrap();
        let mut metadata = ProjectMetadata::new("Project".to_string(), "Author".to_string());
        metadata.manifest.chapters = vec![
            Chapter {
                id: "root".to_string(),
                parent_id: None,
                title: "Root".to_string(),
                filename: "root.md".to_string(),
                word_count: 27,
                order: 4,
                chronological_date: Some("2035-01-01".to_string()),
                abstract_timeframe: None,
                duration: None,
                plotline_tag: None,
                depends_on: None,
                pov_character_id: None,
            },
            Chapter {
                id: "child".to_string(),
                parent_id: Some("root".to_string()),
                title: "Child".to_string(),
                filename: "child.md".to_string(),
                word_count: 63,
                order: 2,
                chronological_date: None,
                abstract_timeframe: None,
                duration: None,
                plotline_tag: None,
                depends_on: Some("root".to_string()),
                pov_character_id: None,
            },
        ];

        save_project_metadata(directory.path(), &metadata)
            .await
            .unwrap();
        metadata.title = "Updated".to_string();
        save_project_metadata(directory.path(), &metadata)
            .await
            .unwrap();
        let loaded = load_project_metadata(directory.path()).await.unwrap();
        assert_eq!(loaded.title, "Updated");

        assert_eq!(loaded.manifest.chapters[0].order, 4);
        assert_eq!(
            loaded.manifest.chapters[1].parent_id.as_deref(),
            Some("root")
        );
        assert_eq!(loaded.manifest.chapters[0].word_count, 27);
        assert_eq!(loaded.manifest.chapters[1].word_count, 63);
        assert_eq!(
            loaded.manifest.chapters[0].chronological_date.as_deref(),
            Some("2035-01-01")
        );
        assert_eq!(
            loaded.manifest.chapters[1].depends_on.as_deref(),
            Some("root")
        );
    }
}
