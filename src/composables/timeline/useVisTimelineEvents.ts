import { type Ref } from 'vue';
import { Timeline } from 'vis-timeline';
import { useTimeline } from '../useTimeline';
import { useTimeHelpers } from '../logic/useTimeHelpers';
import { DataSet } from 'vis-data';
import { type VisTimelineItem, type VisTimelineGroup } from './useVisTimelineData';

export function useVisTimelineEvents(
    timeline: Ref<Timeline | null>,
    groups: DataSet<VisTimelineGroup>
) {
    const { updateNodeTemporal } = useTimeline();
    const { formatDurationFromMillis } = useTimeHelpers();

    async function handleItemMove(item: VisTimelineItem, callback: (item: VisTimelineItem | null) => void) {
        try {
            const startDate = item.start instanceof Date ? item.start : new Date(item.start);
            const updates: Record<string, string | undefined> = {
                chronological_date: startDate.toISOString(),
                plotline_tag: item.group ? String(item.group) : undefined
            };

            if (item.end) {
                const endDate = item.end instanceof Date ? item.end : new Date(item.end);
                const startMs = startDate.getTime();
                const endMs = endDate.getTime();
                const durationMillis = endMs - startMs;
                
                // Only update duration if it's positive
                if (durationMillis > 0) {
                    updates.duration = formatDurationFromMillis(durationMillis);
                }
            }

            await updateNodeTemporal(String(item.id), updates);
            callback(item);
        } catch (e) {
            console.error('Failed to move scene:', e);
            callback(null);
        }
    }

    async function handleDrop(event: DragEvent) {
        const sceneId = event.dataTransfer?.getData('text/plain');
        if (!sceneId || !timeline.value) return;

        const props = timeline.value.getEventProperties(event);
        const time = props.time;
        let targetGroup: string | number | null | undefined = props.group;
        
        if (!targetGroup && groups.length > 0) {
            targetGroup = groups.getIds()[0];
        }

        if (time) {
            await updateNodeTemporal(sceneId, {
                chronological_date: time.toISOString(),
                plotline_tag: String(targetGroup)
            });
        }
    }

    return {
        handleItemMove,
        handleDrop
    };
}
