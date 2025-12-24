import { useSettingsStore } from '../stores/settings';
import { useProjectIO } from '../composables/logic/useProjectIO';

export async function initApp() {
  const settingsStore = useSettingsStore();
  const { loadProject } = useProjectIO();

  try {
    // 1. Load Settings
    await settingsStore.loadSettings();

    // 2. Auto-load last project
    const lastPath = localStorage.getItem('last_opened_project_path');
    if (lastPath) {
      console.debug('Auto-loading project from:', lastPath);
      await loadProject(lastPath);
    }
  } catch (err) {
    console.error('App initialization failed:', err);
  }
}
