import { ref, shallowRef, watch } from 'vue';
import type { FileNode, ProjectSettings, Character, Plotline } from '../../types';

// Singleton state
export const projectData = ref<FileNode[]>([]);
export const activeId = ref<string | undefined>(undefined);
export const projectId = ref<string | undefined>(undefined); // Store active project UUID
export const projectSettings = ref<ProjectSettings | null>(null);
export const projectPlotlines = ref<Plotline[]>([]);
export const projectCharacters = ref<Character[]>([]);

// Optimized lookups
export const nodeMap = shallowRef(new Map<string, FileNode>());
export const flatNodes = shallowRef<FileNode[]>([]);

// Shared debouncers and pending updates (Singleton)
export const pendingMetadataUpdates = new Map<string, any>();
export const syncManifestTimeout = ref<ReturnType<typeof setTimeout> | null>(null);
export const metadataTimeout = ref<ReturnType<typeof setTimeout> | null>(null);

// Internal helper for rebuilding maps
const rebuildMap = (nodes: FileNode[]) => {
    const map = new Map<string, FileNode>();
    const list: FileNode[] = [];
    const traverseNodes = (nodes: FileNode[]) => {
        for (const node of nodes) {
            map.set(node.id, node);
            list.push(node);
            if (node.children) traverseNodes(node.children);
        }
    };
    traverseNodes(nodes);
    nodeMap.value = map;
    flatNodes.value = list;
};

// Global watcher to keep lookups in sync
watch(projectData, (newVal) => {
    rebuildMap(newVal);
}, { immediate: true, deep: false }); // deep: false because we only care about structural changes (replacement of projectData)
