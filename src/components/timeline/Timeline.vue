<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { DataSet } from 'vis-data';
import { Timeline } from 'vis-timeline';
import 'vis-timeline/styles/vis-timeline-graph2d.min.css';
import { useTimeline } from '../../composables/useTimeline';
import HoldingPen from './HoldingPen.vue';
import ScenePopover from './ScenePopover.vue';
import TimelineControls from './TimelineControls.vue';

// Local type definitions for vis-timeline items
interface VisTimelineItem {
    id: string;
    group?: string;
    content: string;
    start: Date;
    end?: Date;
    type?: string;
    className?: string;
    title?: string;
}

const {
    plotlines,
    assignedScenes,
    unassignedScenes,
    paradoxWarnings,
    narrativeConnectors,
    selectNode,
    updateNodeTemporal,
} = useTimeline();

// Refs
const containerRef = ref<HTMLElement | null>(null);
const timeline = ref<Timeline | null>(null);
const showNarrativeConnectors = ref(false);
const hoveredScene = ref<{ id: string; x: number; y: number } | null>(null);

// vis-timeline datasets (using any to avoid type incompatibility between vis-data and vis-timeline)
const items = new DataSet<any>([]);
const groups = new DataSet<any>([]);

// Initialize timeline
onMounted(() => {
    if (!containerRef.value) return;

    const options = {
        orientation: { axis: 'top', item: 'top' },
        stack: false,
        showCurrentTime: false,
        zoomable: true,
        moveable: true,
        editable: {
            updateTime: true,
            updateGroup: true,
            remove: false,
            add: false,
        },
        groupOrder: 'id',
        margin: { item: { horizontal: 5, vertical: 10 } },
        snap: null, // Allow free positioning
        onMove: handleItemMove,
    };

    timeline.value = new Timeline(containerRef.value, items, groups, options);

    // Selection handler
    timeline.value.on('select', (props: { items: string[] }) => {
        if (props.items.length > 0) {
            selectNode(props.items[0]);
        }
    });

    // Hover handler for popover
    timeline.value.on('itemover', (props: { item: string; event: MouseEvent }) => {
        hoveredScene.value = {
            id: props.item,
            x: props.event.clientX,
            y: props.event.clientY,
        };
    });

    timeline.value.on('itemout', () => {
        hoveredScene.value = null;
    });

    // Initial sync
    syncData();
});

onUnmounted(() => {
    timeline.value?.destroy();
});

// Watch for data changes
watch([plotlines, assignedScenes], syncData, { deep: true });

function syncData() {
    // Sync groups (plotlines/swimlanes)
    const groupsData = plotlines.value.map(pl => ({
        id: pl.id,
        content: pl.name,
        style: `border-left: 4px solid ${pl.color}; background: ${pl.color}15;`,
    }));
    groups.clear();
    groups.add(groupsData);

    // Sync items (scenes)
    const itemsData: VisTimelineItem[] = assignedScenes.value.map(scene => {
        const warnings = paradoxWarnings.value.filter(w => w.sceneIds.includes(scene.id));
        const hasWarning = warnings.length > 0;
        const start = scene.chronological_date
            ? new Date(scene.chronological_date)
            : parseAbstractTimeframe(scene.abstract_timeframe);
        const end = computeEndTime(start, scene.duration);

        return {
            id: scene.id,
            group: scene.plotline_tag || 'main',
            content: scene.title,
            start,
            end,
            type: scene.duration ? 'range' : 'box',
            className: hasWarning ? 'timeline-item--warning' : '',
            title: warnings.map(w => w.message).join('\n') || scene.title,
        };
    });

    items.clear();
    items.add(itemsData);
}

function parseAbstractTimeframe(timeframe?: string): Date {
    // Simple parser for abstract timeframes like "Day 1", "Year 5"
    if (!timeframe) return new Date();

    const match = timeframe.match(/(day|week|month|year)\s*(\d+)/i);
    if (match) {
        const unit = match[1].toLowerCase();
        const num = parseInt(match[2], 10);
        const base = new Date('2000-01-01'); // Anchor date for abstract timeline

        if (unit === 'day') base.setDate(base.getDate() + num - 1);
        else if (unit === 'week') base.setDate(base.getDate() + (num - 1) * 7);
        else if (unit === 'month') base.setMonth(base.getMonth() + num - 1);
        else if (unit === 'year') base.setFullYear(base.getFullYear() + num - 1);

        return base;
    }
    return new Date();
}

function computeEndTime(start: Date, duration?: string): Date | undefined {
    if (!duration) return undefined;

    const hours = parseDurationToHours(duration);
    const end = new Date(start.getTime() + hours * 60 * 60 * 1000);
    return end;
}

