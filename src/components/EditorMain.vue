<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { EditorContent } from '@tiptap/vue-3'
import { useProjectData } from '../composables/useProjectData'
import { useGamification } from '../composables/useGamification'
import { useTiptapEditor } from '../composables/useTiptapEditor'
import { useSettings } from '../composables/useSettings'

const { activeId, activeChapter, projectId, renameNode } = useProjectData()
const { addWords } = useGamification()
const { settings } = useSettings()

// --- Title Logic ---
const activeChapterName = ref('');

watch(activeChapter, (chapter) => {
    if (chapter) {
        activeChapterName.value = chapter.name;
    }
}, { immediate: true });

const handleRename = async () => {
    if (activeId.value && activeChapterName.value !== activeChapter.value?.name) {
        await renameNode(activeId.value, activeChapterName.value);
    }
};

// --- Editor Logic ---
const { 
  editor, 
  containerRef, 
  loadChapter, 
  saveChapter 
} = useTiptapEditor(
    (delta) => addWords(delta) // Callback for gamification
)

// Watch Active ID to reload content
watch(activeId, async (newId) => {
    if (newId && projectId.value) {
       await loadChapter(projectId.value, newId);
    }
})

// Auto-save logic
let saveInterval: any
    const setupAutoSave = () => {
        if (saveInterval) clearInterval(saveInterval);
        const intervalMs = (settings.value.general.autoSaveInterval || 30) * 1000;
        saveInterval = setInterval(async () => {
            if (activeChapter.value && activeChapter.value.filename && projectId.value) {
                // Pass filename instead of ID for IO optimization
                await saveChapter(projectId.value, activeChapter.value.filename);
            }
        }, intervalMs);
    };

watch(() => settings.value.general.autoSaveInterval, setupAutoSave);

onMounted(() => {
    setupAutoSave();
    
    // Initial focus
    editor.value?.commands.focus()
    
    // Silence unused ref warning
    if (containerRef.value) { /* Ref is used in template */ }
})

onBeforeUnmount(() => {
    clearInterval(saveInterval)
    editor.value?.destroy()
})

const editorStyles = computed(() => {
    const s = settings.value.editor;
    return {
        fontFamily: s.fontFamily === 'serif' ? 'var(--font-serif)' : 
                    s.fontFamily === 'mono' ? 'var(--font-mono)' : 
                    'var(--font-sans)',
        fontSize: `${s.fontSize}px`,
        lineHeight: s.lineHeight,
        maxWidth: `${s.maxWidth}px`
    };
});
</script>

<template>
  <div ref="containerRef" class="h-full w-full overflow-y-auto scroll-smooth bg-transparent relative" :class="{ 'focus-mode': settings.editor.focusMode }">
     <!-- Brutalist Editor Area -->
     <div 
        class="mx-auto py-24 min-h-[150vh] cursor-text transition-all duration-500" 
        :style="editorStyles"
    >
        <!-- Chapter Title Overlay -->
        <div v-if="activeChapter" class="mb-16 group relative">
            <input 
                v-model="activeChapterName"
                @blur="handleRename"
                @keyup.enter="handleRename"
                class="w-full bg-transparent border-none outline-none text-5xl font-serif font-black text-ink/90 placeholder:text-ink/10 transition-all focus:text-accent selection:bg-accent/20"
                placeholder="Chapter Title"
            />
            <div class="absolute -bottom-4 left-0 w-12 h-1 bg-accent/20 group-focus-within:w-24 group-focus-within:bg-accent transition-all duration-500"></div>
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
