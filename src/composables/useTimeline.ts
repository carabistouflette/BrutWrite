// Default plotline colors for auto-assignment
const PLOTLINE_COLORS = [
    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
    '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1'
];

import { computed } from 'vue';
import { useProjectData } from './useProjectData';
import type { Chapter, Plotline, ParadoxWarning, FileNode } from '../types';

export function useTimeline() {
    const { projectData, activeId, selectNode, plotlines, updatePlotlines, updateNodeTemporal } = useProjectData();

    // Flatten all chapters/scenes from the tree
    const allChapters = computed<Chapter[]>(() => {
        const flatten = (nodes: FileNode[]): Chapter[] => {
            const result: Chapter[] = [];
            nodes.forEach((node, idx) => {
                const chapter: Chapter = {
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
                };
                result.push(chapter);
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
                povGroups.forEach((povScenes, povId) => {
                    if (povScenes.length > 1) {
                        // Only warn if they aren't the exact same scene (unlikely but possible with weird data)
                        // and if they are in arguably "different" locations? 
                        // Realistically just being in two scenes at once is the warning.
                        warnings.push({
                            type: 'simultaneous_presence',
                            sceneIds: povScenes.map(s => s.id),
                            message: `Character (${povId}) appears in multiple scenes at ${timeKey}`,
                        });
                    }
                });
            }
        });

        // 2. Causality violation detection
        const chronoSorted = [...assignedScenes.value].sort((a, b) => {
            const aTime = a.chronological_date || a.abstract_timeframe || '';
            const bTime = b.chronological_date || b.abstract_timeframe || '';
            // Handle ISO dates vs Strings? Simple string compare works for ISO.
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
            
            // Only check gaps between scenes on the SAME plotline? 
            // Or general gaps? The request mentioned "continuity threshold".
            // Let's check general global gaps for now.
            
            const gap = computeTimeGap(prev.chronological_date, curr.chronological_date);
            // Threshold: 3 years (~1095 days)
            if (gap && gap > 1095) { 
                warnings.push({
                    type: 'orphan_gap',
                    sceneIds: [prev.id, curr.id],
                    message: `Large time gap (${Math.floor(gap / 365)} years) between "${prev.title}" and "${curr.title}"`,
                });
            }
        }

        return warnings;
    });

    // Helper: parse duration string to MILLISECONDS
    function parseDurationToMillis(duration?: string): number {
        if (!duration) return 0;
        const lower = duration.toLowerCase().trim();
        const num = parseFloat(lower) || 0;
        
        // Multipliers in milliseconds
        const MINUTE = 60 * 1000;
        const HOUR = 60 * MINUTE;
        const DAY = 24 * HOUR;
        const WEEK = 7 * DAY;
        const MONTH = 30 * DAY; // approx
        const YEAR = 365 * DAY; // approx

        if (lower.includes('minute') || lower.includes('min')) return num * MINUTE;
        if (lower.includes('hour')) return num * HOUR;
        if (lower.includes('week')) return num * WEEK;
        if (lower.includes('month')) return num * MONTH;
        if (lower.includes('year')) return num * YEAR;
        // Default to days if "day" or just number? safest to assume days if explicitly 'day'
        if (lower.includes('day')) return num * DAY;
        
        return 0;
    }

    // Helper: compute gap in days between two ISO dates
    function computeTimeGap(date1?: string, date2?: string): number | null {
        if (!date1 || !date2) return null;
        try {
            const d1 = new Date(date1).getTime();
            const d2 = new Date(date2).getTime();
            if (isNaN(d1) || isNaN(d2)) return null;
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
        if (idx >= 0) { // Allowed to remove first if needed, but usually we keep one.
             // Actually let's allow removing any, but maybe warn if it's the last one?
             // Logic: remove and reassign scenes safely? 
             // For now just remove the definition.
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
        // Only include scenes that are ASSIGNED to the timeline.
        const assignedIds = new Set(assignedScenes.value.map(s => s.id));
        const ordered = allChapters.value.filter(c => assignedIds.has(c.id));
        
        const pairs: { from: string; to: string; isFlashback: boolean }[] = [];

        for (let i = 1; i < ordered.length; i++) {
            const from = ordered[i - 1];
            const to = ordered[i];
            
            // Check dates for flashback detection
            // Simple string compare allows '2023-01' < '2023-02'
            // But abstract "Day 1" < "Day 2" might fail string compare if not padded.
            // Let's assume chronological_date (ISO) for strict flashback checks for now.
            const isFlashback = !!(from.chronological_date && to.chronological_date && to.chronological_date < from.chronological_date);
            
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
        paradoxWarnings,
        narrativeConnectors,
        activeId,

        // Actions
        selectNode,
        addPlotline,
        removePlotline,
        updatePlotline,
        updateNodeTemporal,
        
        // Helpers
        parseDurationToMillis,
        formatDurationFromMillis
    };
}

// Helper: format milliseconds back to string
function formatDurationFromMillis(millis: number): string {
    if (!millis || millis <= 0) return '';
    
    const MINUTE = 60 * 1000;
    const HOUR = 60 * MINUTE;
    const DAY = 24 * HOUR;
    const WEEK = 7 * DAY;
    const MONTH = 30 * DAY; // approx
    const YEAR = 365 * DAY; // approx

    // Determine the best unit
    if (millis >= YEAR && millis % YEAR === 0) return `${millis / YEAR} years`;
    if (millis >= MONTH && millis % MONTH === 0) return `${millis / MONTH} months`;
    if (millis >= WEEK && millis % WEEK === 0) return `${millis / WEEK} weeks`;
    if (millis >= DAY && millis % DAY === 0) return `${millis / DAY} days`;
    if (millis >= HOUR && millis % HOUR === 0) return `${millis / HOUR} hours`;
    if (millis >= MINUTE && millis % MINUTE === 0) return `${millis / MINUTE} minutes`;
    
    // Fallback to days with decimals if needed, or just days
    const days = millis / DAY;
    if (days >= 1) return `${parseFloat(days.toFixed(2))} days`;
    
    return `${Math.floor(millis / MINUTE)} minutes`;
}
