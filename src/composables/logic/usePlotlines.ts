import { useProjectData } from './useProjectData';
import type { Plotline } from '../../types';

// Default plotline colors for auto-assignment
const PLOTLINE_COLORS = [
    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
    '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1'
];

export function usePlotlines() {
    const { plotlines, updatePlotlines } = useProjectData();

    async function addPlotline(name: string) {
        const id = `plotline-${Date.now()}`;
        const colorIdx = plotlines.value.length % PLOTLINE_COLORS.length;
        const newPlotlines = [...plotlines.value, { id, name, color: PLOTLINE_COLORS[colorIdx] }];
        await updatePlotlines(newPlotlines);
        return id;
    }

    async function removePlotline(id: string) {
        const idx = plotlines.value.findIndex((p: Plotline) => p.id === id);
        if (idx >= 0) { 
            const newPlotlines = [...plotlines.value];
            newPlotlines.splice(idx, 1);
            await updatePlotlines(newPlotlines);
        }
    }

    async function updatePlotline(id: string, updates: Partial<Plotline>) {
        const newPlotlines = plotlines.value.map((p: Plotline) =>
            p.id === id ? { ...p, ...updates } : p
        );
        await updatePlotlines(newPlotlines);
    }

    return {
        addPlotline,
        removePlotline,
        updatePlotline
    };
}
