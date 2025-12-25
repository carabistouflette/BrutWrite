use crate::models::research::ResearchArtifact;
use std::collections::HashMap;

pub fn get_file_type_from_name(file_name: &str) -> String {
    let ext = std::path::Path::new(file_name)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    match ext.to_lowercase().as_str() {
        "pdf" => "pdf",
        "png" | "jpg" | "jpeg" | "webp" => "image",
        "md" | "txt" => "text",
        _ => "other",
    }
    .to_string()
}

pub fn reconcile_index(
    disk_files: HashMap<String, String>,
    mut index: HashMap<String, ResearchArtifact>,
) -> HashMap<String, ResearchArtifact> {
    let mut current_artifacts = HashMap::new();

    for (file_path, file_name) in disk_files {
        let existing_id = {
            index.iter().find_map(|(id, art)| {
                if art.path == file_path {
                    Some(id.clone())
                } else {
                    None
                }
            })
        };

        if let Some(id) = existing_id {
            if let Some(mut artifact) = index.remove(&id) {
                artifact.name = file_name;
                current_artifacts.insert(id, artifact);
            }
        } else {
            let file_type = get_file_type_from_name(&file_name);
            let artifact = ResearchArtifact::new(file_path, file_name, file_type);
            current_artifacts.insert(artifact.id.clone(), artifact);
        }
    }
    current_artifacts
}
