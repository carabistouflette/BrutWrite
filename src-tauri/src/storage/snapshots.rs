use super::traits::FileRepository;
use crate::errors::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

const SNAPSHOTS_DIR: &str = ".snapshots";
/// Maximum number of snapshots to keep per chapter
const MAX_SNAPSHOTS_PER_CHAPTER: usize = 50;

/// Cleans up old snapshots, keeping only the most recent MAX_SNAPSHOTS_PER_CHAPTER.
/// This prevents disk space exhaustion over time.
async fn cleanup_old_snapshots<R: FileRepository>(
    repo: &R,
    _snapshots_dir: &Path, // Kept for future logging if needed
    entries: &[std::path::PathBuf],
) -> Result<()> {
    if entries.len() > MAX_SNAPSHOTS_PER_CHAPTER {
        let to_delete = entries.len() - MAX_SNAPSHOTS_PER_CHAPTER;
        // Entries are sorted oldest first, so delete from the beginning
        for path in entries.iter().take(to_delete) {
            // Ignore errors during deletion - best effort cleanup
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
    // 1. Setup paths
    // Structure: <root>/manuscript/.snapshots/<chapter_id>/
    // Note: We use "manuscript" as base because chapters are there?
    // Let's verify where chapters are. MANUSCRIPT_DIR.
    // Ideally .snapshots should be adjacent or inside.
    // Let's go with <root>/manuscript/<chapter_id>/.snapshots is not possible because chapter is a file.
    // So <root>/manuscript/.snapshots/<chapter_id>/
    let snapshots_dir = root_path
        .join(super::consts::MANUSCRIPT_DIR)
        .join(SNAPSHOTS_DIR)
        .join(chapter_id);

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
            if filename_str.contains(&format!("_{}.md", short_hash)) {
                // Content matches, dedupe
                return Ok(None);
            }
        }
    }

    // 4. Create new snapshot
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H%M%S");
    let filename = format!("{}_{}.md", timestamp, short_hash);
    let path = snapshots_dir.join(&filename);

    repo.write_file(&path, content).await?;

    // 5. Cleanup old snapshots to prevent disk exhaustion
    // Re-read entries after adding the new one
    let mut updated_entries = repo.read_dir(&snapshots_dir).await?;
    updated_entries.retain(|e| e.extension().is_some_and(|ext| ext == "md"));
    updated_entries.sort();
    cleanup_old_snapshots(repo, &snapshots_dir, &updated_entries).await?;

    Ok(Some(filename))
}

pub async fn list_snapshots<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
) -> Result<Vec<String>> {
    let snapshots_dir = root_path
        .join(super::consts::MANUSCRIPT_DIR)
        .join(SNAPSHOTS_DIR)
        .join(chapter_id);

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

pub async fn read_snapshot_content<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
    filename: &str,
) -> Result<String> {
    let path = root_path
        .join(super::consts::MANUSCRIPT_DIR)
        .join(SNAPSHOTS_DIR)
        .join(chapter_id)
        .join(filename);

    repo.read_file(&path).await
}

pub async fn restore_snapshot<R: FileRepository>(
    repo: &R,
    root_path: &Path,
    chapter_id: &str,
    snapshot_filename: &str,
    current_content: &str,
    chapter_filename: &str,
) -> Result<String> {
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
