import { ref, computed } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, ProjectSettings } from '../types';
import { useCharacters } from './useCharacters';
import { useAppStatus } from './useAppStatus';
import {
    reconstructHierarchy,
    projectToManifest
} from '../utils/tree';

// Singleton state
const projectData = ref<FileNode[]>([]);
const activeId = ref<string | undefined>(undefined);
const projectId = ref<string | undefined>(undefined); // Store active project UUID
const projectSettings = ref<ProjectSettings | null>(null);
const projectPlotlines = ref<any[]>([]);

// Shared debouncers and pending updates (Singleton)
let syncManifestTimeout: ReturnType<typeof setTimeout> | null = null;
const pendingMetadataUpdates = new Map<string, any>();
let metadataTimeout: ReturnType<typeof setTimeout> | null = null;

export function useProjectData() {
    const { notifyError } = useAppStatus();

    // --- Backend Sync Helpers ---

    const syncManifestDebounced = () => {
        if (syncManifestTimeout) clearTimeout(syncManifestTimeout);
        syncManifestTimeout = setTimeout(async () => {
            if (!projectId.value) return;

            const manifest = projectToManifest(projectData.value);

            try {
                await projectApi.updateManifest(projectId.value, manifest);
                console.debug('Manifest synced');
            } catch (e) {
                notifyError('Failed to sync manifest', e);
            }
        }, 1500); // 1.5s debounce for structural changes
    };

    const syncNodeMetadataDebounced = (nodeId: string, updates: any) => {
        // Merge updates for the same node
        const current = pendingMetadataUpdates.get(nodeId) || {};
        pendingMetadataUpdates.set(nodeId, { ...current, ...updates });

        if (metadataTimeout) clearTimeout(metadataTimeout);
        metadataTimeout = setTimeout(async () => {
            if (!projectId.value) return;
            const updatesToSync = Array.from(pendingMetadataUpdates.entries());
            pendingMetadataUpdates.clear();
            metadataTimeout = null;

            for (const [id, up] of updatesToSync) {
                try {
                    await projectApi.updateNodeMetadata(projectId.value, id, up);
                } catch (e) {
                    notifyError(`Failed to sync metadata for node ${id}`, e);
                }
            }
        }, 1000);
    };

    // --- Optimized Lookups ---
    const nodeMap = computed(() => {
        const map = new Map<string, FileNode>();
        const list: FileNode[] = [];
        const traverseNodes = (nodes: FileNode[]) => {
            for (const node of nodes) {
                map.set(node.id, node);
                list.push(node);
                if (node.children) traverseNodes(node.children);
            }
        };
        traverseNodes(projectData.value);
        return { map, list };
    });

    const flatNodes = computed(() => nodeMap.value.list);

    const activeChapter = computed(() => {
        if (!activeId.value) return undefined;
        return nodeMap.value.map.get(activeId.value);
    });

    const totalWords = computed(() => {
        let total = 0;
        nodeMap.value.map.forEach(node => {
            total += (node.word_count || 0);
        });
        return total;
    });

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

        const node = nodeMap.value.map.get(id);
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
        const node = nodeMap.value.map.get(id);
        if (node && node.name !== newName) {
            node.name = newName;
            syncNodeMetadataDebounced(id, { title: newName });
        }
    };

    const updateStructure = async (newStructure: FileNode[]) => {
        projectData.value = newStructure;
        syncManifestDebounced();
    };

    const updateNodeStats = (id: string, wordCount: number) => {
        const node = nodeMap.value.map.get(id);
        if (node && node.word_count !== wordCount) {
            node.word_count = wordCount;
        }
    };

    const updateNodeTemporal = async (id: string, updates: Partial<FileNode>) => {
        const node = nodeMap.value.map.get(id);
        if (node) {
            // Only allow temporal updates here
            const allowed = ['chronological_date', 'abstract_timeframe', 'duration', 'plotline_tag', 'depends_on', 'pov_character_id'];
            let changed = false;
            const updateForBackend: any = {};
            
            allowed.forEach(key => {
                if (key in updates && (node as any)[key] !== (updates as any)[key]) {
                    (node as any)[key] = (updates as any)[key];
                    (updateForBackend as any)[key] = (updates as any)[key];
                    changed = true;
                }
            });

            if (changed) {
                syncNodeMetadataDebounced(id, updateForBackend);
            }
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
        flatNodes,
        plotlines: computed(() => projectPlotlines.value),
        closeProject
    };
}

