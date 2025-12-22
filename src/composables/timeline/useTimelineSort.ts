import type { FileNode } from '../../types';
import { useProjectData } from '../logic/useProjectData';

export function useTimelineSort() {
    const { projectData, updateStructure } = useProjectData();

    function sortNodesChronologically(nodes: FileNode[]): FileNode[] {
        return [...nodes].sort((a, b) => {
            const dateA = a.chronological_date || a.abstract_timeframe || '';
            const dateB = b.chronological_date || b.abstract_timeframe || '';
            if (!dateA && !dateB) return 0;
            if (!dateA) return 1;
            if (!dateB) return -1;
            return dateA.localeCompare(dateB);
        }).map(node => {
            if (node.children) {
                return { ...node, children: sortNodesChronologically(node.children) };
            }
            return node;
        });
    }

    async function applyChronologicalSort() {
        if (confirm('This will reorder your manuscript chapters based on their chronological time. This cannot be undone easily. Continue?')) {
            const sorted = sortNodesChronologically(projectData.value);
            await updateStructure(sorted);
            return true;
        }
        return false;
    }

    return {
        sortNodesChronologically,
        applyChronologicalSort
    };
}
