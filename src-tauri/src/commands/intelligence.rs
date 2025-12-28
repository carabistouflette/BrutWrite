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
fn find_character_mentions(text: &str, character: &Character) -> Vec<usize> {
    let text_lower = text.to_lowercase();
    let name_lower = character.name.to_lowercase();
    let tag = format!("@{}", name_lower);

    let mut positions = Vec::new();

    // Find name mentions
    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&name_lower) {
        let actual_pos = start + pos;
        positions.push(actual_pos);
        start = actual_pos + name_lower.len();
    }

    // Find @tag mentions
    start = 0;
    while let Some(pos) = text_lower[start..].find(&tag) {
        let actual_pos = start + pos;
        positions.push(actual_pos);
        start = actual_pos + tag.len();
    }

    positions.sort();
    positions.dedup();
    positions
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

    // Step 1: Count mentions per character and find positions
    let mut mention_counts: HashMap<String, u32> = HashMap::new();
    let mut chapter_presences: HashMap<String, HashSet<String>> = HashMap::new(); // char_id -> set of chapter_ids
    let mut char_mentions_by_chapter: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new(); // chapter_id -> char_id -> positions
    let mut first_mentions: HashMap<String, MentionLocation> = HashMap::new(); // char_id -> first mention location

    for (chapter_id, content) in chapter_contents {
        let mut chapter_mentions: HashMap<String, Vec<usize>> = HashMap::new();

        for character in characters {
            let char_id = character.id.to_string();
            let positions = find_character_mentions(content, character);

            if !positions.is_empty() {
                *mention_counts.entry(char_id.clone()).or_default() += positions.len() as u32;

                chapter_presences
                    .entry(char_id.clone())
                    .or_default()
                    .insert(chapter_id.clone());

                // Track first mention
                if !first_mentions.contains_key(&char_id) {
                    first_mentions.insert(
                        char_id.clone(),
                        MentionLocation {
                            chapter_id: chapter_id.clone(),
                            char_offset: positions[0],
                        },
                    );
                }

                chapter_mentions.insert(char_id, positions);
            }
        }

        char_mentions_by_chapter.insert(chapter_id.clone(), chapter_mentions);
    }

    // Step 2: Build nodes
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

    // Step 3: Build interaction matrix
    let n = characters.len();
    let mut interaction_weights: HashMap<(String, String), f32> = HashMap::new();
    let mut interaction_types: HashMap<(String, String), InteractionType> = HashMap::new();

    // Co-presence: characters appearing in same chapter
    for (chapter_id, content) in chapter_contents {
        let mentions = char_mentions_by_chapter.get(chapter_id);
        if mentions.is_none() {
            continue;
        }
        let mentions = mentions.unwrap();

        let present_chars: Vec<&String> = mentions.keys().collect();

        // Co-presence score
        for i in 0..present_chars.len() {
            for j in (i + 1)..present_chars.len() {
                let (a, b) = if present_chars[i] < present_chars[j] {
                    (present_chars[i].clone(), present_chars[j].clone())
                } else {
                    (present_chars[j].clone(), present_chars[i].clone())
                };

                *interaction_weights
                    .entry((a.clone(), b.clone()))
                    .or_default() += 1.0;
                interaction_types
                    .entry((a, b))
                    .or_insert(InteractionType::CoPresence);
            }
        }

        // Proximity bonus within same chapter
        for i in 0..present_chars.len() {
            for j in (i + 1)..present_chars.len() {
                let pos_a = &mentions[present_chars[i]];
                let pos_b = &mentions[present_chars[j]];

                let (a, b) = if present_chars[i] < present_chars[j] {
                    (present_chars[i].clone(), present_chars[j].clone())
                } else {
                    (present_chars[j].clone(), present_chars[i].clone())
                };

                // Calculate proximity bonus for closest mentions
                for &pa in pos_a {
                    for &pb in pos_b {
                        let word_a = char_pos_to_word_index(content, pa);
                        let word_b = char_pos_to_word_index(content, pb);
                        let distance = (word_a as isize - word_b as isize).unsigned_abs();

                        let bonus = proximity_bonus(distance, proximity_window);
                        if bonus > 0.0 {
                            *interaction_weights
                                .entry((a.clone(), b.clone()))
                                .or_default() += bonus;
                        }
                    }
                }
            }
        }
    }

    // Step 4: Build edges (prune below threshold)
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

    // Step 5: Compute metrics
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
) -> crate::errors::Result<CharacterGraphPayload> {
    let (root_path, metadata_arc) = state.projects.get_context(project_id).await?;
    let metadata = metadata_arc.lock().await;

    // Use provided values or defaults
    let proximity_window = proximity_window.unwrap_or(50);
    let prune_threshold = prune_threshold.unwrap_or(0.05);

    // Load all chapter contents
    let repo = LocalFileRepository;
    let mut chapter_contents: Vec<(String, String)> = Vec::new();

    for chapter in &metadata.manifest.chapters {
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
            id: Uuid::parse_str(id).unwrap(),
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
