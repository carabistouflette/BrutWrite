//! Character Graph Intelligence Module
//!
//! Provides semantic analysis of character interactions within a manuscript,
//! building a graph-theory based model of narrative relationships.

use crate::models::{Character, CharacterRole, ProjectMetadata};
use crate::storage::{self, LocalFileRepository};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;
use uuid::Uuid;

// =============================================================================
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
fn role_weight(role: &CharacterRole) -> f32 {
    match role {
        CharacterRole::Protagonist => 4.0,
        CharacterRole::Antagonist => 3.0,
        CharacterRole::Secondary => 2.0,
        CharacterRole::Extra => 1.0,
    }
}

/// Find all positions where a character name or @tag appears in text.
/// Pre-compiled search pattern for a character.
struct CharacterPattern {
    id: String,
    // We use a regex to ensure word boundaries or specific formatting
    pattern: regex::Regex,
}

impl CharacterPattern {
    fn new(c: &Character) -> Self {
        let name_lower = regex::escape(&c.name.to_lowercase());
        let id_ref = regex::escape(&c.id.to_string());

        // Match:
        // 1. Name (case insensitive via flag in search)
        // 2. @Name
        // 3. data-id="UUID"
        // 4. data-entity-id="UUID"
        let raw = format!(
            r"(?i)\b{}\b|@{}|data-(?:entity-)?id=[\x22\x27]{}[\x22\x27]",
            name_lower, name_lower, id_ref
        );

        Self {
            id: c.id.to_string(),
            // Safe fallback: ^$ matches nothing, which is correct behavior for invalid patterns
            pattern: regex::Regex::new(&raw).unwrap_or_else(|_| {
                // This static pattern is guaranteed to compile
                regex::Regex::new(r"^\b$").expect("static regex must compile")
            }),
        }
    }
}

/// Find all mentions of a specific character using Regex.
fn find_character_mentions(text: &str, pattern: &regex::Regex) -> Vec<usize> {
    pattern.find_iter(text).map(|m| m.start()).collect()
}

/// Convert character position to approximate word index.
fn char_pos_to_word_index(text: &str, char_pos: usize) -> usize {
    text[..char_pos.min(text.len())].split_whitespace().count()
}

/// Calculate proximity bonus between two mention positions.
fn proximity_bonus(word_distance: usize, proximity_window: usize) -> f32 {
    if word_distance == 0 || word_distance > proximity_window {
        0.0
    } else {
        0.1 * (proximity_window as f32 / word_distance as f32)
    }
}

/// Union-Find data structure for connected components.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let px = self.find(x);
        let py = self.find(y);
        if px == py {
            return;
        }
        if self.rank[px] < self.rank[py] {
            self.parent[px] = py;
        } else if self.rank[px] > self.rank[py] {
            self.parent[py] = px;
        } else {
            self.parent[py] = px;
            self.rank[px] += 1;
        }
    }

    fn component_sizes(&mut self) -> Vec<u32> {
        let n = self.parent.len();
        let mut sizes: HashMap<usize, u32> = HashMap::new();
        for i in 0..n {
            let root = self.find(i);
            *sizes.entry(root).or_default() += 1;
        }
        sizes.into_values().collect()
    }
}

// =============================================================================
// Core Analysis
// =============================================================================

