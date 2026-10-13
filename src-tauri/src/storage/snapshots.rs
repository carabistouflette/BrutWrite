use super::traits::FileRepository;
use crate::errors::Result;
use sha2::{Digest, Sha256};
use std::path::{Component, Path};

const SNAPSHOTS_DIR: &str = ".snapshots";
/// Maximum number of snapshots to keep per chapter
const MAX_SNAPSHOTS_PER_CHAPTER: usize = 50;

fn validate_chapter_id(chapter_id: &str) -> Result<()> {
    let path = Path::new(chapter_id);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || path.file_name().and_then(|name| name.to_str()) != Some(chapter_id)
    {
        return Err(crate::errors::Error::Validation(
            "Invalid snapshot chapter ID".to_string(),
        ));
    }
    Ok(())
}

fn validate_snapshot_filename(filename: &str) -> Result<()> {
    let path = Path::new(filename);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || path.file_name().and_then(|name| name.to_str()) != Some(filename)
        || path.extension().and_then(|extension| extension.to_str()) != Some("md")
    {
        return Err(crate::errors::Error::Validation(
            "Invalid snapshot filename".to_string(),
        ));
    }
    Ok(())
}

fn snapshot_directory(root_path: &Path, chapter_id: &str) -> Result<std::path::PathBuf> {
    validate_chapter_id(chapter_id)?;
    Ok(root_path
        .join(super::consts::MANUSCRIPT_DIR)
        .join(SNAPSHOTS_DIR)
        .join(chapter_id))
}

/// Cleans up old snapshots, keeping only the most recent MAX_SNAPSHOTS_PER_CHAPTER.
/// This prevents disk space exhaustion over time.
async fn cleanup_old_snapshots<R: FileRepository>(
    repo: &R,
    entries: &[std::path::PathBuf],
) -> Result<()> {
    if entries.len() > MAX_SNAPSHOTS_PER_CHAPTER {
        let to_delete = entries.len() - MAX_SNAPSHOTS_PER_CHAPTER;
        for path in entries.iter().take(to_delete) {
            let _ = repo.delete(path).await;
        }
    }
    Ok(())
}

/// Creates a snapshot of the given content for a chapter.
/// Returns Ok(Some(snapshot_path)) if a snapshot was created,
/// Ok(None) if the content is identical to the latest snapshot (deduplicated),
/// or Err if an IO error occurred.
pub async fn create_snapshot<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
    content: &str,
) -> Result<Option<String>> {
    let snapshots_dir = snapshot_directory(root_path, chapter_id)?;
    crate::validation::validate_content_size(content)?;

    if !repo.exists(&snapshots_dir).await? {
        repo.create_dir_all(&snapshots_dir).await?;
    }

    // 2. Compute hash of current content
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = hex::encode(hasher.finalize());
    let short_hash = &hash[0..8];

    // 3. Check latest snapshot to deduplicate
    // We list files, sort by name (timestamp should ensure order), and check last.
    // But reading file content is slow?
    // Optimization: Store hash in filename? <timestamp>_<hash>.md
    // Then we just check if the last file has the same hash suffix.

    let mut entries = repo.read_dir(&snapshots_dir).await?;
    // Filter for .md files
    entries.retain(|e| e.extension().is_some_and(|ext| ext == "md"));
    // Sort determines order. Timestamps (ISO) are sortable strings.
    entries.sort();

    if let Some(last_entry) = entries.last() {
        if let Some(filename) = last_entry.file_name() {
            let filename_str = filename.to_string_lossy();
            // Check if filename ends with hash
            if filename_str.ends_with(&format!("_{}.md", short_hash)) {
                // Content matches, dedupe
                return Ok(None);
            }
        }
    }

    // 4. Create new snapshot
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H%M%S");
    let mut sequence = 0_u64;
    let (filename, path) = loop {
        let filename = format!("{}_z{:08}_{}.md", timestamp, sequence, short_hash);
        let path = snapshots_dir.join(&filename);
        if !repo.exists(&path).await? {
            break (filename, path);
        }
        sequence += 1;
    };

    repo.write_file(&path, content).await?;

    // 5. Cleanup old snapshots to prevent disk exhaustion
    // Re-read entries after adding the new one
    let mut updated_entries = repo.read_dir(&snapshots_dir).await?;
    updated_entries.retain(|e| e.extension().is_some_and(|ext| ext == "md"));
    updated_entries.sort();
    cleanup_old_snapshots(repo, &updated_entries).await?;

    Ok(Some(filename))
}

pub async fn list_snapshots<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
) -> Result<Vec<String>> {
    let snapshots_dir = snapshot_directory(root_path, chapter_id)?;
    if !repo.exists(&snapshots_dir).await? {
        return Ok(Vec::new());
    }

    let mut entries = repo.read_dir(&snapshots_dir).await?;
    entries.retain(|e| e.extension().is_some_and(|ext| ext == "md"));
    entries.sort();

    // Return filenames
    Ok(entries
        .into_iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect())
}

