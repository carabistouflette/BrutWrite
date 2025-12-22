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
                Ok(_event) => {
                    // Simple logic: Re-scan on any file change for now
                    // Optimization: handle specific events
                    let state = app_handle.state::<ResearchState>();
                    refresh_artifacts(&state);
                    let _ = app_handle.emit("research-update", ());
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

fn scan_artifacts(path: &PathBuf, state: &ResearchState) {
    let mut artifacts = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let ext = path
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

                let artifact = ResearchArtifact::new(
                    path.to_string_lossy().to_string(),
                    name,
                    file_type.to_string(),
                );
                artifacts.insert(artifact.id.clone(), artifact);
            }
        }
    }
    *state.artifacts.lock().unwrap() = artifacts;
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
