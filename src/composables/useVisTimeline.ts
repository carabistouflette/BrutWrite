import { ref, onMounted, onUnmounted, watch, type Ref } from 'vue';
import { DataSet } from 'vis-data';
import { Timeline } from 'vis-timeline';
import 'vis-timeline/styles/vis-timeline-graph2d.min.css';
import { useCalendar } from './useCalendar';
import { useTimeline } from './useTimeline'; // We need some types/helpers from here, but be careful of circular deps if any

// Local type definitions for vis-timeline items
export interface VisTimelineItem {
    id: string;
    group?: string | number;
    content: string;
    start: Date;
    end?: Date;
    type?: string;
    className?: string;
    title?: string;
}

export function useVisTimeline(
    containerRef: Ref<HTMLElement | null> | any, // Vue ref
    onSelect: (sceneId: string | null) => void,
    onHover: (info: { id: string; x: number; y: number } | null) => void,
    onRangeChange: () => void
) {
    const timeline = ref<Timeline | null>(null);
    const items = new DataSet<any>([]);
    const groups = new DataSet<any>([]);
    const isMounted = ref(false);

    const { formatDate, getYear } = useCalendar();
    
    // We import these usage helpers here or pass them in? 
    // Passing project data in might be cleaner to avoid tight coupling inside this generic-ish composable
    // But for now let's reuse the ones available globally or passed as args if needed.
    // Actually, let's keep the data syncing logic INSIDE here to truly clean up the component.
    
    const {
        plotlines,
        assignedScenes,
        paradoxWarnings,
        updateNodeTemporal,
        parseDurationToMillis,
        formatDurationFromMillis
    } = useTimeline();

    onMounted(() => {
        isMounted.value = true;
        if (!containerRef.value) return;

        const options = {
            orientation: { axis: 'top', item: 'top' },
            stack: true, 
            showCurrentTime: false,
            zoomable: true,
            moveable: true,
            editable: {
                updateTime: true, // Allow resizing/dragging
                updateGroup: true, // Allow moving between swimlanes
                remove: false, // Use the editor to remove
                add: false,
            },
            groupOrder: 'id',
            margin: { item: { horizontal: 5, vertical: 20 } },
            snap: null,
            onMove: handleItemMove,
            format: {
                minorLabels: (date: Date, _scale: string, _step: number) => {
                        return formatDate(date);
                },
                majorLabels: (date: Date, _scale: string, _step: number) => {
                        // For major labels we often want just the Year
                        return getYear(date);
                }
            }
        };

        timeline.value = new Timeline(containerRef.value, items, groups, options);

        // Selection handler
        timeline.value.on('select', (props: { items: string[] }) => {
            if (props.items.length > 0) {
                onSelect(props.items[0]);
            } else {
                onSelect(null);
            }
        });

        // Hover handler
        timeline.value.on('itemover', (props: { item: string; event: MouseEvent }) => {
                onHover({
                id: props.item,
                x: props.event.clientX,
                y: props.event.clientY,
            });
        });

        timeline.value.on('itemout', () => {
            onHover(null);
        });
        
        // Range change handler
        timeline.value.on('rangechange', () => {
             onRangeChange();
        });

        // Initial sync
        syncData();
    });

    onUnmounted(() => {
        isMounted.value = false;
        timeline.value?.destroy();
        timeline.value = null;
    });

    // Watch for data changes
    watch([plotlines, assignedScenes], syncData, { deep: true });

    function syncData() {
        if (!isMounted.value || !timeline.value) return;
        // Sync groups
        const groupsData = plotlines.value.map(pl => ({
            id: pl.id,
            content: pl.name,
            style: `border-left: 4px solid ${pl.color}; background: linear-gradient(90deg, ${pl.color}10 0%, transparent 100%);`,
        }));
        
        const existingGroupIds = groups.getIds();
        const newGroupIds = groupsData.map((g: any) => g.id);
        const toRemoveGroups = existingGroupIds.filter((id: any) => !newGroupIds.includes(id));
        
        groups.remove(toRemoveGroups);
        groups.update(groupsData);
        
        if (groups.length === 0) {
            groups.add({ id: 'main', content: 'Main Plot' });
        }

        // Sync items
        const itemsData: VisTimelineItem[] = assignedScenes.value.map(scene => {
            const warnings = paradoxWarnings.value.filter(w => w.sceneIds.includes(scene.id));
            const hasWarning = warnings.length > 0;
            const start = scene.chronological_date
                ? new Date(scene.chronological_date)
                : parseAbstractTimeframe(scene.abstract_timeframe);
                
            let durationMillis = parseDurationToMillis(scene.duration);
            // Default to 2 hours if no duration, so it appears as a resizable range
            const isProjectedDuration = durationMillis <= 0;
            if (isProjectedDuration) {
                durationMillis = 1000 * 60 * 60 * 2; // 2 Hours default visualization
            }

            const end = new Date(start.getTime() + durationMillis);

            const group = scene.plotline_tag && groups.get(scene.plotline_tag) ? scene.plotline_tag : groups.getIds()[0];

            return {
                id: scene.id,
                group: group,
                content: `<span class="scene-item-content">${scene.title}</span>`,
                start,
                end,
                type: 'range', // Always range to allow resizing
                className: hasWarning ? 'timeline-item--warning' : (isProjectedDuration ? 'timeline-item--projected' : 'timeline-item--normal'),
                title: warnings.map(w => w.message).join('\n') || scene.title,
            };
        });

        const existingItemIds = items.getIds();
        const newItemIds = itemsData.map(i => i.id);
        const toRemoveItems = existingItemIds.filter((id: any) => !newItemIds.includes(id));

        items.remove(toRemoveItems);
        items.update(itemsData);
    }

    function parseAbstractTimeframe(timeframe?: string): Date {
        if (!timeframe) return new Date();
        const match = timeframe.match(/(day|week|month|year)\s*(\d+)/i);
        if (match) {
            const unit = match[1].toLowerCase();
            const num = parseInt(match[2], 10);
            const base = new Date('2000-01-01');

            if (unit === 'day') base.setDate(base.getDate() + num - 1);
            else if (unit === 'week') base.setDate(base.getDate() + (num - 1) * 7);
            else if (unit === 'month') base.setMonth(base.getMonth() + num - 1);
            else if (unit === 'year') base.setFullYear(base.getFullYear() + num - 1);

            return base;
        }
        return new Date();
    }

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
    
    function fit() {
        timeline.value?.fit();
    }
    
    function zoomIn(percentage: number) {
        timeline.value?.zoomIn(percentage);
    }
    
    function zoomOut(percentage: number) {
        timeline.value?.zoomOut(percentage);
    }

    return {
        timeline,
        fit,
        zoomIn,
        zoomOut,
        handleDrop,
        isMounted
    };
}