/// Analyze character interactions and build the graph.
async fn build_character_graph(
    metadata: &ProjectMetadata,
    chapter_contents: &[(String, String)], // (chapter_id, content)
    proximity_window: usize,
    prune_threshold: f32,
) -> CharacterGraphPayload {
    let characters = &metadata.characters;

    if characters.is_empty() {
        return CharacterGraphPayload {
            nodes: vec![],
            edges: vec![],
            metrics: GraphMetrics {
                network_density: 0.0,
                connected_components: 0,
                largest_component_size: 0,
                isolation_ratio: 0.0,
            },
        };
    }

    // Maps char_id -> Total references across project
    let mut mention_counts: HashMap<String, u32> = HashMap::new();
    // Maps char_id -> Set(chapter_id)
    let mut chapter_presences: HashMap<String, HashSet<String>> = HashMap::new();
    let mut first_mentions: HashMap<String, MentionLocation> = HashMap::new();

    // Accumulate interaction weights: (id_a, id_b) -> weight
    // Key must be sorted (a < b) to avoid duplicates
    let mut interaction_weights: HashMap<(String, String), f32> = HashMap::new();
    let mut interaction_types: HashMap<(String, String), InteractionType> = HashMap::new();

    // Pre-compile patterns O(N)
    let patterns: Vec<CharacterPattern> = characters.iter().map(CharacterPattern::new).collect();

    // Iterate chapters to process text
    for (chapter_id, content) in chapter_contents {
        // 1. Collect all mentions in this chapter
        //    Format: (char_offset, word_index, char_id)
        let mut chapter_mentions: Vec<(usize, usize, String)> = Vec::new();
        let mut present_in_chapter: HashSet<String> = HashSet::new();

        for pattern in &patterns {
            let char_id = pattern.id.clone();
            let positions = find_character_mentions(content, &pattern.pattern);

            if !positions.is_empty() {
                *mention_counts.entry(char_id.clone()).or_default() += positions.len() as u32;
                chapter_presences
                    .entry(char_id.clone())
                    .or_default()
                    .insert(chapter_id.clone());
                present_in_chapter.insert(char_id.clone());

                if !first_mentions.contains_key(&char_id) {
                    first_mentions.insert(
                        char_id.clone(),
                        MentionLocation {
                            chapter_id: chapter_id.clone(),
                            char_offset: positions[0],
                        },
                    );
                }

                // Add to flat list for sliding window
                for &pos in &positions {
                    let word_idx = char_pos_to_word_index(content, pos);
                    chapter_mentions.push((pos, word_idx, char_id.clone()));
                }
            }
        }

        // 2. Co-Presence Score (Base load)
        //    All characters present in this chapter get +1.0 with each other
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

        // 3. Proximity Bonus (Sliding Window / Look-ahead)
        //    Sort mentions by word index to allow linear scanning
        chapter_mentions.sort_by_key(|k| k.1);

        for i in 0..chapter_mentions.len() {
            let (_, word_idx_a, ref id_a) = chapter_mentions[i];

            // Look ahead until distance > proximity_window
            for (_, word_idx_b, ref id_b) in chapter_mentions.iter().skip(i + 1) {
                let dist = word_idx_b.saturating_sub(word_idx_a);
                if dist > proximity_window {
                    break;
                }

                // If different characters, add proximity bonus
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

    // Step 4: Build nodes
    // Safe handling: iterate original chars list
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

    // Step 5: Build edges (prune)
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

    // Step 6: Compute metrics
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

    // Isolated nodes = characters with 0 edges
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

    // Network density: |E| / (|V| × (|V|-1) / 2)
    let max_edges = if n > 1 { n * (n - 1) / 2 } else { 1 };
    let network_density = edges.len() as f32 / max_edges as f32;

    CharacterGraphPayload {
        nodes,
        edges,
        metrics: GraphMetrics {
            network_density,
            connected_components,
            largest_component_size,
            isolation_ratio,
        },
    }
}

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

    // Load chapter contents (filtered if specified)
    let repo = LocalFileRepository;
    let mut chapter_contents: Vec<(String, String)> = Vec::new();

    for chapter in &metadata.manifest.chapters {
        // Skip if filter is active and chapter not in filter
        if let Some(ref filter) = chapter_filter {
            if !filter.contains(chapter.id.as_str()) {
                continue;
            }
        }

        match storage::read_chapter_content(&repo, &root_path, &metadata, &chapter.id).await {
            Ok(content) => {
                chapter_contents.push((chapter.id.clone(), content));
            }
            Err(_) => {
                // Skip chapters that can't be read
                continue;
            }
        }
    }

    let payload = build_character_graph(
        &metadata,
        &chapter_contents,
        proximity_window,
        prune_threshold,
    )
    .await;

    Ok(payload)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Manifest;

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
        let payload = build_character_graph(&metadata, &[], 50, 0.05).await;

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
        let metadata = make_test_metadata(characters);
        let payload = build_character_graph(&metadata, &[], 50, 0.05).await;

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
        let metadata = make_test_metadata(characters);

        let chapters = vec![(
            "ch1".to_string(),
            "Alice walked into the room. Bob was already there.".to_string(),
        )];

        let payload = build_character_graph(&metadata, &chapters, 50, 0.05).await;

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
        let metadata = make_test_metadata(characters);

        let chapters = vec![
            (
                "ch1".to_string(),
                "Hero does something. Hero again.".to_string(),
            ),
            ("ch2".to_string(), "Extra appears once.".to_string()),
        ];

        let payload = build_character_graph(&metadata, &chapters, 50, 0.05).await;

        let hero = payload.nodes.iter().find(|n| n.label == "Hero").unwrap();
        let extra = payload.nodes.iter().find(|n| n.label == "Extra").unwrap();

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
