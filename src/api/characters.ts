import { invoke } from '@tauri-apps/api/core';
import type { ProjectMetadata, Character } from '../types';

export const charactersApi = {
  save: async (projectId: string, character: Character): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('save_character', { projectId, character });
  },

  delete: async (projectId: string, characterId: string): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('delete_character', {
      projectId,
      characterId,
    });
  },
};
