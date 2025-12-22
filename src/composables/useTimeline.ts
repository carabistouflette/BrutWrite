import { computed } from 'vue';
import { useProjectData } from './useProjectData';
import type { Chapter, TemporalScene } from '../types';
import { useTimeHelpers } from './logic/useTimeHelpers';
import { usePlotlines } from './logic/usePlotlines';
import { useParadoxDetection } from './logic/useParadoxDetection';

export function useTimeline() {
    const { activeId, selectNode, plotlines, updateNodeTemporal, flatNodes } = useProjectData();
    const { parseDurationToMillis, formatDurationFromMillis } = useTimeHelpers();
    const { addPlotline, removePlotline, updatePlotline } = usePlotlines();

    // Extract only temporal metadata for efficient paradox detection tracking
    const scenesTemporalData = computed<TemporalScene[]>(() => {
        return flatNodes.value.map(node => ({
            id: node.id,
            title: node.name,
            chronological_date: node.chronological_date,
            abstract_timeframe: node.abstract_timeframe,
            duration: node.duration,
            plotline_tag: node.plotline_tag,
            depends_on: node.depends_on,
            pov_character_id: node.pov_character_id,
        }));
    });

    const allChapters = computed<Chapter[]>(() => {
        return flatNodes.value.map((node, idx) => ({
            id: node.id,
            parent_id: undefined, 
            title: node.name || 'Untitled',
            filename: node.filename || '',
            word_count: node.word_count || 0,
            order: idx,
            chronological_date: node.chronological_date,
            abstract_timeframe: node.abstract_timeframe,
            duration: node.duration,
            plotline_tag: node.plotline_tag,
            depends_on: node.depends_on,
        }));
    });

    // Scenes that have temporal data (for timeline)
    const assignedScenes = computed(() =>
        scenesTemporalData.value.filter(c => c.chronological_date || c.abstract_timeframe)
    );

    // Scenes without temporal data (for holding pen)
    const unassignedScenes = computed(() =>
        allChapters.value.filter(c => !c.chronological_date && !c.abstract_timeframe)
    );

    const { paradoxWarnings } = useParadoxDetection(assignedScenes);

    // Export narrative connectors data (reading order lines)
    const narrativeConnectors = computed(() => {
        // Return pairs of scene IDs in reading order (manuscript order)
        // Only include scenes that are ASSIGNED to the timeline.
        const assignedIds = new Set(assignedScenes.value.map(s => s.id));
        const ordered = allChapters.value.filter((c) => assignedIds.has(c.id));
        
        const pairs: { from: string; to: string; isFlashback: boolean }[] = [];

        for (let i = 1; i < ordered.length; i++) {
            const from = ordered[i - 1];
            const to = ordered[i];
            
            // Check dates for flashback detection
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
