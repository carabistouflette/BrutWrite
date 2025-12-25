use crate::errors::Result;
use crate::models::research::ResearchArtifact;
use std::collections::HashMap;
use std::path::Path;

const INDEX_FILENAME: &str = ".research-index.json";

pub async fn load_index<P: AsRef<Path>>(path: P) -> HashMap<String, ResearchArtifact> {
    let index_path = path.as_ref().join(INDEX_FILENAME);
    if index_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(index_path).await {
            if let Ok(artifacts) =
                serde_json::from_str::<HashMap<String, ResearchArtifact>>(&content)
            {
                return artifacts;
            }
        }
    }
    HashMap::new()
}

pub async fn save_index<P: AsRef<Path>>(
    path: P,
    artifacts: &HashMap<String, ResearchArtifact>,
) -> Result<()> {
    let index_path = path.as_ref().join(INDEX_FILENAME);
    let new_content = serde_json::to_string_pretty(artifacts)?;

    // Read existing to compare
    if index_path.exists() {
        if let Ok(existing_content) = tokio::fs::read_to_string(&index_path).await {
            if existing_content == new_content {
                return Ok(()); // No change
            }
        }
    }
    tokio::fs::write(index_path, new_content).await?;
    Ok(())
}

pub async fn scan_on_disk<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let mut disk_files = HashMap::new(); // Map<Path, Name>
    let mut entries = match tokio::fs::read_dir(path).await {
        Ok(e) => e,
        Err(_) => return disk_files,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let entry_path = entry.path();
        if entry_path.is_file() {
            let file_name = entry_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Skip the index file itself
            if file_name == INDEX_FILENAME {
                continue;
            }

            disk_files.insert(entry_path.to_string_lossy().to_string(), file_name);
        }
    }
    disk_files
}
