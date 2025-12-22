import { computed, type Ref } from 'vue';
import type { ParadoxWarning, TemporalScene } from '../../types';
import { computeTimeGap } from './useTimeHelpers';

export function useParadoxDetection(assignedScenes: Ref<TemporalScene[]>) {
    
    // Paradox detection - now only re-runs if temporal metadata changes
    const paradoxWarnings = computed<ParadoxWarning[]>(() => {
        const warnings: ParadoxWarning[] = [];
        const activeScenes = assignedScenes.value;

        if (activeScenes.length === 0) return [];

        // 1. Simultaneous presence detection
        const byTime = new Map<string, TemporalScene[]>();
        activeScenes.forEach(scene => {
            const timeKey = scene.chronological_date || scene.abstract_timeframe || '';
            if (timeKey) {
                if (!byTime.has(timeKey)) byTime.set(timeKey, []);
                byTime.get(timeKey)!.push(scene);
            }
        });

        byTime.forEach((scenes, timeKey) => {
            if (scenes.length > 1) {
                const povGroups = new Map<string, TemporalScene[]>();
                scenes.forEach(s => {
                    if (s.pov_character_id) {
                        if (!povGroups.has(s.pov_character_id)) povGroups.set(s.pov_character_id, []);
                        povGroups.get(s.pov_character_id)!.push(s);
                    }
                });
                povGroups.forEach((povScenes, povId) => {
                    if (povScenes.length > 1) {
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
        const sceneMap = new Map<string, TemporalScene>();
        activeScenes.forEach(s => sceneMap.set(s.id, s));

        const chronoSorted = [...activeScenes].sort((a, b) => {
            const aTime = a.chronological_date || a.abstract_timeframe || '';
            const bTime = b.chronological_date || b.abstract_timeframe || '';
            return aTime.localeCompare(bTime);
        });

        chronoSorted.forEach(scene => {
            if (scene.depends_on) {
                const dependency = sceneMap.get(scene.depends_on);
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

    return {
        paradoxWarnings
    };
}
