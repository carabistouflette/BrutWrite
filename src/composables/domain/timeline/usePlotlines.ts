import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { useProjectIO } from '../project/useProjectIO';
import type { Plotline } from '../../../types';

// Default plotline colors for auto-assignment
const PLOTLINE_COLORS = [
  '#3b82f6',
  '#10b981',
  '#f59e0b',
  '#ef4444',
  '#8b5cf6',
  '#ec4899',
  '#06b6d4',
  '#84cc16',
  '#f97316',
  '#6366f1',
];

export function usePlotlines() {
  const projectStore = useProjectStore();
  const { plotlines } = storeToRefs(projectStore);
  const { updatePlotlines } = useProjectIO();

  async function addPlotline(name: string) {
    const id = `plotline-${Date.now()}`;
    const colorIdx = (plotlines.value?.length || 0) % PLOTLINE_COLORS.length;
    const newPlotlines = [
      ...(plotlines.value || []),
      { id, name, color: PLOTLINE_COLORS[colorIdx] },
    ];
    await updatePlotlines(newPlotlines);
    return id;
  }

  async function updatePlotline(id: string, updates: Partial<Plotline>) {
    if (!plotlines.value) return;
    const newPlotlines = plotlines.value.map((p: Plotline) =>
      p.id === id ? { ...p, ...updates } : p
    );
    await updatePlotlines(newPlotlines);
  }

  return {
    plotlines,
    addPlotline,
    updatePlotlines,
    updatePlotline,
  };
}
