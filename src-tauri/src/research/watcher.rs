use crate::models::research::ResearchArtifact;
use crate::research::ResearchState;
use crate::storage;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

pub fn init_research_watcher<R: Runtime>(app: &AppHandle<R>, project_path: PathBuf) {
    let research_path = project_path.join("research");
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        // Ensure directory exists (async)
        if !research_path.exists() {
            let _ = tokio::fs::create_dir_all(&research_path).await;
        }

        let state = app_handle.state::<ResearchState>();

        {
            let mut inner = state.inner.lock().await;
            inner.root_path = Some(research_path.clone());
        }

        // Initial scan
        scan_artifacts(&research_path, &state).await;

        // Setup Watcher
        let (tx, mut rx) = mpsc::unbounded_channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .unwrap();

        let _ = watcher.watch(&research_path, RecursiveMode::Recursive);

        {
            let mut inner = state.inner.lock().await;
            inner.watcher = Some(watcher);
        }

        // Handle events with debounce
        let mut debounce_deadline = tokio::time::Instant::now();
        let mut is_dirty = false;

        loop {
            tokio::select! {
                Some(res) = rx.recv() => {
                    match res {
                        Ok(event) => {
                            // Filter out changes to the index file itself to prevent loops
                            let is_index_change = event.paths.iter().any(|p| {
                                p.file_name()
                                    .map(|n| n.to_string_lossy() == ".research-index.json")
                                    .unwrap_or(false)
                            });

                            if !is_index_change {
                                is_dirty = true;
                                debounce_deadline = tokio::time::Instant::now() + Duration::from_millis(500);
                            }
                        }
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
                _ = tokio::time::sleep_until(debounce_deadline), if is_dirty => {
                    is_dirty = false;
                    let path_to_scan = {
                        let inner = state.inner.lock().await;
                        inner.root_path.clone()
                    };

                    if let Some(path) = path_to_scan {
                        scan_artifacts(&path, &state).await;
                    }
                    let _ = app_handle.emit("research-update", ());
                }
                else => break, // Channel closed
            }
        }
    });
}

async fn scan_artifacts(path: &PathBuf, state: &ResearchState) {
    let mut current_artifacts = HashMap::new();
    let index_data = storage::load_index(path);
    let mut index = index_data;
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

    let mut inner = state.inner.lock().await;
    inner.artifacts = current_artifacts;
}
