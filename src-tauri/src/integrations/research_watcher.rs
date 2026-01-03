use crate::AppState;
use log::{error, warn};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

pub fn init_research_watcher<R: Runtime>(app: &AppHandle<R>, project_path: PathBuf) {
    let research_path = project_path.join("research");
    let app_handle = app.clone();

    // Spawn initialization and watcher setup
    tauri::async_runtime::spawn(async move {
        let app_state = app_handle.state::<AppState>();
        let state = &app_state.research;

        // 1. Synchronize initialization
        let _lock = state.init_lock.lock().await;

        // Ensure we stop any existing watcher before starting a new one
        state.stop().await;

        // 2. Initialize State (Ensure dir exists, scan)
        if let Err(e) = state.initialize(research_path.clone()).await {
            error!("Failed to initialize research state: {:?}", e);
            return;
        }

        // 3. Setup Watcher
        let (tx, mut rx) = mpsc::unbounded_channel();
        let watcher_res = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    // This is expected when the channel is closed
                    warn!("Watcher failed to send event: {}", e);
                }
            },
            Config::default(),
        );

        match watcher_res {
            Ok(mut watcher) => {
                if let Err(e) = watcher.watch(&research_path, RecursiveMode::Recursive) {
                    error!("Failed to watch research directory: {:?}", e);
                    return;
                }

                // Save watcher in state. This drops any previous watcher,
                // which closes its associated channel and stops its task.
                state.set_watcher(watcher).await;

                // 4. Handle Events
                while let Some(res) = rx.recv().await {
                    match res {
                        Ok(event) => {
                            if let Err(e) = state.handle_fs_change(event).await {
                                warn!("Error handling fs change: {:?}", e);
                            } else {
                                let _ = app_handle.emit("research-update", ());
                            }
                        }
                        Err(e) => error!("Watch error: {:?}", e),
                    }
                }
                warn!("Research watcher task for {:?} exiting.", research_path);
            }
            Err(e) => error!("Failed to create watcher: {:?}", e),
        }
    });
}
