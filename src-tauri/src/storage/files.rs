use super::consts::MANUSCRIPT_DIR;
use super::traits::FileRepository;
use crate::errors::{Error, Result};
use crate::models::ProjectMetadata;
use std::path::{Path, PathBuf};

/// Helper to resolve internal path. Pure logic.
pub fn resolve_chapter_path<P: AsRef<Path>>(
    root_path: P,
    metadata: &ProjectMetadata,
    chapter_id: &str,
) -> Result<PathBuf> {
    let root = root_path.as_ref();

    let filename = metadata
        .manifest
        .chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.filename.clone());

    if let Some(fname) = filename {
        let path = Path::new(&fname);
        // Security: Strictly validate components to prevent traversal or absolute path injection
        for component in path.components() {
            match component {
                std::path::Component::Normal(_) => {}
                std::path::Component::CurDir => {} // "." is fine
                _ => {
                    // ParentDir (..), RootDir (/), or Prefix (C:) are not allowed
                    return Err(Error::ChapterNotFound {
                        id: chapter_id.to_string(),
                    });
                }
            }
        }
        Ok(root.join(MANUSCRIPT_DIR).join(path))
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
    let manuscript_dir = root_path.join(MANUSCRIPT_DIR);

    if !repo.exists(&manuscript_dir).await? {
        repo.create_dir_all(&manuscript_dir).await?;
    }

    let file_path = manuscript_dir.join(filename);
    repo.write_file(&file_path, content).await
}

pub async fn delete_chapter_file<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    filename: &str,
) -> Result<()> {
    let file_path = root_path.join(MANUSCRIPT_DIR).join(filename);
    /*
       Note: The original implementation unwrapped try_exists error.
       The trait returns Result<bool>, propagating error is better.
    */
    if repo.exists(&file_path).await? {
        repo.delete(&file_path).await?;
    }
    Ok(())
}
