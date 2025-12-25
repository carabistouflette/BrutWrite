<script setup lang="ts">
import { ref, computed, toRef, onBeforeUnmount, watch } from 'vue';
import { storeToRefs } from 'pinia';
import EditorMain from './EditorMain.vue';
import { useProjectStore } from '../../stores/project';
import { useResearchStore } from '../../stores/research';
import { useProjectNodeOperations } from '../../composables/domain/useProjectNodeOperations';
import { useGamification } from '../../composables/domain/useGamification';
import { useSettingsStore } from '../../stores/settings';
import { projectApi } from '../../api/project';
import { useAppStatus } from '../../composables/ui/useAppStatus';
import { APP_CONSTANTS } from '../../config/constants';
import type { Chapter } from '../../types';
import { useAutoSave } from '../../composables/editor/useAutoSave';
import { useChapterSession } from '../../composables/domain/useChapterSession';

const props = defineProps<{
  chapterId: string;
  projectId: string;
}>();

const projectStore = useProjectStore();
const { activeId, nodeMap } = storeToRefs(projectStore);
const researchStore = useResearchStore();
const { updateNodeStats, renameNode } = useProjectNodeOperations();
const { addWords } = useGamification();
const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);
const { notifyError } = useAppStatus();

// 1. Session Management (Loading content)
// We use toRef to keep reactivity for the composable
const { content: activeChapterContent, isLoading } = useChapterSession(
  toRef(props, 'projectId'),
  toRef(props, 'chapterId')
);

// --- Local Sync State ---
const currentHtml = ref('');
const isDirty = ref(false);

watch(activeChapterContent, (newVal) => {
  currentHtml.value = newVal;
  isDirty.value = false;
});

// Find active chapter metadata
const activeChapter = computed(() => {
  return nodeMap.value.get(props.chapterId);
});

// Rename handler
const handleRename = async (newName: string) => {
  if (props.chapterId && newName !== activeChapter.value?.name) {
    await renameNode(props.chapterId, newName);
  }
};

// Word count/Content change handler
const handleContentChange = (delta: number) => {
  addWords(delta);
  if (props.chapterId) {
    const currentWordCount = activeChapter.value?.word_count || 0;
    updateNodeStats(props.chapterId, currentWordCount + delta);
  }
};

const handleResearchLinkClick = (id: string) => {
  const artifact = researchStore.artifacts.find((a) => a.id === id);
  if (artifact) {
    researchStore.setActiveArtifact(artifact);
  } else {
    researchStore.fetchArtifacts().then(() => {
      const found = researchStore.artifacts.find((a) => a.id === id);
      if (found) researchStore.setActiveArtifact(found);
    });
  }
};

// Auto-save logic
const saveActiveChapter = async (content: string) => {
  if (!props.chapterId || !props.projectId) return;

  try {
    const metadata = await projectApi.saveChapter(props.projectId, props.chapterId, content);

    // Sync word count from backend
    if (activeId.value === props.chapterId) {
      const chapter = metadata.manifest.chapters.find((c: Chapter) => c.id === props.chapterId);
      if (chapter) {
        updateNodeStats(props.chapterId, chapter.word_count, false);
      }
    }
    console.debug(`Auto-saved chapter ${props.chapterId}`);
  } catch (e) {
    notifyError(`Failed to save chapter ${props.chapterId}`, e);
  }
};

const handleSave = async (content: string) => {
  await saveActiveChapter(content);
  isDirty.value = false;
};

// 2. Auto Save
const autoSaveInterval = computed(
  () => settings.value.general.autoSaveInterval || APP_CONSTANTS.EDITOR.AUTO_SAVE_INTERVAL
);

useAutoSave(async () => {
  if (isDirty.value) {
    await saveActiveChapter(currentHtml.value);
    isDirty.value = false;
  }
}, autoSaveInterval);

onBeforeUnmount(async () => {
  // Final save on unmount is critical, so we keep this manual hook
  if (isDirty.value) {
    await saveActiveChapter(currentHtml.value);
  }
});
</script>

<template>
  <div v-if="isLoading" class="h-full flex items-center justify-center">
    <div class="text-ink/40 animate-pulse font-serif italic text-2xl">Loading...</div>
  </div>
  <EditorMain
    v-else-if="activeChapter"
    :id="chapterId"
    :key="chapterId"
    v-model:is-dirty="isDirty"
    :project-id="projectId"
    :title="activeChapter.name"
    :initial-content="activeChapterContent"
    :initial-word-count="activeChapter.word_count"
    :editor-settings="settings.editor"
    @update:title="handleRename"
    @update:content="(val) => (currentHtml = val)"
    @content-change="handleContentChange"
    @save="handleSave"
    @research-link-click="handleResearchLinkClick"
  />
</template>
