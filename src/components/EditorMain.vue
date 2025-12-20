<script setup lang="ts">
import { watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { EditorContent } from '@tiptap/vue-3'
import { useProjectData } from '../composables/useProjectData'
import { useGamification } from '../composables/useGamification'
import { useTiptapEditor } from '../composables/useTiptapEditor'
import { useSettings } from '../composables/useSettings'

const { activeId, projectId } = useProjectData()
const { addWords } = useGamification()
const { settings } = useSettings()

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
        if (activeId.value && projectId.value) {
            await saveChapter(projectId.value, activeId.value);
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
        @click="editor?.commands.focus()"
    >
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
