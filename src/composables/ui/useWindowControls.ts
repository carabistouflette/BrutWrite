import { getCurrentWindow } from '@tauri-apps/api/window';
import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useProjectStore } from '../../stores/project';
import { useProjectSync } from '../domain/project/useProjectSync';
import { flushChapterSessions } from '../domain/project/useChapterSession';
import { flushResearchNoteSessions } from '../domain/research/useResearchNoteSession';
import { useAppStatus } from './useAppStatus';

export function useWindowControls() {
  // Check if running in Tauri environment
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  const projectStore = useProjectStore();
  const { flush } = useProjectSync();
  const { notifyError } = useAppStatus();
  let unlistenClose: UnlistenFn | undefined;
  let disposed = false;
  let closing = false;

  onMounted(async () => {
    if (!isTauri) return;
    try {
      const unlisten = await getCurrentWindow().onCloseRequested(async (event) => {
        if (closing) {
          event.preventDefault();
          return;
        }
        closing = true;
        const projectId = projectStore.projectId;
        const projectPath = projectStore.path;
        const appRoot = document.getElementById('app');
        const wasInert = appRoot?.inert ?? false;
        if (appRoot) appRoot.inert = true;
        try {
          await flushChapterSessions(projectStore);
          if (projectId && projectPath) {
            await flushResearchNoteSessions(projectStore, projectId, projectPath);
          }
          await flush(projectId);
        } catch (error) {
          if (appRoot) appRoot.inert = wasInert;
          event.preventDefault();
          notifyError('Failed to save the project before closing', error);
        } finally {
          closing = false;
        }
      });
      if (disposed) unlisten();
      else unlistenClose = unlisten;
    } catch (error) {
      notifyError('Failed to register the save-before-close handler', error);
    }
  });

  onBeforeUnmount(() => {
    disposed = true;
    unlistenClose?.();
  });

  const minimize = () => {
    if (isTauri) {
      getCurrentWindow().minimize();
    }
    // Non-Tauri environments: no-op (e.g., browser dev mode)
  };

  const toggleMaximize = () => {
    if (isTauri) {
      getCurrentWindow().toggleMaximize();
    }
    // Non-Tauri environments: no-op
  };

  const close = () => {
    if (isTauri) {
      getCurrentWindow().close();
    }
    // Non-Tauri environments: no-op
  };

  return {
    minimize,
    toggleMaximize,
    close,
    isTauri,
  };
}
