import { type Ref } from 'vue';
import { Timeline } from 'vis-timeline';
import { useTimeline } from '../useTimeline';
import { useTimeHelpers } from '../logic/useTimeHelpers';
import { DataSet } from 'vis-data';

export function useVisTimelineEvents(
    timeline: Ref<Timeline | null>,
    groups: DataSet<any>
) {
    const { updateNodeTemporal } = useTimeline();
    const { formatDurationFromMillis } = useTimeHelpers();

    async function handleItemMove(item: any, callback: (item: any) => void) {
        try {
            const updates: any = {
                chronological_date: item.start.toISOString(),
                plotline_tag: item.group
            };

            if (item.end) {
                const startMs = item.start.getTime();
                const endMs = item.end.getTime();
                const durationMillis = endMs - startMs;
                
                // Only update duration if it's positive
                if (durationMillis > 0) {
                    updates.duration = formatDurationFromMillis(durationMillis);
                }
            }

            await updateNodeTemporal(item.id, updates);
            callback(item);
        } catch (e) {
            console.error('Failed to move scene:', e);
            callback(null);
        }
    }

    async function handleDrop(event: DragEvent) {
        const sceneId = event.dataTransfer?.getData('text/plain');
        if (!sceneId || !timeline.value) return;

        const props = (timeline.value as any).getEventProperties(event);
        const time = props.time;
        let targetGroup = props.group;
        
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
