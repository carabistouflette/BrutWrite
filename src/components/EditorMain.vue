<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import { useProjectData } from '../composables/useProjectData'

const { activeId, projectData } = useProjectData()
const container = ref<HTMLElement | null>(null)

// Mock content based on ID
const loadContent = (id: string) => {
    // In a real app, this would fetch from backend/fs
    const chapterName = findName(projectData.value, id) || 'Untitled'
    return `
# ${chapterName}

Start writing here...

## Section 1

*   Item 1
*   Item 2

\`\`\`javascript
console.log('Hello BrutWrite');
\`\`\`

**Bold text** and *Italic text*.
    `
}

const findName = (list: any[], id: string): string | null => {
    for (const item of list) {
        if (item.id === id) return item.name
        if (item.children) {
            const found = findName(item.children, id)
            if (found) return found
        }
    }
    return null
}

const editor = useEditor({
  content: activeId.value ? loadContent(activeId.value) : '',
  extensions: [
    StarterKit.configure({
        heading: {
            levels: [1, 2, 3]
        }
    }),
    Link.configure({
      openOnClick: false,
    }),
  ],
  editorProps: {
    attributes: {
      class: 'prose prose-invert prose-lg max-w-none focus:outline-none min-h-screen p-16',
    },
  },
  onUpdate: () => {
      handleScroll()
  },
  onSelectionUpdate: () => {
      handleScroll()
  }
})

const handleScroll = () => {
    // Defer slightly to ensure DOM update
    requestAnimationFrame(() => {
        if (!editor.value || !container.value) return
        
        const { from } = editor.value.state.selection
        const view = editor.value.view
        
        // Get cursor coordinates (viewport relative)
        const coords = view.coordsAtPos(from)
        
        // Calculate offset relative to container
        const containerRect = container.value.getBoundingClientRect()
        
        // coords.top is viewport y. containerRect.top is viewport y.
        // relativeTop is the position of cursor inside the VIEWPORT relative to container top?
        // No, we want the cursor to be at the middle of the container's visible area.
        
        // If containerRect.top is 0 (top of screen), and cursor is at 500.
        // relativeTop = 500. Target = height/2.
        
        // Just use valid calculation:
        // Center point of container in viewport
        const containerCenter = containerRect.top + containerRect.height / 2
        
        // Cursor position
        const cursorY = coords.top
        
        // Difference
        const diff = cursorY - containerCenter
        
        // If diff is significant, scroll
        if (Math.abs(diff) > 50) { // Threshold 50px
            container.value.scrollBy({
                top: diff,
                behavior: 'smooth'
            })
        }
    })
}

// Watch Active ID to reload content
watch(activeId, (newId) => {
    if (editor.value && newId) {
        editor.value.commands.setContent(loadContent(newId))
    }
})

// Auto-save
let saveInterval: any
onMounted(() => {
    saveInterval = setInterval(() => {
        if (activeId.value && editor.value) {
            const content = editor.value.getHTML()
            // Backend save would go here
            console.log(`[Auto-save] Saving ${activeId.value}...`, content.length)
        }
    }, 30000)
    
    // Initial focus
    editor.value?.commands.focus()
})

onBeforeUnmount(() => {
    clearInterval(saveInterval)
    editor.value?.destroy()
})

</script>

<template>
  <div ref="container" class="h-full w-full overflow-y-auto scroll-smooth bg-transparent relative">
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

/* Hide TipTap Bubble Menu if we had one (we don't) */
</style>
