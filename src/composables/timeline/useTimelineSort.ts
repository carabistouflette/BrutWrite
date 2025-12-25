import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../stores/project';
import { useProjectNodeOperations } from '../logic/useProjectNodeOperations';
import type { FileNode } from '../../types';

export function useTimelineSort() {
  const projectStore = useProjectStore();
  const { nodes: projectData } = storeToRefs(projectStore);
  const { updateStructure } = useProjectNodeOperations();

  function sortNodesChronologically(nodes: FileNode[]): FileNode[] {
    return [...nodes]
      .sort((a, b) => {
        const dateA = a.chronological_date || a.abstract_timeframe || '';
        const dateB = b.chronological_date || b.abstract_timeframe || '';
        if (!dateA && !dateB) return 0;
        if (!dateA) return 1;
        if (!dateB) return -1;
        return dateA.localeCompare(dateB);
      })
      .map((node) => {
        if (node.children) {
          return { ...node, children: sortNodesChronologically(node.children) };
        }
        return node;
      });
  }

  async function applyChronologicalSort() {
    const sorted = sortNodesChronologically(projectData.value);
    await updateStructure(sorted);
    return true;
  }

  return {
    sortNodesChronologically,
    applyChronologicalSort,
  };
}
