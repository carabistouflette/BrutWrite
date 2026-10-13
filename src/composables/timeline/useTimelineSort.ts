import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../stores/project';
import { useProjectNodeOperations } from '../domain/project/useProjectNodeOperations';
import { parseAbstractTimeframe } from '../domain/timeline/useTimeHelpers';
import type { FileNode, TemporalScene } from '../../types';

type ChronologicalValue = Pick<TemporalScene, 'chronological_date' | 'abstract_timeframe'>;

function sortableTimestamp(value: ChronologicalValue): number | undefined {
  if (value.chronological_date) {
    const timestamp = Date.parse(value.chronological_date);
    return Number.isNaN(timestamp) ? undefined : timestamp;
  }

  if (value.abstract_timeframe && /\b(day|week|month|year)\s*\d+/i.test(value.abstract_timeframe)) {
    return parseAbstractTimeframe(value.abstract_timeframe).getTime();
  }
}

export function compareChronologicalOrder(a: ChronologicalValue, b: ChronologicalValue) {
  const labelA = a.chronological_date || a.abstract_timeframe || '';
  const labelB = b.chronological_date || b.abstract_timeframe || '';
  if (!labelA && !labelB) return 0;
  if (!labelA) return 1;
  if (!labelB) return -1;

  const timestampA = sortableTimestamp(a);
  const timestampB = sortableTimestamp(b);
  if (timestampA !== undefined && timestampB !== undefined && timestampA !== timestampB) {
    return timestampA - timestampB;
  }

  return labelA.localeCompare(labelB, undefined, { numeric: true, sensitivity: 'base' });
}

export function sortNodesChronologically(nodes: FileNode[]): FileNode[] {
  return [...nodes]
    .sort(compareChronologicalOrder)
    .map((node) =>
      node.children ? { ...node, children: sortNodesChronologically(node.children) } : node
    );
}

export function useTimelineSort() {
  const projectStore = useProjectStore();
  const { nodes: projectData } = storeToRefs(projectStore);
  const { updateStructure } = useProjectNodeOperations();

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
