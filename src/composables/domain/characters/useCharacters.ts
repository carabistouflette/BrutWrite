import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { charactersApi } from '../../../api/characters';
import type { Character } from '../../../types';
import { useAppStatus } from '../../ui/useAppStatus';

export function useCharacters() {
  const { notifyError } = useAppStatus();
  const projectStore = useProjectStore();
  const { characters: projectCharacters } = storeToRefs(projectStore);

  /**
   * Set the full list of characters (e.g., after loading a project)
   */
  const setCharacters = (list: Character[]) => {
    projectStore.setCharacters(list);
  };

  /**
   * Save or update a character
   */
  const saveCharacter = async (projectId: string, character: Character) => {
    try {
      const metadata = await charactersApi.save(projectId, character);
      // The backend returns the updated metadata.
      // Optimistically update or use backend source of truth?
      // Backend returns full list in metadata.characters
      projectStore.setCharacters(metadata.characters);
      return metadata;
    } catch (e) {
      notifyError('Failed to save character', e);
      throw e;
    }
  };

  /**
   * Delete a character by ID
   */
  const deleteCharacter = async (projectId: string, characterId: string) => {
    try {
      const metadata = await charactersApi.delete(projectId, characterId);
      projectStore.setCharacters(metadata.characters);
      return metadata;
    } catch (e) {
      notifyError('Failed to delete character', e);
      throw e;
    }
  };

  return {
    characters: projectCharacters,
    setCharacters,
    saveCharacter,
    deleteCharacter,
  };
}
