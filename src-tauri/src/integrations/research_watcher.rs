use crate::research::ResearchState;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

pub fn init_research_watcher<R: Runtime>(app: &AppHandle<R>, project_path: PathBuf) {
    let research_path = project_path.join("research");
    let app_handle = app.clone();

    // Spawn initialization and watcher setup
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<ResearchState>();

        // 1. Initialize State (Ensure dir exists, scan)
        if let Err(e) = state.initialize(research_path.clone()).await {
            println!("Failed to initialize research state: {:?}", e);
            return;
        }

        // 2. Setup Watcher
        let (tx, mut rx) = mpsc::unbounded_channel();
        let watcher_res = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        );

        match watcher_res {
            Ok(mut watcher) => {
                if let Err(e) = watcher.watch(&research_path, RecursiveMode::Recursive) {
                    println!("Failed to watch research directory: {:?}", e);
                    return;
                }

                // Save watcher in state
                state.set_watcher(watcher).await;

                // 3. Handle Events
                while let Some(res) = rx.recv().await {
                    match res {
                        Ok(event) => {
                            // Handle incremental change
                            // Debouncing could be added here or in state.
                            // For now, let's trust simple events.
                            // To avoid spamming frontend, we might want to debounce emissions.
                            // But let's start simple.
                            if let Err(e) = state.handle_fs_change(event).await {
                                println!("Error handling fs change: {:?}", e);
                            } else {
                                let _ = app_handle.emit("research-update", ());
                            }
                        }
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Failed to create watcher: {:?}", e),
        }
    });
}
