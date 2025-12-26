<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { EditorContent } from '@tiptap/vue-3';
import { useTiptapEditor } from '../../composables/editor/useTiptapEditor';
import { APP_CONSTANTS } from '../../config/constants';
import type { EditorSettings } from '../../config/defaultSettings';

const props = defineProps<{
  id: string;
  projectId: string;
  title: string;
  initialContent?: string;
  initialWordCount?: number;
  editorSettings: EditorSettings;
}>();

const emit = defineEmits<{
  (e: 'update:title', newTitle: string): void;
  (e: 'update:isDirty', value: boolean): void;
  (e: 'update:content', value: string): void;
  (e: 'content-change', delta: number): void;
  (e: 'save', content: string): void;
  (e: 'research-link-click', id: string): void;
  (e: 'open-history'): void;
}>();

const isDirtyModel = defineModel<boolean>('isDirty', { default: false });

// --- Title Logic ---
const editableTitle = ref(props.title);

watch(
  () => props.title,
  (newTitle) => {
    editableTitle.value = newTitle;
  },
  { immediate: true }
);

const handleTitleBlur = () => {
  if (editableTitle.value !== props.title) {
    emit('update:title', editableTitle.value);
  }
};

// --- Link Handler ---
const handleResearchClick = (e: MouseEvent) => {
  const target = (e.target as HTMLElement).closest('a');
  if (target && target.href.startsWith(APP_CONSTANTS.RESEARCH.PROTOCOL_PREFIX)) {
    e.preventDefault();
    const id = target.href.replace(APP_CONSTANTS.RESEARCH.PROTOCOL_PREFIX, '');
    emit('research-link-click', id);
  }
};

// --- Editor Logic ---
const { editor, isDirty, setContent, focus } = useTiptapEditor((payload) => {
  emit('content-change', payload.delta);
  emit('update:content', payload.html);
});

watch(isDirty, (newVal) => {
  isDirtyModel.value = newVal;
});

watch(isDirtyModel, (newVal) => {
  if (newVal === false) {
    isDirty.value = false;
  }
});

// Initialize content when editor is ready or initialContent changes
watch(
  [() => props.initialContent, editor],
  ([newContent, pEd]) => {
    if (pEd && newContent !== undefined) {
      setContent(newContent);

      // Focus editor after load
      setTimeout(() => {
        pEd.commands.focus();
      }, 50);
    }
  },
  { immediate: true }
);

onMounted(() => {
  focus();
});

onBeforeUnmount(() => {
  editor.value?.destroy();
});

const editorStyles = computed(() => {
  const s = props.editorSettings;
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

// Expose setContent
const setContentExposed = (html: string) => setContent(html);

defineExpose({
  focus,
  setContent: setContentExposed,
});
</script>

<template>
  <div
    ref="containerRef"
    class="h-full w-full overflow-y-auto scroll-smooth bg-transparent relative"
    :class="{ 'focus-mode': props.editorSettings.focusMode }"
    @click="handleResearchClick"
  >
    <!-- Brutalist Editor Area -->
    <div
      class="mx-auto py-24 min-h-[150vh] cursor-text transition-all duration-500"
      :style="editorStyles"
    >
      <!-- Chapter Title Overlay -->
      <div class="mb-16 group relative flex items-center justify-between">
        <div class="relative flex-1">
          <input
            v-model="editableTitle"
            class="w-full bg-transparent border-none outline-none text-5xl font-serif font-black text-ink/90 placeholder:text-ink/10 transition-all focus:text-accent selection:bg-accent/20"
            :placeholder="APP_CONSTANTS.STRINGS.PLACEHOLDERS.CHAPTER_TITLE"
            @blur="handleTitleBlur"
            @keyup.enter="handleTitleBlur"
          />
          <div
            class="absolute -bottom-4 left-0 w-12 h-1 bg-accent/20 group-focus-within:w-24 group-focus-within:bg-accent transition-all duration-500"
          ></div>
        </div>

        <button
          class="opacity-0 group-hover:opacity-100 transition-opacity p-2 text-ink/40 hover:text-accent"
          title="History & Snapshots"
          @click="$emit('open-history')"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 3v5h5" />
            <path d="M3.05 13A9 9 0 1 0 6 5.3L3 8" />
          </svg>
        </button>
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
  background: var(--ink);
  opacity: 0.2;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}
</style>
