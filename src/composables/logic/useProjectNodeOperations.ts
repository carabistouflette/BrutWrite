import type { FileNode } from '../../types';
import { projectApi } from '../../api/project';
import { useAppStatus } from '../ui/useAppStatus';
import { reconstructHierarchy } from '../../utils/tree';
import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../stores/project';
import { useProjectSync } from './useProjectSync';

export function useProjectNodeOperations() {
  const { notifyError } = useAppStatus();
  const { syncNodeMetadataDebounced, syncManifestDebounced } = useProjectSync();
  const projectStore = useProjectStore();
  const { nodes: projectData, projectId, activeId, nodeMap } = storeToRefs(projectStore);

  const addChapter = async () => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.createNode(projectId.value, undefined, 'New Chapter');
      const newChapters = reconstructHierarchy(metadata.manifest.chapters);

      // Update store
      projectStore.updateStructure(newChapters);

      const roots = projectData.value;
      if (roots.length > 0) {
        const newChapter = roots[roots.length - 1];
        projectStore.setActiveId(newChapter.id);
        return newChapter.id;
      }
    } catch (e) {
      notifyError('Failed to create chapter', e);
    }
  };

  const addSection = async (parentId: string) => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.createNode(projectId.value, parentId, 'New Section');
      projectStore.updateStructure(reconstructHierarchy(metadata.manifest.chapters));
    } catch (e) {
      notifyError('Failed to create section', e);
    }
  };

  const deleteNode = async (id: string) => {
    if (!projectId.value) return;

    try {
      const metadata = await projectApi.deleteNode(projectId.value, id);

      // Atomic sync from backend
      projectStore.updateStructure(reconstructHierarchy(metadata.manifest.chapters));
      if (activeId.value === id) projectStore.setActiveId(undefined);
    } catch (e) {
      notifyError(`Failed to delete node ${id}`, e);
    }
  };

  const renameNode = async (id: string, newName: string) => {
    const node = nodeMap.value.get(id);
    if (node && node.name !== newName) {
      // Direct mutation of the node in the store is fine as it's a ref,
      // but strictly we might want an action. For now, matching previous behavior.
      node.name = newName;
      syncNodeMetadataDebounced(id, { title: newName });
    }
  };

  const updateStructure = async (newStructure: FileNode[]) => {
    projectStore.updateStructure(newStructure);
    syncManifestDebounced();
  };

  const updateNodeStats = (id: string, wordCount: number, shouldSync = true) => {
    const node = nodeMap.value.get(id);
    if (node && node.word_count !== wordCount) {
      node.word_count = wordCount;
      // Only sync if requested (avoid echo-syncs when loading from backend)
      if (shouldSync) {
        syncNodeMetadataDebounced(id, { word_count: wordCount });
      }
    }
  };

  const updateNodeTemporal = (id: string, updates: Partial<FileNode>) => {
    const node = nodeMap.value.get(id);
    if (node) {
      const temporalKeys = [
        'chronological_date',
        'abstract_timeframe',
        'duration',
        'plotline_tag',
        'depends_on',
        'pov_character_id',
      ] as const;

      let changed = false;
      const updateForBackend: Partial<FileNode> = {};

      for (const key of temporalKeys) {
        if (key in updates) {
          const newVal = updates[key];
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          if ((node as any)[key] !== newVal) {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (node as any)[key] = newVal;
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (updateForBackend as any)[key] = newVal;
            changed = true;
          }
        }
      }

      if (changed) {
        syncNodeMetadataDebounced(id, updateForBackend);
      }
    }
  };

  return {
    addChapter,
    addSection,
    deleteNode,
    renameNode,
    updateStructure,
    updateNodeStats,
    updateNodeTemporal,
  };
}
