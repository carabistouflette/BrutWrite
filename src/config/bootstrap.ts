import { useSettingsStore } from '../stores/settings';
import { useProjectIO } from '../composables/domain/useProjectIO';
import { useProjectSession } from '../composables/domain/useProjectSession';
import { useTheme } from '../composables/ui/useTheme';
import { useAppStatus } from '../composables/ui/useAppStatus';

/**
 * Global application bootstrapper.
 * Handles loading settings, initializing theme watchers, and auto-loading the last project.
 */
export async function initApp() {
  const settingsStore = useSettingsStore();
  const { loadProject } = useProjectIO();
  const { restoreSession } = useProjectSession();
  const { initTheme } = useTheme();

  // 1. Initialize Theme Watchers (Calculates CSS variables, handles dark mode)
  initTheme();
  const { notifyError } = useAppStatus();

  try {
    // 2. Load Settings & 3. Auto-load Last Project in parallel
    const lastPath = localStorage.getItem('last_opened_project_path');

    // Optimistic restore for immediate UI feedback
    if (lastPath) {
      const restored = restoreSession(lastPath);
      if (restored) {
        console.debug('Session restored from cache', lastPath);
      }
    }

    await Promise.allSettled([
      settingsStore.loadSettings(),
      lastPath
        ? (async () => {
            // Even if restored, we fetch fresh data
            console.debug('Loading fresh project data:', lastPath);
            await loadProject(lastPath);
          })()
        : Promise.resolve(),
    ]);
  } catch (err) {
    notifyError('App initialization failed', err);
  }
}
