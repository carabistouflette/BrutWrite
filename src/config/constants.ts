export const APP_CONSTANTS = {
  // General App Info
  APP_NAME: 'BrutWrite',
  VERSION: '0.1.0',

  // UI Defaults
  UI: {
    DEFAULT_SCALE: 100,
    MIN_SCALE: 50,
    MAX_SCALE: 300,
    CYBER_GLASS: {
      MAX_BLUR: 40,
      MIN_OPACITY: 0.1,
      VAR_OPACITY: 0.5,
    },
    ANIMATION_DURATION: {
      FAST: 200,
      NORMAL: 300,
      SLOW: 500,
    },
  },

  // Editor Defaults
  EDITOR: {
    DEFAULT_FONT_SIZE: 16,
    DEFAULT_LINE_HEIGHT: 1.6,
    DEFAULT_MAX_WIDTH: 800,
    AUTO_SAVE_INTERVAL: 30, // seconds
    MIN_AUTO_SAVE_INTERVAL: 5,
  },

  // Gamification & Goals
  GOALS: {
    DEFAULT_DAILY_GOAL: 500,
    DEFAULT_PROJECT_TARGET: 50000,
    HISTORY_DAYS: 7,
  },

  // Research Panel
  RESEARCH: {
    PROTOCOL_PREFIX: 'research://',
    PDF_VIEWER_SCALE_STEP: 0.1,
    DEFAULT_PDF_SCALE: 1.0,
  },

  // Events
  EVENTS: {
    RESEARCH_UPDATE: 'research-update',
    CLOSE: 'close',
  },

  // Timeouts (ms)
  TIMEOUTS: {
    DEBOUNCE_DEFAULT: 300,
    LONG_PRESS: 500,
  },

  // User Facing Strings (Localization Base)
  STRINGS: {
    PROMPTS: {
      NEW_NOTE_TITLE: 'New Research Note',
      NEW_NOTE_MESSAGE: 'Note Name:',
      NEW_NOTE_PLACEHOLDER: 'e.g. Character Profile',
      RENAME_ARTIFACT_TITLE: 'Rename Artifact',
      RENAME_ARTIFACT_MESSAGE: 'New Name:',
      DELETE_ARTIFACT_TITLE: 'Delete Artifact',
      DELETE_ARTIFACT_MESSAGE: (name: string) =>
        `Are you sure you want to delete "${name}"? This cannot be undone.`,
      UNSAVED_CHANGES_TITLE: 'Unsaved Changes',
      UNSAVED_CHANGES_MESSAGE: 'You have unsaved changes. Close anyway?',
      SORT_CONFIRM_TITLE: 'Apply Chronological Sort',
      SORT_CONFIRM_MESSAGE:
        'This will reorder your manuscript chapters based on their chronological time. This cannot be undone easily. Continue?',
      DELETE_CHAR_TITLE: 'Delete Character',
      DELETE_CHAR_MESSAGE: (name: string) =>
        `Are you sure you want to delete ${name}? This action cannot be undone.`,
    },
    PLACEHOLDERS: {
      CHAPTER_TITLE: 'Chapter Title',
      SEARCH: 'Filter sources or #tags...',
    },
  },
} as const;