pub async fn delete_chapter_snapshots<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
) -> Result<()> {
    let snapshots_dir = snapshot_directory(root_path, chapter_id)?;
    if !repo.exists(&snapshots_dir).await? {
        return Ok(());
    }

    let entries = repo.read_dir(&snapshots_dir).await?;
    for entry in entries {
        if entry.parent() == Some(snapshots_dir.as_path())
            && entry.extension().is_some_and(|extension| extension == "md")
        {
            repo.delete(&entry).await?;
        }
    }

    repo.remove_dir(&snapshots_dir).await
}

pub async fn read_snapshot_content<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
    filename: &str,
) -> Result<String> {
    let snapshots_dir = snapshot_directory(root_path, chapter_id)?;
    validate_snapshot_filename(filename)?;
    let path = snapshots_dir.join(filename);

    let content = repo.read_file(&path).await?;
    crate::validation::validate_content_size(&content)?;
    Ok(content)
}

pub async fn restore_snapshot<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
    snapshot_filename: &str,
    current_content: &str,
    chapter_filename: &str,
) -> Result<String> {
    validate_chapter_id(chapter_id)?;
    validate_snapshot_filename(snapshot_filename)?;

    // 1. Create safety snapshot of current state
    create_snapshot(repo, root_path, chapter_id, current_content).await?;

    // 2. Read snapshot content
    let content = read_snapshot_content(repo, root_path, chapter_id, snapshot_filename).await?;

    // 3. Write back to chapter file
    let chapter_path = root_path
        .join(super::consts::MANUSCRIPT_DIR)
        .join(chapter_filename);

    repo.write_file(&chapter_path, &content).await?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MockFileRepository;

    fn snapshots_dir(root: &Path, chapter_id: &str) -> std::path::PathBuf {
        root.join(super::super::consts::MANUSCRIPT_DIR)
            .join(SNAPSHOTS_DIR)
            .join(chapter_id)
    }

    #[tokio::test]
    async fn rejects_snapshot_path_traversal() {
        let repo = MockFileRepository::new();
        let root = Path::new("/project");

        assert!(
            read_snapshot_content(&repo, root, "../private", "secret.md")
                .await
                .is_err()
        );
        assert!(
            read_snapshot_content(&repo, root, "chapter-1", "../../secret.md")
                .await
                .is_err()
        );
        assert!(create_snapshot(&repo, root, "../private", "draft")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn restoring_an_empty_snapshot_writes_empty_content() {
        let repo = MockFileRepository::new();
        let root = Path::new("/project");
        let chapter_id = "chapter-1";
        let directory = snapshots_dir(root, chapter_id);
        let filename = "2024-01-01T000000_00000000_e3b0c442.md";
        repo.set_exists(directory.clone(), true);
        repo.set_content(directory.join(filename), String::new());

        let restored = restore_snapshot(
            &repo,
            root,
            chapter_id,
            filename,
            "<p>Current draft</p>",
            "chapter-1.md",
        )
        .await
        .unwrap();

        assert_eq!(restored, "");
        assert_eq!(
            repo.get_content(
                &root
                    .join(super::super::consts::MANUSCRIPT_DIR)
                    .join("chapter-1.md")
            )
            .as_deref(),
            Some("")
        );
    }

    #[tokio::test]
    async fn retention_keeps_only_the_latest_snapshot_limit() {
        let repo = MockFileRepository::new();
        let root = Path::new("/project");
        let directory = snapshots_dir(root, "chapter-1");
        repo.set_exists(directory, true);

        for index in 0..=MAX_SNAPSHOTS_PER_CHAPTER {
            create_snapshot(&repo, root, "chapter-1", &format!("Draft version {index}"))
                .await
                .unwrap();
        }

        let snapshots = list_snapshots(&repo, root, "chapter-1").await.unwrap();
        assert_eq!(snapshots.len(), MAX_SNAPSHOTS_PER_CHAPTER);
    }

    #[tokio::test]
    async fn retention_deletes_entries_in_oldest_first_order() {
        let repo = MockFileRepository::new();
        let root = Path::new("/project");
        let directory = snapshots_dir(root, "chapter-1");
        let entries: Vec<_> = (0..=MAX_SNAPSHOTS_PER_CHAPTER)
            .map(|index| {
                let path = directory.join(format!("2020-01-01T000000_{index:08}.md"));
                repo.set_content(path.clone(), format!("Draft version {index}"));
                path
            })
            .collect();

        cleanup_old_snapshots(&repo, &entries).await.unwrap();

        assert_eq!(repo.get_content(&entries[0]), None);
        assert!(entries[1..]
            .iter()
            .all(|entry| repo.get_content(entry).is_some()));
    }
    #[tokio::test]
    async fn deleting_chapter_snapshots_removes_files_and_directory() {
        let repo = MockFileRepository::new();
        let root = Path::new("/project");
        let directory = snapshots_dir(root, "chapter-1");
        let snapshot = directory.join("snapshot.md");
        repo.set_exists(directory.clone(), true);
        repo.set_content(snapshot.clone(), "History".to_string());

        delete_chapter_snapshots(&repo, root, "chapter-1")
            .await
            .unwrap();

        assert_eq!(repo.get_content(&snapshot), None);
        assert!(!repo.exists(&directory).await.unwrap());
    }
}
