//! Character Graph Intelligence Module
//!
//! Provides semantic analysis of character interactions within a manuscript,
//! building a graph-theory based model of narrative relationships.

use crate::AppState;
use tauri::State;
use uuid::Uuid;

use crate::intelligence::models::CharacterGraphPayload;

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

    // Critical: Clone metadata and release DB lock immediately to avoid blocking UI
    let metadata = {
        let guard = metadata_arc.lock().await;
        guard.clone()
    };

    // Default configuration constants
    const DEFAULT_PROXIMITY_WINDOW: usize = 50;
    const DEFAULT_PRUNE_THRESHOLD: f32 = 0.05;

    // Use provided values or defaults
    let proximity_window = proximity_window.unwrap_or(DEFAULT_PROXIMITY_WINDOW);
    let prune_threshold = prune_threshold.unwrap_or(DEFAULT_PRUNE_THRESHOLD);

    // Create filter set if chapter IDs provided
    let chapter_filter: Option<std::collections::HashSet<String>> =
        chapter_ids.map(|ids| ids.into_iter().collect());

    let options = crate::intelligence::coordinator::AnalysisOptions {
        proximity_window,
        prune_threshold,
        chapter_filter,
    };

    state
        .intelligence
        .analyze_project(project_id, &root_path, &metadata, options)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::graph::build_character_graph_cached;
    use crate::intelligence::scanner::CharacterScanner;
    use crate::models::{Character, CharacterRole, Manifest, ProjectMetadata};
    use std::collections::HashMap;

    use crate::models::utils::WordIndexer;

    /// Helper to bridge old test format to new cached format
    async fn helper_build_graph(
        metadata: &ProjectMetadata,
        chapters: &[(String, String)],
        proximity: usize,
        prune: f32,
        scanner: Option<&CharacterScanner>,
    ) -> CharacterGraphPayload {
        let mut chapter_mentions: HashMap<String, std::sync::Arc<Vec<(usize, usize, uuid::Uuid)>>> =
            HashMap::new();

        if let Some(s) = scanner {
            for (id, content) in chapters {
                let raw_mentions = s.scan(content);
                let indexer = WordIndexer::new(content);

                let mentions: Vec<(usize, usize, uuid::Uuid)> = raw_mentions
                    .into_iter()
                    .map(|(char_idx, uuid)| (char_idx, indexer.get_word_index(char_idx), uuid))
                    .collect();

                chapter_mentions.insert(id.clone(), std::sync::Arc::new(mentions));
            }
        }

        build_character_graph_cached(metadata, &chapter_mentions, proximity, prune, None)
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
            aliases: vec![],
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
