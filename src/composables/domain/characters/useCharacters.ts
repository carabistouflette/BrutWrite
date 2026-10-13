import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { useProjectSync } from '../project/useProjectSync';
import { charactersApi } from '../../../api/characters';
import type { Character, ProjectMetadata } from '../../../types';
import { useAppStatus } from '../../ui/useAppStatus';
import { useCharacterGraph } from '../../domain/intelligence/useCharacterGraph';

export function useCharacters() {
  const { notify, notifyError } = useAppStatus();
  const projectStore = useProjectStore();
  const { characters: projectCharacters } = storeToRefs(projectStore);

  const normalizeAlias = (alias: string) => alias.trim().toLowerCase();

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
      if (projectStore.projectId === projectId) {
        projectStore.setCharacters(metadata.characters);
      }
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
      if (projectStore.projectId !== projectId) return;

      const { flush } = useProjectSync();
      await flush(projectId);
      if (projectStore.projectId !== projectId) return;

      const metadata = await charactersApi.delete(projectId, characterId);
      if (projectStore.projectId === projectId) {
        projectStore.setCharacters(metadata.characters);
        for (const node of projectStore.flatNodes) {
          if (node.pov_character_id?.toLowerCase() === characterId.toLowerCase()) {
            projectStore.updateNodeMetadataAction(node.id, { pov_character_id: undefined });
          }
        }
      }
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
    if (projectStore.projectId !== projectId) throw new Error('The active project changed');

    const normalizedAlias = normalizeAlias(alias);
    if (!normalizedAlias) throw new Error('Alias cannot be empty');

    const char = projectCharacters.value.find((character) => character.id === characterId);
    if (!char) throw new Error('Character not found');

    const currentAliases = char.aliases || [];
    const hasTargetAlias = currentAliases.some(
      (currentAlias) => normalizeAlias(currentAlias) === normalizedAlias
    );
    let result: ProjectMetadata | undefined;

    if (!hasTargetAlias) {
      result = await saveCharacter(projectId, {
        ...char,
        aliases: [...currentAliases, alias.trim()],
      });
    }
    if (projectStore.projectId !== projectId) return result;

    const previousOwners = projectCharacters.value.filter(
      (character) =>
        character.id !== characterId &&
        (character.aliases || []).some(
          (currentAlias) => normalizeAlias(currentAlias) === normalizedAlias
        )
    );
    for (const previousOwner of previousOwners) {
      if (projectStore.projectId !== projectId) return result;

      result = await saveCharacter(projectId, {
        ...previousOwner,
        aliases: (previousOwner.aliases || []).filter(
          (currentAlias) => normalizeAlias(currentAlias) !== normalizedAlias
        ),
      });
    }

    if (projectStore.projectId !== projectId || !result) return result;

    const graphStore = useCharacterGraph();
    graphStore.analyze();
    notify(`Associated "${alias.trim()}" with ${char.name}`);
    return result;
  };

  /**
   * Remove an alias from a character (by alias text)
   * Finds the character that has this alias and removes it.
   */
  const removeAliasFromCharacter = async (projectId: string, alias: string) => {
    if (projectStore.projectId !== projectId) throw new Error('The active project changed');

    const normalizedAlias = normalizeAlias(alias);
    const owners = projectCharacters.value.filter((character) =>
      (character.aliases || []).some(
        (currentAlias) => normalizeAlias(currentAlias) === normalizedAlias
      )
    );

    if (owners.length === 0) throw new Error('No character associated with this alias');

    let result: ProjectMetadata | undefined;
    for (const owner of owners) {
      if (projectStore.projectId !== projectId) return result;

      result = await saveCharacter(projectId, {
        ...owner,
        aliases: (owner.aliases || []).filter(
          (currentAlias) => normalizeAlias(currentAlias) !== normalizedAlias
        ),
      });
    }
    if (projectStore.projectId !== projectId) return result;

    const graphStore = useCharacterGraph();
    graphStore.analyze();
    notify(`Removed association "${alias}"`);
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
