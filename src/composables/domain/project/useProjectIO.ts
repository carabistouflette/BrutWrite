import { storeToRefs } from 'pinia';
import { projectApi } from '../../../api/project';
import { useAppStatus } from '../../ui/useAppStatus';
import { useCharacters } from '../characters/useCharacters';
import { useRecentProjects } from './useRecentProjects';
import { reconstructHierarchy } from '../../../utils/tree';
import type { ProjectSettings, Plotline } from '../../../types';
import { useProjectStore } from '../../../stores/project';
import { useProjectNodeOperations } from './useProjectNodeOperations';
import { useProjectSession } from './useProjectSession';

export function useProjectIO() {
  const { notifyError } = useAppStatus();
  const { addChapter } = useProjectNodeOperations();
  const { addRecentProject } = useRecentProjects();
  const projectStore = useProjectStore();
  const {
    projectId,
    settings: projectSettings,
    plotlines: projectPlotlines,
    nodes: projectData,
    activeId,
  } = storeToRefs(projectStore);

  const { saveToCache } = useProjectSession();

  const loadProject = async (path: string) => {
    try {
      const metadata = await projectApi.load(path);

      // Sync characters to their dedicated store
      const { setCharacters } = useCharacters();
      setCharacters(metadata.characters);

      // Reconstruct hierarchy
      const hierarchy = reconstructHierarchy(metadata.manifest.chapters);

      // Set project data in store
      projectStore.setProjectData(metadata.id, path, hierarchy, metadata.settings);

      projectPlotlines.value = metadata.plotlines;

      localStorage.setItem('last_opened_project_path', path);
      addRecentProject(path);

      if (projectData.value.length > 0) {
        // preserve activeId if it's already set (e.g. by cache) AND valid
        // Otherwise default to first node
        if (!projectId.value || !activeId.value) {
          // This check is tricky because we just set projectId
          projectStore.setActiveId(projectData.value[0].id);
        }
      }

      // Update Cache
      saveToCache(path, {
        id: metadata.id,
        nodes: hierarchy,
        settings: metadata.settings,
        plotlines: metadata.plotlines,
        characters: metadata.characters,
        activeId: activeId.value,
      });
    } catch (e) {
      notifyError('Failed to load project', e);
      // Rethrow to allow caller to handle UI state (e.g. abort navigation)
      throw e;
    }
  };

  const createProject = async (path: string, name: string, author: string) => {
    try {
      const metadata = await projectApi.create(path, name, author);

      // Reset character store
      const { setCharacters } = useCharacters();
      setCharacters([]);

      // Set project data in store
      projectStore.setProjectData(metadata.id, path, [], metadata.settings);
      projectPlotlines.value = metadata.plotlines;

      localStorage.setItem('last_opened_project_path', path);
      addRecentProject(path);

      // Add a default chapter
      await addChapter();

      // Initial Cache
      saveToCache(path, {
        id: metadata.id,
        nodes: projectData.value,
        settings: metadata.settings,
        plotlines: metadata.plotlines,
        characters: [],
        activeId: activeId.value,
      });
    } catch (e) {
      notifyError('Failed to create project', e);
    }
  };

  const closeProject = () => {
    projectStore.closeProject();

    const { setCharacters } = useCharacters();
    setCharacters([]);

    localStorage.removeItem('last_opened_project_path');
  };

  const updateSettings = async (settings: ProjectSettings) => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.updateSettings(projectId.value, settings);
      projectSettings.value = metadata.settings;
      // Update Cache (lazy way: read whole store? or just patch?)
      // For simplicity, we won't partial-update cache here unless necessary.
      // But ideally we should.
    } catch (e) {
      notifyError('Failed to update project settings', e);
    }
  };

  const updatePlotlines = async (plotlines: Plotline[]) => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.updatePlotlines(projectId.value, plotlines);
      projectPlotlines.value = metadata.plotlines;
    } catch (e) {
      notifyError('Failed to update plotlines', e);
    }
  };

  return {
    loadProject,
    createProject,
    closeProject,
    updateSettings,
    updatePlotlines,
  };
}
