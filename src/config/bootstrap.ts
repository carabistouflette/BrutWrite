import { useSettingsStore } from '../stores/settings';
import { useProjectIO } from '../composables/domain/useProjectIO';
import { useTheme } from '../composables/ui/useTheme';
import { useAppStatus } from '../composables/ui/useAppStatus';

/**
 * Global application bootstrapper.
 * Handles loading settings, initializing theme watchers, and auto-loading the last project.
 */
export async function initApp() {
  const settingsStore = useSettingsStore();
  const { loadProject } = useProjectIO();
  const { initTheme } = useTheme();

  // 1. Initialize Theme Watchers (Calculates CSS variables, handles dark mode)
  initTheme();
  const { notifyError } = useAppStatus();

  try {
    // 2. Load Settings & 3. Auto-load Last Project in parallel
    const lastPath = localStorage.getItem('last_opened_project_path');

    await Promise.allSettled([
      settingsStore.loadSettings(),
      lastPath
        ? (async () => {
            console.debug('Auto-loading project from:', lastPath);
            await loadProject(lastPath);
          })()
        : Promise.resolve(),
    ]);
  } catch (err) {
    notifyError('App initialization failed', err);
  }
}
