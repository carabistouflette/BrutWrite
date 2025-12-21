// Default plotline colors for auto-assignment
const PLOTLINE_COLORS = [
    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
    '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1'
];

import { computed } from 'vue';
import { useProjectData } from './useProjectData';
import type { Chapter, Plotline, ParadoxWarning, TimelineScene, FileNode } from '../types';

export function useTimeline() {
    const { projectData, activeId, selectNode, plotlines, updatePlotlines } = useProjectData();

    // Flatten all chapters/scenes from the tree
    const allChapters = computed<Chapter[]>(() => {
        const flatten = (nodes: FileNode[]): Chapter[] => {
            const result: Chapter[] = [];
            nodes.forEach((node, idx) => {
                result.push({
                    id: node.id,
                    title: node.name || 'Untitled',
                    filename: node.filename || '',
                    word_count: node.word_count || 0,
                    order: idx,
                    // Temporal fields populated from backend if present
                    chronological_date: node.chronological_date,
                    abstract_timeframe: node.abstract_timeframe,
                    duration: node.duration,
                    plotline_tag: node.plotline_tag,
                    depends_on: node.depends_on,
                    pov_character_id: node.pov_character_id,
                });
                if (node.children?.length) {
                    result.push(...flatten(node.children));
                }
            });
            return result;
        };
        return flatten(projectData.value);
    });

    // Scenes that have temporal data (for timeline)
    const assignedScenes = computed(() =>
        allChapters.value.filter(c => c.chronological_date || c.abstract_timeframe)
    );

    // Scenes without temporal data (for holding pen)
    const unassignedScenes = computed(() =>
        allChapters.value.filter(c => !c.chronological_date && !c.abstract_timeframe)
    );

    // Convert assigned scenes to timeline positions
    const timelineScenes = computed<TimelineScene[]>(() => {
        return assignedScenes.value.map((chapter, idx) => {
            const plotlineIdx = plotlines.value.findIndex((p: Plotline) => p.id === chapter.plotline_tag);
            return {
                chapter,
                x: idx * 150, // Basic positioning, refined by vis-timeline
                width: parseDurationToPixels(chapter.duration),
                plotlineLaneY: plotlineIdx >= 0 ? plotlineIdx : 0,
            };
        });
    });

    // Paradox detection
    const paradoxWarnings = computed<ParadoxWarning[]>(() => {
        const warnings: ParadoxWarning[] = [];

        // 1. Simultaneous presence detection
        const byTime = new Map<string, Chapter[]>();
        assignedScenes.value.forEach(scene => {
            const timeKey = scene.chronological_date || scene.abstract_timeframe || '';
            if (timeKey) {
                if (!byTime.has(timeKey)) byTime.set(timeKey, []);
                byTime.get(timeKey)!.push(scene);
            }
        });

        byTime.forEach((scenes, timeKey) => {
            if (scenes.length > 1) {
                const povGroups = new Map<string, Chapter[]>();
                scenes.forEach(s => {
                    if (s.pov_character_id) {
                        if (!povGroups.has(s.pov_character_id)) povGroups.set(s.pov_character_id, []);
                        povGroups.get(s.pov_character_id)!.push(s);
                    }
                });
                povGroups.forEach((povScenes) => {
                    if (povScenes.length > 1) {
                        const uniquePlotlines = new Set(povScenes.map(s => s.plotline_tag).filter(Boolean));
                        if (uniquePlotlines.size > 1) {
                            warnings.push({
                                type: 'simultaneous_presence',
                                sceneIds: povScenes.map(s => s.id),
                                message: `Character appears in multiple locations at ${timeKey}`,
                            });
                        }
                    }
                });
            }
        });

        // 2. Causality violation detection
        const chronoSorted = [...assignedScenes.value].sort((a, b) => {
            const aTime = a.chronological_date || a.abstract_timeframe || '';
            const bTime = b.chronological_date || b.abstract_timeframe || '';
            return aTime.localeCompare(bTime);
        });

        chronoSorted.forEach(scene => {
            if (scene.depends_on) {
                const dependency = chronoSorted.find(s => s.id === scene.depends_on);
                if (dependency) {
                    const sceneTime = scene.chronological_date || scene.abstract_timeframe || '';
                    const depTime = dependency.chronological_date || dependency.abstract_timeframe || '';
                    if (sceneTime && depTime && sceneTime < depTime) {
                        warnings.push({
                            type: 'causality_violation',
                            sceneIds: [scene.id, dependency.id],
                            message: `"${scene.title}" occurs before its cause "${dependency.title}"`,
                        });
                    }
                }
            }
        });

        // 3. Orphan gap detection (large time gaps)
        for (let i = 1; i < chronoSorted.length; i++) {
            const prev = chronoSorted[i - 1];
            const curr = chronoSorted[i];
            const gap = computeTimeGap(prev.chronological_date, curr.chronological_date);
            if (gap && gap > 365 * 3) { // 3 year gap threshold
                warnings.push({
                    type: 'orphan_gap',
                    sceneIds: [prev.id, curr.id],
                    message: `Large time gap (${Math.floor(gap / 365)} years) between "${prev.title}" and "${curr.title}"`,
                });
            }
        }

        return warnings;
    });

    // Helper: parse duration string to approximate pixel width
    function parseDurationToPixels(duration?: string): number {
        if (!duration) return 50; // default width
        const lower = duration.toLowerCase();
        if (lower.includes('minute')) {
            const num = parseInt(lower) || 30;
            return Math.max(30, num / 2);
        }
        if (lower.includes('hour')) {
            const num = parseInt(lower) || 1;
            return Math.max(50, num * 30);
        }
        if (lower.includes('day')) {
            const num = parseInt(lower) || 1;
            return Math.max(80, num * 50);
        }
        return 50;
    }

    // Helper: compute gap in days between two ISO dates
    function computeTimeGap(date1?: string, date2?: string): number | null {
        if (!date1 || !date2) return null;
        try {
            const d1 = new Date(date1).getTime();
            const d2 = new Date(date2).getTime();
            return Math.abs(d2 - d1) / (1000 * 60 * 60 * 24);
        } catch {
            return null;
        }
    }

    // Actions
    async function addPlotline(name: string) {
        const id = `plotline-${Date.now()}`;
        const colorIdx = plotlines.value.length % PLOTLINE_COLORS.length;
        const newPlotlines = [...plotlines.value, { id, name, color: PLOTLINE_COLORS[colorIdx] }];
        await updatePlotlines(newPlotlines);
        return id;
    }

    async function removePlotline(id: string) {
        const idx = plotlines.value.findIndex((p: Plotline) => p.id === id);
        if (idx > 0) { // Don't remove default 'main' 
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

    // Export narrative connectors data (reading order lines)
    const narrativeConnectors = computed(() => {
        // Return pairs of scene IDs in reading order (manuscript order)
        const ordered = [...allChapters.value];
        const pairs: { from: string; to: string; isFlashback: boolean }[] = [];

        for (let i = 1; i < ordered.length; i++) {
            const from = ordered[i - 1];
            const to = ordered[i];
            const fromTime = from.chronological_date || from.abstract_timeframe || '';
            const toTime = to.chronological_date || to.abstract_timeframe || '';
            const isFlashback = !!(fromTime && toTime && toTime < fromTime);
            pairs.push({ from: from.id, to: to.id, isFlashback });
        }

        return pairs;
    });

    return {
        // State
        plotlines,
        allChapters,
        assignedScenes,
        unassignedScenes,
        timelineScenes,
        paradoxWarnings,
        narrativeConnectors,
        activeId,

        // Actions
        selectNode,
        addPlotline,
        removePlotline,
        updatePlotline,
    };
}
