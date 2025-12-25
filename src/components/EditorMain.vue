<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { EditorContent } from '@tiptap/vue-3';
import { useGamification } from '../composables/logic/useGamification';
import { useTiptapEditor } from '../composables/editor/useTiptapEditor';
import { useSettingsStore } from '../stores/settings';
import { useProjectStore } from '../stores/project';
import { useResearchStore } from '../stores/research';
import { useProjectNodeOperations } from '../composables/logic/useProjectNodeOperations';
import { APP_CONSTANTS } from '../config/constants';

const props = defineProps<{
  chapterId: string;
  projectId: string;
}>();

const projectStore = useProjectStore();
const { updateNodeStats, renameNode } = useProjectNodeOperations();

const { addWords } = useGamification();
const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);
const researchStore = useResearchStore();

const activeChapter = computed(() => {
  // Helper to find node recursively
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const findNode = (nodes: any[], id: string): any => {
    for (const node of nodes) {
      if (node.id === id) return node;
      if (node.children) {
        const found = findNode(node.children, id);
        if (found) return found;
      }
    }
    return null;
  };
  return findNode(projectStore.nodes, props.chapterId);
});

// --- Title Logic ---
const activeChapterName = ref('');

watch(
  activeChapter,
  (chapter) => {
    if (chapter) {
      activeChapterName.value = chapter.name;
    }
  },
  { immediate: true }
);

const handleRename = async () => {
  if (props.chapterId && activeChapterName.value !== activeChapter.value?.name) {
    await renameNode(props.chapterId, activeChapterName.value);
  }
};

// --- Link Handler ---
const handleResearchClick = (e: MouseEvent) => {
  const target = (e.target as HTMLElement).closest('a');
  if (target && target.href.startsWith(APP_CONSTANTS.RESEARCH.PROTOCOL_PREFIX)) {
    e.preventDefault();
    // Format: research://<id>
    const id = target.href.replace(APP_CONSTANTS.RESEARCH.PROTOCOL_PREFIX, '');
    // We need to find the artifact. The store might not be loaded if we didn't open the panel yet.
    // So we force a fetch if needed, then find it.
    // For now, assume store.fetchArtifacts() is called or we call it.

    // Optimistic lookup
    const artifact = researchStore.artifacts.find((a) => a.id === id);
    if (artifact) {
      researchStore.setActiveArtifact(artifact);
    } else {
      // Fetch and try again
      researchStore.fetchArtifacts().then(() => {
        const found = researchStore.artifacts.find((a) => a.id === id);
        if (found) researchStore.setActiveArtifact(found);
      });
    }
  }
};

// --- Editor Logic ---
const { editor, containerRef, loadChapter, saveChapter } = useTiptapEditor((delta) => {
  addWords(delta);
  if (props.chapterId) {
    const activeWordCount = activeChapter.value?.word_count || 0;
    updateNodeStats(props.chapterId, activeWordCount + delta);
  }
});

// Watch Props and Editor instance to reload content
let lastLoadedId = '';
watch(
  [() => props.chapterId, () => props.projectId, editor],
  async ([newId, newPid, pEd]) => {
    // 1. Save old chapter if it exists and has changes
    // Note: oldId comes from the previous watcher state, which might be tricky with props.
    // Ideally we save explicitly before switching or rely on auto-save/unmount.
    // Use lastLoadedId for saving the previous one if we are switching.
    if (lastLoadedId && newPid && pEd && lastLoadedId !== newId) {
      await saveChapter(newPid, lastLoadedId);
    }

    // 2. Load new chapter
    if (newId && newPid && pEd) {
      // Only load if the ID has changed or we haven't loaded anything yet
      if (newId !== lastLoadedId) {
        await loadChapter(newPid, newId);
        lastLoadedId = newId;

        // Focus editor after load to ensure seamless writing experience
        setTimeout(() => {
          pEd.commands.focus();
        }, 50);
      }
    }
  },
  { immediate: true }
);

// Auto-save logic
let saveInterval: ReturnType<typeof setInterval> | undefined;
const setupAutoSave = () => {
  if (saveInterval) clearInterval(saveInterval);
  const intervalMs =
    (settings.value.general.autoSaveInterval || APP_CONSTANTS.EDITOR.AUTO_SAVE_INTERVAL) * 1000;
  saveInterval = setInterval(async () => {
    if (props.chapterId && props.projectId) {
      await saveChapter(props.projectId, props.chapterId);
    }
  }, intervalMs);
};

watch(() => settings.value.general.autoSaveInterval, setupAutoSave);

onMounted(() => {
  setupAutoSave();

  // Initial focus
  editor.value?.commands.focus();

  // Silence unused ref warning
  if (containerRef.value) {
    /* Ref is used in template */
  }
});

onBeforeUnmount(async () => {
  clearInterval(saveInterval);

  // Final save on unmount if there are changes
  if (props.chapterId && props.projectId) {
    await saveChapter(props.projectId, props.chapterId);
  }

  editor.value?.destroy();
});

const editorStyles = computed(() => {
  const s = settings.value.editor;
  return {
    fontFamily:
      s.fontFamily === 'serif'
        ? 'var(--font-serif)'
        : s.fontFamily === 'mono'
          ? 'var(--font-mono)'
          : 'var(--font-sans)',
    fontSize: `${s.fontSize}px`,
    lineHeight: s.lineHeight,
    maxWidth: `${s.maxWidth}px`,
  };
});
</script>

<template>
  <div
    ref="containerRef"
    class="h-full w-full overflow-y-auto scroll-smooth bg-transparent relative"
    :class="{ 'focus-mode': settings.editor.focusMode }"
    @click="handleResearchClick"
  >
    <!-- Brutalist Editor Area -->
    <div
      class="mx-auto py-24 min-h-[150vh] cursor-text transition-all duration-500"
      :style="editorStyles"
    >
      <!-- Chapter Title Overlay -->
      <div v-if="activeChapter" class="mb-16 group relative">
        <input
          v-model="activeChapterName"
          class="w-full bg-transparent border-none outline-none text-5xl font-serif font-black text-ink/90 placeholder:text-ink/10 transition-all focus:text-accent selection:bg-accent/20"
          :placeholder="APP_CONSTANTS.STRINGS.PLACEHOLDERS.CHAPTER_TITLE"
          @blur="handleRename"
          @keyup.enter="handleRename"
        />
        <div
          class="absolute -bottom-4 left-0 w-12 h-1 bg-accent/20 group-focus-within:w-24 group-focus-within:bg-accent transition-all duration-500"
        ></div>
      </div>

      <editor-content :editor="editor" />
    </div>
  </div>
</template>

<style scoped>
/* Focus Mode */
:deep(.focus-mode .ProseMirror > *) {
  opacity: 0.2;
  transition: opacity 0.5s ease;
  filter: blur(1px);
}

:deep(.focus-mode .ProseMirror > *.has-focus) {
  opacity: 1;
  filter: blur(0);
}

/* Custom Scrollbar for Brutalist feel */
::-webkit-scrollbar {
  width: 8px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: theme('colors.ink');
  opacity: 0.2;
}
::-webkit-scrollbar-thumb:hover {
  background: theme('colors.accent');
}
</style>
