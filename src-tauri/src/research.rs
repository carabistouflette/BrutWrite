use crate::models::research::ResearchArtifact;
use crate::storage;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::{mpsc, Mutex};

pub struct ResearchState {
    pub watcher: Mutex<Option<RecommendedWatcher>>,
    pub artifacts: Mutex<HashMap<String, ResearchArtifact>>,
    pub root_path: Mutex<Option<PathBuf>>,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            watcher: Mutex::new(None),
            artifacts: Mutex::new(HashMap::new()),
            root_path: Mutex::new(None),
        }
    }
}

impl ResearchState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn init_research_watcher<R: Runtime>(app: &AppHandle<R>, project_path: PathBuf) {
    let research_path = project_path.join("research");

    // Ensure directory exists
    if !research_path.exists() {
        let _ = std::fs::create_dir_all(&research_path);
    }

    // Initial scan and setup
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<ResearchState>();
        *state.root_path.lock().await = Some(research_path.clone());

        // Initial scan
        scan_artifacts(&research_path, &state).await;

        // Setup Watcher
        let (tx, mut rx) = mpsc::channel(1);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.blocking_send(res);
            },
            Config::default(),
        )
        .unwrap();

        let _ = watcher.watch(&research_path, RecursiveMode::Recursive);

        *state.watcher.lock().await = Some(watcher);

        // Handle events
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
                        if let Some(path) = state.root_path.lock().await.as_ref() {
                            scan_artifacts(path, &state).await;
                        }
                        let _ = app_handle.emit("research-update", ());
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}

async fn scan_artifacts(path: &std::path::Path, state: &ResearchState) {
    let mut current_artifacts = HashMap::new();
    let mut index = storage::load_index(path);
    let disk_files = storage::scan_on_disk(path);

    // Reconcile index with disk
    for (file_path, file_name) in disk_files {
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
            // New file found
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
    if let Err(e) = storage::save_index(path, &current_artifacts) {
        println!("Failed to save research index: {:?}", e);
    }

    *state.artifacts.lock().await = current_artifacts;
}
