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
import ConfirmationModal from '../base/ConfirmationModal.vue';

const { selectNode, unassignedScenes } = useTimeline();
const { exportTimeline } = useTimelineExport();
const { applyChronologicalSort } = useTimelineSort();

// Refs
const containerRef = ref<HTMLElement | null>(null);
const narrativeOverlayRef = ref<InstanceType<typeof NarrativeOverlay> | null>(null);
const showCalendarSettings = ref(false);
const selectedSceneId = ref<string | null>(null);
const hoveredScene = ref<{ id: string; x: number; y: number } | null>(null);

// Sort Confirmation
const showSortConfirm = ref(false);

const handleSortRequest = () => {
  showSortConfirm.value = true;
};

const confirmSort = async () => {
  await applyChronologicalSort();
  showSortConfirm.value = false;
};

// Initialize vis-timeline using our new composable
const {
  isMounted,
  fit: fitTimeline,
  zoomIn: timelineZoomIn,
  zoomOut: timelineZoomOut,
  handleDrop,
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
  (info) => {
    hoveredScene.value = info;
  },
  () => {
    if (narrativeOverlayRef.value?.showNarrativeConnectors) {
      requestAnimationFrame(narrativeOverlayRef.value.updateConnectorPositions);
    }
  }
);

function zoomIn() {
  timelineZoomIn(0.5);
}
function zoomOut() {
  timelineZoomOut(0.5);
}

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
      @apply-chronological="handleSortRequest"
      @export="handleExport"
      @open-calendar="showCalendarSettings = true"
    />

    <div class="timeline-container">
      <!-- Holding Pen Sidebar -->
      <HoldingPen :scenes="unassignedScenes" />

      <!-- Main Timeline -->
      <div class="timeline-canvas-wrapper">
        <div ref="containerRef" class="timeline-canvas" @dragover.prevent @drop="handleDrop"></div>

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

    <CalendarSettings v-if="showCalendarSettings" @close="showCalendarSettings = false" />

    <ConfirmationModal
      :show="showSortConfirm"
      title="Apply Chronological Order"
      message="This will reorder your manuscript chapters based on their chronological time. This cannot be undone easily. Continue?"
      confirm-label="Apply Sort"
      :is-destructive="true"
      @close="showSortConfirm = false"
      @confirm="confirmSort"
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
  background-color: var(--bg-primary);
  background-image:
    linear-gradient(rgba(var(--color-ink-rgb), 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(var(--color-ink-rgb), 0.05) 1px, transparent 1px);
  background-size: 50px 50px;
}

.timeline-canvas {
  width: 100%;
  height: 100%;
}

:deep(.narrative-overlay) {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 10;
}

:deep(.flashback-connector) {
  stroke-dasharray: 8 4;
  animation: flashback-pulse 2s ease-in-out infinite;
}

@keyframes flashback-pulse {
  0%,
  100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
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
  transition:
    transform 0.1s ease,
    box-shadow 0.1s ease;
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
:deep(.vis-drag-left),
:deep(.vis-drag-right) {
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
:deep(.vis-drag-left::after),
:deep(.vis-drag-right::after) {
  content: '';
  display: block;
  width: 6px;
  height: 24px;
  background: var(--color-ink);
  border-radius: 4px;
  opacity: 0.8;
  transition: all 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

/* Hover Effect */
:deep(.vis-drag-left:hover::after),
:deep(.vis-drag-right:hover::after) {
  height: 32px;
  width: 8px;
  background: var(--color-accent);
  opacity: 1;
}

:deep(.vis-drag-left) {
  left: -12px;
}
:deep(.vis-drag-right) {
  right: -12px;
}

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
  box-shadow: 6px 6px 0px 0px rgba(0, 0, 0, 0.2); /* Soften shadow for selected or keep hard? Let's keep hard but maybe accent color */
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
