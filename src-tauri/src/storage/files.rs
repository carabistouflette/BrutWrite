use super::consts::MANUSCRIPT_DIR;
use super::traits::FileRepository;
use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::path::{Path, PathBuf};

/// Helper to resolve internal path from a known filename. Pure logic.
pub fn resolve_chapter_path_from_filename<P: AsRef<Path>>(
    root_path: P,
    filename: &str,
) -> Result<PathBuf> {
    let root = root_path.as_ref();
    crate::validation::validate_name(filename)?;
    if filename.contains('/') || filename.contains('\\') {
        return Err(Error::ChapterNotFound {
            id: format!("Invalid filename: {}", filename),
        });
    }
    let path = Path::new(filename);
    let mut components = path.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(Error::ChapterNotFound {
            id: format!("Invalid filename: {}", filename),
        });
    }
    Ok(root.join(MANUSCRIPT_DIR).join(path))
}

/// Helper to resolve internal path. Pure logic.
pub fn resolve_chapter_path<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<PathBuf> {
    let filename = metadata
        .manifest
        .chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.filename.clone());

    if let Some(fname) = filename {
        resolve_chapter_path_from_filename(root_path, &fname)
    } else {
        Err(Error::ChapterNotFound {
            id: chapter_id.to_string(),
        })
    }
}

pub async fn read_chapter_content<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<String> {
    let chapter_path = resolve_chapter_path(root_path, metadata, chapter_id)?;

    if !repo.exists(&chapter_path).await? {
        return Ok(String::new());
    }

    repo.read_file(&chapter_path).await
}

pub async fn write_chapter_file<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    filename: &str,
    content: &str,
) -> Result<()> {
    let file_path = resolve_chapter_path_from_filename(root_path, filename)?;
    let manuscript_dir = file_path.parent().ok_or_else(|| Error::ChapterNotFound {
        id: format!("Invalid filename: {}", filename),
    })?;

    if !repo.exists(manuscript_dir).await? {
        repo.create_dir_all(manuscript_dir).await?;
    }

    repo.write_file(&file_path, content).await
}

pub async fn delete_chapter_file<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    filename: &str,
) -> Result<()> {
    let file_path = resolve_chapter_path_from_filename(root_path, filename)?;
    if repo.exists(&file_path).await? {
        repo.delete(&file_path).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_filename_rejects_path_traversal_and_absolute_paths() {
        assert!(resolve_chapter_path_from_filename("/project", "../chapter.md").is_err());
        assert!(resolve_chapter_path_from_filename("/project", "/chapter.md").is_err());
        assert!(resolve_chapter_path_from_filename("/project", "nested/chapter.md").is_err());
    }

    #[test]
    fn chapter_filename_resolves_within_manuscript_directory() {
        let path = resolve_chapter_path_from_filename("/project", "chapter.md").unwrap();
        assert_eq!(
            path,
            PathBuf::from("/project")
                .join(MANUSCRIPT_DIR)
                .join("chapter.md")
        );
    }
}
