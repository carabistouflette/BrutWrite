/**
 * Character Graph Intelligence Types
 *
 * TypeScript definitions for the character interaction graph analysis.
 * Mirrors the Rust types in src-tauri/src/commands/intelligence.rs.
 */

/**
 * Type of interaction between two characters.
 */
export type InteractionType = 'co_presence' | 'reference';

/**
 * Location of a character mention in the manuscript.
 */
export interface MentionLocation {
  /** Chapter ID where the mention appears. */
  chapterId: string;
  /** Character offset within the chapter content. */
  charOffset: number;
}

/**
 * A node in the character graph representing a single character.
 */
export interface GraphNode {
  /** UUID of the character. */
  id: string;
  /** Character's display name. */
  label: string;
  /** Scalar importance metric: ln(1 + mentionCount) × roleWeight. */
  valence: number;
  /** Total number of mentions across all chapters. */
  mentionCount: number;
  /** True if the character has at least 1 mention. */
  isMapped: boolean;
  /** Location of the first mention (for click-to-jump). */
  firstMention: MentionLocation | null;
}

/**
 * An edge in the character graph representing an interaction link.
 */
export interface GraphEdge {
  /** Source character ID. */
  source: string;
  /** Target character ID. */
  target: string;
  /** Aggregate interaction strength. */
  weight: number;
  /** Type of interaction. */
  interactionType: InteractionType;
}

/**
 * Graph-level metrics for narrative health diagnostics.
 */
export interface GraphMetrics {
  /** Network density: |E| / (|V| × (|V|-1) / 2). */
  networkDensity: number;
  /** Number of connected components. */
  connectedComponents: number;
  /** Size of the largest connected component. */
  largestComponentSize: number;
  /** Ratio of isolated nodes to total nodes. */
  isolationRatio: number;
}

/**
 * Complete payload returned by the analyze_character_graph command.
 */
export interface CharacterGraphPayload {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metrics: GraphMetrics;
}

/**
 * Alert codes for narrative health diagnostics.
 */
export type AlertCode =
  | 'SOLIPSISM'
  | 'SATELLITE'
  | 'GHOST'
  | 'PROTAGONIST_ABSENT'
  | 'ANTAGONIST_ISOLATED'
  | 'CHARACTER_DROPOFF';

/**
 * Alert definition for the Metrics HUD.
 */
export interface GraphAlert {
  code: AlertCode;
  primaryText: string;
  tooltip: string;
}

/**
 * Pre-defined alert configurations.
 */
export const GRAPH_ALERTS: Record<AlertCode, Omit<GraphAlert, 'code'>> = {
  SOLIPSISM: {
    primaryText: 'Low Connectivity',
    tooltip: 'Over 50% of your cast has no meaningful interactions.',
  },
  SATELLITE: {
    primaryText: 'Isolated Subplots',
    tooltip: 'There are disconnected groups of characters that never interact with the main cast.',
  },
  GHOST: {
    primaryText: 'Unmapped Characters',
    tooltip: 'Characters declared but with 0 mentions.',
  },
  PROTAGONIST_ABSENT: {
    primaryText: 'Protagonist Fading',
    tooltip: 'Your protagonist has very few mentions in the manuscript.',
  },
  ANTAGONIST_ISOLATED: {
    primaryText: 'Antagonist Isolated',
    tooltip: 'Antagonist never shares a chapter with the protagonist.',
  },
  CHARACTER_DROPOFF: {
    primaryText: 'Character Fades',
    tooltip: 'A character disappears from the story after initial appearances.',
  },
};
