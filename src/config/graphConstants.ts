/**
 * Constants for the D3 Character Graph Simulation.
 * These control the physics and visual behavior of the force-directed graph.
 */
export const GRAPH_CONFIG = {
  /** Repulsion force between nodes (negative = repel) */
  CHARGE_STRENGTH: -400,
  /** Base distance for links before weight adjustment */
  LINK_BASE_DISTANCE: 120,
  /** Offset to prevent division by zero in link distance */
  LINK_WEIGHT_OFFSET: 0.1,
  /** Gravity strength toward center */
  CENTER_STRENGTH: 0.05,
  /** Simulation damping factor (0-1, higher = slower) */
  VELOCITY_DECAY: 0.85,
  /** Ticks for static layout in reduced-motion mode */
  STATIC_TICKS: 300,
} as const;
