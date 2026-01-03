//! Character Graph Intelligence Module
//!
//! Provides semantic analysis of character interactions within a manuscript,
//! building a graph-theory based model of narrative relationships.

use crate::models::{Character, CharacterRole, ProjectMetadata};
use crate::storage::LocalFileRepository;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;
use uuid::Uuid;

use crate::models::utils::{UnionFind, WordIndexer};

use aho_corasick::{AhoCorasick, MatchKind};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// =============================================================================
// Constants
// =============================================================================

const MAX_NAME_LEN: usize = 64;

// Role weights for valence calculation
const WEIGHT_PROTAGONIST: f32 = 2.0;
const WEIGHT_ANTAGONIST: f32 = 1.8;
const WEIGHT_SECONDARY: f32 = 1.5;
const WEIGHT_EXTRA: f32 = 1.0;
// Types
// =============================================================================

/// Type of interaction between two characters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    /// Both characters appear in the same scene/chapter file.
    CoPresence,
    /// One character is mentioned in context of another (proximity-based).
    Reference,
}

/// Location of a character mention in the manuscript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentionLocation {
    /// Chapter ID where the mention appears.
    pub chapter_id: String,
    /// Character offset within the chapter content.
    pub char_offset: usize,
}

/// A node in the character graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    /// UUID of the character.
    pub id: String,
    /// Character's display name.
    pub label: String,
    /// Scalar importance metric: ln(1 + mention_count) × role_weight.
    pub valence: f32,
    /// Total number of mentions across all chapters.
    pub mention_count: u32,
    /// True if the character has at least 1 mention.
    pub is_mapped: bool,
    /// Location of the first mention (for click-to-jump).
    pub first_mention: Option<MentionLocation>,
}

/// An edge in the character graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    /// Source character ID.
    pub source: String,
    /// Target character ID.
    pub target: String,
    /// Aggregate interaction strength.
    pub weight: f32,
    /// Type of interaction.
    pub interaction_type: InteractionType,
}

/// Graph-level metrics for narrative health diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphMetrics {
    /// Network density: |E| / (|V| × (|V|-1) / 2).
    pub network_density: f32,
    /// Number of connected components.
    pub connected_components: u32,
    /// Size of the largest connected component.
    pub largest_component_size: u32,
    /// Ratio of isolated nodes to total nodes.
    pub isolation_ratio: f32,
}

/// Complete payload returned by the analyze command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterGraphPayload {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metrics: GraphMetrics,
}

// =============================================================================
// Internal Helpers
// =============================================================================

/// Get the role weight for valence calculation.
/// Get the role weight for valence calculation.
/// Uses a flattened curve to avoid over-biasing based on purely static roles.
fn role_weight(role: &CharacterRole) -> f32 {
    match role {
        CharacterRole::Protagonist => WEIGHT_PROTAGONIST,
        CharacterRole::Antagonist => WEIGHT_ANTAGONIST,
        CharacterRole::Secondary => WEIGHT_SECONDARY,
        CharacterRole::Extra => WEIGHT_EXTRA,
    }
}

/// Pre-compiled search patterns for all characters.
#[derive(Clone, Debug)]
pub struct CharacterScanner {
    /// Aho-Corasick automaton for O(n) multi-pattern search.
    ac: AhoCorasick,
    /// Maps pattern index from AC -> index in `ids` vector
    pattern_to_char_idx: Vec<usize>,
    /// Unique character IDs
    ids: Vec<String>,
    /// Bitmask indicating which patterns require strict word boundary checks.
    /// (Optimized: we could use a specific separate vector or encode it in pattern ID,
    /// but separate vec is clear).
    requires_boundary: Vec<bool>,
}

