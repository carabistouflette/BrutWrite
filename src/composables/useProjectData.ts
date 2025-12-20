import { ref, computed, triggerRef } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, Character } from '../types';
import {
    findNode,
    deleteFromList,
    findAndAdd,
    findAndRename,
    reconstructHierarchy,
    projectToManifest
} from '../utils/tree';

// Singleton state
const projectData = ref<FileNode[]>([]);
const activeId = ref<string | undefined>(undefined);
const projectId = ref<string | undefined>(undefined); // Store active project UUID
const characters = ref<Character[]>([]);

export function useProjectData() {

    // --- Backend Sync Helpers ---

    const syncManifest = async () => {
        if (!projectId.value) return;

        const manifest = projectToManifest(projectData.value);

        try {
            await projectApi.updateManifest(projectId.value, manifest);
            console.debug('Manifest synced');
        } catch (e) {
            console.error('Failed to sync manifest:', e);
        }
    };

    // --- Actions ---

    const loadProject = async (path: string) => {
        try {
            const metadata = await projectApi.load(path);
            projectId.value = metadata.id;

            localStorage.setItem('last_opened_project_path', path);

            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
            characters.value = metadata.characters;

            if (projectData.value.length > 0) {
                activeId.value = projectData.value[0].id;
            }
        } catch (e) {
            console.error('Failed to load project:', e);
            localStorage.removeItem('last_opened_project_path');
        }
    };

    const createProject = async (path: string, name: string, author: string) => {
        try {
            const metadata = await projectApi.create(path, name, author);
            projectId.value = metadata.id;
            projectData.value = [];
            characters.value = [];

            await addChapter();
        } catch (e) {
            console.error('Failed to create project:', e);
        }
    }

    const selectNode = (id: string) => {
        activeId.value = id;
    };

    const addChapter = async () => {
        const newId = `chapter-${Date.now()}`;
        const newNode: FileNode = {
            id: newId,
            name: 'New Chapter',
            filename: `${newId}.md`,
            children: []
        };

        projectData.value.push(newNode);
        activeId.value = newId;

        await syncManifest();
        return newId;
    };

    const addSection = async (parentId: string) => {
        const newId = `sec-${Date.now()}`;
        const newNode: FileNode = {
            id: newId,
            name: 'New Section',
            filename: `${newId}.md`,
            children: []
        };

        if (findAndAdd(projectData.value, parentId, newNode)) {
            await syncManifest();
        }
    };

    const deleteNode = async (id: string) => {
        if (deleteFromList(projectData.value, id)) {
            if (activeId.value === id) activeId.value = undefined;
            await syncManifest();
        }
    };

    const renameNode = async (id: string, newName: string) => {
        if (findAndRename(projectData.value, id, newName)) {
            // Use triggerRef to force a reactivity update on children components 
            // (like the Sidebar/FileTree) without replacing the underlying array reference.
            triggerRef(projectData);
            await syncManifest();
        }
    };

    const activeChapter = computed(() => {
        if (!activeId.value) return undefined;
        return findNode(projectData.value, activeId.value);
    });

    const saveCharacter = async (character: Character) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.saveCharacter(projectId.value, character);
            characters.value = metadata.characters;
        } catch (e) {
            console.error('Failed to save character:', e);
            throw e;
        }
    };

    const deleteCharacter = async (characterId: string) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.deleteCharacter(projectId.value, characterId);
            characters.value = metadata.characters;
        } catch (e) {
            console.error('Failed to delete character:', e);
            throw e;
        }
    };

    const updateStructure = async (newStructure: FileNode[]) => {
        projectData.value = newStructure;
        await syncManifest();
    };

    return {
        projectData,
        activeId,
        activeChapter,
        projectId,
        characters,
        loadProject,
        createProject,
        selectNode,
        addChapter,
        addSection,
        deleteNode,
        renameNode,
        saveCharacter,
        deleteCharacter,
        updateStructure
    };
}
