<script setup lang="ts">
import { ref, toRef } from 'vue';
import { storeToRefs } from 'pinia';
import EditorMain from './EditorMain.vue';
import SnapshotManager from '../snapshots/SnapshotManager.vue';
import { useSettingsStore } from '../../stores/settings';
import { useEditorSession } from '../../composables/editor/useEditorSession';

const props = defineProps<{
  chapterId: string;
  projectId: string;
}>();

const showSnapshotManager = ref(false);
const editorRef = ref<InstanceType<typeof EditorMain> | null>(null);
const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);

// Use new composable
const {
  activeChapter,
  currentHtml,
  isLoading,
  isDirty,
  handleRename,
  handleContentChange,
  handleSave,
  restoreSnapshot,
  branchSnapshot,
  openResearchArtifact,
} = useEditorSession(toRef(props, 'projectId'), toRef(props, 'chapterId'));

const onRestore = async (_content: string, filename: string) => {
  showSnapshotManager.value = false;
  const newContent = await restoreSnapshot(filename);
  if (newContent && editorRef.value) {
    editorRef.value.setContent(newContent);
  }
};

const onBranch = async (_content: string, filename: string) => {
  showSnapshotManager.value = false;
  await branchSnapshot(filename);
};
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
    v-model:is-dirty="isDirty"
    :project-id="projectId"
    :title="activeChapter.name"
    :initial-content="currentHtml"
    :initial-word-count="activeChapter.word_count"
    :editor-settings="settings.editor"
    @update:title="handleRename"
    @update:content="(val) => (currentHtml = val)"
    @content-change="handleContentChange"
    @save="handleSave"
    @research-link-click="openResearchArtifact"
    @open-history="showSnapshotManager = true"
  />

  <SnapshotManager
    v-if="showSnapshotManager && props.chapterId"
    :chapter-id="props.chapterId"
    :current-content="currentHtml"
    @close="showSnapshotManager = false"
    @restore="onRestore"
    @branch="onBranch"
  />
</template>
