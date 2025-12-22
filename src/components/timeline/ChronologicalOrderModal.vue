<script setup lang="ts">
import { ref } from 'vue';
import ConfirmationModal from '../base/ConfirmationModal.vue';
import { useProjectData } from '../../composables/logic/useProjectData';
import { useTimeline } from '../../composables/timeline/useTimeline';

const { updateStructure, nodeMap } = useProjectData();
const { assignedScenes } = useTimeline();

const showModal = ref(false);
const isApplying = ref(false);

function open() {
  showModal.value = true;
}

function close() {
  showModal.value = false;
}

async function applyChronologicalOrder() {
  isApplying.value = true;
  try {
    // Sort assigned scenes by chronological date
    const sorted = [...assignedScenes.value].sort((a, b) => {
      const aTime = a.chronological_date || a.abstract_timeframe || '';
      const bTime = b.chronological_date || b.abstract_timeframe || '';
      return aTime.localeCompare(bTime);
    });

    // Rebuild project data in chronological order
    // We use the nodeMap to get the full node data
    const reorderedData = sorted.map((scene) => {
      const fullNode = nodeMap.value.get(scene.id);
      return (
        fullNode || {
          id: scene.id,
          name: scene.title,
          children: [],
          filename: `${scene.id}.md`,
          word_count: 0,
        }
      );
    });

    await updateStructure(reorderedData);
    close();
  } catch (e) {
    console.error('Failed to apply chronological order:', e);
  } finally {
    isApplying.value = false;
  }
}

defineExpose({ open });
</script>

<template>
  <ConfirmationModal
    :show="showModal"
    title="Apply Chronological Order"
    confirm-text="Reorder Manuscript"
    cancel-text="Cancel"
    variant="warning"
    @confirm="applyChronologicalOrder"
    @cancel="close"
  >
    <div class="chrono-modal-content">
      <div class="warning-banner">
        <svg
          class="warning-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"
          />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <span>This action will permanently reorder your manuscript chapters.</span>
      </div>

      <p class="description">
        All chapters with temporal data will be reordered to match their chronological (story-time)
        sequence. Chapters without temporal data will be placed at the end.
      </p>

      <div class="preview-section">
        <h4 class="preview-title">Preview: New Order ({{ assignedScenes.length }} scenes)</h4>
        <ol class="preview-list">
          <li v-for="scene in assignedScenes.slice(0, 5)" :key="scene.id" class="preview-item">
            <span class="scene-name">{{ scene.title }}</span>
            <span class="scene-time">{{
              scene.chronological_date || scene.abstract_timeframe
            }}</span>
          </li>
          <li v-if="assignedScenes.length > 5" class="preview-more">
            ... and {{ assignedScenes.length - 5 }} more scenes
          </li>
        </ol>
      </div>

      <div v-if="isApplying" class="applying-state">
        <div class="spinner"></div>
        <span>Reordering manuscript...</span>
      </div>
    </div>
  </ConfirmationModal>
</template>

<style scoped>
.chrono-modal-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.warning-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: rgba(245, 158, 11, 0.15);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 8px;
  color: var(--color-warning);
  font-size: 0.875rem;
  font-weight: 500;
}

.warning-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  stroke: var(--color-warning);
}

.description {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.preview-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
}

.preview-title {
  margin: 0 0 10px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
}

.preview-list {
  margin: 0;
  padding-left: 20px;
  font-size: 0.813rem;
}

.preview-item {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  color: var(--text-primary);
}

.scene-name {
  font-weight: 500;
}

.scene-time {
  font-size: 0.75rem;
  color: var(--text-tertiary);
  font-family: monospace;
}

.preview-more {
  color: var(--text-tertiary);
  font-style: italic;
}

.applying-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border-color);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
