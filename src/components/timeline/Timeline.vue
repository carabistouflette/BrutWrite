<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { DataSet } from 'vis-data';
import { Timeline } from 'vis-timeline';
import 'vis-timeline/styles/vis-timeline-graph2d.min.css';
import { useTimeline } from '../../composables/useTimeline';
import HoldingPen from './HoldingPen.vue';
import SceneEditor from './SceneEditor.vue';
import CalendarSettings from './CalendarSettings.vue';
import TimelineControls from './TimelineControls.vue';
import html2canvas from 'html2canvas';
import jsPDF from 'jspdf';
import { useProjectData } from '../../composables/useProjectData';
import { useCalendar } from '../../composables/useCalendar';
import type { FileNode } from '../../types';

// Local type definitions for vis-timeline items
interface VisTimelineItem {
    id: string;
    group?: string | number;
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
    parseDurationToMillis,
    formatDurationFromMillis
} = useTimeline();

const { formatDate, getYear } = useCalendar();

// Refs
const containerRef = ref<HTMLElement | null>(null);
const timeline = ref<Timeline | null>(null);
const showNarrativeConnectors = ref(false);
const showCalendarSettings = ref(false);
const selectedSceneId = ref<string | null>(null);
const hoveredScene = ref<{ id: string; x: number; y: number } | null>(null);

// vis-timeline datasets
const items = new DataSet<any>([]);
const groups = new DataSet<any>([]);

// Initialize timeline
onMounted(() => {
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
            selectedSceneId.value = props.items[0];
            selectNode(props.items[0]); // Sync with project store
        } else {
            selectedSceneId.value = null;
        }
    });

    // Hover handler (optional, can keep for tooltips)
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
    
    // Zoom/Scroll listener to redraw connectors
    timeline.value.on('rangechange', () => {
        if (showNarrativeConnectors.value) {
            requestAnimationFrame(updateConnectorPositions);
        }
    });

    // Initial sync
    syncData();
});

onUnmounted(() => {
    timeline.value?.destroy();
});

// Watch for data changes
watch([plotlines, assignedScenes], syncData, { deep: true });
watch(showNarrativeConnectors, (val) => {
    if (val) requestAnimationFrame(updateConnectorPositions);
});

