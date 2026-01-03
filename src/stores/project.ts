import { defineStore } from 'pinia';
import { ref, shallowRef, computed, watch } from 'vue';
import type { FileNode, ProjectSettings, Character, Plotline } from '../types';

export const useProjectStore = defineStore('project', () => {
  // State
  const nodes = ref<FileNode[]>([]);
  const activeId = ref<string | undefined>(undefined);
  const projectId = ref<string | undefined>(undefined);
  const path = ref<string | undefined>(undefined);
  const settings = ref<ProjectSettings | null>(null);
  const plotlines = ref<Plotline[]>([]);
  const characters = ref<Character[]>([]);

  // Derived State (Optimized Lookups)
  const nodeMap = shallowRef(new Map<string, FileNode>());
  const flatNodes = shallowRef<FileNode[]>([]);

  // Character map for O(1) access
  const characterMap = computed(() => {
    const map = new Map<string, Character>();
    for (const char of characters.value) {
      map.set(char.id, char);
    }
    return map;
  });

  // Internal Helper
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const rebuildMap = (fileNodes: FileNode[]) => {
    if (debounceTimer) clearTimeout(debounceTimer);

    debounceTimer = setTimeout(() => {
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
    }, 50); // 50ms Debounce
  };

  // Watcher to keep lookups in sync
  watch(nodes, (newVal) => rebuildMap(newVal), { deep: true }); // Need deep true to catch children changes

  // --- Getters ---

  const activeChapter = computed(() => {
    if (!activeId.value) return undefined;
    return nodeMap.value.get(activeId.value);
  });

  /**
   * Get a chapter/node by ID with O(1) complexity.
   */
  const chapterById = (id: string) => nodeMap.value.get(id);

  /**
   * Get a character by ID with O(1) complexity.
   */
  const characterById = (id: string) => characterMap.value.get(id);

  /**
   * Total word count of all nodes in the project.
   */
  const totalWordCount = computed(() => {
    return flatNodes.value.reduce((sum, node) => sum + (node.word_count || 0), 0);
  });

  // --- Actions ---

  /**
   * Initialize the store with full project data from the backend.
   */
  // --- Actions ---

  /**
   * Initialize the store with full project data from the backend.
   */
  function setProjectData(
    id: string,
    projectPath: string,
    fileNodes: FileNode[],
    projectSettingsData: ProjectSettings
  ) {
    projectId.value = id;
    path.value = projectPath;
    nodes.value = fileNodes;
    settings.value = projectSettingsData;
    // Reset active ID on new project load
    activeId.value = undefined;
  }

  /**
   * Set the currently active node/chapter ID.
   */
  function setActiveId(id: string | undefined) {
    activeId.value = id;
  }

  /**
   * Update the entire file structure (manifest).
   */
  function updateStructure(newNodes: FileNode[]) {
    nodes.value = newNodes;
  }

  /**
   * Clear all project data from the store.
   */
  function closeProject() {
    projectId.value = undefined;
    path.value = undefined;
    nodes.value = [];
    activeId.value = undefined;
    settings.value = null;
    plotlines.value = [];
    characters.value = [];

    // Clear characters
    characters.value = [];

    localStorage.removeItem('last_opened_project_path');
  }

  // Granular Mutations

  /**
   * Optimistically rename a node in the local store.
   */
  function renameNodeAction(id: string, name: string) {
    const node = nodeMap.value.get(id);
    if (node) {
      node.name = name;
    }
  }

  /**
   * Optimistically update word count for a node.
   */
  function updateNodeStatsAction(id: string, wordCount: number) {
    const node = nodeMap.value.get(id);
    if (node) {
      node.word_count = wordCount;
    }
  }

  /**
   * Optimistically update generic metadata for a node.
   */
  function updateNodeMetadataAction(id: string, updates: Partial<FileNode>) {
    const node = nodeMap.value.get(id);
    if (node) {
      Object.assign(node, updates);
    }
  }

  // --- Async Actions (Moved to useProjectLoader to avoid circular deps) ---
  // The logic for loading/creating projects is now in useProjectLoader.ts
  // The store only provides setters for the data.

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

    // Getters
    activeChapter,
    chapterById,
    characterById,
    totalWordCount,

    // Actions
    setProjectData,
    setActiveId,
    updateStructure,
    closeProject,
    renameNodeAction,
    updateNodeStatsAction,
    updateNodeMetadataAction,
  };
});
