import { useProjectStore } from '../../../stores/project';
import { invoke } from '@tauri-apps/api/core';
import { useCharacters } from '../characters/useCharacters';
import { useRecentProjects } from './useRecentProjects';
import { useProjectSession } from './useProjectSession';
import { projectApi } from '../../../api/project';
import { chaptersApi } from '../../../api/chapters';
import { reconstructHierarchy } from '../../../utils/tree';
import type { ProjectSettings, Plotline } from '../../../types';

export function useProjectLoader() {
  const projectStore = useProjectStore();
  const { setCharacters } = useCharacters();
  const { addRecentProject } = useRecentProjects();
  const { saveToCache } = useProjectSession();

  /**
   * Load a project from a path and populate all stores
   */
  const loadProject = async (projectPath: string) => {
    // 1. Load Metadata
    const metadata = await projectApi.load(projectPath);

    // 2. Sync characters
    setCharacters(metadata.characters);

    // 3. Reconstruct hierarchy
    const hierarchy = reconstructHierarchy(metadata.manifest.chapters);

    // 4. Set project data in store
    projectStore.setProjectData(metadata.id, projectPath, hierarchy, metadata.settings);
    projectStore.setPlotlines(metadata.plotlines);

    // 5. Side effects
    localStorage.setItem('last_opened_project_path', projectPath);
    addRecentProject(projectPath);

    // 6. Set active ID
    if (projectStore.nodes.length > 0 && !projectStore.activeId) {
      projectStore.setActiveId(projectStore.nodes[0].id);
    }

    // 7. Update Cache
    saveToCache(projectPath, {
      id: metadata.id,
      nodes: hierarchy,
      settings: metadata.settings,
      plotlines: metadata.plotlines,
      characters: metadata.characters,
      activeId: projectStore.activeId,
    });
  };

  /**
   * Create a new project and populate stores
   */
  const createProject = async (projectPath: string, name: string, author: string) => {
    // 1. Create on Backend
    const metadata = await projectApi.create(projectPath, name, author);

    // 2. Reset characters
    setCharacters([]);

    // 3. Set data (empty initially)
    projectStore.setProjectData(metadata.id, projectPath, [], metadata.settings);
    projectStore.setPlotlines(metadata.plotlines);

    localStorage.setItem('last_opened_project_path', projectPath);
    addRecentProject(projectPath);

    // 4. Add default chapter
    const chapterMetadata = await chaptersApi.createNode(metadata.id, undefined, 'New Chapter');
    const hierarchy = reconstructHierarchy(chapterMetadata.manifest.chapters);
    projectStore.updateStructure(hierarchy);

    if (hierarchy.length > 0) {
      projectStore.setActiveId(hierarchy[0].id);
    }

    // 5. Initial Cache
    saveToCache(projectPath, {
      id: metadata.id,
      nodes: hierarchy,
      settings: metadata.settings,
      plotlines: metadata.plotlines,
      characters: [],
      activeId: projectStore.activeId,
    });
  };

  /**
   * Update settings
   */
  const updateSettings = async (newSettings: ProjectSettings) => {
    if (!projectStore.projectId) return;
    const metadata = await projectApi.updateSettings(projectStore.projectId, newSettings);
    projectStore.setSettings(metadata.settings);
  };

  /**
   * Update plotlines
   */
  const updatePlotlines = async (newPlotlines: Plotline[]) => {
    if (!projectStore.projectId) return;
    const metadata = await projectApi.updatePlotlines(projectStore.projectId, newPlotlines);
    projectStore.setPlotlines(metadata.plotlines);
  };

  /**
   * Create and seed a demo project
   */
  const createDemoProject = async (projectPath: string) => {
    // 1. Seed on backend
    await invoke('seed_demo_project', { path: projectPath });
    // 2. Load it
    await loadProject(projectPath);
  };

  return {
    loadProject,
    createProject,
    updateSettings,
    updatePlotlines,
    createDemoProject,
  };
}
