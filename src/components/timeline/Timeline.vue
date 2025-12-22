<script setup lang="ts">
import { ref } from 'vue';
import { useTimeline } from '../../composables/timeline/useTimeline';
import HoldingPen from './HoldingPen.vue';
import SceneEditor from './SceneEditor.vue';
import CalendarSettings from './CalendarSettings.vue';
import TimelineControls from './TimelineControls.vue';
import NarrativeOverlay from './NarrativeOverlay.vue';
import { useTimelineExport } from '../../composables/timeline/useTimelineExport';
import { useTimelineSort } from '../../composables/timeline/useTimelineSort';
import { useVisTimeline } from '../../composables/timeline/useVisTimeline';

import './timeline.css';

const { selectNode, unassignedScenes } = useTimeline();
const { exportTimeline } = useTimelineExport();
const { applyChronologicalSort } = useTimelineSort();

// Refs
const containerRef = ref<HTMLElement | null>(null);
const narrativeOverlayRef = ref<InstanceType<typeof NarrativeOverlay> | null>(null);
const showCalendarSettings = ref(false);
const selectedSceneId = ref<string | null>(null);
const hoveredScene = ref<{ id: string; x: number; y: number } | null>(null);

// Initialize vis-timeline using our new composable
const {
    isMounted,
    fit: fitTimeline,
    zoomIn: timelineZoomIn,
    zoomOut: timelineZoomOut,
    handleDrop
} = useVisTimeline(
    containerRef,
    (sceneId) => {
        if (sceneId) {
            selectedSceneId.value = sceneId;
            selectNode(sceneId);
        } else {
            selectedSceneId.value = null;
        }
    },
    (info) => { hoveredScene.value = info; },
    () => {
         if (narrativeOverlayRef.value?.showNarrativeConnectors) {
            requestAnimationFrame(narrativeOverlayRef.value.updateConnectorPositions);
        }
    }
);

function zoomIn() { timelineZoomIn(0.5); }
function zoomOut() { timelineZoomOut(0.5); }

async function handleExport(format: 'png' | 'pdf') {
    if (!containerRef.value) return;
    await exportTimeline(containerRef.value, format);
}
</script>

<template>
    <div class="timeline-view">
        <!-- Control Panel -->
        <TimelineControls
            :show-connectors="narrativeOverlayRef?.showNarrativeConnectors || false"
            @toggle-connectors="narrativeOverlayRef?.toggleConnectors"
            @fit="fitTimeline"
            @zoom-in="zoomIn"
            @zoom-out="zoomOut"
            @apply-chronological="applyChronologicalSort"
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

                <NarrativeOverlay
                    ref="narrativeOverlayRef"
                    :container="containerRef"
                    :is-mounted="isMounted"
                />
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
/* Component-specific non-theme styles can go here if needed,
   but most are in timeline.css now. */
</style>
