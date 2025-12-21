import { ref, computed, triggerRef } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, ProjectSettings } from '../types';
import { useCharacters } from './useCharacters';
import { useAppStatus } from './useAppStatus';
import {
    findNode,
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
const projectPlotlines = ref<any[]>([]);

export function useProjectData() {
    const { notifyError } = useAppStatus();

    // --- Backend Sync Helpers ---

    const syncManifest = async () => {
        if (!projectId.value) return;

        const manifest = projectToManifest(projectData.value);

        try {
            await projectApi.updateManifest(projectId.value, manifest);
            console.debug('Manifest synced');
        } catch (e) {
            notifyError('Failed to sync manifest', e);
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
            projectPlotlines.value = metadata.plotlines;

            localStorage.setItem('last_opened_project_path', path);

            projectData.value = reconstructHierarchy(metadata.manifest.chapters);

            if (projectData.value.length > 0) {
                activeId.value = projectData.value[0].id;
            }
        } catch (e) {
            notifyError('Failed to load project', e);
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
            projectPlotlines.value = metadata.plotlines;

            await addChapter();
        } catch (e) {
            notifyError('Failed to create project', e);
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

        const node = findNode(projectData.value, id);
        if (!node) return;

        const collectFilenames = (n: FileNode, acc: string[]) => {
            if (n.filename) acc.push(n.filename);
            if (n.children) {
                n.children.forEach(child => collectFilenames(child, acc));
            }
        };

        const filesToDelete: string[] = [];
        collectFilenames(node, filesToDelete);

        try {
            const metadata = await projectApi.deleteNode(projectId.value, id, filesToDelete);

            // Atomic sync from backend
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
            if (activeId.value === id) activeId.value = undefined;

        } catch (e) {
            notifyError(`Failed to delete node ${id}`, e);
        }
    };

    const renameNode = async (id: string, newName: string) => {
        if (findAndRename(projectData.value, id, newName)) {
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
            triggerRef(projectData);
        }
    };

    const updateNodeTemporal = async (id: string, updates: Partial<FileNode>) => {
        const node = findNode(projectData.value, id);
        if (node) {
            // Only allow temporal updates here
            const allowed = ['chronological_date', 'abstract_timeframe', 'duration', 'plotline_tag', 'depends_on', 'pov_character_id'];
            allowed.forEach(key => {
                if (key in updates) {
                    (node as any)[key] = (updates as any)[key];
                }
            });
            triggerRef(projectData);
            await syncManifest();
        }
    };

    const updateSettings = async (settings: ProjectSettings) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.updateSettings(projectId.value, settings);
            projectSettings.value = metadata.settings;
        } catch (e) {
            notifyError('Failed to update project settings', e);
        }
    };

    const updatePlotlines = async (plotlines: any[]) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.updatePlotlines(projectId.value, plotlines);
            projectPlotlines.value = metadata.plotlines;
        } catch (e) {
            notifyError('Failed to update plotlines', e);
        }
    };

    const closeProject = () => {
        projectId.value = undefined;
        projectData.value = [];
        activeId.value = undefined;
        projectSettings.value = null;
        projectPlotlines.value = [];

        const { setCharacters } = useCharacters();
        setCharacters([]);

        localStorage.removeItem('last_opened_project_path');
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
        updateSettings,
        updatePlotlines,
        updateNodeStats,
        updateNodeTemporal,
        plotlines: computed(() => projectPlotlines.value),
        closeProject
    };
}
