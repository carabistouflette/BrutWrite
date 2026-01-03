/**
 * Constants for the Character Graph visualization.
 * Centralized configuration for D3 force simulation and visual elements.
 */

export const GRAPH_CONSTANTS = {
  /**
   * D3 Force simulation parameters
   */
  FORCE: {
    /** Repulsion strength between nodes (negative = repel) */
    CHARGE_STRENGTH: -400,
    /** Strength of center force (0-1) */
    CENTER_STRENGTH: 0.05,
    /** Velocity decay per tick (0-1, higher = more friction) */
    VELOCITY_DECAY: 0.85,
  },

  /**
   * Node visual properties
   */
  NODE: {
    /** Minimum node radius in pixels */
    MIN_RADIUS: 10,
    /** Maximum node radius in pixels */
    MAX_RADIUS: 28,
    /** Stroke width for node circles */
    STROKE_WIDTH: 2,
  },

  /**
   * Zoom behavior constraints
   */
  ZOOM: {
    /** Minimum zoom scale */
    MIN_SCALE: 0.5,
    /** Maximum zoom scale */
    MAX_SCALE: 3,
  },

  /**
   * Animation and transition durations
   */
  ANIMATION: {
    /** Number of ticks to run when reduced motion is preferred */
    REDUCED_MOTION_TICKS: 300,
    /** Transition duration in milliseconds */
    TRANSITION_DURATION: 300,
    /** Highlight fade duration */
    HIGHLIGHT_DURATION: 200,
  },

  /**
   * Accessibility settings
   */
  ACCESSIBILITY: {
    /** Tab index for focusable nodes */
    TAB_INDEX: 0,
  },
} as const;

/**
 * Layout constants for resizable panels
 */
export const LAYOUT_CONSTANTS = {
  SIDEBAR: {
    INITIAL_WIDTH: 256,
    MIN_WIDTH: 200,
    MAX_WIDTH: 600,
  },
  RESEARCH_PANEL: {
    INITIAL_WIDTH: 384,
    MIN_WIDTH: 280,
    MAX_WIDTH: 700,
  },
} as const;

/**
 * Timing constants for debouncing and throttling
 */
export const TIMING_CONSTANTS = {
  /** Debounce time for tree rebuilds in ms */
  TREE_REBUILD_DEBOUNCE: 50,
  /** Settings save debounce in ms */
  SETTINGS_SAVE_DEBOUNCE: 1000,
  /** Minimum auto-save interval in seconds */
  MIN_AUTO_SAVE_INTERVAL: 5,
} as const;