function parseDurationToHours(duration: string): number {
    const lower = duration.toLowerCase();
    const num = parseInt(lower) || 1;

    if (lower.includes('minute')) return num / 60;
    if (lower.includes('hour')) return num;
    if (lower.includes('day')) return num * 24;
    if (lower.includes('week')) return num * 24 * 7;
    if (lower.includes('month')) return num * 24 * 30;
    return 1;
}

async function handleItemMove(item: any, callback: (item: any) => void) {
    try {
        await updateNodeTemporal(item.id, {
            chronological_date: item.start.toISOString(),
            plotline_tag: item.group
        });
        callback(item);
    } catch (e) {
        console.error('Failed to move scene:', e);
        callback(null); // Cancel move
    }
}

async function handleDrop(event: DragEvent) {
    const sceneId = event.dataTransfer?.getData('text/plain');
    if (!sceneId || !timeline.value) return;

    // Get time and group from the drop position
    const time = (timeline.value as any).getEventTime(event);
    const group = (timeline.value as any).getEventGroup(event);

    if (time && group) {
        await updateNodeTemporal(sceneId, {
            chronological_date: time.toISOString(),
            plotline_tag: String(group)
        });
    }
}

function toggleNarrativeConnectors() {
    showNarrativeConnectors.value = !showNarrativeConnectors.value;
}

function fitTimeline() {
    timeline.value?.fit();
}

function zoomIn() {
    timeline.value?.zoomIn(0.5);
}

function zoomOut() {
    timeline.value?.zoomOut(0.5);
}
</script>

<template>
    <div class="timeline-view">
        <!-- Control Panel -->
        <TimelineControls
            :show-connectors="showNarrativeConnectors"
            @toggle-connectors="toggleNarrativeConnectors"
            @fit="fitTimeline"
            @zoom-in="zoomIn"
            @zoom-out="zoomOut"
        />

        <div class="timeline-container">
            <!-- Holding Pen Sidebar -->
            <HoldingPen :scenes="unassignedScenes" />

            <!-- Main Timeline -->
            <div class="timeline-canvas-wrapper">
                <div 
                    ref="containerRef" 
                    class="timeline-canvas"
                    @dragover.prevent
                    @drop="handleDrop"
                ></div>

                <!-- Narrative Connectors Overlay -->
                <svg
                    v-if="showNarrativeConnectors"
                    class="narrative-overlay"
                    :viewBox="`0 0 ${containerRef?.clientWidth || 1000} ${containerRef?.clientHeight || 400}`"
                >
                    <defs>
                        <marker id="arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
                            <path d="M0,0 L0,6 L6,3 z" fill="var(--color-primary)" />
                        </marker>
                        <marker id="arrow-flashback" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
                            <path d="M0,0 L0,6 L6,3 z" fill="var(--color-warning)" />
                        </marker>
                    </defs>
                    <path
                        v-for="(conn, idx) in narrativeConnectors"
                        :key="idx"
                        :d="`M ${50 + idx * 100} 50 Q ${75 + idx * 100} 100 ${150 + idx * 100} 50`"
                        :stroke="conn.isFlashback ? 'var(--color-warning)' : 'var(--color-primary)'"
                        stroke-width="2"
                        fill="none"
                        :marker-end="conn.isFlashback ? 'url(#arrow-flashback)' : 'url(#arrow)'"
                        :class="{ 'flashback-connector': conn.isFlashback }"
                    />
                </svg>
            </div>
        </div>

        <!-- Scene Popover -->
        <ScenePopover
            v-if="hoveredScene"
            :scene-id="hoveredScene.id"
            :x="hoveredScene.x"
            :y="hoveredScene.y"
        />
    </div>
</template>

<style scoped>
.timeline-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
}

.timeline-container {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.timeline-canvas-wrapper {
    position: relative;
    flex: 1;
    overflow: hidden;
}

.timeline-canvas {
    width: 100%;
    height: 100%;
}

.narrative-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 10;
}

.flashback-connector {
    stroke-dasharray: 8 4;
    animation: flashback-pulse 2s ease-in-out infinite;
}

@keyframes flashback-pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
}

/* Global vis-timeline overrides */
:deep(.vis-timeline) {
    border: none;
    background: transparent;
}

:deep(.vis-panel.vis-center) {
    background: var(--bg-secondary);
}

:deep(.vis-item) {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.875rem;
    padding: 4px 8px;
}

:deep(.vis-item.vis-selected) {
    background: var(--color-primary);
    border-color: var(--color-primary-dark);
    color: white;
}

:deep(.vis-item.timeline-item--warning) {
    border: 2px solid var(--color-danger);
    box-shadow: 0 0 8px var(--color-danger);
}

:deep(.vis-label) {
    color: var(--text-secondary);
    font-weight: 500;
}

:deep(.vis-time-axis .vis-text) {
    color: var(--text-tertiary);
    font-size: 0.75rem;
}

:deep(.vis-group) {
    border-bottom: 1px solid var(--border-color);
}
</style>
