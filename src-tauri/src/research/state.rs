use crate::models::research::ResearchArtifact;
use notify::RecommendedWatcher;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct ResearchInner {
    pub watcher: Option<RecommendedWatcher>,
    pub artifacts: HashMap<String, ResearchArtifact>,
    pub root_path: Option<PathBuf>,
}

pub struct ResearchState {
    pub inner: Mutex<ResearchInner>,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ResearchInner {
                watcher: None,
                artifacts: HashMap::new(),
                root_path: None,
            }),
        }
    }
}

impl ResearchState {
    pub fn new() -> Self {
        Self::default()
    }
}
