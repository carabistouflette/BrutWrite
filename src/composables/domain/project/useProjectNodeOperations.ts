import { storeToRefs } from 'pinia';
import { chaptersApi } from '../../../api/chapters';
import { buildNodeMap, reconstructHierarchy } from '../../../utils/tree';
import type { Chapter, FileNode } from '../../../types';
import { useProjectStore } from '../../../stores/project';
import { useProjectSync } from './useProjectSync';
import { flushChapterSessions } from './useChapterSession';

const findParentId = (nodes: FileNode[], id: string, parentId?: string): string | undefined => {
  for (const node of nodes) {
    if (node.id === id) return parentId;
    if (node.children) {
      const found = findParentId(node.children, id, node.id);
      if (found !== undefined) return found;
    }
  }
  return undefined;
};

const removeNode = (nodes: FileNode[], id: string): FileNode[] => {
  const index = nodes.findIndex((node) => node.id === id);
  if (index !== -1) return [...nodes.slice(0, index), ...nodes.slice(index + 1)];

  for (let index = 0; index < nodes.length; index += 1) {
    const node = nodes[index];
    if (!node.children) continue;
    const nextChildren = removeNode(node.children, id);
    if (nextChildren !== node.children) {
      const nextNodes = [...nodes];
      nextNodes[index] = { ...node, children: nextChildren };
      return nextNodes;
    }
  }
  return nodes;
};

const appendChildren = (nodes: FileNode[], parentId: string, children: FileNode[]): FileNode[] => {
  const index = nodes.findIndex((node) => node.id === parentId);
  if (index !== -1) {
    const nextNodes = [...nodes];
    const parent = nodes[index];
    nextNodes[index] = { ...parent, children: [...(parent.children || []), ...children] };
    return nextNodes;
  }

  for (let index = 0; index < nodes.length; index += 1) {
    const node = nodes[index];
    if (!node.children) continue;
    const nextChildren = appendChildren(node.children, parentId, children);
    if (nextChildren !== node.children) {
      const nextNodes = [...nodes];
      nextNodes[index] = { ...node, children: nextChildren };
      return nextNodes;
    }
  }
  return nodes;
};

const getSubtreeIds = (node: FileNode): Set<string> => {
  const ids = new Set<string>();
  const stack = [node];
  while (stack.length > 0) {
    const current = stack.pop()!;
    ids.add(current.id);
    if (current.children) {
      for (const child of current.children) stack.push(child);
    }
  }
  return ids;
};

const getCreatedNodes = (
  chapters: Chapter[],
  parentId: string | undefined,
  existingIds: Set<string>
): FileNode[] => {
  const responseNodes = buildNodeMap(reconstructHierarchy(chapters));
  return chapters
    .filter(
      (chapter) =>
        (parentId === undefined ? !chapter.parent_id : chapter.parent_id === parentId) &&
        !existingIds.has(chapter.id)
    )
    .map((chapter) => responseNodes.get(chapter.id))
    .filter((node): node is FileNode => node !== undefined);
};

export function useProjectNodeOperations() {
  const { syncNodeMetadataDebounced, syncManifestDebounced, flush } = useProjectSync();
  const projectStore = useProjectStore();
  const { nodes: projectData, projectId, activeId, nodeMap } = storeToRefs(projectStore);

  const addChapter = async () => {
    const currentProjectId = projectId.value;
    if (!currentProjectId) return;

    await flush(currentProjectId);
    if (projectId.value !== currentProjectId) return;

    const existingIds = new Set(nodeMap.value.keys());
    const metadata = await chaptersApi.createNode(currentProjectId, undefined, 'New Chapter');
    if (projectId.value !== currentProjectId) return;
    for (const id of nodeMap.value.keys()) existingIds.add(id);

    const newChapters = getCreatedNodes(metadata.manifest.chapters, undefined, existingIds);
    if (newChapters.length === 0) return;

    projectStore.updateStructure([...projectData.value, ...newChapters]);
    const newChapter = newChapters[newChapters.length - 1];
    projectStore.setActiveId(newChapter.id);
    syncManifestDebounced();
    return newChapter.id;
  };

  const addSection = async (parentId: string) => {
    const currentProjectId = projectId.value;
    if (!currentProjectId) return;

    await flush(currentProjectId);
    if (projectId.value !== currentProjectId || !nodeMap.value.has(parentId)) return;

    const existingIds = new Set(nodeMap.value.keys());
    const metadata = await chaptersApi.createNode(currentProjectId, parentId, 'New Section');
    if (projectId.value !== currentProjectId) return;
    for (const id of nodeMap.value.keys()) existingIds.add(id);

    const newSections = getCreatedNodes(metadata.manifest.chapters, parentId, existingIds);
    if (newSections.length === 0) return;

    const nextStructure = appendChildren(projectData.value, parentId, newSections);
    if (nextStructure === projectData.value) return;

    projectStore.updateStructure(nextStructure);
    projectStore.setActiveId(newSections[newSections.length - 1].id);
    syncManifestDebounced();
    return newSections[newSections.length - 1].id;
  };

  const deleteNode = async (id: string) => {
    const currentProjectId = projectId.value;
    if (!currentProjectId) return;

    const node = nodeMap.value.get(id);
    if (!node) return;

    await flushChapterSessions(projectStore);
    if (projectId.value !== currentProjectId) return;
    await flush(currentProjectId);
    if (projectId.value !== currentProjectId) return;

    const parentId = findParentId(projectData.value, id);
    const deletedIds = getSubtreeIds(node);

    await chaptersApi.deleteNode(currentProjectId, id);
    if (projectId.value !== currentProjectId) return;

    const nextStructure = removeNode(projectData.value, id);
    if (nextStructure !== projectData.value) {
      projectStore.updateStructure(nextStructure);
      for (const survivingNode of projectStore.flatNodes) {
        if (survivingNode.depends_on && deletedIds.has(survivingNode.depends_on)) {
          projectStore.updateNodeMetadataAction(survivingNode.id, { depends_on: undefined });
        }
      }
      syncManifestDebounced();
    }

    if (activeId.value && deletedIds.has(activeId.value)) {
      const fallbackId =
        parentId && projectStore.nodeMap.has(parentId) ? parentId : nextStructure[0]?.id;
      projectStore.setActiveId(fallbackId);
    }
  };

  const renameNode = async (id: string, newName: string) => {
    const node = nodeMap.value.get(id);
    if (node && node.name !== newName) {
      projectStore.renameNodeAction(id, newName);
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
      projectStore.updateNodeStatsAction(id, wordCount);
      if (shouldSync) syncNodeMetadataDebounced(id, { word_count: wordCount });
    }
  };

  const updateNodeTemporal = (id: string, updates: Partial<FileNode>) => {
    const node = nodeMap.value.get(id);
    if (!node) return;

    const temporalKeys = [
      'chronological_date',
      'abstract_timeframe',
      'duration',
      'plotline_tag',
      'depends_on',
      'pov_character_id',
    ] as const;

    const updateForBackend: Partial<FileNode> = {};
    for (const key of temporalKeys) {
      if (key in updates && node[key] !== updates[key]) {
        updateForBackend[key] = updates[key];
      }
    }

    if (Object.keys(updateForBackend).length > 0) {
      projectStore.updateNodeMetadataAction(id, updateForBackend);
      syncNodeMetadataDebounced(id, updateForBackend);
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
