import { defineStore } from 'pinia';
import { ref, shallowRef, computed, watch } from 'vue';
import type { FileNode, ProjectSettings, Character, Plotline } from '../types';

export const useProjectStore = defineStore('project', () => {
  // State
  const nodes = ref<FileNode[]>([]);
  const activeId = ref<string | undefined>(undefined);
  const projectId = ref<string | undefined>(undefined);
  const settings = ref<ProjectSettings | null>(null);
  const plotlines = ref<Plotline[]>([]);
  const characters = ref<Character[]>([]);

  // Derived State (Optimized Lookups)
  const nodeMap = shallowRef(new Map<string, FileNode>());
  const flatNodes = shallowRef<FileNode[]>([]);

  // Internal Helper
  const rebuildMap = (fileNodes: FileNode[]) => {
    const map = new Map<string, FileNode>();
    const list: FileNode[] = [];
    const stack: FileNode[] = [...fileNodes].reverse();

    while (stack.length > 0) {
      const node = stack.pop()!;
      map.set(node.id, node);
      list.push(node);

      if (node.children && node.children.length > 0) {
        for (let i = node.children.length - 1; i >= 0; i--) {
          stack.push(node.children[i]);
        }
      }
    }
    nodeMap.value = map;
    flatNodes.value = list;
  };

  // Watcher to keep lookups in sync
  watch(nodes, (newVal) => rebuildMap(newVal), { deep: false });

  // Actions
  function setProjectData(id: string, fileNodes: FileNode[], projectSettingsData: ProjectSettings) {
    projectId.value = id;
    nodes.value = fileNodes;
    settings.value = projectSettingsData;
    // Reset active ID on new project load
    activeId.value = undefined;
  }

  function setActiveId(id: string | undefined) {
    activeId.value = id;
  }

  function updateStructure(newNodes: FileNode[]) {
    nodes.value = newNodes;
  }

  function closeProject() {
    projectId.value = undefined;
    nodes.value = [];
    activeId.value = undefined;
    settings.value = null;
    plotlines.value = [];
    characters.value = [];
  }

  const activeChapter = computed(() => {
    if (!activeId.value) return undefined;
    return nodeMap.value.get(activeId.value);
  });

  return {
    // State
    nodes,
    activeId,
    projectId,
    settings,
    plotlines,
    characters,
    nodeMap,
    flatNodes,
    activeChapter,

    // Actions
    setProjectData,
    setActiveId,
    updateStructure,
    closeProject,
  };
});
