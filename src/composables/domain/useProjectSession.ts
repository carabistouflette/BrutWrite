import { storeToRefs } from 'pinia';
import { watch } from 'vue';
import { useProjectStore } from '../../stores/project';
import { useCharacters } from './useCharacters';
import type { FileNode, ProjectSettings, Plotline, Character } from '../../types';
import { APP_CONSTANTS } from '../../config/constants';

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

  const getCacheKey = (path: string) => `${APP_CONSTANTS.CACHE.KEY_PREFIX}${path}`;

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
      const expiryMs = APP_CONSTANTS.CACHE.EXPIRY_DAYS * 24 * 60 * 60 * 1000;

      // Optional: Check if cache is too old
      if (Date.now() - data.timestamp > expiryMs) {
        localStorage.removeItem(getCacheKey(path));
        return false;
      }

      console.debug('Restoring project from cache:', path);

      // 1. Restore Characters
      const { setCharacters } = useCharacters();
      if (data.characters) setCharacters(data.characters);

      // 2. Restore Project Data
      projectStore.setProjectData(data.id, path, data.nodes, data.settings);

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

  const setupAutoSave = () => {
    let timeout: ReturnType<typeof setTimeout> | null = null;

    const { nodes, settings, plotlines, characters, activeId } = storeToRefs(projectStore);

    // Watch for ANY change in the session-critical data
    // We watch deep on specific objects that are mutable but might not trigger shallow reference changes if mutated in place (though store refs should trigger)
    watch(
      [nodes, settings, plotlines, characters, activeId],
      () => {
        // Debounce
        if (timeout) clearTimeout(timeout);

        timeout = setTimeout(() => {
          // Guard: Only save if we have a valid open project and path
          if (!projectStore.projectId || !projectStore.path) return;

          saveToCache(projectStore.path, {
            id: projectStore.projectId,
            nodes: projectStore.nodes,
            settings: projectStore.settings || { daily_target: 0, word_target: 0 }, // Fallback if null (shouldn't happen in active project)
            plotlines: projectStore.plotlines,
            characters: projectStore.characters,
            activeId: projectStore.activeId,
          });

          console.debug('Project session auto-saved to cache');
        }, APP_CONSTANTS.CACHE.DEBOUNCE_MS);
      },
      { deep: true }
    );
  };

  return {
    saveToCache,
    restoreSession,
    setupAutoSave,
  };
}
