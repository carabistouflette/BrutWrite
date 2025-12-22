import { ref, computed, watch } from 'vue';
import type { FileNode } from '../types';
import { 
    projectData, 
    activeId, 
    projectId, 
    projectSettings, 
    projectPlotlines 
} from './state/projectState';
import { useProjectIO } from './logic/useProjectIO';
import { useProjectNodeOperations } from './logic/useProjectNodeOperations';

export function useProjectData() {
    
    // --- Optimized Lookups ---
    const _nodeMap = ref(new Map<string, FileNode>());
    const _flatNodes = ref<FileNode[]>([]);

    const rebuildMap = () => {
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
        _nodeMap.value = map;
        _flatNodes.value = list;
    };

    // Only rebuild on structural changes (when projectData.value is replaced)
    watch(projectData, rebuildMap, { immediate: true });

    const flatNodes = computed(() => _flatNodes.value);

    // Initialize logic composables
    const { 
        loadProject, 
        createProject, 
        closeProject, 
        updateSettings, 
        updatePlotlines 
    } = useProjectIO();

    const { 
        addChapter, 
        addSection, 
        deleteNode, 
        renameNode, 
        updateStructure, 
        updateNodeStats, 
        updateNodeTemporal 
    } = useProjectNodeOperations();

    const activeChapter = computed(() => {
        if (!activeId.value) return undefined;
        return _nodeMap.value.get(activeId.value);
    });

    const totalWords = computed(() => {
        let total = 0;
        _flatNodes.value.forEach(node => {
            total += (node.word_count || 0);
        });
        return total;
    });

    const selectNode = (id: string) => {
        activeId.value = id;
    };

    // Wrapper functions to pass nodeMap where needed
    const wrappedDeleteNode = (id: string) => deleteNode(id, _nodeMap.value);
    const wrappedRenameNode = (id: string, newName: string) => renameNode(id, newName, _nodeMap.value);
    const wrappedUpdateNodeStats = (id: string, wordCount: number) => updateNodeStats(id, wordCount, _nodeMap.value);
    const wrappedUpdateNodeTemporal = (id: string, updates: Partial<FileNode>) => updateNodeTemporal(id, updates, _nodeMap.value);

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
        deleteNode: wrappedDeleteNode,
        renameNode: wrappedRenameNode,
        updateStructure,
        updateSettings,
        updatePlotlines,
        updateNodeStats: wrappedUpdateNodeStats,
        updateNodeTemporal: wrappedUpdateNodeTemporal,
        flatNodes,
        plotlines: computed(() => projectPlotlines.value),
        closeProject
    };
}
