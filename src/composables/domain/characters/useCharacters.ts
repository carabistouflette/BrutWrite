import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { charactersApi } from '../../../api/characters';
import type { Character } from '../../../types';
import { useAppStatus } from '../../ui/useAppStatus';
import { useCharacterGraph } from '../../domain/intelligence/useCharacterGraph';

export function useCharacters() {
  const { notify, notifyError } = useAppStatus();
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

  /**
   * Add an alias to a character
   */
  const addAliasToCharacter = async (projectId: string, characterId: string, alias: string) => {
    const char = projectCharacters.value.find((c) => c.id === characterId);
    if (!char) throw new Error('Character not found');

    // Avoid duplicates
    const currentAliases = char.aliases || [];
    if (currentAliases.includes(alias)) return;

    const updatedChar = {
      ...char,
      aliases: [...currentAliases, alias],
    };

    const result = await saveCharacter(projectId, updatedChar);

    // Trigger graph refresh to reflect new alias
    const graphStore = useCharacterGraph();
    // We don't await this to keep UI snappy, it will update in background
    graphStore.analyze();

    notify(`Associated "${alias}" with ${char.name}`);
    return result;
  };

  /**
   * Remove an alias from a character (by alias text)
   * Finds the character that has this alias and removes it.
   */
  const removeAliasFromCharacter = async (projectId: string, alias: string) => {
    // Case-insensitive search
    const normalizedAlias = alias.toLowerCase().trim();
    const char = projectCharacters.value.find((c) =>
      (c.aliases || []).some((a) => a.toLowerCase().trim() === normalizedAlias)
    );

    if (!char) throw new Error('No character associated with this alias');

    const updatedAliases = (char.aliases || []).filter(
      (a) => a.toLowerCase().trim() !== normalizedAlias
    );

    const updatedChar = {
      ...char,
      aliases: updatedAliases,
    };

    const result = await saveCharacter(projectId, updatedChar);

    // Trigger graph refresh
    const graphStore = useCharacterGraph();
    graphStore.analyze();

    notify(`Removed association "${alias}" from ${char.name}`);
    return result;
  };

  return {
    characters: projectCharacters,
    setCharacters,
    saveCharacter,
    deleteCharacter,
    addAliasToCharacter,
    removeAliasFromCharacter,
  };
}
