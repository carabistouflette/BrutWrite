/**
 * Centralized localStorage keys for consistent naming across the application.
 * All keys should use the 'brutwrite_' prefix for namespace isolation.
 */
export const STORAGE_KEYS = {
  /** Path to the last opened project */
  LAST_OPENED_PATH: 'brutwrite_last_opened_path',
  /** List of recently opened project paths */
  RECENT_PROJECTS: 'brutwrite_recent_projects',
  /** Gamification state (history, targets) */
  GAMIFICATION: 'brutwrite_gamification',
  /** Session cache prefix (append project path) */
  SESSION_PREFIX: 'brutwrite_session_',
} as const;

export type StorageKey = (typeof STORAGE_KEYS)[keyof typeof STORAGE_KEYS];
