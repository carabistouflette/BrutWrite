<script setup lang="ts">
defineProps<{
  showConnectors: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connectors'): void;
  (e: 'fit'): void;
  (e: 'zoom-in'): void;
  (e: 'zoom-out'): void;
  (e: 'apply-chronological'): void;
  (e: 'export', format: 'png' | 'pdf'): void;
  (e: 'open-calendar'): void;
}>();
import BaseIcon from '../base/BaseIcon.vue';
import './TimelineControls.css';
</script>

<template>
  <div class="timeline-controls">
    <div class="controls-group">
      <span class="group-label">View</span>

      <button class="control-btn" title="Calendar Settings" @click="emit('open-calendar')">
        <BaseIcon name="calendar" />
        <span class="btn-label">Calendar</span>
      </button>

      <button
        class="control-btn"
        :class="{ active: showConnectors }"
        title="Toggle narrative connectors"
        @click="emit('toggle-connectors')"
      >
        <BaseIcon name="connectors" />
        <span class="btn-label">Connectors</span>
      </button>

      <button class="control-btn" title="Fit to view" @click="emit('fit')">
        <BaseIcon name="fit" />
      </button>

      <div class="zoom-controls">
        <button class="control-btn" title="Zoom out" @click="emit('zoom-out')">
          <BaseIcon name="minus" />
        </button>
        <button class="control-btn" title="Zoom in" @click="emit('zoom-in')">
          <BaseIcon name="plus" />
        </button>
      </div>
    </div>

    <div class="controls-group">
      <span class="group-label">Actions</span>

      <button
        class="control-btn action-btn"
        title="Reorder manuscript to chronological order"
        @click="emit('apply-chronological')"
      >
        <BaseIcon name="sort" />
        <span class="btn-label">Apply Order</span>
      </button>
    </div>

    <div class="controls-group">
      <span class="group-label">Export</span>

      <button class="control-btn" title="Export as PNG" @click="emit('export', 'png')">
        <BaseIcon name="image" />
        <span class="btn-label">PNG</span>
      </button>

      <button class="control-btn" title="Export as PDF" @click="emit('export', 'pdf')">
        <BaseIcon name="file" />
        <span class="btn-label">PDF</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Styles moved to TimelineControls.css */
</style>
