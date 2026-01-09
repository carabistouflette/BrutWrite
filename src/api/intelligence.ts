/**
 * Intelligence API
 *
 * Tauri command wrappers for character graph analysis.
 */

import { invoke } from '@tauri-apps/api/core';
import type { CharacterGraphPayload } from '../types/intelligence';

export interface AnalysisOptions {
  proximityWindow?: number;
  pruneThreshold?: number;
  chapterIds?: string[];
}

export const intelligenceApi = {
  /**
   * Analyze character interactions and build a weighted graph.
   *
   * @param projectId - UUID of the project to analyze.
   * @param options - Optional analysis parameters.
   * @returns Promise resolving to the character graph payload.
   */
  getCharacterGraph: (
    projectId: string,
    options?: AnalysisOptions
  ): Promise<CharacterGraphPayload> =>
    invoke('analyze_character_graph', {
      projectId,
      proximityWindow: options?.proximityWindow,
      pruneThreshold: options?.pruneThreshold,
      chapterIds: options?.chapterIds,
    }),
};
