use crate::intelligence::coordinator::IntelligenceCoordinator;
use crate::intelligence::models::CharacterGraphPayload;
use crate::intelligence::scanner::CharacterScanner;
use crate::models::ProjectMetadata;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type IntelligenceCache = RwLock<HashMap<Uuid, (u64, CharacterScanner)>>;
pub type ChapterContentCache = RwLock<HashMap<String, (u64, u64, Vec<(usize, Uuid)>)>>;

pub struct IntelligenceService {
    intelligence_cache: Arc<IntelligenceCache>,
    chapter_content_cache: Arc<ChapterContentCache>,
}

impl IntelligenceService {
    pub fn new() -> Self {
        Self {
            intelligence_cache: Arc::new(RwLock::new(HashMap::new())),
            chapter_content_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn analyze_project(
        &self,
        project_id: Uuid,
        root_path: &std::path::Path,
        metadata: &ProjectMetadata,
        options: crate::intelligence::coordinator::AnalysisOptions,
    ) -> crate::errors::Result<CharacterGraphPayload> {
        let coordinator = IntelligenceCoordinator::new(
            self.intelligence_cache.clone(),
            self.chapter_content_cache.clone(),
        );

        coordinator
            .analyze_project(project_id, root_path, metadata, options)
            .await
    }
}

impl Default for IntelligenceService {
    fn default() -> Self {
        Self::new()
    }
}
