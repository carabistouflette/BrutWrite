import { ref } from 'vue';

const recentProjects = ref<string[]>([]);
const STORAGE_KEY = 'recent_projects';

export function useRecentProjects() {
    
    const loadRecentProjects = () => {
        const recentStr = localStorage.getItem(STORAGE_KEY) || '[]';
        try {
            recentProjects.value = JSON.parse(recentStr);
        } catch (e) {
            console.error('Failed to parse recent projects', e);
            recentProjects.value = [];
        }
    };

    const addRecentProject = (path: string) => {
        // We reload to ensure we have the latest if multiple tabs/windows (though unlikely in Tauri)
        // or just to be safe.
        const recentStr = localStorage.getItem(STORAGE_KEY) || '[]';
        let current: string[] = [];
        try {
            current = JSON.parse(recentStr);
        } catch {
            current = [];
        }

        const filtered = current.filter(p => p !== path);
        const newRecent = [path, ...filtered].slice(0, 5);
        
        recentProjects.value = newRecent;
        localStorage.setItem(STORAGE_KEY, JSON.stringify(newRecent));
    };

    const removeRecentProject = (path: string) => {
        const recentStr = localStorage.getItem(STORAGE_KEY) || '[]';
        let current: string[] = [];
        try {
            current = JSON.parse(recentStr);
        } catch {
            current = [];
        }

        const newRecent = current.filter(p => p !== path);
        recentProjects.value = newRecent;
        localStorage.setItem(STORAGE_KEY, JSON.stringify(newRecent));
    };

    // Initial load
    if (recentProjects.value.length === 0) {
        loadRecentProjects();
    }

    return {
        recentProjects,
        loadRecentProjects,
        addRecentProject,
        removeRecentProject
    };
}
