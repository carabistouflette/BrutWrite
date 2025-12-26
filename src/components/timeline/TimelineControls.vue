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
.timeline-controls {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.controls-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.group-label {
  font-size: 0.688rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
  margin-right: 6px;
}

.control-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.control-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
  background: rgba(
    var(--color-accent-rgb),
    0.05
  ); /* Using rgb var if available or just opacity on hex if supported or hardcode */
}

.control-btn.active {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: white;
}

.control-btn svg {
  width: 16px;
  height: 16px;
}

.btn-label {
  display: none;
}

@media (min-width: 900px) {
  .btn-label {
    display: inline;
  }
}

.zoom-controls {
  display: flex;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
}

.zoom-controls .control-btn {
  border: none;
  border-radius: 0;
  padding: 6px 8px;
}

.zoom-controls .control-btn:first-child {
  border-right: 1px solid var(--border-color);
}

.action-btn {
  background: rgba(var(--color-accent-rgb), 0.1);
  border-color: rgba(var(--color-accent-rgb), 0.3);
  color: var(--color-accent);
}

.action-btn:hover {
  background: var(--color-accent);
  color: white;
}
</style>
