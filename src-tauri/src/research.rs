use crate::models::research::ResearchArtifact;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

pub struct ResearchState {
    pub watcher: Mutex<Option<RecommendedWatcher>>,
    pub artifacts: Mutex<HashMap<String, ResearchArtifact>>,
    pub root_path: Mutex<Option<PathBuf>>,
}

impl ResearchState {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
            artifacts: Mutex::new(HashMap::new()),
            root_path: Mutex::new(None),
        }
    }
}

pub fn init_research_watcher<R: Runtime>(app: &AppHandle<R>, project_path: PathBuf) {
    let research_path = project_path.join("research");

    // Ensure directory exists
    if !research_path.exists() {
        let _ = std::fs::create_dir_all(&research_path);
    }

    let state = app.state::<ResearchState>();
    *state.root_path.lock().unwrap() = Some(research_path.clone());

    // Initial scan
    scan_artifacts(&research_path, &state);

    // Setup Watcher
    let (tx, mut rx) = mpsc::channel(1);
    let app_handle = app.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )
    .unwrap();

    let _ = watcher.watch(&research_path, RecursiveMode::Recursive);

    *state.watcher.lock().unwrap() = Some(watcher);

    // Handle events in a separate task
    tauri::async_runtime::spawn(async move {
        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => {
                    // Filter out changes to the index file itself to prevent loops
                    let is_index_change = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|n| n.to_string_lossy() == ".research-index.json")
                            .unwrap_or(false)
                    });

                    if !is_index_change {
                        let state = app_handle.state::<ResearchState>();
                        refresh_artifacts(&state);
                        let _ = app_handle.emit("research-update", ());
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    fn refresh_artifacts(state: &ResearchState) {
        if let Some(path) = state.root_path.lock().unwrap().as_ref() {
            scan_artifacts(path, state);
        }
    }
}

fn load_index(path: &std::path::Path) -> HashMap<String, ResearchArtifact> {
    let index_path = path.join(".research-index.json");
    if index_path.exists() {
        if let Ok(content) = std::fs::read_to_string(index_path) {
            if let Ok(artifacts) =
                serde_json::from_str::<HashMap<String, ResearchArtifact>>(&content)
            {
                return artifacts;
            }
        }
    }
    HashMap::new()
}

fn save_index(path: &std::path::Path, artifacts: &HashMap<String, ResearchArtifact>) {
    let index_path = path.join(".research-index.json");
    if let Ok(new_content) = serde_json::to_string_pretty(artifacts) {
        // Read existing to compare
        if index_path.exists() {
            if let Ok(existing_content) = std::fs::read_to_string(&index_path) {
                if existing_content == new_content {
                    return; // No change
                }
            }
        }
        let _ = std::fs::write(index_path, new_content);
    }
}

fn scan_artifacts(path: &std::path::Path, state: &ResearchState) {
    let mut current_artifacts = HashMap::new();
    let mut index = load_index(path);
    let mut disk_files = HashMap::new(); // Map<Path, Name>

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let file_name = entry_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                // Skip the index file itself
                if file_name == ".research-index.json" {
                    continue;
                }

                disk_files.insert(entry_path.to_string_lossy().to_string(), file_name);
            }
        }
    }

    // Reconcile index with disk
    for (file_path, file_name) in disk_files {
        // Check if we already have an entry for this path
        // In a real robust system, we might track by inode or hash to handle renames better
        // For now, we trust path. If path is new, we check if there's an orphan in index with same name (rename heuristic)

        let existing_id = index.iter().find_map(|(id, art)| {
            if art.path == file_path {
                Some(id.clone())
            } else {
                None
            }
        });

        if let Some(id) = existing_id {
            // Keep existing
            if let Some(mut artifact) = index.remove(&id) {
                artifact.name = file_name; // Update name just in case
                current_artifacts.insert(id, artifact);
            }
        } else {
            // New file found.
            // Stretch: Check for rename (same name, different path? or same hash?)
            // For MVP: Treat as new artifact
            let ext = std::path::Path::new(&file_name)
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let file_type = match ext.to_lowercase().as_str() {
                "pdf" => "pdf",
                "png" | "jpg" | "jpeg" | "webp" => "image",
                "md" | "txt" => "text",
                _ => "other",
            };

            let artifact = ResearchArtifact::new(file_path, file_name, file_type.to_string());
            current_artifacts.insert(artifact.id.clone(), artifact);
        }
    }

    // Save updated index
    save_index(path, &current_artifacts);

    *state.artifacts.lock().unwrap() = current_artifacts;
}

#[tauri::command]
pub fn get_research_artifacts(state: tauri::State<ResearchState>) -> Vec<ResearchArtifact> {
    let artifacts = state.artifacts.lock().unwrap();
    artifacts.values().cloned().collect()
}

#[tauri::command]
pub async fn add_research_files(
    state: tauri::State<'_, ResearchState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let root_path = state.root_path.lock().unwrap().clone();
    if let Some(root) = root_path {
        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                let file_name = path.file_name().unwrap();
                let dest = root.join(file_name);
                tokio::fs::copy(path, dest)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn update_research_artifact(
    state: tauri::State<ResearchState>,
    artifact: ResearchArtifact,
) -> Result<(), String> {
    let mut artifacts = state.artifacts.lock().unwrap();
    if artifacts.contains_key(&artifact.id) {
        artifacts.insert(artifact.id.clone(), artifact);

        // Persist
        let root = state.root_path.lock().unwrap().clone();
        if let Some(path) = root {
            save_index(&path, &artifacts);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_research_note(
    state: tauri::State<'_, ResearchState>,
    name: String,
) -> Result<ResearchArtifact, String> {
    let root = state.root_path.lock().unwrap().clone();
    if let Some(path) = root {
        let mut final_name = name;
        if !final_name.ends_with(".md") {
            final_name.push_str(".md");
        }

        let file_path = path.join(&final_name);
        if file_path.exists() {
            return Err("Note already exists".to_string());
        }

        tokio::fs::write(&file_path, "")
            .await
            .map_err(|e| e.to_string())?;

        // Manually create artifact to return immediately
        let artifact = ResearchArtifact::new(
            file_path.to_string_lossy().to_string(),
            final_name,
            "text".to_string(),
        );

        // The watcher will eventually pick it up, but we can add it to state now for responsiveness
        {
            let mut artifacts = state.artifacts.lock().unwrap();
            artifacts.insert(artifact.id.clone(), artifact.clone());
            save_index(&path, &artifacts);
        }

        return Ok(artifact);
    }
    Err("Research vault not initialized".to_string())
}

#[tauri::command]
pub async fn update_note_content(
    state: tauri::State<'_, ResearchState>,
    id: String,
    content: String,
) -> Result<(), String> {
    let artifact = {
        let artifacts = state.artifacts.lock().unwrap();
        artifacts.get(&id).cloned()
    };

    if let Some(artifact) = artifact {
        tokio::fs::write(&artifact.path, content)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Artifact not found".to_string())
    }
}
