use crate::intelligence::graph::build_character_graph_cached;
use crate::intelligence::models::CharacterGraphPayload;
use crate::intelligence::scanner::CharacterScanner;
use crate::models::utils::WordIndexer;
use crate::models::ProjectMetadata;
use crate::storage::traits::{FileMetadata, FileRepository};
use crate::storage::{resolve_chapter_path, LocalFileRepository};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Cache Types
/// Keys:
/// - IntelligenceCache: Project ID -> (Hash of Characters, Scanner)
/// - ChapterContentCache: Chapter ID -> (Length, Modified, ScannerHash, Mentions)
///
/// Note: Hash is now storing 64 bits derived from SHA256 for efficient checking,
/// but the calculation is deterministic.
pub type IntelligenceCache = RwLock<HashMap<Uuid, (u64, Arc<CharacterScanner>)>>;
pub type ChapterContentCache = RwLock<HashMap<String, (u64, u64, u64, Vec<(usize, usize, Uuid)>)>>;

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
        let (scanner_arc, scanner_hash) = self.get_or_create_scanner(project_id, metadata).await?;

        // 3. Process Chapters
        let (chapter_texts, chapter_mentions) = self
            .process_chapters(root_path, metadata, scanner_arc, scanner_hash, &options)
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
    ) -> crate::errors::Result<(Arc<CharacterScanner>, u64)> {
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
        let scanner_arc = Arc::new(scanner);

        // Write to cache
        let mut cache = self.scanner_cache.write().await;
        cache.insert(project_id, (current_hash, scanner_arc.clone()));
        Ok((scanner_arc, current_hash))
    }

    async fn process_chapters(
        &self,
        root_path: &std::path::Path,
        metadata: &ProjectMetadata,
        scanner: Arc<CharacterScanner>,
        scanner_hash: u64,
        options: &AnalysisOptions,
    ) -> crate::errors::Result<(
        HashMap<String, String>,
        HashMap<String, Vec<(usize, usize, Uuid)>>,
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

        // 2. Parallel Processing (Metadata Check -> Read + Scan if needed)
        let mut join_set = tokio::task::JoinSet::new();
        for (cid, path) in tasks {
            let repo = repo.clone();
            let content_cache = self.content_cache.clone();
            let scanner = scanner.clone(); // Arc clone
            let cid_clone = cid.clone();

            join_set.spawn(async move {
                // A. Check Metadata
                let file_meta = match repo.get_metadata(&path).await {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("Failed to get metadata for chapter {}: {}", cid_clone, e);
                        return (cid_clone, None);
                    }
                };

                // B. Check Cache
                {
                    let cache = content_cache.read().await;
                    if let Some((cached_len, cached_mod, cached_scanner_hash, mentions)) =
                        cache.get(&cid_clone)
                    {
                        if *cached_len == file_meta.len
                            && *cached_mod == file_meta.modified
                            && *cached_scanner_hash == scanner_hash
                        {
                            // Cache Hit! Return mentions, but NO content (we don't need it for graph if we have word indices)
                            // Wait, graph ONLY needs mentions now.
                            // But if we want to return content map, we might need to read it?
                            // The `graph.rs` NO LONGER needs contents if we give it word indices.
                            // BUT `Coordinator` returns `HashMap<String, String>` (chapter_texts).
                            // Does `graph.rs` use it for anything else?
                            // `graph.rs` used `chapter_contents` for `WordIndexer`.
                            // If we remove that dependency, we don't need to return content to it.
                            // However, we still need to return the map if the function signature requires it.
                            // Let's modify the signature to return Option<String>?
                            // Or just empty string if not needed?
                            // For safety, let's look at `analyze_project`. It calls `build_character_graph_cached`.
                            // If `graph.rs` logic is pure math on indices, we can pass empty strings.
                            return (cid_clone, Some((None, mentions.clone())));
                        }
                    }
                }

                // C. Cache Miss - Read & Scan
                let content = match repo.read_file(&path).await {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to read chapter {}: {}", cid_clone, e);
                        return (cid_clone, None);
                    }
                };

                if content.is_empty() {
                    return (cid_clone, Some((Some(content), vec![])));
                }

                let mentions = Self::scan_and_cache(
                    &content_cache,
                    &cid_clone,
                    &content,
                    file_meta,
                    &scanner,
                    scanner_hash,
                )
                .await;

                (cid_clone, Some((Some(content), mentions)))
            });
        }

        let mut chapter_texts = HashMap::new();
        let mut chapter_mentions = HashMap::new();

        // 3. Process Results
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok((cid, data_opt)) => {
                    if let Some((content_opt, mentions)) = data_opt {
                        chapter_mentions.insert(cid.clone(), mentions);
                        // We only insert content if we actully read it?
                        // If we returned `None` content (Cache Hit), we put ""?
                        // `graph.rs` signature still takes `chapter_contents`. We will fix that next.
                        if let Some(c) = content_opt {
                            chapter_texts.insert(cid, c);
                        } else {
                            // If cache hit, we didn't read file.
                            // We can simulate it or just omit it.
                            // Omitting is safer if we change graph.rs to not require it.
                        }
                    }
                }
                Err(e) => log::error!("Task join error: {}", e),
            }
        }

        Ok((chapter_texts, chapter_mentions))
    }

    async fn scan_and_cache(
        cache_arc: &Arc<ChapterContentCache>,
        cid: &str,
        content: &str,
        meta: FileMetadata,
        scanner: &CharacterScanner,
        scanner_hash: u64,
    ) -> Vec<(usize, usize, Uuid)> {
        // Scan
        let raw_mentions = scanner.scan(content);

        // Index Words
        let indexer = WordIndexer::new(content);
        let mentions_with_words: Vec<(usize, usize, Uuid)> = raw_mentions
            .into_iter()
            .map(|(char_idx, uuid)| {
                let word_idx = indexer.get_word_index(char_idx);
                (char_idx, word_idx, uuid)
            })
            .collect();

        // Write Cache
        let mut cache = cache_arc.write().await;
        cache.insert(
            cid.to_string(),
            (
                meta.len,
                meta.modified,
                scanner_hash,
                mentions_with_words.clone(),
            ),
        );
        mentions_with_words
    }
}

pub struct AnalysisOptions {
    pub proximity_window: usize,
    pub prune_threshold: f32,
    pub chapter_filter: Option<HashSet<String>>,
}
