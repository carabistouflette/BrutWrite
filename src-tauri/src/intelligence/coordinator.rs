use crate::intelligence::graph::build_character_graph_cached;
use crate::intelligence::models::CharacterGraphPayload;
use crate::intelligence::scanner::CharacterScanner;
use crate::models::ProjectMetadata;
use crate::storage::traits::FileRepository;
use crate::storage::{resolve_chapter_path, LocalFileRepository};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Cache Types (Aliased from lib.rs for clarity, though typically we'd import or define shared types)
pub type IntelligenceCache = Mutex<HashMap<Uuid, (u64, CharacterScanner)>>;
pub type ChapterContentCache = Mutex<HashMap<String, (u64, Vec<(usize, String)>)>>;

pub struct IntelligenceCoordinator<'a> {
    scanner_cache: &'a IntelligenceCache,
    content_cache: &'a ChapterContentCache,
}

impl<'a> IntelligenceCoordinator<'a> {
    pub fn new(
        scanner_cache: &'a IntelligenceCache,
        content_cache: &'a ChapterContentCache,
    ) -> Self {
        Self {
            scanner_cache,
            content_cache,
        }
    }

    pub async fn analyze_project(
        &self,
        project_id: Uuid,
        root_path: &std::path::Path,
        metadata: &ProjectMetadata,
        options: AnalysisOptions,
    ) -> crate::errors::Result<CharacterGraphPayload> {
        // 1. Check for empty characters
        if metadata.characters.is_empty() {
            return build_character_graph_cached(
                metadata,
                &HashMap::new(),
                &HashMap::new(),
                options.proximity_window,
                options.prune_threshold,
            );
        }

        // 2. Initialize Scanner (Cached)
        let scanner = self.get_or_create_scanner(project_id, metadata).await?;

        // 3. Process Chapters
        let (chapter_texts, chapter_mentions) = self
            .process_chapters(root_path, metadata, &scanner, &options)
            .await?;

        // 4. Build Graph
        build_character_graph_cached(
            metadata,
            &chapter_texts,
            &chapter_mentions,
            options.proximity_window,
            options.prune_threshold,
        )
    }

    async fn get_or_create_scanner(
        &self,
        project_id: Uuid,
        metadata: &ProjectMetadata,
    ) -> crate::errors::Result<CharacterScanner> {
        let current_hash = {
            let mut s = DefaultHasher::new();
            for c in &metadata.characters {
                c.id.hash(&mut s);
                c.name.hash(&mut s);
            }
            s.finish()
        };

        let mut cache = self.scanner_cache.lock().await;

        if let Some((hash, scanner)) = cache.get(&project_id) {
            if *hash == current_hash {
                return Ok(scanner.clone());
            }
        }

        // Rebuild
        let scanner = CharacterScanner::try_new(&metadata.characters)
            .map_err(crate::errors::Error::Intelligence)?;

        cache.insert(project_id, (current_hash, scanner.clone()));
        Ok(scanner)
    }

    async fn process_chapters(
        &self,
        root_path: &std::path::Path,
        metadata: &ProjectMetadata,
        scanner: &CharacterScanner,
        options: &AnalysisOptions,
    ) -> crate::errors::Result<(
        HashMap<String, String>,
        HashMap<String, Vec<(usize, String)>>,
    )> {
        let repo = LocalFileRepository;
        let mut tasks = Vec::new();

        // 1. Identify valid chapters
        for chapter in &metadata.manifest.chapters {
            if let Some(ref filter) = options.chapter_filter {
                if !filter.contains(chapter.id.as_str()) {
                    continue;
                }
            }

            match resolve_chapter_path(root_path, metadata, &chapter.id) {
                Ok(path) => tasks.push((chapter.id.clone(), path)),
                Err(e) => log::warn!(
                    "Skipping chapter {}: path resolution failed ({})",
                    chapter.id,
                    e
                ),
            }
        }

        // 2. Parallel Read
        let mut join_set = tokio::task::JoinSet::new();
        for (cid, path) in tasks {
            let repo = repo.clone();
            join_set.spawn(async move {
                match repo.read_file(&path).await {
                    Ok(content) => (cid, Some(content)),
                    Err(e) => {
                        log::error!("Failed to read chapter {}: {}", cid, e);
                        (cid, None)
                    }
                }
            });
        }

        let mut chapter_texts = HashMap::new();
        let mut chapter_mentions = HashMap::new();

        // 3. Process Results
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok((cid, content_opt)) => {
                    if let Some(content) = content_opt {
                        if content.is_empty() {
                            continue;
                        }

                        let mentions = self.get_mentions_for_chapter(&cid, &content, scanner).await;
                        chapter_texts.insert(cid.clone(), content);
                        chapter_mentions.insert(cid, mentions);
                    }
                }
                Err(e) => log::error!("Task join error: {}", e),
            }
        }

        Ok((chapter_texts, chapter_mentions))
    }

    async fn get_mentions_for_chapter(
        &self,
        cid: &str,
        content: &str,
        scanner: &CharacterScanner,
    ) -> Vec<(usize, String)> {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = hasher.finish();

        let mut cache = self.content_cache.lock().await;

        if let Some((cached_hash, matches)) = cache.get(cid) {
            if *cached_hash == content_hash {
                return matches.clone();
            }
        }

        // Scan
        let matches = scanner.scan(content);
        cache.insert(cid.to_string(), (content_hash, matches.clone()));
        matches
    }
}

pub struct AnalysisOptions {
    pub proximity_window: usize,
    pub prune_threshold: f32,
    pub chapter_filter: Option<HashSet<String>>,
}
