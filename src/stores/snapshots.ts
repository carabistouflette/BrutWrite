import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { useProjectStore } from './project';

export const useSnapshotStore = defineStore('snapshots', () => {
  const snapshots = ref<string[]>([]);
  const loading = ref(false);
  const projectStore = useProjectStore();

  async function fetchSnapshots(chapterId: string) {
    if (!projectStore.projectId) return;
    loading.value = true;
    try {
      const result = await invoke<string[]>('list_snapshots', {
        projectId: projectStore.projectId,
        chapterId,
      });
      // Sort descending (newest first)
      snapshots.value = result.sort().reverse();
    } catch (e) {
      console.error('Failed to fetch snapshots', e);
      snapshots.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function loadSnapshotContent(chapterId: string, filename: string): Promise<string> {
    if (!projectStore.projectId) return '';
    try {
      return await invoke<string>('load_snapshot_content', {
        projectId: projectStore.projectId,
        chapterId,
        filename,
      });
    } catch (e) {
      console.error('Failed to load snapshot content', e);
      throw e;
    }
  }

  return {
    snapshots,
    loading,
    fetchSnapshots,
    loadSnapshotContent,
  };
});
