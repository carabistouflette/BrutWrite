import { ref, computed, triggerRef } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, ProjectSettings } from '../types';
import { useCharacters } from './useCharacters';
import { useAppStatus } from './useAppStatus';
import {
    findNode,
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

    const updateRecentProjects = (path: string) => {
        const recentStr = localStorage.getItem('recent_projects') || '[]';
        let recent: string[] = JSON.parse(recentStr);
        recent = [path, ...recent.filter(p => p !== path)].slice(0, 5);
        localStorage.setItem('recent_projects', JSON.stringify(recent));
    };

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
            updateRecentProjects(path);

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

            localStorage.setItem('last_opened_project_path', path);
            updateRecentProjects(path);

            await addChapter();
        } catch (e) {
            notifyError('Failed to create project', e);
        }
    }

    const selectNode = (id: string) => {
        activeId.value = id;
    };

    const addChapter = async () => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.createNode(projectId.value, undefined, 'New Chapter');
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
            
            // Find the newly created node (highest order root node)
            // Or simplified: just filtering by what we know we created, but we don't know the ID easily unless we check diff.
            // Actually, we can just find the one with the highest order or specific name if we want to auto-select.
            // For now, let's find the 'New Chapter' with highest order/index ?
            // Better: The backend returns metadata. The new node has a generated UUID.
            // We can iterate to find the node that didn't exist before? No, too expensive.
            // Let's just not auto-select for a micro-second, or...
            // Wait, failure to auto-select is fine, but user expects it.
            // Implementation detail: createNode returns metadata.
            // We can return the ID from backend?
            // Actually, let's just find the node with name "New Chapter" and highest order for now as a heuristic, 
            // OR finding the one that is not in `oldIds`.
            // But let's look at `create_node` in backend... it returns metadata.
            
            // Heuristic: Find the chapter with highest order among roots.
            // (Assuming we just added it to the end).
             const roots = projectData.value;
             if (roots.length > 0) {
                 const newChapter = roots[roots.length - 1];
                 activeId.value = newChapter.id;
                 return newChapter.id;
             }
        } catch (e) {
            notifyError('Failed to create chapter', e);
        }
    };

    const addSection = async (parentId: string) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.createNode(projectId.value, parentId, 'New Section');
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
        } catch (e) {
            notifyError('Failed to create section', e);
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

    // --- Optimized Lookups ---
    const nodeMap = computed(() => {
        const map = new Map<string, FileNode>();
        const traverseNodes = (nodes: FileNode[]) => {
            for (const node of nodes) {
                map.set(node.id, node);
                if (node.children) traverseNodes(node.children);
            }
        };
        traverseNodes(projectData.value);
        return map;
    });

    const activeChapter = computed(() => {
        if (!activeId.value) return undefined;
        return nodeMap.value.get(activeId.value);
    });

    const totalWords = computed(() => {
        let total = 0;
        nodeMap.value.forEach(node => {
            total += (node.word_count || 0);
        });
        return total;
    });

    const updateStructure = async (newStructure: FileNode[]) => {
        projectData.value = newStructure;
        await syncManifest();
    };

    const updateNodeStats = (id: string, wordCount: number) => {
        const node = nodeMap.value.get(id);
        if (node) {
            node.word_count = wordCount;
            triggerRef(projectData);
        }
    };

    const updateNodeTemporal = async (id: string, updates: Partial<FileNode>) => {
        const node = nodeMap.value.get(id);
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
        activeId.value = undefined;
        projectData.value = [];
        projectId.value = undefined;
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
