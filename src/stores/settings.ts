import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { BaseDirectory, readTextFile, writeTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import { type AppSettings, defaultSettings } from '../config/defaultSettings';

import { useAppStatus } from '../composables/ui/useAppStatus';

const SETTINGS_FILE = 'settings.json';

function isObject(item: unknown): item is Record<string, unknown> {
  return !!(item && typeof item === 'object' && !Array.isArray(item));
}

export const useSettingsStore = defineStore('settings', () => {
  const { notifyError } = useAppStatus();
  const settings = ref<AppSettings>(JSON.parse(JSON.stringify(defaultSettings)));
  const isLoaded = ref(false);

  // Helper for merging
  function recursiveMerge<T extends object>(defaults: T, loaded: Record<string, unknown>): T {
    const result = { ...defaults } as Record<string, unknown>;
    for (const key in loaded) {
      if (Object.prototype.hasOwnProperty.call(defaults, key)) {
        const defaultValue = (defaults as Record<string, unknown>)[key];
        const loadedValue = loaded[key];
        if (isObject(defaultValue) && isObject(loadedValue)) {
          result[key] = recursiveMerge(
            defaultValue as Record<string, unknown>,
            loadedValue as Record<string, unknown>
          );
        } else if (loadedValue !== undefined) {
          result[key] = loadedValue;
        }
      }
    }
    return result as T;
  }

  // Persistence Logic
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  async function performSave() {
    try {
      await mkdir('', { baseDir: BaseDirectory.AppConfig, recursive: true });
      await writeTextFile(SETTINGS_FILE, JSON.stringify(settings.value, null, 2), {
        baseDir: BaseDirectory.AppConfig,
      });
    } catch (err) {
      notifyError('Failed to save settings', err);
    }
  }

  async function saveSettings(immediate = false) {
    if (!immediate) {
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = setTimeout(async () => {
        await performSave();
        saveTimeout = null;
      }, 1000);
      return;
    }
    await performSave();
  }

  async function loadSettings() {
    try {
      const fileExists = await exists(SETTINGS_FILE, { baseDir: BaseDirectory.AppConfig });
      if (fileExists) {
        const content = await readTextFile(SETTINGS_FILE, { baseDir: BaseDirectory.AppConfig });
        const parsed = JSON.parse(content);
        settings.value = recursiveMerge(defaultSettings, parsed);
      } else {
        await saveSettings(true);
      }
    } catch (err) {
      notifyError('Failed to load settings', err);
    } finally {
      setTimeout(() => {
        isLoaded.value = true;
      }, 100);
    }
  }

  // Auto-save watcher
  watch(
    settings,
    () => {
      if (isLoaded.value) {
        saveSettings();
      }
    },
    { deep: true }
  );

  return {
    settings,
    isLoaded,
    loadSettings,
    saveSettings,
  };
});
