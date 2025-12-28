/**
 * Intelligence API
 *
 * Tauri command wrappers for character graph analysis.
 */

import { invoke } from '@tauri-apps/api/core';
import type { CharacterGraphPayload } from '../types/intelligence';

export const intelligenceApi = {
  /**
   * Analyze character interactions and build a weighted graph.
   *
   * @param projectId - UUID of the project to analyze.
   * @returns Promise resolving to the character graph payload.
   */
  getCharacterGraph: (projectId: string): Promise<CharacterGraphPayload> =>
    invoke('analyze_character_graph', { projectId }),
};
