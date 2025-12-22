import { ref, watch } from 'vue';
import { BaseDirectory, readTextFile, writeTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import { type AppSettings, defaultSettings } from '../config/defaultSettings';

function isObject(item: unknown): item is Record<string, unknown> {
    return !!(item && typeof item === 'object' && !Array.isArray(item));
}

// Global state
const settings = ref<AppSettings>(JSON.parse(JSON.stringify(defaultSettings)));
const isLoaded = ref(false);

const SETTINGS_FILE = 'settings.json';

// Singleton debouncer and watcher
let saveTimeout: ReturnType<typeof setTimeout> | null = null;

const performSave = async () => {
    try {
        // Ensure the directory exists (mkdir -p behavior)
        await mkdir('', { baseDir: BaseDirectory.AppConfig, recursive: true });

        await writeTextFile(
            SETTINGS_FILE,
            JSON.stringify(settings.value, null, 2),
            { baseDir: BaseDirectory.AppConfig }
        );
    } catch (err) {
        console.error('Failed to save settings:', err);
    }
};

const saveSettings = async (immediate = false) => {
    if (!immediate) {
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(async () => {
            await performSave();
            saveTimeout = null;
        }, 1000); // 1 second debounce
        return;
    }
    await performSave();
};

// Start watching once (Singleton)
watch(settings, () => {
    if (isLoaded.value) {
        saveSettings();
    }
}, { deep: true });

export function useSettings() {

    const loadSettings = async () => {
        try {
            const fileExists = await exists(SETTINGS_FILE, { baseDir: BaseDirectory.AppConfig });

            if (fileExists) {
                const content = await readTextFile(SETTINGS_FILE, { baseDir: BaseDirectory.AppConfig });
                const parsed = JSON.parse(content);

                // Merge loaded settings with defaults to ensure all keys exist
                settings.value = recursiveMerge(defaultSettings, parsed);

            } else {
                // First time: save default settings
                await saveSettings(true);
            }
        } catch (err) {
            console.error('Failed to load settings:', err);
        } finally {
            // Set isLoaded at the very end to ensure any reactive updates 
            // from the load process are finished before we start auto-saving.
            setTimeout(() => {
                isLoaded.value = true;
            }, 100);
        }
    };

    // Helper for merging defaults (source) with loaded (override)
    function recursiveMerge(defaults: any, loaded: any): any {
        const result = { ...defaults };

        for (const key in loaded) {
            if (key in defaults) {
                if (isObject(defaults[key]) && isObject(loaded[key])) {
                    result[key] = recursiveMerge(defaults[key], loaded[key]);
                } else {
                    result[key] = loaded[key];
                }
            }
        }
        return result;
    }

    return {
        settings,
        loadSettings,
        saveSettings
    };
}

