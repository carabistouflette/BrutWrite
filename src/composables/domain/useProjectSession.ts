import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../stores/project';
import { useCharacters } from './useCharacters';
import type { FileNode, ProjectSettings, Plotline, Character } from '../../types';

export interface SessionData {
  id: string;
  nodes: FileNode[];
  settings: ProjectSettings;
  plotlines: Plotline[];
  characters: Character[];
  activeId: string | undefined;
}

export function useProjectSession() {
  const projectStore = useProjectStore();
  const { plotlines: projectPlotlines } = storeToRefs(projectStore);

  const getCacheKey = (path: string) => `project_session_${path}`;

  const saveToCache = (path: string, data: SessionData) => {
    try {
      localStorage.setItem(
        getCacheKey(path),
        JSON.stringify({
          ...data,
          timestamp: Date.now(),
        })
      );
    } catch (e) {
      console.warn('Failed to cache project session', e);
    }
  };

  const restoreSession = (path: string): boolean => {
    try {
      const cached = localStorage.getItem(getCacheKey(path));
      if (!cached) return false;

      const data = JSON.parse(cached) as SessionData & { timestamp: number };
      // Optional: Check if cache is too old (e.g. > 7 days)
      if (Date.now() - data.timestamp > 7 * 24 * 60 * 60 * 1000) {
        localStorage.removeItem(getCacheKey(path));
        return false;
      }

      console.debug('Restoring project from cache:', path);

      // 1. Restore Characters
      const { setCharacters } = useCharacters();
      if (data.characters) setCharacters(data.characters);

      // 2. Restore Project Data
      projectStore.setProjectData(data.id, data.nodes, data.settings);

      // 3. Restore Plotlines
      if (data.plotlines) projectPlotlines.value = data.plotlines;

      // 4. Restore Active ID (if it was valid)
      if (data.activeId) {
        // We need to wait for nodeMap to be rebuilt which happens in watcher
        // But since we set nodes above, it should trigger.
        // Ideally we check if id exists in nodes, but for speed we just set it.
        projectStore.setActiveId(data.activeId);
      } else if (data.nodes.length > 0) {
        projectStore.setActiveId(data.nodes[0].id);
      }

      return true;
    } catch (e) {
      console.warn('Failed to restore session', e);
      return false;
    }
  };

  return {
    saveToCache,
    restoreSession,
  };
}
