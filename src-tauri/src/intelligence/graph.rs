use crate::models::utils::{UnionFind, WordIndexer};
use crate::models::{CharacterRole, ProjectMetadata};
use std::collections::{HashMap, HashSet};

use super::models::{
    CharacterGraphPayload, GraphEdge, GraphMetrics, GraphNode, InteractionType, MentionLocation,
};

// Role weights for valence calculation
const WEIGHT_PROTAGONIST: f32 = 2.0;
const WEIGHT_ANTAGONIST: f32 = 1.8;
const WEIGHT_SECONDARY: f32 = 1.5;
const WEIGHT_EXTRA: f32 = 1.0;

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

/// Calculate proximity bonus between two mention positions.
fn proximity_bonus(word_distance: usize, proximity_window: usize) -> f32 {
    if word_distance == 0 || word_distance > proximity_window {
        0.0
    } else {
        0.1 * (proximity_window as f32 / word_distance as f32)
    }
}

/// Optimized builder that uses pre-scanned mentions
pub fn build_character_graph_cached(
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
