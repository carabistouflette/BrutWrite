use crate::intelligence::graph::build_character_graph_cached;
use crate::intelligence::models::CharacterGraphPayload;
use crate::intelligence::scanner::CharacterScanner;
use crate::models::ProjectMetadata;
use crate::storage::traits::FileRepository;
use crate::storage::{resolve_chapter_path, LocalFileRepository};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cache Types
/// Keys:
/// - IntelligenceCache: Project ID -> (Hash of Characters, Scanner)
/// - ChapterContentCache: Chapter ID -> (Hash of Content, Mentions)
///
/// Note: Hash is now storing 64 bits derived from SHA256 for efficient checking,
/// but the calculation is deterministic.
pub type IntelligenceCache = RwLock<HashMap<Uuid, (u64, CharacterScanner)>>;
pub type ChapterContentCache = RwLock<HashMap<String, (u64, u64, Vec<(usize, String)>)>>;

pub struct IntelligenceCoordinator {
    scanner_cache: Arc<IntelligenceCache>,
    content_cache: Arc<ChapterContentCache>,
}

impl IntelligenceCoordinator {
    pub fn new(
        scanner_cache: Arc<IntelligenceCache>,
        content_cache: Arc<ChapterContentCache>,
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
                None,
            );
        }

        // 2. Initialize Scanner (Cached)
        let (scanner, scanner_hash) = self.get_or_create_scanner(project_id, metadata).await?;

        // 3. Process Chapters
        let (chapter_texts, chapter_mentions) = self
            .process_chapters(root_path, metadata, &scanner, scanner_hash, &options)
            .await?;

        // 4. Build Graph
        build_character_graph_cached(
            metadata,
            &chapter_texts,
            &chapter_mentions,
            options.proximity_window,
            options.prune_threshold,
            None,
        )
    }

    async fn get_or_create_scanner(
        &self,
        project_id: Uuid,
        metadata: &ProjectMetadata,
    ) -> crate::errors::Result<(CharacterScanner, u64)> {
        // Calculate deterministic hash
        let mut hasher = Sha256::new();
        // Sort to ensure order independence if chars were reordered but not changed?
        // No, scanner depends on list order index usually, unless scanner is ID-based.
        // Assuming metadata.characters order matters for the scanner internal logic
        // (if it builds regexes based on sequence).
        // Let's just hash them in order.
        for c in &metadata.characters {
            hasher.update(c.id.as_bytes());
            hasher.update(c.name.as_bytes());
            for alias in &c.aliases {
                hasher.update(alias.as_bytes());
            }
            // also hash role as it changes weighting? No, scanner only cares about names.
            // But if we rename a char, we need new scanner.
        }
        let hash_result = hasher.finalize();
        // Take first 8 bytes as u64 signature
        let current_hash = u64::from_le_bytes(
            hash_result[0..8]
                .try_into()
                .expect("Sha256 output is 32 bytes, slice is 8 bytes"),
        );

        // Optimistic Read
        {
            let cache = self.scanner_cache.read().await;
            if let Some((hash, scanner)) = cache.get(&project_id) {
                if *hash == current_hash {
                    return Ok((scanner.clone(), *hash));
                }
            }
        }

        // Rebuild (Internal logic only, no lock needed yet)
        let scanner = CharacterScanner::try_new(&metadata.characters)
            .map_err(crate::errors::Error::Intelligence)?;

        // Write to cache
        let mut cache = self.scanner_cache.write().await;
        cache.insert(project_id, (current_hash, scanner.clone()));
        Ok((scanner, current_hash))
    }

    async fn process_chapters(
        &self,
        root_path: &std::path::Path,
        metadata: &ProjectMetadata,
        scanner: &CharacterScanner,
        scanner_hash: u64,
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

        // 2. Parallel Processing (Read + Scan)
        let mut join_set = tokio::task::JoinSet::new();
        for (cid, path) in tasks {
            let repo = repo.clone();
            let content_cache = self.content_cache.clone();
            let scanner = scanner.clone();
            let cid_clone = cid.clone();

            join_set.spawn(async move {
                let content = match repo.read_file(&path).await {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to read chapter {}: {}", cid_clone, e);
                        return (cid_clone, None);
                    }
                };

                if content.is_empty() {
                    return (cid_clone, Some((content, vec![])));
                }

                let mentions = Self::get_mentions_static(
                    &content_cache,
                    &cid_clone,
                    &content,
                    &scanner,
                    scanner_hash,
                )
                .await;

                (cid_clone, Some((content, mentions)))
            });
        }

        let mut chapter_texts = HashMap::new();
        let mut chapter_mentions = HashMap::new();

        // 3. Process Results
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok((cid, data_opt)) => {
                    if let Some((content, mentions)) = data_opt {
                        if !content.is_empty() {
                            chapter_texts.insert(cid.clone(), content);
                            chapter_mentions.insert(cid, mentions);
                        }
                    }
                }
                Err(e) => log::error!("Task join error: {}", e),
            }
        }

        Ok((chapter_texts, chapter_mentions))
    }

    async fn get_mentions_static(
        cache_arc: &Arc<ChapterContentCache>,
        cid: &str,
        content: &str,
        scanner: &CharacterScanner,
        scanner_hash: u64,
    ) -> Vec<(usize, String)> {
        // Deterministic content hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash_result = hasher.finalize();
        let content_hash = u64::from_le_bytes(
            hash_result[0..8]
                .try_into()
                .expect("Sha256 output is 32 bytes, slice is 8 bytes"),
        );

        // Optimistic Read
        {
            let cache = cache_arc.read().await;
            if let Some((cached_content_hash, cached_scanner_hash, matches)) = cache.get(cid) {
                if *cached_content_hash == content_hash && *cached_scanner_hash == scanner_hash {
                    return matches.clone();
                }
            }
        }

        // Scan (CPU intensive, done without lock)
        let matches = scanner.scan(content);

        // Write Cache
        let mut cache = cache_arc.write().await;
        cache.insert(
            cid.to_string(),
            (content_hash, scanner_hash, matches.clone()),
        );
        matches
    }

    // Deprecated / Unused instance method (kept but privatized/renamed if needed, or deleted)
    // We replaced it with get_mentions_static
    #[allow(dead_code)]
    async fn get_mentions_for_chapter_legacy(
        &self,
        cid: &str,
        content: &str,
        scanner: &CharacterScanner,
        scanner_hash: u64,
    ) -> Vec<(usize, String)> {
        // Deterministic content hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        // Also mix in scanner hash logic?
        // Ideally cache Key should be (CID + ContentHash + ScannerHash).
        // But here cache is keyed by CID.
        // If the scanner changes, the coordinator SHOULD invalidate the content cache or the caller
        // usually recreates the coordinator?
        // Actually `state.chapter_content_cache` is global app state.
        // We suffer a stale cache issue if Scanner changes (e.g. new char added) but Chapter content is same.
        // We must mix scanner signature into the content hash check or store it.
        // For now, let's keep it simple: WE MUST re-scan if scanner changed.

        // HOWEVER, the `Coordinator` is ephemeral per request usually?
        // No, `state` is passed in.

        let hash_result = hasher.finalize();
        let content_hash = u64::from_le_bytes(
            hash_result[0..8]
                .try_into()
                .expect("Sha256 output is 32 bytes, slice is 8 bytes"),
        );

        // We really need to verify if the scanner used for the cached mentions is compatible.
        // For this audit fix, I will assume the caller manages cache invalidation or that collision is rare enough.
        // Ideally we'd store (content_hash, scanner_hash, mentions).

        // Optimistic Read
        {
            let cache = self.content_cache.read().await;
            if let Some((cached_content_hash, cached_scanner_hash, matches)) = cache.get(cid) {
                if *cached_content_hash == content_hash && *cached_scanner_hash == scanner_hash {
                    return matches.clone();
                }
            }
        }

        // Scan (CPU intensive, done without lock)
        let matches = scanner.scan(content);

        // Write Cache
        let mut cache = self.content_cache.write().await;
        cache.insert(
            cid.to_string(),
            (content_hash, scanner_hash, matches.clone()),
        );
        matches
    }
}

pub struct AnalysisOptions {
    pub proximity_window: usize,
    pub prune_threshold: f32,
    pub chapter_filter: Option<HashSet<String>>,
}
