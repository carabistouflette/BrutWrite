<script setup lang="ts">
import { ref } from 'vue';
import FileTree from './FileTree.vue';
import type { FileNode } from '../types';

// Mock data for initial development
const projectData = ref<FileNode[]>([
  {
    id: '1',
    name: 'Chapter 1: The Beginning',
    children: [
      { id: '1-1', name: 'Section 1.1: Intro' },
      { id: '1-2', name: 'Section 1.2: The Incident' }
    ]
  },
  {
    id: '2',
    name: 'Chapter 2: The Middle',
    children: []
  }
]);

const activeId = ref<string | undefined>(undefined);

const handleSelect = (id: string) => {
  activeId.value = id;
  console.log('Selected item:', id);
  // specific tauri logic will go here
};

const addChapter = () => {
  const newId = `chapter-${Date.now()}`;
  projectData.value.push({
    id: newId,
    name: 'New Chapter',
    children: []
  });
  // Auto-select new chapter
  activeId.value = newId;
};

const handleDelete = (id: string) => {
  // Recursive delete function
  const deleteFromList = (list: FileNode[]): boolean => {
    const index = list.findIndex(item => item.id === id);
    if (index !== -1) {
      list.splice(index, 1);
      return true;
    }
    for (const item of list) {
      if (item.children && deleteFromList(item.children)) {
        return true;
      }
    }
    return false;
  };
  
  deleteFromList(projectData.value);
  if (activeId.value === id) {
    activeId.value = undefined;
  }
};
</script>

<template>
  <div class="flex flex-1 w-full bg-paper text-ink font-sans overflow-hidden bg-noise relative">
    
    <!-- Background Texture Layer -->
    <div class="absolute inset-0 bg-grid-dots pointer-events-none z-0"></div>

    <!-- Sidebar -->
    <aside class="w-64 flex flex-col border-r border-stone/60 h-full bg-paper/80 backdrop-blur-md relative z-10 shadow-[4px_0_24px_rgba(0,0,0,0.02)]">
      <div class="p-6 font-serif italic font-bold text-2xl tracking-tight text-ink select-none relative">
        BrutWrite
        <span class="absolute -bottom-1 left-6 w-8 h-0.5 bg-accent/60"></span>
      </div>
      
      <div class="flex-1 overflow-y-auto px-4 py-2">
        <FileTree 
          v-model="projectData" 
          :active-id="activeId"
          @select="handleSelect"
          @delete="handleDelete"
        />
        
        <!-- Root Add Button -->
        <button 
          @click="addChapter"
          class="mt-4 w-full py-2 border border-dashed border-stone/50 text-xs text-ink/40 hover:text-accent hover:border-accent/50 transition-colors uppercase tracking-wider flex items-center justify-center gap-2"
        >
          <span>+ New Chapter</span>
        </button>
      </div>

      <div class="p-4 border-t border-stone/50">
        <!-- Settings -->
        <button class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left">
          Settings
        </button>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full bg-white relative">
      <!-- Top Bar / Header -->
      <div class="h-16 px-8 flex justify-between items-center bg-transparent">
        <h1 class="font-normal text-sm text-ink/40 uppercase tracking-widest">Editor</h1>
        <div class="space-x-2">
            <!-- Toolbar -->
        </div>
      </div>

      <!-- Editor Canvas -->
      <div class="flex-1 px-12 pb-12 overflow-y-auto max-w-3xl mx-auto w-full">
        <slot></slot>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Brutalist specific overrides if tailwind isn't enough */
</style>
