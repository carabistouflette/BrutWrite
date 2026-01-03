import { defineStore } from 'pinia';
import { ref, shallowRef, computed, watch } from 'vue';
import type { FileNode, ProjectSettings, Character, Plotline } from '../types';
import { reconstructHierarchy } from '../utils/tree';

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

    // Clear characters in dedicated store too
    // Note: We avoid importing useCharacters at top level if possible to avoid cycles
    // But since useCharacters imports projectStore, we have a cycle if we import it top level.
    // Dynamic import or function-scoped usage is safer.
    import('../composables/domain/characters/useCharacters').then(({ useCharacters }) => {
      const { setCharacters } = useCharacters();
      setCharacters([]);
    });

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

  // --- Async Actions (Merged from useProjectIO) ---

  async function loadProject(projectPath: string) {
    // Dynamic imports to avoid circular dependencies with composables that use this store
    const { projectApi } = await import('../api/project');
    const { useCharacters } = await import('../composables/domain/characters/useCharacters');
    const { useRecentProjects } = await import('../composables/domain/project/useRecentProjects');
    const { useProjectSession } = await import('../composables/domain/project/useProjectSession');

    const metadata = await projectApi.load(projectPath);

    // Sync characters
    const { setCharacters } = useCharacters();
    setCharacters(metadata.characters);

    // Reconstruct hierarchy
    const hierarchy = reconstructHierarchy(metadata.manifest.chapters);

    // Set project data
    setProjectData(metadata.id, projectPath, hierarchy, metadata.settings);
    plotlines.value = metadata.plotlines;

    // Side effects
    localStorage.setItem('last_opened_project_path', projectPath);
    const { addRecentProject } = useRecentProjects();
    addRecentProject(projectPath);

    // Set active ID
    if (nodes.value.length > 0) {
      if (!activeId.value) {
        setActiveId(nodes.value[0].id);
      }
    }

    // Update Cache
    const { saveToCache } = useProjectSession();
    saveToCache(projectPath, {
      id: metadata.id,
      nodes: hierarchy,
      settings: metadata.settings,
      plotlines: metadata.plotlines,
      characters: metadata.characters,
      activeId: activeId.value,
    });
  }

  async function createProject(projectPath: string, name: string, author: string) {
    const { projectApi } = await import('../api/project');
    const { useCharacters } = await import('../composables/domain/characters/useCharacters');
    const { useRecentProjects } = await import('../composables/domain/project/useRecentProjects');
    const { useProjectSession } = await import('../composables/domain/project/useProjectSession');
    const { chaptersApi } = await import('../api/chapters');

    const metadata = await projectApi.create(projectPath, name, author);

    // Reset characters
    const { setCharacters } = useCharacters();
    setCharacters([]);

    // Set data (empty initially)
    setProjectData(metadata.id, projectPath, [], metadata.settings);
    plotlines.value = metadata.plotlines;

    localStorage.setItem('last_opened_project_path', projectPath);
    const { addRecentProject } = useRecentProjects();
    addRecentProject(projectPath);

    // Add default chapter
    const chapterMetadata = await chaptersApi.createNode(metadata.id, undefined, 'New Chapter');
    const hierarchy = reconstructHierarchy(chapterMetadata.manifest.chapters);
    updateStructure(hierarchy);
    if (hierarchy.length > 0) {
      setActiveId(hierarchy[0].id);
    }

    // Initial Cache
    const { saveToCache } = useProjectSession();
    saveToCache(projectPath, {
      id: metadata.id,
      nodes: hierarchy,
      settings: metadata.settings,
      plotlines: metadata.plotlines,
      characters: [],
      activeId: activeId.value,
    });
  }

  async function updateSettingsAction(newSettings: ProjectSettings) {
    if (!projectId.value) return;
    const { projectApi } = await import('../api/project');
    const metadata = await projectApi.updateSettings(projectId.value, newSettings);
    settings.value = metadata.settings;
  }

  async function updatePlotlinesAction(newPlotlines: Plotline[]) {
    if (!projectId.value) return;
    const { projectApi } = await import('../api/project');
    const metadata = await projectApi.updatePlotlines(projectId.value, newPlotlines);
    plotlines.value = metadata.plotlines;
  }

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

    // Async Actions
    loadProject,
    createProject,
    updateSettingsAction,
    updatePlotlinesAction,
  };
});
