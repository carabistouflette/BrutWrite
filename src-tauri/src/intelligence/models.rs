use serde::{Deserialize, Serialize};

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
