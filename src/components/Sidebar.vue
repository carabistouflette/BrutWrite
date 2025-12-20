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

const handleSelect = (id: string) => {
  console.log('Selected item:', id);
  // specific tauri logic will go here
};
</script>

<template>
  <div class="flex flex-1 w-full bg-paper text-ink font-sans overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-64 flex flex-col border-r border-stone h-full bg-paper/50 backdrop-blur-sm">
      <div class="p-6 font-medium text-sm tracking-widest uppercase text-ink/70 select-none">
        BrutWrite
      </div>
      
      <div class="flex-1 overflow-y-auto px-4 py-2">
        <FileTree 
          v-model="projectData" 
          @select="handleSelect"
        />
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
