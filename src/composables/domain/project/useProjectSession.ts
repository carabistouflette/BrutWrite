import { watch, onUnmounted, getCurrentInstance } from 'vue';
import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { useCharacters } from '../characters/useCharacters';
import type { FileNode, ProjectSettings, Plotline, Character } from '../../../types';
import { APP_CONSTANTS } from '../../../config/constants';

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
      // CRITICAL FIX: Do NOT save the entire nodes tree to localStorage!
      // The nodes tree can be 100s of KB (or MB for large projects) and will:
      // 1. Hit the 5MB localStorage quota
      // 2. Block the main thread with expensive JSON.stringify
      // Instead, we save only lightweight metadata.
      const lightweightCache = {
        id: data.id,
        // nodes: data.nodes, // REMOVED - this is the performance killer
        settings: data.settings,
        plotlines: data.plotlines,
        characters: data.characters,
        activeId: data.activeId,
        timestamp: Date.now(),
      };

      localStorage.setItem(getCacheKey(path), JSON.stringify(lightweightCache));
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

      // 2. Restore Project Data (BUT NOT NODES - they come from disk via loadProject)
      // We only restore the ID and path here as a hint that this project was open.
      projectStore.projectId = data.id;
      projectStore.path = path;

      // 3. Restore Plotlines
      if (data.plotlines) projectPlotlines.value = data.plotlines;

      // 4. Restore Active ID (if it was valid)
      // NOTE: We can't validate this until nodes are loaded from disk.
      // The main loadProject flow will handle setting activeId properly.
      if (data.activeId) {
        projectStore.setActiveId(data.activeId);
      }

      // IMPORTANT: The caller (bootstrap or loadProject) must call the backend
      // to actually load the nodes from disk. This cache is just for metadata.
      return true;
    } catch (e) {
      console.warn('Failed to restore session', e);
      return false;
    }
  };

  const setupAutoSave = () => {
    let timeout: ReturnType<typeof setTimeout> | null = null;
    let unsubscribeActions: (() => void) | null = null;

    const triggerSave = () => {
      // Debounce
      if (timeout) clearTimeout(timeout);

      timeout = setTimeout(() => {
        // Guard: Only save if we have a valid open project and path
        if (!projectStore.projectId || !projectStore.path) return;

        saveToCache(projectStore.path, {
          id: projectStore.projectId,
          nodes: projectStore.nodes, // This is NOT saved to localStorage (see saveToCache)
          settings: projectStore.settings || { daily_target: 0, word_target: 0 },
          plotlines: projectStore.plotlines,
          characters: projectStore.characters,
          activeId: projectStore.activeId,
        });

        console.debug('Project session auto-saved to cache');
      }, APP_CONSTANTS.CACHE.DEBOUNCE_MS);
    };

    // 1. Structural Changes: Use Action Subscription to filter out noise
    unsubscribeActions = projectStore.$onAction(({ name, after }) => {
      // Ignore high-frequency stats updates
      if (name === 'updateNodeStatsAction') return;

      after(() => {
        triggerSave();
      });
    });

    // 2. Specific Watchers for non-action state changes
    const { activeId, settings, plotlines, characters } = storeToRefs(projectStore);

    // Watch settings deeply (small object)
    watch(settings, triggerSave, { deep: true });

    // Watch activeId (primitive)
    watch(activeId, triggerSave);

    // Watch arrays shallowly (reference change)
    // If plotlines/characters are mutated in place WITHOUT an action, this won't catch it,
    // but good practice is to use actions.
    watch([plotlines, characters], triggerSave);

    // Cleanup if used in a component
    if (getCurrentInstance()) {
      onUnmounted(() => {
        if (timeout) clearTimeout(timeout);
        if (unsubscribeActions) unsubscribeActions();
      });
    }
  };

  return {
    saveToCache,
    restoreSession,
    setupAutoSave,
  };
}