impl CharacterScanner {
    pub fn try_new(characters: &[Character]) -> Result<Self, String> {
        let mut patterns = Vec::new();
        let mut pattern_to_char_idx = Vec::new();
        let mut requires_boundary = Vec::new();
        let mut ids = Vec::new();

        for (i, c) in characters.iter().enumerate() {
            if c.name.len() > MAX_NAME_LEN {
                log::warn!(
                    "Character name '{}' is too long, shortening for analysis",
                    c.name
                );
            }
            let safe_name = c.name[..c.name.len().min(MAX_NAME_LEN)].to_lowercase();
            let id_str = c.id.to_string();

            // 1. Literal Name (Requires \b checks)
            patterns.push(safe_name.clone());
            pattern_to_char_idx.push(i);
            requires_boundary.push(true);

            // 2. @Mention (No left boundary check needed if @ is non-word, but we rely on AC)
            // Ideally we just want to match "@Name"
            patterns.push(format!("@{}", safe_name));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false); // explicit symbol prefix usually suffices

            // 3. data-id="..." (Exact machine match)
            // Handle both single and double quotes if needed, though usually standardizing is better.
            // We'll add both common variants to be safe.
            patterns.push(format!("data-id=\"{}\"", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            patterns.push(format!("data-id='{}'", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            // Store ID
            ids.push(id_str);
        }

        if patterns.is_empty() {
            return Err("No characters to analyze".into());
        }

        let ac = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::LeftmostLongest) // Match "Christopher" over "Chris"
            .build(&patterns)
            .map_err(|e| {
                log::error!("Failed to build Aho-Corasick automaton: {}", e);
                e.to_string()
            })?;

        Ok(Self {
            ac,
            pattern_to_char_idx,
            ids,
            requires_boundary,
        })
    }

    /// Scans text and returns mentions as (offset, char_id)
    pub fn scan(&self, text: &str) -> Vec<(usize, String)> {
        let mut mentions = Vec::new();
        // Aho-Corasick find_iter returns non-overlapping matches by default
        for mat in self.ac.find_iter(text) {
            let pattern_idx = mat.pattern().as_usize();
            let start = mat.start();
            let end = mat.end();

            // Check boundaries if required
            if self.requires_boundary[pattern_idx] && !is_word_boundary(text, start, end) {
                continue;
            }

            // Map back to ID
            let char_idx = self.pattern_to_char_idx[pattern_idx];
            if let Some(id) = self.ids.get(char_idx) {
                mentions.push((start, id.clone()));
            }
        }
        mentions
    }
}

/// Check if a match at `start..end` is surrounded by word boundaries.
/// Equivalent to regex `\bMATCH\b`.
fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    // Check left
    if start > 0 {
        if let Some(prev_char) = text[..start].chars().last() {
            if prev_char.is_alphanumeric() || prev_char == '_' {
                return false;
            }
        }
    }
    // Check right
    if end < text.len() {
        if let Some(next_char) = text[end..].chars().next() {
            if next_char.is_alphanumeric() || next_char == '_' {
                return false;
            }
        }
    }
    true
}

/// Calculate proximity bonus between two mention positions.
fn proximity_bonus(word_distance: usize, proximity_window: usize) -> f32 {
    if word_distance == 0 || word_distance > proximity_window {
        0.0
    } else {
        0.1 * (proximity_window as f32 / word_distance as f32)
    }
}

// =============================================================================
// Core Analysis
// =============================================================================

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

/// Optimized builder that uses pre-scanned mentions
/// Optimized builder that uses pre-scanned mentions
fn build_character_graph_cached(
    metadata: &ProjectMetadata,
    chapter_contents: &HashMap<String, String>,
    chapter_mentions: &HashMap<String, Vec<(usize, String)>>,
    proximity_window: usize,
    prune_threshold: f32,
) -> crate::errors::Result<CharacterGraphPayload> {
    let characters = &metadata.characters;

    if characters.is_empty() {
        return Ok(CharacterGraphPayload {
            nodes: vec![],
            edges: vec![],
            metrics: GraphMetrics {
                network_density: 0.0,
                connected_components: 0,
                largest_component_size: 0,
                isolation_ratio: 0.0,
            },
        });
    }

    let mut mention_counts: HashMap<String, u32> = HashMap::new();
    let mut first_mentions: HashMap<String, MentionLocation> = HashMap::new();
    let mut interaction_weights: HashMap<(String, String), f32> = HashMap::new();
    let mut interaction_types: HashMap<(String, String), InteractionType> = HashMap::new();

    for (chapter_id, mentions) in chapter_mentions {
        // We need content only for word indexer
        let content = chapter_contents.get(chapter_id).ok_or_else(|| {
            crate::errors::Error::Intelligence(format!(
                "Content missing for analyzed chapter: {}",
                chapter_id
            ))
        })?;

        let word_indexer = WordIndexer::new(content);

        let mut current_chapter_formatted: Vec<(usize, usize, String)> =
            Vec::with_capacity(mentions.len());
        let mut present_in_chapter: HashSet<String> = HashSet::new();

        for (pos, char_id) in mentions {
            *mention_counts.entry(char_id.clone()).or_default() += 1;
            present_in_chapter.insert(char_id.clone());

            if !first_mentions.contains_key(char_id) {
                first_mentions.insert(
                    char_id.clone(),
                    MentionLocation {
                        chapter_id: chapter_id.clone(),
                        char_offset: *pos,
                    },
                );
            }

            let word_idx = word_indexer.get_word_index(*pos);
            current_chapter_formatted.push((*pos, word_idx, char_id.clone()));
        }

        // Co-Presence
        let present_list: Vec<String> = present_in_chapter.into_iter().collect();
        for i in 0..present_list.len() {
            for j in (i + 1)..present_list.len() {
                let (a, b) = if present_list[i] < present_list[j] {
                    (present_list[i].clone(), present_list[j].clone())
                } else {
                    (present_list[j].clone(), present_list[i].clone())
                };

                *interaction_weights
                    .entry((a.clone(), b.clone()))
                    .or_default() += 1.0;
                interaction_types
                    .entry((a, b))
                    .or_insert(InteractionType::CoPresence);
            }
        }

        // Proximity
        current_chapter_formatted.sort_by_key(|k| k.1);
        for i in 0..current_chapter_formatted.len() {
            let (_, word_idx_a, ref id_a) = current_chapter_formatted[i];
            for (_, word_idx_b, ref id_b) in current_chapter_formatted.iter().skip(i + 1) {
                let dist = word_idx_b.saturating_sub(word_idx_a);
                if dist > proximity_window {
                    break;
                }
                if id_a != id_b {
                    let (a, b) = if id_a < id_b {
                        (id_a.clone(), id_b.clone())
                    } else {
                        (id_b.clone(), id_a.clone())
                    };
                    let bonus = proximity_bonus(dist, proximity_window);
                    if bonus > 0.0 {
                        *interaction_weights.entry((a, b)).or_default() += bonus;
                    }
                }
            }
        }
    }

    // Build Nodes
    let nodes: Vec<GraphNode> = characters
        .iter()
        .map(|c| {
            let char_id = c.id.to_string();
            let count = *mention_counts.get(&char_id).unwrap_or(&0);
            let valence = (1.0 + count as f32).ln() * role_weight(&c.role);
            GraphNode {
                id: char_id.clone(),
                label: c.name.clone(),
                valence,
                mention_count: count,
                is_mapped: count > 0,
                first_mention: first_mentions.get(&char_id).cloned(),
            }
        })
        .collect();

    // Build Edges
    let edges: Vec<GraphEdge> = interaction_weights
        .into_iter()
        .filter(|(_, weight)| *weight >= prune_threshold)
        .map(|((source, target), weight)| {
            let interaction_type = interaction_types
                .get(&(source.clone(), target.clone()))
                .cloned()
                .unwrap_or(InteractionType::Reference);
            GraphEdge {
                source,
                target,
                weight,
                interaction_type,
            }
        })
        .collect();

    // Metrics (UnionFind logic copied)
    let n = characters.len();
    let char_id_to_index: HashMap<String, usize> = characters
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.to_string(), i))
        .collect();

    let mut uf = UnionFind::new(n);
    for edge in &edges {
        if let (Some(&i), Some(&j)) = (
            char_id_to_index.get(&edge.source),
            char_id_to_index.get(&edge.target),
        ) {
            uf.union(i, j);
        }
    }

    let component_sizes = uf.component_sizes();
    let connected_components = component_sizes.len() as u32;
    let largest_component_size = component_sizes.iter().copied().max().unwrap_or(0);

    let connected_chars: HashSet<&String> =
        edges.iter().flat_map(|e| [&e.source, &e.target]).collect();
    let isolated_count = characters
        .iter()
        .filter(|c| !connected_chars.contains(&c.id.to_string()))
        .count();
    let isolation_ratio = if n > 0 {
        isolated_count as f32 / n as f32
    } else {
        0.0
    };

    let max_edges = if n > 1 { n * (n - 1) / 2 } else { 1 };
    let network_density = edges.len() as f32 / max_edges as f32;

    Ok(CharacterGraphPayload {
        nodes,
        edges,
        metrics: GraphMetrics {
            network_density,
            connected_components,
            largest_component_size,
            isolation_ratio,
        },
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Manifest;

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

    #[tokio::test]
    async fn test_proximity_bonus() {
        let bonus = proximity_bonus(10, 50);
        assert!(bonus > 0.0);

        let bonus_far = proximity_bonus(100, 50);
        assert_eq!(bonus_far, 0.0);

        let bonus_close = proximity_bonus(5, 50);
        assert!(bonus_close > bonus);
    }
}
