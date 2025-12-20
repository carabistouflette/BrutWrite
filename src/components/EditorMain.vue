<script setup lang="ts">
import { watch, onMounted, onBeforeUnmount } from 'vue'
import { EditorContent } from '@tiptap/vue-3'
import { useProjectData } from '../composables/useProjectData'
import { useGamification } from '../composables/useGamification'
import { useTiptapEditor } from '../composables/useTiptapEditor'

const { activeId, projectId } = useProjectData()
const { addWords } = useGamification()

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
onMounted(() => {
    saveInterval = setInterval(async () => {
        if (activeId.value && projectId.value) {
            await saveChapter(projectId.value, activeId.value);
            // console.debug(`[Auto-save] Saved ${activeId.value}`);
        }
    }, 30000)
    
    // Initial focus
    editor.value?.commands.focus()
    
    // Silence TS unused warning
    if (containerRef.value) { console.debug('Editor mounted') }
})

onBeforeUnmount(() => {
    clearInterval(saveInterval)
    editor.value?.destroy()
})

</script>

<template>
  <div ref="containerRef" class="h-full w-full overflow-y-auto scroll-smooth bg-transparent relative">
     <!-- Brutalist Editor Area -->
     <div class="max-w-3xl mx-auto py-24 min-h-[150vh] cursor-text" @click="editor?.commands.focus()">
        <editor-content :editor="editor" />
     </div>
  </div>
</template>

<style scoped>
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