function syncData() {
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
    
    if (showNarrativeConnectors.value) {
        requestAnimationFrame(updateConnectorPositions);
    }
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

// Connector Overlay Logic
const connectorPaths = ref<{ d: string; color: string; isFlashback: boolean }[]>([]);

function updateConnectorPositions() {
    if (!timeline.value || !showNarrativeConnectors.value) {
        connectorPaths.value = [];
        return;
    }

    const paths: { d: string; color: string; isFlashback: boolean }[] = [];
    const containerRect = containerRef.value?.getBoundingClientRect();
    if (!containerRect) return;

    narrativeConnectors.value.forEach(conn => {
        const fromEl = containerRef.value?.querySelector(`.vis-item[data-id="${conn.from}"]`);
        const toEl = containerRef.value?.querySelector(`.vis-item[data-id="${conn.to}"]`);

        if (fromEl && toEl) {
            const fromRect = fromEl.getBoundingClientRect();
            const toRect = toEl.getBoundingClientRect();

            const x1 = fromRect.right - containerRect.left;
            const y1 = fromRect.top + fromRect.height / 2 - containerRect.top;
            const x2 = toRect.left - containerRect.left;
            const y2 = toRect.top + toRect.height / 2 - containerRect.top;

            const dist = Math.abs(x2 - x1);
            const cp1x = x1 + dist * 0.5;
            const cp1y = y1;
            const cp2x = x2 - dist * 0.5;
            const cp2y = y2;
            
            const isFlashback = x2 < x1;
            const pathD = `M ${x1} ${y1} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${x2} ${y2}`;

            paths.push({
                d: pathD,
                color: isFlashback ? 'var(--color-warning)' : 'var(--color-primary)',
                isFlashback
            });
        }
    });

    connectorPaths.value = paths;
}

// --- Export & Sort Logic ---

const { projectData, updateStructure } = useProjectData();

function sortNodesChronologically(nodes: FileNode[]): FileNode[] {
    return [...nodes].sort((a, b) => {
        const dateA = a.chronological_date || a.abstract_timeframe || '';
        const dateB = b.chronological_date || b.abstract_timeframe || '';
        if (!dateA && !dateB) return 0;
        if (!dateA) return 1;
        if (!dateB) return -1;
        return dateA.localeCompare(dateB);
    }).map(node => {
        if (node.children) {
            return { ...node, children: sortNodesChronologically(node.children) };
        }
        return node;
    });
}

async function handleApplyChronological() {
    if (confirm('This will reorder your manuscript chapters based on their chronological time. This cannot be undone easily. Continue?')) {
        const sorted = sortNodesChronologically(projectData.value);
        await updateStructure(sorted);
    }
}

import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';

// ... (existing imports)

async function handleExport(format: 'png' | 'pdf') {
    if (!containerRef.value) return;
    try {
        const canvas = await html2canvas(containerRef.value, {
            scale: 2,
            backgroundColor: '#1a1a1a', 
            ignoreElements: (element: Element) => element.classList.contains('narrative-overlay'),
        });

        const defaultName = `timeline_export_${new Date().toISOString().split('T')[0]}`;
        let fileData: Uint8Array;
        let filters = [];

        if (format === 'png') {
            const dataUrl = canvas.toDataURL('image/png');
            // Convert DataURL to Uint8Array
            const res = await fetch(dataUrl);
            const blob = await res.blob();
            fileData = new Uint8Array(await blob.arrayBuffer());
            filters = [{ name: 'PNG Image', extensions: ['png'] }];
        } else {
            const imgData = canvas.toDataURL('image/png');
            const pdf = new jsPDF({
                orientation: 'landscape',
                unit: 'px',
                format: [canvas.width, canvas.height] 
            });
            pdf.addImage(imgData, 'PNG', 0, 0, canvas.width, canvas.height);
            fileData = new Uint8Array(pdf.output('arraybuffer'));
            filters = [{ name: 'PDF Document', extensions: ['pdf'] }];
        }

        // Open Save Dialog
        const path = await save({
            defaultPath: defaultName,
            filters: filters
        });

        if (path) {
            await writeFile(path, fileData);
            console.log(`Export saved to ${path}`);
        }

    } catch (e) {
        console.error('Export failed:', e);
        // Fallback or User Notification could be added here
        alert('Failed to export: ' + e);
    }
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
            @apply-chronological="handleApplyChronological"
            @export="handleExport"
            @open-calendar="showCalendarSettings = true"
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
                <!-- We disable pointer events so clicks pass through to vis-timeline -->
                <svg
                    v-if="showNarrativeConnectors"
                    class="narrative-overlay"
                    xmlns="http://www.w3.org/2000/svg"
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
                        v-for="(path, idx) in connectorPaths"
                        :key="idx"
                        :d="path.d"
                        :stroke="path.color"
                        stroke-width="2"
                        fill="none"
                        :marker-end="path.isFlashback ? 'url(#arrow-flashback)' : 'url(#arrow)'"
                        :class="{ 'flashback-connector': path.isFlashback }"
                    />
                </svg>
            </div>
        </div>

        <!-- Scene Editor (Persistent Selection) -->
        <SceneEditor
            v-if="selectedSceneId"
            :scene-id="selectedSceneId"
            @close="selectedSceneId = null"
        />

        <CalendarSettings 
            v-if="showCalendarSettings"
            @close="showCalendarSettings = false"
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
    /* Data Grid Background */
    background-color: var(--color-paper);
    background-image: 
        linear-gradient(color-mix(in srgb, var(--color-stone), transparent 60%) 1px, transparent 1px),
        linear-gradient(90deg, color-mix(in srgb, var(--color-stone), transparent 60%) 1px, transparent 1px);
    background-size: 50px 50px;
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

/* --- ENHANCED ITEM STYLES --- */
:deep(.vis-item) {
    /* Brutalist High Contrast Style */
    background: var(--color-paper); /* Solid background to cover grid */
    border: 2px solid var(--color-ink); /* Solid dark border */
    border-radius: 4px;
    color: var(--color-ink);
    font-size: 1rem;
    font-weight: 700; /* Bolder text */
    padding: 0;
    
    /* Hard Shadow for Pop */
    box-shadow: 4px 4px 0px 0px var(--color-ink);
    
    height: 64px; /* Taller blocks */
    margin-bottom: 10px; /* Space for shadow */
    opacity: 1; /* No transparency */
    transition: transform 0.1s ease, box-shadow 0.1s ease;
    z-index: 1;
}

/* Hover effect */
:deep(.vis-item:hover) {
    transform: translate(-1px, -1px);
    box-shadow: 5px 5px 0px 0px var(--color-ink);
    z-index: 5;
}

:deep(.vis-item .vis-item-overflow) {
    overflow: visible;
}

/* Resize handles visibility */
:deep(.vis-drag-center) {
    cursor: grab;
}
:deep(.vis-drag-left), :deep(.vis-drag-right) {
    /* Larger hit area, transparent background */
    background: transparent;
    width: 24px; 
    cursor: col-resize;
    top: 0;
    bottom: 0;
    height: 100%;
    z-index: 100; /* Ensure on top */
    display: flex;
    align-items: center;
    justify-content: center;
    touch-action: none;
}

/* Visual Grip Indicator */
:deep(.vis-drag-left)::after, :deep(.vis-drag-right)::after {
    content: '';
    display: block;
    width: 6px;
    height: 24px; 
    background: var(--color-ink);
    border-radius: 4px;
    opacity: 0.8;
    transition: all 0.2s ease;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}

/* Hover Effect */
:deep(.vis-drag-left):hover::after, :deep(.vis-drag-right):hover::after {
    height: 32px;
    width: 8px;
    background: var(--color-accent);
    opacity: 1;
}

:deep(.vis-drag-left) { left: -12px; }
:deep(.vis-drag-right) { right: -12px; }

/* Custom content wrapper for items */
:deep(.scene-item-content) {
    display: flex;
    align-items: center;
    padding: 0 12px;
    height: 100%;
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1;
}

:deep(.vis-item.vis-selected) {
    /* Inverted style for selected */
    background: var(--color-ink);
    color: var(--color-paper);
    border-color: var(--color-ink);
    z-index: 10;
    box-shadow: 6px 6px 0px 0px rgba(0,0,0,0.2); /* Soften shadow for selected or keep hard? Let's keep hard but maybe accent color */
    box-shadow: 4px 4px 0px 0px var(--color-accent);
}

:deep(.vis-item.vis-range .vis-item-overflow) {
    overflow: visible;
}

:deep(.vis-item.timeline-item--warning) {
    border: 2px solid var(--color-danger);
    color: var(--color-danger);
}

:deep(.vis-item.timeline-item--projected) {
    border-style: dashed;
    opacity: 0.8;
}

:deep(.vis-label) {
    color: var(--color-ink);
    font-weight: 600;
}

:deep(.vis-time-axis .vis-text) {
    color: var(--color-ink);
    opacity: 0.7;
    font-size: 0.75rem;
    font-weight: 500;
}

:deep(.vis-group) {
    border-bottom: 1px solid var(--color-stone);
    border-right: 1px solid var(--color-stone); /* Grid like feeling */
    overflow: visible !important;
}

:deep(.vis-foreground) {
    overflow: visible;
}

:deep(.vis-panel) {
    overflow: visible;
}
</style>
