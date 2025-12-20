import { ref, computed, triggerRef } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, ProjectSettings } from '../types';
import { useCharacters } from './useCharacters';
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
const projectSettings = ref<ProjectSettings | null>(null);

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

            // Sync characters to their dedicated store
            const { setCharacters } = useCharacters();
            setCharacters(metadata.characters);

            // Set settings
            projectSettings.value = metadata.settings;

            localStorage.setItem('last_opened_project_path', path);

            projectData.value = reconstructHierarchy(metadata.manifest.chapters);

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

            // Reset character store
            const { setCharacters } = useCharacters();
            setCharacters([]);

            projectSettings.value = metadata.settings;

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
        if (!projectId.value) return;

        // Helper to collect all filenames in a subtree
        const collectFilenames = (node: FileNode, acc: string[]) => {
            if (node.filename) acc.push(node.filename);
            if (node.children) {
                node.children.forEach(child => collectFilenames(child, acc));
            }
        };

        const node = findNode(projectData.value, id);
        if (node) {
            const filesToDelete: string[] = [];
            collectFilenames(node, filesToDelete);

            // Delete files from backend
            for (const filename of filesToDelete) {
                try {
                    await projectApi.deleteChapter(projectId.value, filename);
                } catch (e) {
                    console.error(`Failed to delete file ${filename}:`, e);
                }
            }
        }

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

    const totalWords = computed(() => {
        const countWords = (nodes: FileNode[]): number => {
            return nodes.reduce((sum, node) => {
                const childSum = node.children ? countWords(node.children) : 0;
                return sum + (node.word_count || 0) + childSum;
            }, 0);
        };
        return countWords(projectData.value);
    });

    const updateStructure = async (newStructure: FileNode[]) => {
        projectData.value = newStructure;
        await syncManifest();
    };

    const updateNodeStats = (id: string, wordCount: number) => {
        const node = findNode(projectData.value, id);
        if (node) {
            node.word_count = wordCount;
            // Trigger reactivity without full array replacement if possible, 
            // but for deep partial updates to refs, standard Vue reactivity handles it
            // if we mutate the object directly.
            triggerRef(projectData);
        }
    };

    const updateContextSettings = async (settings: ProjectSettings) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.updateSettings(projectId.value, settings);
            projectSettings.value = metadata.settings;
        } catch (e) {
            console.error('Failed to update project settings:', e);
        }
    };

    return {
        projectData,
        activeId,
        activeChapter,
        projectId,
        settings: computed(() => projectSettings.value),
        totalWords,
        loadProject,
        createProject,
        selectNode,
        addChapter,
        addSection,
        deleteNode,
        renameNode,
        updateStructure,
        updateSettings: updateContextSettings,
        updateNodeStats
    };
}
