<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { storeToRefs } from 'pinia';
import EditorMain from './EditorMain.vue';
import { useProjectStore } from '../stores/project';
import { useResearchStore } from '../stores/research';
import { useProjectNodeOperations } from '../composables/domain/useProjectNodeOperations';
import { useGamification } from '../composables/domain/useGamification';
import { useSettingsStore } from '../stores/settings';
import { projectApi } from '../api/project';
import { useAppStatus } from '../composables/ui/useAppStatus';
import { APP_CONSTANTS } from '../config/constants';
import type { Chapter } from '../types';

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

// Find active chapter data from nodes
const activeChapterContent = ref('');
const isLoading = ref(false);

const activeChapter = computed(() => {
  return nodeMap.value.get(props.chapterId);
});

const loadContent = async () => {
  if (!props.chapterId || !props.projectId) return;
  isLoading.value = true;
  try {
    const content = await projectApi.loadChapter(props.projectId, props.chapterId);
    activeChapterContent.value = content;
  } catch (e) {
    notifyError('Failed to load chapter content', e);
    activeChapterContent.value = '<h1>Error</h1><p>Could not load content.</p>';
  } finally {
    isLoading.value = false;
  }
};

watch(() => props.chapterId, loadContent, { immediate: true });

// Rename handler
const handleRename = async (newName: string) => {
  if (props.chapterId && newName !== activeChapter.value?.name) {
    // Note: useProjectNodeOperations handles errors internally for now,
    // but we should eventually catch them here if it throws.
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
let saveInterval: ReturnType<typeof setInterval> | undefined;

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

// Editor ref for manual saves
const editorRef = ref<InstanceType<typeof EditorMain> | null>(null);

const handleSave = async (content: string) => {
  await saveActiveChapter(content);
};

const setupAutoSave = () => {
  if (saveInterval) clearInterval(saveInterval);
  const intervalMs =
    (settings.value.general.autoSaveInterval || APP_CONSTANTS.EDITOR.AUTO_SAVE_INTERVAL) * 1000;
  saveInterval = setInterval(async () => {
    if (editorRef.value) {
      const content = editorRef.value.getContent();
      if (editorRef.value.isDirty()) {
        await saveActiveChapter(content);
        editorRef.value.markAsClean();
      }
    }
  }, intervalMs);
};

watch(() => settings.value.general.autoSaveInterval, setupAutoSave);

onMounted(() => {
  setupAutoSave();
});

onBeforeUnmount(async () => {
  clearInterval(saveInterval);
  if (editorRef.value && editorRef.value.isDirty()) {
    await saveActiveChapter(editorRef.value.getContent());
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
    ref="editorRef"
    :project-id="projectId"
    :title="activeChapter.name"
    :initial-content="activeChapterContent"
    :initial-word-count="activeChapter.word_count"
    :editor-settings="settings.editor"
    @update:title="handleRename"
    @content-change="handleContentChange"
    @save="handleSave"
    @research-link-click="handleResearchLinkClick"
  />
</template>
