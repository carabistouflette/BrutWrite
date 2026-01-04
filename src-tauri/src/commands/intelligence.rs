//! Character Graph Intelligence Module
//!
//! Provides semantic analysis of character interactions within a manuscript,
//! building a graph-theory based model of narrative relationships.

use crate::storage::LocalFileRepository;
use crate::AppState;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tauri::State;
use uuid::Uuid;

use crate::intelligence::graph::build_character_graph_cached;
use crate::intelligence::models::CharacterGraphPayload;
use crate::intelligence::scanner::CharacterScanner;

// =============================================================================
// Tauri Command
// =============================================================================

/// Analyze character graph for a project.
///
/// Scans all chapters for character mentions and co-presence,
/// building a weighted interaction graph.
#[tauri::command]
pub async fn analyze_character_graph(
    state: State<'_, AppState>,
    project_id: Uuid,
    proximity_window: Option<usize>,
    prune_threshold: Option<f32>,
    chapter_ids: Option<Vec<String>>,
) -> crate::errors::Result<CharacterGraphPayload> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let metadata = metadata_arc.lock().await;

    // Use provided values or defaults
    let proximity_window = proximity_window.unwrap_or(50);
    let prune_threshold = prune_threshold.unwrap_or(0.05);

    // Create filter set if chapter IDs provided
    let chapter_filter: Option<std::collections::HashSet<&str>> = chapter_ids
        .as_ref()
        .map(|ids| ids.iter().map(|s| s.as_str()).collect());

    // Check for empty characters before scanner logic
    if metadata.characters.is_empty() {
        return build_character_graph_cached(
            &metadata,
            &HashMap::new(),
            &HashMap::new(),
            proximity_window,
            prune_threshold,
        );
    }

    // Cache Logic (Scanner Init)
    let current_hash = {
        let mut s = DefaultHasher::new();
        for c in &metadata.characters {
            c.id.hash(&mut s);
            c.name.hash(&mut s);
        }
        s.finish()
    };

    // Initialize Scanner (once per analysis run)
    let scanner = {
        let cache = state.intelligence_cache.lock().await; // Changed to async lock
        if let Some((hash, scanner)) = cache.get(&project_id) {
            if *hash == current_hash {
                Some(scanner.clone())
            } else {
                None
            }
        } else {
            None
        }
    };

    let scanner = if let Some(s) = scanner {
        s
    } else {
        // Rebuild if stale or missing
        match CharacterScanner::try_new(&metadata.characters) {
            Ok(s) => {
                let mut cache = state.intelligence_cache.lock().await; // Changed to async lock
                cache.insert(project_id, (current_hash, s.clone()));
                s
            }
            Err(e) => return Err(crate::errors::Error::Intelligence(e)),
        }
    };

    // 4. Process chapters with concurrency
    // -------------------------------------------------------------------------
    let mut chapter_mentions_map: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    let mut chapter_texts: HashMap<String, String> = HashMap::new();

    let repo = LocalFileRepository;

    // Prepare tasks for parallel reading
    let mut tasks = Vec::new();
    for chapter in &metadata.manifest.chapters {
        if let Some(ref filter) = chapter_filter {
            if !filter.contains(chapter.id.as_str()) {
                continue;
            }
        }

        // Resolve path synchronously to avoid passing entire metadata clone to tasks
        // We log warnings instead of failing the whole batch if one file is missing
        match crate::storage::resolve_chapter_path(&root_path, &metadata, &chapter.id) {
            Ok(path) => tasks.push((chapter.id.clone(), path)),
            Err(e) => log::warn!("Skipping chapter {}: {}", chapter.id, e),
        }
    }

    // Spawn parallel reads using Tokio JoinSet
    // Robustness: Handle File I/O completely separately from logic
    let mut join_set = tokio::task::JoinSet::new();

    for (cid, path) in tasks {
        let repo = repo.clone();
        join_set.spawn(async move {
            use crate::storage::traits::FileRepository;
            // Robustness: Treat read failure as empty content rather than panic.
            // This ensures analysis continues even if one file is locked/missing transiently.
            let content = repo.read_file(&path).await.unwrap_or_default();
            (cid, content)
        });
    }

    // Process results as they arrive
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok((cid, content)) => {
                if content.is_empty() {
                    continue;
                }

                // 2. Compute Hash (CPU)
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                current_hash.hash(&mut hasher);
                let combined_hash = hasher.finish();

                // 3. Cache Check & Update (Sync - Lock)
                let mentions = {
                    let mut content_cache = state.chapter_content_cache.lock().await; // Changed to async lock

                    if let Some((cached_hash, matches)) = content_cache.get(&cid) {
                        if *cached_hash == combined_hash {
                            // HIT
                            matches.clone()
                        } else {
                            // MISS (Content changed)
                            let m = scanner.scan(&content);
                            content_cache.insert(cid.clone(), (combined_hash, m.clone()));
                            m
                        }
                    } else {
                        // MISS (New chapter)
                        let m = scanner.scan(&content);
                        content_cache.insert(cid.clone(), (combined_hash, m.clone()));
                        m
                    }
                };

                // 4. Accumulate
                chapter_mentions_map.insert(cid.clone(), mentions);
                chapter_texts.insert(cid, content);
            }
            Err(e) => log::error!("Analysis task join error: {}", e),
        }
    }

    let payload = build_character_graph_cached(
        &metadata,
        &chapter_texts,
        &chapter_mentions_map,
        proximity_window,
        prune_threshold,
    )?;

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Character, CharacterRole, Manifest, ProjectMetadata};

    /// Helper to bridge old test format to new cached format
    async fn helper_build_graph(
        metadata: &ProjectMetadata,
        chapters: &[(String, String)],
        proximity: usize,
        prune: f32,
        scanner: Option<&CharacterScanner>,
    ) -> CharacterGraphPayload {
        let mut chapter_mentions = HashMap::new();
        let mut chapter_texts = HashMap::new();

        if let Some(s) = scanner {
            for (id, content) in chapters {
                let mentions = s.scan(content);
                chapter_mentions.insert(id.clone(), mentions);
                chapter_texts.insert(id.clone(), content.clone());
            }
        }

        build_character_graph_cached(
            metadata,
            &chapter_texts,
            &chapter_mentions,
            proximity,
            prune,
        )
        .expect("Graph build failed in test")
    }

    fn make_test_character(id: &str, name: &str, role: CharacterRole) -> Character {
        Character {
            id: Uuid::parse_str(id).expect("Invalid UUID in test setup"),
            name: name.to_string(),
            role,
            archetype: String::new(),
            description: String::new(),
            engine: Default::default(),
            physical_features: String::new(),
            traits: vec![],
            arc: String::new(),
            notes: String::new(),
        }
    }

    fn make_test_metadata(characters: Vec<Character>) -> ProjectMetadata {
        ProjectMetadata {
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            author: "Author".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            settings: Default::default(),
            manifest: Manifest { chapters: vec![] },
            characters,
            plotlines: vec![],
        }
    }

    #[tokio::test]
    async fn test_empty_project() {
        let metadata = make_test_metadata(vec![]);
        let payload = helper_build_graph(&metadata, &[], 50, 0.05, None).await;

        assert!(payload.nodes.is_empty());
        assert!(payload.edges.is_empty());
        assert_eq!(payload.metrics.connected_components, 0);
    }

    #[tokio::test]
    async fn test_single_character_no_mentions() {
        let characters = vec![make_test_character(
            "00000000-0000-0000-0000-000000000001",
            "Alice",
            CharacterRole::Protagonist,
        )];
        let metadata = make_test_metadata(characters.clone());
        let scanner = CharacterScanner::try_new(&characters).expect("Scanner init failed");
        let payload = helper_build_graph(&metadata, &[], 50, 0.05, Some(&scanner)).await;

        assert_eq!(payload.nodes.len(), 1);
        assert_eq!(payload.nodes[0].mention_count, 0);
        assert!(!payload.nodes[0].is_mapped);
    }

    #[tokio::test]
    async fn test_co_presence_detection() {
        let characters = vec![
            make_test_character(
                "00000000-0000-0000-0000-000000000001",
                "Alice",
                CharacterRole::Protagonist,
            ),
            make_test_character(
                "00000000-0000-0000-0000-000000000002",
                "Bob",
                CharacterRole::Secondary,
            ),
        ];
        let metadata = make_test_metadata(characters.clone());
        let scanner = CharacterScanner::try_new(&characters).expect("Scanner init failed");

        let chapters = vec![(
            "ch1".to_string(),
            "Alice walked into the room. Bob was already there.".to_string(),
        )];

        let payload = helper_build_graph(&metadata, &chapters, 50, 0.05, Some(&scanner)).await;

        assert_eq!(payload.nodes.len(), 2);
        assert!(payload.nodes.iter().all(|n| n.is_mapped));
        assert_eq!(payload.edges.len(), 1);
        assert!(payload.edges[0].weight >= 1.0);
    }

    #[tokio::test]
    async fn test_valence_calculation() {
        let characters = vec![
            make_test_character(
                "00000000-0000-0000-0000-000000000001",
                "Hero",
                CharacterRole::Protagonist,
            ),
            make_test_character(
                "00000000-0000-0000-0000-000000000002",
                "Extra",
                CharacterRole::Extra,
            ),
        ];
        let metadata = make_test_metadata(characters.clone());
        let scanner = CharacterScanner::try_new(&characters).expect("Scanner init failed");

        let chapters = vec![
            (
                "ch1".to_string(),
                "Hero does something. Hero again.".to_string(),
            ),
            ("ch2".to_string(), "Extra appears once.".to_string()),
        ];

        let payload = helper_build_graph(&metadata, &chapters, 50, 0.05, Some(&scanner)).await;

        let hero = payload
            .nodes
            .iter()
            .find(|n| n.label == "Hero")
            .expect("Hero node missing");
        let extra = payload
            .nodes
            .iter()
            .find(|n| n.label == "Extra")
            .expect("Extra node missing");

        // Protagonist with 2 mentions should have higher valence than Extra with 1
        assert!(hero.valence > extra.valence);
    }
}
