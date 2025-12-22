import { projectCharacters } from './state/projectState';
import { projectApi } from '../api/project';
import type { Character } from '../types';
import { useAppStatus } from './useAppStatus';

export function useCharacters() {
    const { notifyError } = useAppStatus();

    /**
     * Set the full list of characters (e.g., after loading a project)
     */
    const setCharacters = (list: Character[]) => {
        projectCharacters.value = list;
    };

    /**
     * Save or update a character
     */
    const saveCharacter = async (projectId: string, character: Character) => {
        try {
            const metadata = await projectApi.saveCharacter(projectId, character);
            // The backend returns the updated metadata, including the full list of characters
            projectCharacters.value = metadata.characters;
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
            const metadata = await projectApi.deleteCharacter(projectId, characterId);
            projectCharacters.value = metadata.characters;
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
        deleteCharacter
    };
}
