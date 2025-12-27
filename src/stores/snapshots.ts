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

  async function createSnapshot(chapterId: string, content: string) {
    if (!projectStore.projectId) return;
    try {
      await invoke('create_snapshot', {
        projectId: projectStore.projectId,
        chapterId,
        content,
      });
      await fetchSnapshots(chapterId);
    } catch (e) {
      console.error('Failed to create snapshot', e);
      throw e;
    }
  }

  async function restoreSnapshot(chapterId: string, snapshotFilename: string) {
    if (!projectStore.projectId) return;
    try {
      const newContent = await invoke<string>('restore_snapshot', {
        projectId: projectStore.projectId,
        chapterId,
        snapshotFilename,
      });
      await fetchSnapshots(chapterId);
      return newContent;
    } catch (e) {
      console.error('Failed to restore snapshot', e);
      throw e;
    }
  }

  async function branchSnapshot(chapterId: string, snapshotFilename: string) {
    if (!projectStore.projectId) return;
    try {
      await invoke('branch_snapshot', {
        projectId: projectStore.projectId,
        snapshotChapterId: chapterId,
        snapshotFilename,
      });
      await projectStore.loadProject(projectStore.projectId);
    } catch (e) {
      console.error('Failed to branch snapshot', e);
      throw e;
    }
  }

  return {
    snapshots,
    loading,
    fetchSnapshots,
    loadSnapshotContent,
    createSnapshot,
    restoreSnapshot,
    branchSnapshot,
  };
});
