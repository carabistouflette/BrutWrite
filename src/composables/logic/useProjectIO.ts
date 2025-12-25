import { storeToRefs } from 'pinia';
import { projectApi } from '../../api/project';
import { useAppStatus } from '../ui/useAppStatus';
import { useCharacters } from './useCharacters';
import { useRecentProjects } from './useRecentProjects';
import { reconstructHierarchy } from '../../utils/tree';
import type { ProjectSettings, Plotline } from '../../types';
import { useProjectStore } from '../../stores/project';
import { useProjectNodeOperations } from './useProjectNodeOperations';

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
  } = storeToRefs(projectStore);

  const loadProject = async (path: string) => {
    try {
      const metadata = await projectApi.load(path);

      // Sync characters to their dedicated store
      const { setCharacters } = useCharacters();
      setCharacters(metadata.characters);

      // Set project data in store
      projectStore.setProjectData(
        metadata.id,
        reconstructHierarchy(metadata.manifest.chapters),
        metadata.settings
      );

      projectPlotlines.value = metadata.plotlines;

      localStorage.setItem('last_opened_project_path', path);
      addRecentProject(path);

      if (projectData.value.length > 0) {
        projectStore.setActiveId(projectData.value[0].id);
      }
    } catch (e) {
      notifyError('Failed to load project', e);
      localStorage.removeItem('last_opened_project_path');
    }
  };

  const createProject = async (path: string, name: string, author: string) => {
    try {
      const metadata = await projectApi.create(path, name, author);

      // Reset character store
      const { setCharacters } = useCharacters();
      setCharacters([]);

      // Set project data in store
      projectId.value = metadata.id;
      projectData.value = [];
      projectSettings.value = metadata.settings;
      projectPlotlines.value = metadata.plotlines;

      localStorage.setItem('last_opened_project_path', path);
      addRecentProject(path);

      // Add a default chapter
      await addChapter();
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
