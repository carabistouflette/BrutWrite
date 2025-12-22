import { computed, watch } from 'vue';
import type { FileNode } from '../types';
import { 
    projectData, 
    activeId, 
    projectId, 
    projectSettings, 
    projectPlotlines,
    nodeMap,
    flatNodes
} from './state/projectState';
import { useProjectIO } from './logic/useProjectIO';
import { useProjectNodeOperations } from './logic/useProjectNodeOperations';

export function useProjectData() {
    
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
        nodeMap.value = map;
        flatNodes.value = list;
    };

    // Only rebuild on structural changes (when projectData.value is replaced)
    watch(projectData, rebuildMap, { immediate: true });

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
        return nodeMap.value.get(activeId.value);
    });

    const totalWords = computed(() => {
        let total = 0;
        flatNodes.value.forEach(node => {
            total += (node.word_count || 0);
        });
        return total;
    });

    const selectNode = (id: string) => {
        activeId.value = id;
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
        flatNodes: computed(() => flatNodes.value),
        nodeMap: computed(() => nodeMap.value),
        plotlines: computed(() => projectPlotlines.value),
        closeProject
    };
}
