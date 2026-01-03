import { defineStore } from 'pinia';
import { ref, shallowRef, computed, watch } from 'vue';
import type { FileNode, ProjectSettings, Character, Plotline } from '../types';

export const useProjectStore = defineStore('project', () => {
  // State
  const nodes = shallowRef<FileNode[]>([]);
  const activeId = ref<string | undefined>(undefined);
  const projectId = ref<string | undefined>(undefined);
  const path = ref<string | undefined>(undefined);
  const settings = ref<ProjectSettings | null>(null);
  const plotlines = ref<Plotline[]>([]);
  const characters = ref<Character[]>([]);

  // Statistics (tracked separately to avoid O(N) traversals on every word count change)
  const totalWordCount = ref(0);

  // Derived State (Optimized Lookups)
  const nodeMap = shallowRef(new Map<string, FileNode>());
  const flatNodes = shallowRef<FileNode[]>([]);

  // Tracking version to trigger reactive updates for non-structural changes if needed
  const statsVersion = ref(0);

  // Character map for O(1) access
  const characterMap = computed(() => {
    const map = new Map<string, Character>();
    for (const char of characters.value) {
      map.set(char.id, char);
    }
    return map;
  });

  const rebuildMap = (fileNodes: FileNode[]) => {
    const map = new Map<string, FileNode>();
    const list: FileNode[] = [];
    const stack: FileNode[] = [...fileNodes].reverse();
    let totalWc = 0;

    while (stack.length > 0) {
      const node = stack.pop()!;
      map.set(node.id, node);
      list.push(node);
      totalWc += node.word_count || 0;

      if (node.children && node.children.length > 0) {
        for (let i = node.children.length - 1; i >= 0; i--) {
          stack.push(node.children[i]);
        }
      }
    }
    nodeMap.value = map;
    flatNodes.value = list;
    totalWordCount.value = totalWc;
  };

  // Watcher to keep lookups in sync
  watch(nodes, (newVal) => rebuildMap(newVal ?? []), { immediate: true });

  // --- Getters ---

  const activeChapter = computed(() => {
    if (!activeId.value) return undefined;
    return nodeMap.value.get(activeId.value);
  });

  const chapterById = (id: string) => nodeMap.value.get(id);
  const characterById = (id: string) => characterMap.value.get(id);

  // --- Actions ---

  function setProjectData(
    id: string,
    projectPath: string,
    fileNodes: FileNode[],
    projectSettingsData: ProjectSettings
  ) {
    projectId.value = id;
    path.value = projectPath;
    nodes.value = fileNodes; // Triggers rebuildMap
    settings.value = projectSettingsData;
    activeId.value = undefined;
  }

  function setActiveId(id: string | undefined) {
    activeId.value = id;
  }

  function updateStructure(newNodes: FileNode[]) {
    nodes.value = [...newNodes]; // Force shallow trigger
  }

  function closeProject() {
    projectId.value = undefined;
    path.value = undefined;
    nodes.value = [];
    activeId.value = undefined;
    settings.value = null;
    plotlines.value = [];
    characters.value = [];
    totalWordCount.value = 0;

    localStorage.removeItem('last_opened_project_path');
  }

  // Granular Mutations (Optimized)

  function renameNodeAction(id: string, name: string) {
    const node = nodeMap.value.get(id);
    if (node) {
      node.name = name;
      statsVersion.value++; // Signal change for minor UI updates
    }
  }

  function updateNodeStatsAction(id: string, wordCount: number) {
    const node = nodeMap.value.get(id);
    if (node) {
      const diff = wordCount - (node.word_count || 0);
      node.word_count = wordCount;
      totalWordCount.value += diff;
      statsVersion.value++;
    }
  }

  function updateNodeMetadataAction(id: string, updates: Partial<FileNode>) {
    const node = nodeMap.value.get(id);
    if (node) {
      Object.assign(node, updates);
      statsVersion.value++;
    }
  }

  const setSettings = (newSettings: ProjectSettings) => {
    settings.value = newSettings;
  };

  const setPlotlines = (newPlotlines: Plotline[]) => {
    plotlines.value = newPlotlines;
  };

  return {
    // State
    nodes,
    activeId,
    projectId,
    path,
    settings,
    plotlines,
    characters,
    nodeMap,
    flatNodes,
    totalWordCount,
    statsVersion,

    // Getters
    activeChapter,
    chapterById,
    characterById,

    // Actions
    setProjectData,
    setActiveId,
    updateStructure,
    closeProject,
    renameNodeAction,
    updateNodeStatsAction,
    updateNodeMetadataAction,
    setSettings,
    setPlotlines,
  };
});
