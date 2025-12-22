<script setup lang="ts">
import { computed, ref, watch } from 'vue';

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

async function handleUnschedule() {
  if (!props.sceneId) return;
  if (confirm('Remove this scene from the timeline? It will return to the Holding Pen.')) {
    await updateNodeTemporal(props.sceneId, {
      chronological_date: undefined,
      plotline_tag: undefined,
    });
    emit('close');
  }
}
import './SceneEditor.css';
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
    </div>
  </div>
</template>

<style scoped>
/* Styles moved to SceneEditor.css */
</style>
