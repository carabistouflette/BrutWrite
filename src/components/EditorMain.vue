<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { EditorContent } from '@tiptap/vue-3'
import { useProjectData } from '../composables/useProjectData'
import { useGamification } from '../composables/useGamification'
import { useTiptapEditor } from '../composables/useTiptapEditor'
import { useSettings } from '../composables/useSettings'

const { activeId, activeChapter, projectId, renameNode, updateNodeStats } = useProjectData()
const currentProjectId = projectId.value; // Capture the ID for use in unmount/cleanup
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
    (delta) => {
        addWords(delta);
        if (activeId.value) {
            const currentCount = activeChapter.value?.word_count || 0;
            updateNodeStats(activeId.value, currentCount + delta);
        }
    }
)

// Watch Active ID to reload content
watch(activeId, async (newId, oldId) => {
    // 1. Save old chapter if it exists and has changes
    if (oldId && currentProjectId) {
        await saveChapter(currentProjectId, oldId);
    }

    // 2. Load new chapter
    if (newId && currentProjectId) {
       await loadChapter(currentProjectId, newId);
       
       // Focus editor after load to ensure seamless writing experience
       setTimeout(() => {
           editor.value?.commands.focus();
       }, 50);
    }
}, { immediate: true });

// Auto-save logic
let saveInterval: ReturnType<typeof setInterval> | undefined;
    const setupAutoSave = () => {
        if (saveInterval) clearInterval(saveInterval);
        const intervalMs = (settings.value.general.autoSaveInterval || 30) * 1000;
        saveInterval = setInterval(async () => {
            if (activeId.value && currentProjectId) {
                await saveChapter(currentProjectId, activeId.value);
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

onBeforeUnmount(async () => {
    clearInterval(saveInterval)
    
    // Final save on unmount if there are changes
    if (activeId.value && currentProjectId) {
        await saveChapter(currentProjectId, activeId.value);
    }
    
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
