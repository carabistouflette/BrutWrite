<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import ConfirmationModal from '../base/ConfirmationModal.vue';
import { useTimeline } from '../../composables/timeline/useTimeline';

const props = defineProps<{
  sceneId: string | null;
}>();

const emit = defineEmits(['close']);

const { allChapters, updateNodeTemporal } = useTimeline();

const scene = computed(() => {
  if (!props.sceneId) return null;
  return allChapters.value.find((c) => c.id === props.sceneId);
});

// Local state for editing to avoid constant re-renders/sync issues while typing
const editDate = ref('');
const editDuration = ref('');

// Sync local state when scene changes
watch(
  scene,
  (newScene) => {
    if (newScene) {
      editDate.value = newScene.chronological_date || '';
      editDuration.value = newScene.duration || '';
    }
  },
  { immediate: true }
);

async function saveChanges() {
  if (!props.sceneId) return;
  await updateNodeTemporal(props.sceneId, {
    chronological_date: editDate.value,
    duration: editDuration.value,
  });
  // Optional: emit close or show success? For now, stays open to allow more edits.
}

// ... (existing imports moved or kept as needed, assuming this block replaces the script setup body appropriately or I insert carefully)
// Wait, I need to be careful not to break existing imports.
// Let's replace the whole script setup block to be safe or insert modal logic.

// Modal State
const showConfirm = ref(false);
let confirmResolve: ((value: boolean) => void) | null = null;

const closeConfirm = () => {
  showConfirm.value = false;
  if (confirmResolve) confirmResolve(false);
  confirmResolve = null;
};

const onConfirm = () => {
  showConfirm.value = false;
  if (confirmResolve) confirmResolve(true);
  confirmResolve = null;
};

async function handleUnschedule() {
  if (!props.sceneId) return;

  showConfirm.value = true;
  const confirmed = await new Promise<boolean>((resolve) => {
    confirmResolve = resolve;
  });

  if (confirmed) {
    await updateNodeTemporal(props.sceneId, {
      chronological_date: undefined,
      plotline_tag: undefined,
    });
    emit('close');
  }
}
</script>

<template>
  <div v-if="scene" class="scene-editor-panel">
    <div class="editor-header">
      <h3>{{ scene.title }}</h3>
      <button class="close-btn" @click="emit('close')">×</button>
    </div>

    <div class="editor-body">
      <div class="form-group">
        <label>Start Date (ISO)</label>
        <input v-model="editDate" type="datetime-local" class="brut-input" @change="saveChanges" />
      </div>

      <div class="form-group">
        <label>Duration</label>
        <input
          v-model="editDuration"
          type="text"
          placeholder="e.g. 2 hours, 3 days"
          class="brut-input"
          @change="saveChanges"
        />
        <small class="hint">Natural language supported</small>
      </div>

      <div class="stats-grid">
        <div class="stat">
          <span class="label">Words</span>
          <span class="value">{{ scene.word_count || 0 }}</span>
        </div>
        <div class="stat">
          <span class="label">POV</span>
          <span class="value">{{ scene.pov_character_id || '-' }}</span>
        </div>
      </div>

      <div class="actions">
        <button class="brut-btn danger" @click="handleUnschedule">Unschedule Scene</button>
      </div>

      <ConfirmationModal
        :show="showConfirm"
        title="Unschedule Scene"
        message="Remove this scene from the timeline? It will return to the Holding Pen."
        :is-destructive="true"
        @close="closeConfirm"
        @cancel="closeConfirm"
        @confirm="onConfirm"
      />
    </div>
  </div>
</template>

<style scoped>
.scene-editor-panel {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 320px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  z-index: 2000; /* Above timeline overlay */
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
}

.editor-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
}

.close-btn:hover {
  color: var(--text-primary);
}

.editor-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.getInput {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 8px 12px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 0.875rem;
}

.brut-input {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 8px 12px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 0.875rem;
}

.brut-input:focus {
  outline: 2px solid var(--color-accent);
  border-color: transparent;
}

.hint {
  font-size: 0.7rem;
  color: var(--text-tertiary);
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat .label {
  font-size: 0.7rem;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stat .value {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.actions {
  margin-top: 16px;
}

.brut-btn {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.brut-btn.danger {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-danger);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.brut-btn.danger:hover {
  background: rgba(239, 68, 68, 0.2);
}
</style>
