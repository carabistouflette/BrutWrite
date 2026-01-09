use crate::models::utils::{UnionFind, WordIndexer};
use crate::models::{CharacterRole, ProjectMetadata};
use std::collections::{HashMap, HashSet};

use super::models::{
    CharacterGraphPayload, GraphEdge, GraphMetrics, GraphNode, InteractionType, MentionLocation,
};

// =============================================================================
// Configuration
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub struct GraphWeights {
    pub protagonist: f32,
    pub antagonist: f32,
    pub secondary: f32,
    pub extra: f32,
    pub base_proximity_bonus: f32,
}

impl Default for GraphWeights {
    fn default() -> Self {
        Self {
            protagonist: 2.0,
            antagonist: 1.8,
            secondary: 1.5,
            extra: 1.0,
            base_proximity_bonus: 0.1,
        }
    }
}

// =============================================================================
// Logic
// =============================================================================

/// Get the role weight for valence calculation.
fn role_weight(role: &CharacterRole, weights: &GraphWeights) -> f32 {
    match role {
        CharacterRole::Protagonist => weights.protagonist,
        CharacterRole::Antagonist => weights.antagonist,
        CharacterRole::Secondary => weights.secondary,
        CharacterRole::Extra => weights.extra,
    }
}

/// Calculate proximity bonus between two mention positions.
fn proximity_bonus(word_distance: usize, proximity_window: usize, base_bonus: f32) -> f32 {
    if word_distance == 0 || word_distance > proximity_window {
        0.0
    } else {
        base_bonus * (proximity_window as f32 / word_distance as f32)
    }
}

/// Optimized builder that uses pre-scanned mentions and integer-based indexing
pub fn build_character_graph_cached(
    metadata: &ProjectMetadata,
    chapter_contents: &HashMap<String, String>,
    chapter_mentions: &HashMap<String, Vec<(usize, uuid::Uuid)>>,
    proximity_window: usize,
    prune_threshold: f32,
    custom_weights: Option<GraphWeights>,
) -> crate::errors::Result<CharacterGraphPayload> {
    let characters = &metadata.characters;
    let n_chars = characters.len();

    if n_chars == 0 {
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

    let weights = custom_weights.unwrap_or_default();

    // 1. Map Character IDs to Integers for O(1) array access and cheap copying
    let char_id_to_idx: HashMap<uuid::Uuid, usize> = characters
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id, i))
        .collect();

    let mut mention_counts: Vec<u32> = vec![0; n_chars];
    let mut first_mentions: Vec<Option<MentionLocation>> = vec![None; n_chars];

    // Use an adjacency matrix (flattened or nested vec) for interaction accumulation
    // weight_matrix[i][j] where i < j
    let mut weight_matrix: HashMap<(usize, usize), f32> =
        HashMap::with_capacity(n_chars * n_chars / 2);
    let mut type_matrix: HashMap<(usize, usize), InteractionType> =
        HashMap::with_capacity(n_chars * n_chars / 2);

    // 2. Process Chapters
    for (chapter_id, mentions) in chapter_mentions {
        if mentions.is_empty() {
            continue;
        }

        let content = chapter_contents.get(chapter_id).ok_or_else(|| {
            crate::errors::Error::Intelligence(format!(
                "Content missing for analyzed chapter: {}",
                chapter_id
            ))
        })?;

        let word_indexer = WordIndexer::new(content);

        // Transform mentions to (word_index, char_index)
        let mut linear_mentions: Vec<(usize, usize, usize)> = Vec::with_capacity(mentions.len());

        for (char_offset, char_uuid) in mentions {
            if let Some(&idx) = char_id_to_idx.get(char_uuid) {
                mention_counts[idx] += 1;

                if first_mentions[idx].is_none() {
                    first_mentions[idx] = Some(MentionLocation {
                        chapter_id: chapter_id.clone(),
                        char_offset: *char_offset,
                    });
                }

                let word_idx = word_indexer.get_word_index(*char_offset);
                linear_mentions.push((*char_offset, word_idx, idx));
            }
        }

        // Sort by word index to enable sliding window
        linear_mentions.sort_by_key(|k| k.1);

        // 3. Sliding Window Co-Presence Algorithm (O(M * Window)) instead of O(M^2)
        // For each mention, look ahead only within proximity_window

        let len = linear_mentions.len();
        for i in 0..len {
            let (_, word_idx_a, idx_a) = linear_mentions[i];

            // Look ahead
            for (_, word_idx_b, idx_b) in linear_mentions.iter().skip(i + 1) {
                let word_idx_b = *word_idx_b;
                let idx_b = *idx_b;

                let dist = word_idx_b.saturating_sub(word_idx_a);
                if dist > proximity_window {
                    break; // Left window bound exceeded
                }

                if idx_a != idx_b {
                    let (min, max) = if idx_a < idx_b {
                        (idx_a, idx_b)
                    } else {
                        (idx_b, idx_a)
                    };

                    let bonus =
                        proximity_bonus(dist, proximity_window, weights.base_proximity_bonus);
                    *weight_matrix.entry((min, max)).or_default() += bonus;
                    // Mark as Proxy if they are close (we can refine interaction types later if needed)
                    type_matrix
                        .entry((min, max))
                        .or_insert(InteractionType::CoPresence);
                }
            }
        }
    }

    // 4. Build Nodes
    let nodes: Vec<GraphNode> = characters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let count = mention_counts[i];
            let valence = (1.0 + count as f32).ln() * role_weight(&c.role, &weights);
            GraphNode {
                id: c.id.to_string(),
                label: c.name.clone(),
                valence,
                mention_count: count,
                is_mapped: count > 0,
                first_mention: first_mentions[i].clone(),
            }
        })
        .collect();

    // 5. Build Edges & Calculate Metrics Data
    let mut uf = UnionFind::new(n_chars);
    let mut edges: Vec<GraphEdge> = Vec::with_capacity(weight_matrix.len());
    let mut connected_indices = HashSet::new();

    for ((idx_a, idx_b), weight) in weight_matrix {
        if weight < prune_threshold {
            continue;
        }

        uf.union(idx_a, idx_b);
        connected_indices.insert(idx_a);
        connected_indices.insert(idx_b);

        // Safe unwrap because indices come from range 0..n_chars
        let id_a = characters[idx_a].id.to_string();
        let id_b = characters[idx_b].id.to_string();

        let interaction_type = type_matrix
            .get(&(idx_a, idx_b))
            .cloned()
            .unwrap_or(InteractionType::Reference);

        edges.push(GraphEdge {
            source: id_a,
            target: id_b,
            weight,
            interaction_type,
        });
    }

    // 6. Metrics
    let component_sizes = uf.component_sizes();
    let connected_components = component_sizes.len() as u32;
    let largest_component_size = component_sizes.iter().copied().max().unwrap_or(0);

    let isolated_count = if n_chars > 0 {
        n_chars - connected_indices.len()
    } else {
        0
    };

    let isolation_ratio = if n_chars > 0 {
        isolated_count as f32 / n_chars as f32
    } else {
        0.0
    };

    let max_edges = if n_chars > 1 {
        n_chars * (n_chars - 1) / 2
    } else {
        1
    };
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
