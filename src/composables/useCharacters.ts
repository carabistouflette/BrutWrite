import { ref } from 'vue';
import { projectApi } from '../api/project';
import type { Character } from '../types';

// Singleton state for characters (shared across components)
const characters = ref<Character[]>([]);

export function useCharacters() {

    /**
     * Set the full list of characters (e.g., after loading a project)
     */
    const setCharacters = (list: Character[]) => {
        characters.value = list;
    };

    /**
     * Save or update a character
     */
    const saveCharacter = async (projectId: string, character: Character) => {
        try {
            const metadata = await projectApi.saveCharacter(projectId, character);
            // The backend returns the updated metadata, including the full list of characters
            characters.value = metadata.characters;
            return metadata;
        } catch (e) {
            console.error('Failed to save character:', e);
            throw e;
        }
    };

    /**
     * Delete a character by ID
     */
    const deleteCharacter = async (projectId: string, characterId: string) => {
        try {
            const metadata = await projectApi.deleteCharacter(projectId, characterId);
            characters.value = metadata.characters;
            return metadata;
        } catch (e) {
            console.error('Failed to delete character:', e);
            throw e;
        }
    };

    return {
        characters,
        setCharacters,
        saveCharacter,
        deleteCharacter
    };
}
