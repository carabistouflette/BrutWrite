import type { FileNode } from "../../types";
import { projectApi } from "../../api/project";
import { useAppStatus } from "../ui/useAppStatus";
import { reconstructHierarchy } from "../../utils/tree";
import {
  projectData,
  projectId,
  activeId,
  nodeMap,
} from "../state/projectState";
import { useProjectSync } from "./useProjectSync";

export function useProjectNodeOperations() {
  const { notifyError } = useAppStatus();
  const { syncNodeMetadataDebounced, syncManifestDebounced } = useProjectSync();

  const addChapter = async () => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.createNode(
        projectId.value,
        undefined,
        "New Chapter"
      );
      projectData.value = reconstructHierarchy(metadata.manifest.chapters);

      const roots = projectData.value;
      if (roots.length > 0) {
        const newChapter = roots[roots.length - 1];
        activeId.value = newChapter.id;
        return newChapter.id;
      }
    } catch (e) {
      notifyError("Failed to create chapter", e);
    }
  };

  const addSection = async (parentId: string) => {
    if (!projectId.value) return;
    try {
      const metadata = await projectApi.createNode(
        projectId.value,
        parentId,
        "New Section"
      );
      projectData.value = reconstructHierarchy(metadata.manifest.chapters);
    } catch (e) {
      notifyError("Failed to create section", e);
    }
  };

  const deleteNode = async (id: string) => {
    if (!projectId.value) return;

    const node = nodeMap.value.get(id);
    if (!node) return;

    const collectFilenames = (n: FileNode, acc: string[]) => {
      if (n.filename) acc.push(n.filename);
      if (n.children) {
        n.children.forEach((child) => collectFilenames(child, acc));
      }
    };

    const filesToDelete: string[] = [];
    collectFilenames(node, filesToDelete);

    try {
      const metadata = await projectApi.deleteNode(
        projectId.value,
        id,
        filesToDelete
      );

      // Atomic sync from backend
      projectData.value = reconstructHierarchy(metadata.manifest.chapters);
      if (activeId.value === id) activeId.value = undefined;
    } catch (e) {
      notifyError(`Failed to delete node ${id}`, e);
    }
  };

  const renameNode = async (id: string, newName: string) => {
    const node = nodeMap.value.get(id);
    if (node && node.name !== newName) {
      node.name = newName;
      syncNodeMetadataDebounced(id, { title: newName });
    }
  };

  const updateStructure = async (newStructure: FileNode[]) => {
    projectData.value = newStructure;
    syncManifestDebounced();
  };

  const updateNodeStats = (
    id: string,
    wordCount: number,
    shouldSync = true
  ) => {
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
        "chronological_date",
        "abstract_timeframe",
        "duration",
        "plotline_tag",
        "depends_on",
        "pov_character_id",
      ] as const;

      let changed = false;
      const updateForBackend: Partial<FileNode> = {};

      for (const key of temporalKeys) {
        if (key in updates) {
          const newVal = updates[key];
          if (node[key] !== newVal) {
            node[key] = newVal;
            updateForBackend[key] = newVal;
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
