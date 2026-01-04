import { defineStore } from 'pinia';
import { ref, shallowRef, computed, triggerRef } from 'vue';
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
  // REMOVED: statsVersion hack. We now use reactive() on nodes.

  // Character map for O(1) access
  const characterMap = computed(() => {
    const map = new Map<string, Character>();
    for (const char of characters.value) {
      map.set(char.id, char);
    }
    return map;
  });

  const indexNodes = (fileNodes: FileNode[]) => {
    const map = new Map<string, FileNode>();
    const stack: FileNode[] = [...fileNodes].reverse();

    while (stack.length > 0) {
      const node = stack.pop()!;
      map.set(node.id, node);

      if (node.children && node.children.length > 0) {
        for (let i = node.children.length - 1; i >= 0; i--) {
          stack.push(node.children[i]);
        }
      }
    }
    nodeMap.value = map;
  };

  const updateDerived = (fileNodes: FileNode[]) => {
    // Rebuild flat list and total word count
    // This is O(N) but valid for structure updates.
    // Optimizing map creation is the bigger win as it involves hashing.
    const list: FileNode[] = [];
    const stack: FileNode[] = [...fileNodes].reverse();
    let totalWc = 0;

    while (stack.length > 0) {
      const node = stack.pop()!;
      list.push(node);
      totalWc += node.word_count || 0;

      if (node.children && node.children.length > 0) {
        for (let i = node.children.length - 1; i >= 0; i--) {
          stack.push(node.children[i]);
        }
      }
    }
    flatNodes.value = list;
    totalWordCount.value = totalWc;
  };

  const rebuildAll = (fileNodes: FileNode[]) => {
    indexNodes(fileNodes);
    updateDerived(fileNodes);
  };

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
    // Make nodes deep reactive
    // OPTIMIZATION: Use plain objects with shallowRef + triggerRef to avoid overhead of 1000s of Proxies
    nodes.value = fileNodes;
    // Safe to use triggerRef because we just replaced the entire array.
    triggerRef(nodes);
    rebuildAll(fileNodes); // Full rebuild on load

    settings.value = projectSettingsData;
    activeId.value = undefined;
  }

  function setActiveId(id: string | undefined) {
    activeId.value = id;
  }

  function updateStructure(newNodes: FileNode[]) {
    nodes.value = [...newNodes];
    // Structure update (drag & drop) usually doesn't add/remove nodes,
    // so map (id->node) might stay valid, but let's be safe and rebuild indexes
    // if we suspect new nodes. For pure reorder, we could just updateDerived.
    // But for now, let's assume updateStructure might include new nodes.
    // If we knew it was just reorder, we'd call updateDerived(newNodes).
    rebuildAll(newNodes);
    triggerRef(nodes);
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

    // Clear derived state
    rebuildAll([]);
    triggerRef(nodes);
  }

  // Granular Mutations (Optimized)

  function renameNodeAction(id: string, name: string) {
    const node = nodeMap.value.get(id);
    if (node) {
      node.name = name;
      triggerRef(nodes);
    }
  }

  function updateNodeStatsAction(id: string, wordCount: number) {
    const node = nodeMap.value.get(id);
    if (node) {
      const diff = wordCount - (node.word_count || 0);
      node.word_count = wordCount;
      totalWordCount.value += diff;
      triggerRef(nodes);
    }
  }

  function updateNodeMetadataAction(id: string, updates: Partial<FileNode>) {
    const node = nodeMap.value.get(id);
    if (node) {
      Object.assign(node, updates);
      triggerRef(nodes);
    }
  }

  const setSettings = (newSettings: ProjectSettings) => {
    settings.value = newSettings;
  };

  const setPlotlines = (newPlotlines: Plotline[]) => {
    plotlines.value = newPlotlines;
  };

  // Character Actions
  const setCharacters = (list: Character[]) => {
    characters.value = list;
  };

  const updateCharacter = (character: Character) => {
    const index = characters.value.findIndex((c) => c.id === character.id);
    if (index !== -1) {
      characters.value[index] = character;
    } else {
      characters.value.push(character);
    }
    triggerRef(characters); // Ensure deep reactivity triggers if needed
  };

  const removeCharacter = (id: string) => {
    characters.value = characters.value.filter((c) => c.id !== id);
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
    setCharacters,
    updateCharacter,
    removeCharacter,
  };
});
