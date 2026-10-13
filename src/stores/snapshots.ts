import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref, watch } from 'vue';
import { useProjectStore } from './project';
import { reconstructHierarchy } from '../utils/tree';
import type { ProjectMetadata } from '../types';

export const useSnapshotStore = defineStore('snapshots', () => {
  const snapshots = ref<string[]>([]);
  const loading = ref(false);
  const projectStore = useProjectStore();
  let snapshotListRequest = 0;

  watch(
    () => projectStore.projectId,
    () => {
      snapshotListRequest++;
      snapshots.value = [];
      loading.value = false;
    }
  );

  async function fetchSnapshots(chapterId: string) {
    const projectId = projectStore.projectId;
    if (!projectId) {
      snapshots.value = [];
      loading.value = false;
      return;
    }

    const request = ++snapshotListRequest;
    loading.value = true;
    try {
      const result = await invoke<string[]>('list_snapshots', { projectId, chapterId });
      if (request === snapshotListRequest && projectStore.projectId === projectId) {
        snapshots.value = [...result].sort().reverse();
      }
    } catch (error) {
      console.error('Failed to fetch snapshots', error);
      if (request === snapshotListRequest && projectStore.projectId === projectId) {
        snapshots.value = [];
      }
    } finally {
      if (request === snapshotListRequest) loading.value = false;
    }
  }

  async function loadSnapshotContent(chapterId: string, filename: string): Promise<string> {
    const projectId = projectStore.projectId;
    if (!projectId) throw new Error('Cannot load a snapshot without an open project');
    try {
      return await invoke<string>('load_snapshot_content', {
        projectId,
        chapterId,
        filename,
      });
    } catch (error) {
      console.error('Failed to load snapshot content', error);
      throw error;
    }
  }

  async function createSnapshot(chapterId: string, content: string) {
    const projectId = projectStore.projectId;
    if (!projectId) throw new Error('Cannot create a snapshot without an open project');
    try {
      await invoke('create_snapshot', { projectId, chapterId, content });
      if (projectStore.projectId === projectId) await fetchSnapshots(chapterId);
    } catch (error) {
      console.error('Failed to create snapshot', error);
      throw error;
    }
  }

  async function restoreSnapshot(chapterId: string, snapshotFilename: string): Promise<string> {
    const projectId = projectStore.projectId;
    if (!projectId) throw new Error('Cannot restore a snapshot without an open project');
    try {
      const newContent = await invoke<string>('restore_snapshot', {
        projectId,
        chapterId,
        snapshotFilename,
      });
      if (projectStore.projectId === projectId) await fetchSnapshots(chapterId);
      return newContent;
    } catch (error) {
      console.error('Failed to restore snapshot', error);
      throw error;
    }
  }

  async function branchSnapshot(
    chapterId: string,
    snapshotFilename: string
  ): Promise<ProjectMetadata> {
    const projectId = projectStore.projectId;
    if (!projectId) throw new Error('Cannot branch a snapshot without an open project');
    const existingIds = new Set(projectStore.nodeMap.keys());
    try {
      const metadata = await invoke<ProjectMetadata>('branch_snapshot', {
        projectId,
        snapshotChapterId: chapterId,
        snapshotFilename,
      });
      if (projectStore.projectId === projectId) {
        projectStore.updateStructure(reconstructHierarchy(metadata.manifest.chapters));
        const branch = metadata.manifest.chapters.find((item) => !existingIds.has(item.id));
        if (branch) projectStore.setActiveId(branch.id);
      }
      return metadata;
    } catch (error) {
      console.error('Failed to branch snapshot', error);
      throw error;
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
